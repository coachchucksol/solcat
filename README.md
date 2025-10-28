# SOLCAT Diamond Hands Program

Hi! I'm [@CoachChuckSOL](https://github.com/coachchucksol) ( or [@CoachChuckFF](https://x.com/CoachChuckFF) ), I made this little diamond hands program to serve as a tutorial, starting point and or example code to further help people learn Solana development!

This idea all came from aping into [$SOLCAT](https://pump.fun/coin/84Y6h6XoaLAD1zxoQ2CDhcZYRpNsSBKsXULCnpjXpump), here is the [tweet](https://x.com/CoachChuckFF/status/1966574911071416414). I bought ~5% of the supply and have locked up my tokens for 6 months: HERE:

CA: `84Y6h6XoaLAD1zxoQ2CDhcZYRpNsSBKsXULCnpjXpump`
Note - It bothers me that people use the term CA for solana tokens. CA stands for `Contract Address` which comes from Ethereum. In Solana, all tokens come from the Token or Token2022 program. Its not a new "Contract". And the explorers call them: "Tokens" which is also confusing, because the `CA` is actually the `Mint`, and the `Token Account` is what holds the minted tokens. So, `CA` is the `Mint` and `Token` is the `Mint` and your `Token Account` is what holds the minted tokens. Anyways, thats my rant, thank you for coming to my ted talk!

## IMPORTANT

For legal reasons, I put out this program to serve as a tutorial, starting point and or example code to further help people learn Solana development! It should not be used for anything other than learning purposes. If you choose to use this program, you do so at your own risk and I cannot be held responsible for any losses incurred.

Additionally, I am NOT the creator of [$SOLCAT](https://pump.fun/coin/84Y6h6XoaLAD1zxoQ2CDhcZYRpNsSBKsXULCnpjXpump), I did buy it ~5% of supply, I am locking it up using this program ( 6 months, ~until March 2026 ), and I am using it to promote Solana development. I am not endorsing the token or promoting it in any way. And nothing here is financial advice.

### Accounts/Links

PLEASE DOUBLE CHECK YOU USE THE CORRECT ACCOUNTS AND LINKS TO AVOID LOSING YOUR TOKENS IF YOU DECIDE TO USE THIS PROGRAM

ProgramID: `CATvuZTNuyeBkoo5Tpeqtxcn51NDLNMExWPZ5vzQxkEg`
SOLCAT Mint: `84Y6h6XoaLAD1zxoQ2CDhcZYRpNsSBKsXULCnpjXpump`
My SOLCAT Wallet: `98DTkcLHy56bMFqCeWG2VsTkxf47ocQVbtkZfpHBw3v4`

### Proof of Lockup

Here is the proof of lockup my lockup: [link](https://explorer.solana.com/address/84Y6h6XoaLAD1zxoQ2CDhcZYRpNsSBKsXULCnpjXpump)

You can view it using the included CLI:

```bash
# Build and install CLI
./ci.sh
solcat-diamond-hands-cli \
    --rpc https://api.mainnet-beta.solana.com \
    view \
    --wallet 98DTkcLHy56bMFqCeWG2VsTkxf47ocQVbtkZfpHBw3v4
```

## How to lock up your tokens

*NOTE*: Although I trust my work, and I am okay with the risk of putting my tokens in a vault for 6 months, I am also aware that there is alaways risk of losing my tokens if the program is not working as expected or a bug is found! So for that reason, I cannot take responsibility for any loss of funds.

With that out of the way. You have a couple of options to lock up your tokens:

1. **Manual Lockup**: Use the CLI to manually lock up your tokens.
2. **UI Lockup**: Use the vibe-coded frontend to lock up your tokens.

### Manual Lockup
I will leave #1 as an exercise for the reader ( see the tutorial below )

### UI Lockup
I may host a locking site at some point, however, for legal reasons right now, I am going to suggest you run it locally.

ALSO NOTE - I am proud of the rust code in this repo. However, the frontend is vibe coded, it works but it is NOT a good production-ready solution. Its janky, but it works. Maybe someday, I program it correctly. That being said, the one design pattern I wanted to point out is that only the `server` can call RPC methods as to not leak the RPC endpoint to the client. This is important!

To run the UI locally, follow these steps:

1. Clone the repository
```bash
git clone https://github.com/coachchucksol/solcat
```

2. Navigate to the frontend app
```bash
cd solcat/app
```
3. Setup the `.env` file
```bash
cp .env.example .env.local
```

4. Edit the `.env` file to use a mainnet RPC, and double check the SOLCAT mint it correct! ( Yes, you could lock up any token if you wanted to! )
```bash
# This RPC is safe to use, since it is only used client-side, this should be your faster RPC
RPC_ENDPOINT=https://api.mainnet-beta.solana.com
# NOTE! This is public, only use an RPC that you are okay exposing, like `https://api.mainnet-beta.solana.com`
NEXT_PUBLIC_RPC_ENDPOINT=https://api.mainnet-beta.solana.com
# SOLCAT Accounts - these are also public
NEXT_PUBLIC_SOLCAT_DIAMOND_HANDS_ID=CATvuZTNuyeBkoo5Tpeqtxcn51NDLNMExWPZ5vzQxkEg
NEXT_PUBLIC_SOLCAT_MINT=84Y6h6XoaLAD1zxoQ2CDhcZYRpNsSBKsXULCnpjXpump
```

5. Run the frontend
```bash
npm run dev
```

6. Open your browser and navigate to http://localhost:3000
7. Set how many slots you want to lock up for your vault ( 1 slot ~= 500ms)
I would reccomend setting it to 1 slot and try a full cycle of lock/unlock before you commit to something bigger.
8. Click the `Create Vault` button to lock up your vault
9. Wait until the slots have passed and click `Empty Vault`

## License

MIT License - see [LICENSE.md](LICENSE.md) file for details

# How to Use this Repo

Instead of a guide in the readme here, I went through and annotated a lot of the code to help you understand what's going on. I would highly recommend you deploying the program locally, and see if you can add any new features!

[CHALLENGE] - Take this program and add a feature to take a small vault fee to unlock the vault early!

## Features

- 🚀 **High Performance**: Built with Pinocchio - Low CU BABY!
- 🧪 **Testing Ready**: Complete test suite using `solana-program-test` - the best balance between control and overhead.
- 📦 **Modular Design**: Clean separation between program, SDK, CLI, and tests
- 🔧 **Developer Friendly**: No magic black boxs - all in simple Rust! I also follow the principle of As Few Crates As Possible (AFCAP)
- 🤖 **Vibe Coded UI**: The NextJS UI was vibecoded, its kinda janky and ugly. I may come back and make it better at some point.

## Installation

```bash
git clone https://github.com/coachchucksol/solcat
./ci.sh
```

# SolCat Diamond Hands - Local Development Guide

This guide walks you through building, deploying, and testing the SolCat Diamond Hands vault program locally.

---

## Prerequisites

- [Solana CLI tools installed](https://solana.com/docs/intro/installation)
- A good attitude!

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

Start a local Solana test validator in a new terminal:

```bash
solana-test-validator
```

or, if you're on the cutting edge, use [surfpool!](https://surfpool.run/)

**Note:** Keep this terminal open. The validator will run in the foreground.

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

You should get an output like this:
```bash
Creating token 4Q4nUGn1MtGF5Y4NYAdQtFCUkTr4jwaN4uLrZzY4E1Fi under program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA

Address:  4Q4nUGn1MtGF5Y4NYAdQtFCUkTr4jwaN4uLrZzY4E1Fi
Decimals:  9

Signature: 65D8BjKCZkNbobLumyhpyBJjVsZYEnbuvcEzAPbHmqUTb9dhbaGSw1M5yaHkC9aCJusTqDDdsSGpL4GS9e1dBUrM
```

**Save the mint address (`Address`)** - you'll need it for the CLI commands!

```bash
# MINT_ADDRESS=4Q4nUGn1MtGF5Y4NYAdQtFCUkTr4jwaN4uLrZzY4E1Fi
MINT_ADDRESS=<MINT_ADDRESS>
```

Now create a token account and mint some tokens!

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

# View the vault
solcat-diamond-hands-cli \
    --rpc http://localhost:8899 \
    view \
    --wallet $(solana address)

# Check that all of your tokens have been moved and locked
spl-token balance $MINT_ADDRESS

# Wait for 10 slots ( almost instant ) and Empty the vault
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

### Clean Up Test Ledger

Remove the test ledger data:

```bash
rm -rf test-ledger/
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
