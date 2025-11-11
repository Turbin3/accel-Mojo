# Accel-Mojo

This is the Solana Program which is an account factory for the Mojo-sdk written with Pinocchio, bytemuck, sha2.

# Running the tests

```bash
cd mojo-program
solana config set --url devnet
solana-keygen new -s -o dev_wallet.json
solana airdrop 1 $(solana address -k dev_wallet.json)
```

Moodboard 1 - https://excalidraw.com/#room=a46b67cad46194a6070f,KQR06GWzcammufK6P9A7uQ

Tasks Sheet - https://docs.google.com/spreadsheets/d/1TqDlBIDCJ5K4CVYf0-OmwYorXIHBadmHVW4ndQMU79w/edit?hl=en-GB&gid=0#gid=0

Scratchboard - https://gist.github.com/inspi-writer001/aa5020faafd44e320a0a0e0c5e71d344
