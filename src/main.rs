use std::error::Error;
use std::fs::{self};
use std::io::{self};
use std::path::PathBuf;
use std::process::Command;
use std::env;

const RPC_URL: &str = "YOUR_RPC_URL_HERE";

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "generate" if args.len() == 4 => {
            let num_wallets: usize = args[2].parse()?;
            let wallet_dir = PathBuf::from(&args[3]);
            generate_wallets(num_wallets, &wallet_dir)?;
        },
        "distribute" if args.len() == 5 || args.len() == 6 => {
            let source_keypair = PathBuf::from(&args[2]);
            let wallet_dir = PathBuf::from(&args[3]);
            let amount_sol: f64 = args[4].parse()?;
            let priority_fee: Option<u64> = if args.len() == 6 { Some(args[5].parse()?) } else { None };
            distribute_sol(&source_keypair, &wallet_dir, amount_sol, priority_fee)?;
        },
        "view_balances" if args.len() == 3 => {
            let wallet_dir = PathBuf::from(&args[2]);
            view_balances(&wallet_dir)?;
        },
        "consolidate" if args.len() == 4 || args.len() == 5 => {
            let source_wallet_dir = PathBuf::from(&args[2]);
            let target_keypair = PathBuf::from(&args[3]);
            let priority_fee: Option<u64> = if args.len() == 5 { Some(args[4].parse()?) } else { None };
            consolidate_funds(&source_wallet_dir, &target_keypair, priority_fee)?;
        },
        _ => print_usage(),
    }

    Ok(())
}

fn print_usage() {
    println!("Usage:");
    println!("  generate <num_wallets> <directory> - Generates new wallets");
    println!("  distribute <source_keypair> <directory> <amount> [priority_fee] - Distributes SOL");
    println!("  view_balances <directory> - Views balances of wallets");
    println!("  consolidate <source_directory> <target_keypair> [priority_fee] - Consolidates SOL back");
}

fn generate_wallets(num_wallets: usize, wallet_dir: &PathBuf) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(wallet_dir)?;
    for i in 0..num_wallets {
        let wallet_path = wallet_dir.join(format!("wallet_{}.json", i));
        Command::new("solana-keygen")
            .args(&["new", "--no-bip39-passphrase", "--outfile", wallet_path.to_str().unwrap()])
            .status()?;
    }
    Ok(())
}

fn distribute_sol(source_keypair: &PathBuf, wallet_dir: &PathBuf, amount_sol: f64, priority_fee: Option<u64>) -> Result<(), Box<dyn Error>> {
    let wallets = fs::read_dir(wallet_dir)?
        .collect::<Result<Vec<_>, io::Error>>()?; // Collect to avoid double borrow
    let wallet_count = wallets.len();

    if wallet_count == 0 {
        return Err("No wallet files found in the specified directory.".into());
    }

    let amount_per_wallet = amount_sol / wallet_count as f64;
    let amount_per_wallet_str = amount_per_wallet.to_string(); // Create a let binding
    let fee_str = priority_fee.map(|fee| fee.to_string()); // Create an Option<String>

    for wallet in wallets {
        let wallet_path = wallet.path();
        let pubkey = extract_pubkey(&wallet_path)?;

        let mut attempts = 0;
        let max_attempts = 5;

        loop {
            let mut args = vec![
                "transfer",
                "--from",
                source_keypair.to_str().unwrap(),
                &pubkey,
                &amount_per_wallet_str, // Use the let binding
                "--fee-payer",
                source_keypair.to_str().unwrap(),
                "--allow-unfunded-recipient",
                "--commitment",
                "finalized",
                "--url",
                RPC_URL,
            ];

            if let Some(ref fee) = fee_str {
                args.extend(&["--with-compute-unit-price", fee]);
            }

            let output = Command::new("solana")
                .args(&args)
                .output()?;

            attempts += 1;
            if output.status.success() {
                println!("Successfully transferred {} SOL to {}", amount_per_wallet, pubkey);
                break;
            } else if attempts >= max_attempts {
                eprintln!("Failed to transfer to {} after {} attempts: {}", pubkey, max_attempts, String::from_utf8_lossy(&output.stderr));
                break;
            }

            println!("Attempt {} failed, retrying...", attempts);
        }
    }

    Ok(())
}

fn view_balances(wallet_dir: &PathBuf) -> Result<(), Box<dyn Error>> {
    let wallets = fs::read_dir(wallet_dir)?;
    for wallet in wallets {
        let wallet_path = wallet?.path();
        let pubkey = extract_pubkey(&wallet_path)?;
        let output = Command::new("solana")
            .args(&["balance", &pubkey, "--commitment", "finalized", "--url", RPC_URL])
            .output()?;
        println!("{}: {}", pubkey, String::from_utf8_lossy(&output.stdout).trim());
    }
    Ok(())
}

fn consolidate_funds(source_wallet_dir: &PathBuf, target_keypair: &PathBuf, priority_fee: Option<u64>) -> Result<(), Box<dyn Error>> {
    let target_pubkey = extract_pubkey(target_keypair)?;
    let source_keypairs = fs::read_dir(source_wallet_dir)?
        .collect::<Result<Vec<_>, io::Error>>()?; // Collect to avoid double borrow
    let fee_str = priority_fee.map(|fee| fee.to_string()); // Create an Option<String>

    let max_attempts = 5;

    for keypair_entry in source_keypairs {
        let keypair_path = keypair_entry.path();
        let source_pubkey = extract_pubkey(&keypair_path)?;

        let mut attempts = 0;
        loop {
            let mut args = vec![
                "transfer",
                "--from",
                keypair_path.to_str().unwrap(),
                &target_pubkey,
                "ALL",
                "--fee-payer",
                target_keypair.to_str().unwrap(),
                "--url",
                RPC_URL,
            ];

            if let Some(ref fee) = fee_str {
                args.extend(&["--with-compute-unit-price", fee]);
            }

            let output = Command::new("solana")
                .args(&args)
                .output()?;

            attempts += 1;
            if output.status.success() {
                println!("Successfully consolidated funds from {} to {}", source_pubkey, target_pubkey);
                break;
            } else if attempts >= max_attempts {
                eprintln!(
                    "Failed to consolidate funds from {} after {} attempts: {}",
                    source_pubkey,
                    max_attempts,
                    String::from_utf8_lossy(&output.stderr)
                );
                break;
            }

            println!("Attempt {} failed, retrying...", attempts);
        }
    }

    Ok(())
}

fn extract_pubkey(wallet_path: &PathBuf) -> Result<String, Box<dyn Error>> {
    let output = Command::new("solana-keygen")
        .args(&["pubkey", wallet_path.to_str().unwrap()])
        .output()?;
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}