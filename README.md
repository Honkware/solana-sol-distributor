# Solana Wallet Utility

This utility is designed to manage and operate on Solana wallets. It allows generating wallets, distributing SOL, viewing balances, and consolidating funds.

## Installation

Ensure that Rust is installed on your system. You can download and install Rust from [https://rustup.rs/](https://rustup.rs/).

### Solana CLI

Install the latest version of the Solana CLI using the following command:

```bash
sh -c "$(curl -sSfL https://release.solana.com/v1.18.4/install)"
```

## Usage

The utility supports the following commands:

- **Generate Wallets**
  ```
  cargo run generate <num_wallets> <directory>
  ```
  Generates a specified number of new wallets and stores them in the provided directory.

- **Distribute SOL**
  ```
  cargo run distribute <source_keypair> <directory> <amount> [priority_fee]
  ```
  Distributes SOL from a source keypair to all wallets in a directory, optionally with a priority fee.

- **View Balances**
  ```
  cargo run view_balances <directory>
  ```
  Displays the balances of all wallets in the specified directory.

- **Consolidate Funds**
  ```
  cargo run consolidate <source_directory> <target_keypair> [priority_fee]
  ```
  Consolidates all funds from the wallets in the source directory to a single target keypair, optionally with a priority fee.

Ensure you replace `YOUR_RPC_URL_HERE` in the code with your actual RPC URL.

For more details on each command, you can run them without arguments to see usage instructions.
