# Mojo

A Solana game engine SDK that lets developers build on-chain games without blockchain expertise. Powered by [MagicBlock's Ephemeral Rollups](https://magicblock.gg) for 1ms gameplay with Solana finality.

## Overview

Mojo abstracts blockchain complexity through a simple API: `create_world()`, `read_state()`, `write_state()`. Games run on ephemeral rollups for instant response times, then commit final state to Solana for persistence.

**Stack:**
- **mojo-program**: On-chain Solana program (Pinocchio, account factory pattern)
- **mojo-sdk**: Rust SDK for game developers
- **game-examples**: Reference implementations (Tic-Tac-Toe, Pong, Moving-Box) built with Bevy

**Program ID**: `3zt2gQuNsVRG8PAbZdYS2mgyzhUqG8sNwcaGJ1DYvECo`

## Quick Start

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) installed
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools) configured

### 1. Build & Deploy Program

```bash
cd mojo-program

# Configure Solana CLI for devnet
solana config set --url devnet

# Build the program
cargo build-sbf

# Verify program ID matches lib.rs
solana address -k target/deploy/mojo_program-keypair.json
# Update lib.rs if needed: pinocchio_pubkey::declare_id!("YOUR_ADDRESS");

# Deploy to devnet
solana program deploy --program-id target/deploy/mojo_program-keypair.json target/deploy/mojo_program.so
```

### 2. Run Tests

```bash
cd mojo-program

# Create test wallet
solana-keygen new -s -o dev_wallet.json

# Fund wallet
solana airdrop 1 $(solana address -k dev_wallet.json)

# Run tests
cargo build-sbf && cargo test -- --nocapture
```

### 3. Run Example Games

```bash
# Tic-Tac-Toe
cd game-examples/tic-tac-toe
cargo run

# Pong
cd game-examples/pong
cargo run

# Moving Box
cd game-examples/moving-box
cargo run
```

## SDK Usage

```rust
use mojo_sdk::*;

// Define game state
#[derive(Pod, Zeroable, Copy, Clone)]
#[repr(C)]
struct GameState {
    score: u64,
    turn: u8,
}
impl_mojo_state_pod!(GameState);

// Initialize client & create world
let client = SdkClient::new(RpcType::ERDev);
let creator = read_keypair_file("wallet.json")?;
let world = client.create_world(&creator, "my_game", GameState { score: 0, turn: 1 })?;

// Update state (runs on ephemeral rollup)
client.write_state(&world, "game_state", &creator, GameState { score: 100, turn: 2 })?;

// Commit to Solana
world.commit_and_undelegate()?;
```

## Project Structure

```
mojo-2/
├── mojo-program/       # Solana program (Pinocchio)
│   ├── src/
│   │   ├── instructions/   # Create, Delegate, Update, Commit, Undelegate
│   │   ├── state.rs        # Account schemas
│   │   └── lib.rs          # Entry point
│   └── Cargo.toml
├── mojo-sdk/           # Client library
│   ├── src/
│   │   ├── client.rs       # SdkClient
│   │   ├── sdk/world.rs    # World container
│   │   └── sdk/state.rs    # MojoState trait
│   └── Cargo.toml
└── game-examples/      # Bevy game templates
    ├── tic-tac-toe/
    ├── pong/
    └── moving-box/
```

## Key Features

- **Ephemeral Rollups Integration**: 1ms block times during gameplay, Solana finality on commit
- **Developer-Friendly API**: No Solana program writing required
- **Generic State Types**: Any type implementing `MojoState` works
- **Account Factory Pattern**: Deterministic PDAs via SHA256 seeding
- **Bevy Game Engine Support**: ECS architecture for game logic

## Resources

- [Moodboard](https://excalidraw.com/#room=a46b67cad46194a6070f,KQR06GWzcammufK6P9A7uQ)
- [Tasks Sheet](https://docs.google.com/spreadsheets/d/1TqDlBIDCJ5K4CVYf0-OmwYorXIHBadmHVW4ndQMU79w/edit?hl=en-GB&gid=0#gid=0)
- [Scratchboard](https://gist.github.com/inspi-writer001/aa5020faafd44e320a0a0e0c5e71d344)
