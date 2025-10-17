# Pinocchio Template

A simple Solana program template built with the Pinocchio framework for high-performance program development.

## Features

- 🚀 **High Performance**: Built with Pinocchio - Low CU BABY!
- 🧪 **Testing Ready**: Complete test suite using `solana-program-test` - the best balance between control and overhead
- 📦 **Modular Design**: Clean separation between program, SDK, CLI, and tests
- 🔧 **Developer Friendly**: No magic black box - all in simple Rust!

## Installation

```bash
git clone <your-repo-url>
cd pinocchio-template
./test.sh
```

## Design Decisions

Inspired by ( https://github.com/Nagaprasadvr/solana-pinocchio-starter )
Found from ( https://solana.stackexchange.com/questions/21489/are-there-any-examples-of-the-pinocchio-framework-that-i-can-study )


- **No Direct Dependencies**: No crate should use `solcat-diamond-hands-program` directly - the SDK forwards all important exports
- **Zero-Copy Performance**: Uses Pinocchio for minimal runtime overhead
- **Comprehensive Testing**: Uses Solana Program Test for integration tests with realistic program interactions. Local validator and other testing frameworks did not meet our needs.
- **Workspace Structure**: Organized as a Cargo workspace for better dependency management

## Contributing

Feel free to make PRs to make this template better!

1. Fork the repository
2. Make your changes
3. Run `./test.sh` to ensure tests pass
4. Submit a pull request

## License

MIT License - see [LICENSE.md](LICENSE.md) file for details


# SolCat Diamond Hands - Local Development Guide

This guide walks you through building, deploying, and testing the SolCat Diamond Hands vault program locally.

---

## Prerequisites

- Solana CLI tools installed
- Rust and Cargo installed
- Anchor framework installed (if using Anchor)

---

## Step 1: Build the Program

Build the program using the build script or CI script:

```bash
# Option 1: Using build script
./build.sh

# Option 2: Using CI script (also runs tests and installs CLI)
./ci.sh
```

This will compile the program and generate the `.so` file in `target/deploy/`.

---

## Step 2: Start Local Validator

Start a local Solana test validator:

```bash
solana-test-validator
```

**Note:** Keep this terminal open. The validator will run in the foreground.

**Optional flags:**
- `--reset` - Start with a clean ledger
- `--quiet` - Reduce log output
- `--bpf-program <PROGRAM_ID> <PROGRAM_PATH>` - Preload a program

---

## Step 3: Configure Solana CLI for Localhost

In a new terminal, configure the Solana CLI to use localhost:

```bash
solana config set -u localhost
```

Verify the configuration:

```bash
solana config get
```

You should see:
```
RPC URL: http://localhost:8899
```

---

## Step 4: Create an Example Token

Create a new SPL token mint for testing:

```bash
# Create a new mint (returns the MINT_ADDRESS)
spl-token create-token
```

**Save the mint address** - you'll need it for the CLI commands!
```bash
MINT_ADDRESS=<MINT_ADDRESS>
```

```bash
# Create a token account for your wallet
spl-token create-account $MINT_ADDRESS

# Mint some tokens to your account (e.g., 1000 tokens)
spl-token mint $MINT_ADDRESS 1000

# Verify your token balance
spl-token balance $MINT_ADDRESS
```

---

## Step 5: Deploy SolCat Program to Local Validator

Deploy the compiled program to your local validator:

```bash
solana program deploy target/deploy/solcat_diamond_hands_program.so --program-id target/deploy/solcat_diamond_hands_program-keypair.json
```

**Note:** This will output your program ID. Save it for reference, though it should match your declared program ID.

Verify the program is deployed:

```bash
solana program show CATvuZTNuyeBkoo5Tpeqtxcn51NDLNMExWPZ5vzQxkEg
```

---

## Step 6: Run CLI Commands

Lock tokens in a vault using the CLI:

```bash
# Lock All tokens for 10 slots
solcat-diamond-hands-cli \
    --rpc http://localhost:8899 \
    lock \
    --keypair ~/.config/solana/id.json \
    --mint $MINT_ADDRESS \
    --slots-to-lock 10

solcat-diamond-hands-cli \
    --rpc http://localhost:8899 \
    view \
    --wallet $(solana address)

# Empty the vault
solcat-diamond-hands-cli \
    --rpc http://localhost:8899 \
    empty \
    --keypair ~/.config/solana/id.json \
    --mint $MINT_ADDRESS

# Assure you have all of your tokens back
spl-token balance $MINT_ADDRESS
```

---

## Step 9: Shut Down Validator and Clean Up

### Stop the Validator

In the terminal running `solana-test-validator`, press:

```bash
Ctrl + C
```

### Clean Up Test Ledger (Optional)

Remove the test ledger data:

```bash
rm -rf test-ledger/
```

### Reset Solana Config (Optional)

If you want to switch back to devnet/mainnet:

```bash
# Devnet
solana config set -u devnet

# Mainnet
solana config set -u mainnet-beta
```

---

## Troubleshooting

### Program Deployment Fails
- Ensure your local validator is running
- Check you have enough SOL: `solana balance`
- Airdrop SOL if needed: `solana airdrop 2`

### Lock Command Fails
- Verify you have tokens: `spl-token balance <MINT_ADDRESS>`
- Ensure the program is deployed: `solana program show <PROGRAM_ID>`
- Check your keypair is set correctly: `solana address`

### Empty Command Fails with "Vault Locked"
- Check vault status: `solcat view --admin $(solana address)`
- Wait for the required slots to pass
- The vault displays remaining slots and epochs until unlock

### CLI Not Found
- Ensure you ran `./ci.sh` or `cargo install --path cli`
- Check your PATH includes `~/.cargo/bin`

---

## Quick Reference

### Useful Commands

```bash
# Check SOL balance
solana balance

# Airdrop SOL (localhost only)
solana airdrop 2

# Check token balance
spl-token balance <MINT_ADDRESS>

# View all token accounts
spl-token accounts

# Check validator status
solana cluster-version

# View program account
solana program show <PROGRAM_ID>
```

---

## Advanced: Fast-Forward Time

To test lockup periods without waiting:

```bash
# Start validator with faster slots
solana-test-validator --slots-per-epoch 100

# Or warp to a specific slot
solana-test-validator --warp-slot 1000000
```

---

## Next Steps

- Test edge cases (empty vault, double lock, etc.)
- Try locking different amounts and durations
- Integrate with a UI or other tools
- Deploy to devnet for broader testing

---

**Happy Building! 💎🙌**
