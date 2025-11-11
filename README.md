# Accel-Mojo

This is the Solana Program which is an account factory for the Mojo-sdk written with Pinocchio, bytemuck, sha2.

## Running the tests

_🚨 it is expected that you have already set up your Solana cli_

```bash
 # enter the program directory
cd mojo-program
 # set environment to devnet
solana config set --url devnet
 # create a new keypair to test on devnet
solana-keygen new -s -o dev_wallet.json
 # airdrop some SOL to that wallet
solana airdrop 1 $(solana address -k dev_wallet.json)
 # build and test the program
cargo build-sbf && cargo test -- --nocapture
```

Moodboard 1 - https://excalidraw.com/#room=a46b67cad46194a6070f,KQR06GWzcammufK6P9A7uQ

Tasks Sheet - https://docs.google.com/spreadsheets/d/1TqDlBIDCJ5K4CVYf0-OmwYorXIHBadmHVW4ndQMU79w/edit?hl=en-GB&gid=0#gid=0

Scratchboard - https://gist.github.com/inspi-writer001/aa5020faafd44e320a0a0e0c5e71d344
