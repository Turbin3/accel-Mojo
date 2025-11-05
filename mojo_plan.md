# Mojo Game Engine - Architecture & Design Document

**Last Updated:** 2025-11-03
**Status:** Planning Phase - Pre-Implementation

---

## 1. Vision & Goals

### What is Mojo?
Mojo is a game engine SDK built on top of Solana and MagicBlock's Ephemeral Rollups that **abstracts away blockchain complexity** from game developers.

### Core Value Proposition
- Developers write **pure client-side game code** (Bevy/Rust) - NO Solana program knowledge required
- Simple API: `create_world()`, `read_state()`, `write_state()`
- MagicBlock integration handled automatically (delegate, commit, undelegate)
- Fast gameplay on ephemeral rollups, final state committed to Solana mainnet

### Target Use Cases (POC)
- Turn-based games: Tic-tac-toe, Chess
- Real-time multiplayer: Skribbl.io style drawing games
- Simple casual games with on-chain state

---

## 2. Architecture Overview

### High-Level Components

```
┌─────────────────────────────────────────────────────────────┐
│                      GAME DEVELOPER'S APP                    │
│                      (Bevy Frontend)                         │
│                   All game logic lives here                  │
└───────────────────────┬─────────────────────────────────────┘
                        │ Uses simple API
                        ▼
┌─────────────────────────────────────────────────────────────┐
│                      MOJO SDK (Rust Crate)                   │
│  ┌─────────────┬─────────────┬──────────────┐              │
│  │ create_world│ read_state  │ write_state  │              │
│  └─────────────┴─────────────┴──────────────┘              │
│  - Abstracts MagicBlock delegation/commit                   │
│  - Builds transactions automatically                         │
│  - Handles PDA derivation                                    │
└───────────────────────┬─────────────────────────────────────┘
                        │ Calls instructions
                        ▼
┌─────────────────────────────────────────────────────────────┐
│               MOJO PROGRAM (On-chain Solana)                 │
│  - Owns all game state PDAs (Mother Program)                │
│  - Instructions: Initialize, CreateAccount, etc.            │
│  - Validates basic ownership/authority                       │
│  - Integrates with MagicBlock via CPIs                      │
└───────────────────────┬─────────────────────────────────────┘
                        │ Delegates to
                        ▼
┌─────────────────────────────────────────────────────────────┐
│              MAGICBLOCK EPHEMERAL ROLLUPS                    │
│  - Ultra-fast state updates (1ms block time)                │
│  - Temporary compute during gameplay                         │
│  - Commits final state back to Solana                       │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. Ownership & Authority Model

### Hierarchy

```
Mojo Program ID (Superior owner of all PDAs)
  └─ World PDA (Authority over child states)
      ├─ State PDA 1 (e.g., GameBoard)
      ├─ State PDA 2 (e.g., Player1 state)
      └─ State PDA 3 (e.g., Player2 state)
```

### Key Decisions (CONFIRMED)
- **Mojo Program** owns all PDAs (not the game developer's wallet/program)
- **World PDA** acts as authority for child state PDAs
- **Developers don't write Solana programs** - just client code + SDK
- This enables fast manipulation and transaction processing

---

## 4. PDA Structure & Derivation

### World PDA
```rust
PDA Seeds: [b"world", creator.pubkey(), world_name.as_bytes()]
Owner: Mojo Program ID

Structure:
{
    creator: Pubkey,      // Who created this world
    name: String,         // e.g., "tictactoe_game_123"
    created_at: i64,      // Timestamp
    is_delegated: bool,   // Is this on ephemeral rollup?
    // Additional metadata as needed
}
```

### State PDA (Game Entities)
```rust
PDA Seeds: [b"state", world_pda.key(), state_type_id, owner.pubkey()]
Owner: Mojo Program ID
Authority: World PDA

// state_type_id: u8 identifier for the state type
// Examples:
// - 0x01 for GameBoard
// - 0x02 for Player
// - 0x03 for Bird
// etc.

Structure: Developer-defined (via MojoState macro)
```

### Tradeoffs
- **State identifier (u8):** Adds 1 byte overhead but enables multiple state types per world
- **Separate PDAs per player:** Avoids account size limits, scales better (CONFIRMED: Option B)

---

## 5. Developer Experience - SDK API

### Example: Tic-Tac-Toe Game

```rust
use mojo_sdk::{World, MojoState};

// 1. Developer defines their game state
#[derive(MojoState)]
struct GameBoard {
    cells: [u8; 9],        // 0=empty, 1=X, 2=O
    current_player: u8,
    winner: u8,
}

#[derive(MojoState)]
struct Player {
    pubkey: Pubkey,
    symbol: u8,            // 1=X, 2=O
    wins: u32,
}

// 2. Create world (auto-delegates to ephemeral)
fn setup_game(player1: Pubkey, player2: Pubkey) -> Result<World> {
    let world = World::create("tictactoe_game_123")?;

    // 3. Initialize game state
    let board = GameBoard {
        cells: [0; 9],
        current_player: 1,
        winner: 0,
    };
    world.write_state(player1, board)?;

    // 4. Initialize players
    world.write_state(player1, Player { pubkey: player1, symbol: 1, wins: 0 })?;
    world.write_state(player2, Player { pubkey: player2, symbol: 2, wins: 0 })?;

    Ok(world)
}

// 5. Gameplay (fast updates on ephemeral)
fn make_move(world: &World, player: Pubkey, position: usize) -> Result<()> {
    // Read current state
    let mut board = world.read_state::<GameBoard>(player)?;

    // Update state (client-side validation)
    if board.cells[position] != 0 {
        return Err("Cell already occupied");
    }
    board.cells[position] = board.current_player;
    board.current_player = if board.current_player == 1 { 2 } else { 1 };

    // Write back to chain
    world.write_state(player, board)?;

    Ok(())
}

// 6. End game (commits to mainnet)
fn end_game(world: &World) -> Result<()> {
    world.commit_and_undelegate()?;
    Ok(())
}
```

---

## 6. Validation Strategy (HYBRID APPROACH)

### Client-Side Validation
- **Game-specific rules** (e.g., tic-tac-toe move validity)
- **Business logic** (e.g., score calculations)
- **UI/UX constraints** (e.g., player can't move out of turn)

**Why:** Game rules vary wildly - can't have generic on-chain validation

### On-Chain Validation (Mojo Program)
- **Authority checks** (e.g., is the signer authorized to update this state?)
- **Ownership verification** (e.g., does the World PDA own this state PDA?)
- **Basic integrity** (e.g., is the account initialized properly?)

**Why:** Prevent unauthorized state manipulation

### For High-Stakes Games
- Developers can **optionally write custom Solana programs** for additional validation
- Can integrate with Mojo via CPIs or separate validation layer
- **Out of scope for POC** but good to keep in mind

### Tradeoff
- Lower trust games: Client validation is sufficient
- Higher stakes games: Add custom program validation
- Flexibility > Security for POC phase

---

## 7. MagicBlock Integration

### Dependency
```toml
[dependencies]
ephemeral_rollups_sdk = "0.x.x"  # Add to mojo-program
```

### CPI Functions
- `ephemeral_rollups_sdk::cpi::delegate_account()` - Delegate PDA to ephemeral
- `ephemeral_rollups_sdk::cpi::commit()` - Sync state to mainnet
- `ephemeral_rollups_sdk::cpi::commit_and_undelegate()` - Sync + return authority
- `ephemeral_rollups_sdk::cpi::undelegate()` - Return authority without commit

### Transaction Flow Example (Tic-Tac-Toe)

#### Setup Phase (On Solana → Ephemeral)
```
Transaction 1: Create & Delegate World
  Instruction 1: create_world (Mojo program)
  Instruction 2: delegate_account (MagicBlock CPI)
  Result: World PDA now on ephemeral rollup

Transaction 2: Create & Delegate Game State
  Instruction 1: write_state<GameBoard> (Mojo program)
  Instruction 2: delegate_account (MagicBlock CPI)
  Result: GameBoard PDA now on ephemeral rollup
```

#### Gameplay Phase (On Ephemeral - Ultra Fast)
```
Transaction 3: Player 1 move (1ms latency)
  write_state<GameBoard> - Update cells[0] = X

Transaction 4: Player 2 move (1ms latency)
  write_state<GameBoard> - Update cells[1] = O

... 5 more moves (all on ephemeral, super fast) ...
```

#### Finalization Phase (Ephemeral → Solana)
```
Transaction 9: End game
  commit_and_undelegate (MagicBlock CPI)
  Result: Final GameBoard state synced to Solana mainnet
          World PDA authority returned
```

### Key Behaviors (CONFIRMED)
- **World PDA stays delegated entire game session** - This defines a "round" of gameplay
- **Payment for compute:** Players pay (TBD - needs research)
- **No session ID tracking needed for POC**

---

## 8. Program Instructions

### Current Implementation Status

| Instruction | Status | Purpose |
|-------------|--------|---------|
| Initialize | ✅ Implemented | Creates state accounts with custom data |
| CreateAccount | Planned | (May be same as Initialize?) |
| DelegateAccount | Planned | Wrapper for MagicBlock delegate CPI |
| Commit | Planned | Wrapper for MagicBlock commit CPI |
| UpdateDelegatedAccount | Planned | Write state while delegated |
| UnDelegateAccount | Planned | Wrapper for MagicBlock undelegate CPI |

### For POC - Simplified Instruction Set

We need at minimum:
1. **CreateWorld** - Create World PDA + delegate to ephemeral
2. **WriteState** - Create/update state PDA + delegate if needed
3. **CommitWorld** - Commit all state + undelegate

The rest can be abstracted by the SDK combining these primitives.

---

## 9. Data Structures

### Instruction Data Format

Based on current implementation in `create_account.rs`:

```rust
Instruction Data Layout:
[discriminator: u8][GenIxHandler: 16 bytes][actual_state: variable]

Where GenIxHandler:
{
    seeds: [u8; 8],   // Seeds for PDA derivation
    size: [u8; 8],    // Size of state data (as u64 in little-endian)
}
```

### For POC Consideration
- **Add state_type_id: u8** to identify which struct type this is
- Enables multiple state types per world
- 1 byte overhead is acceptable tradeoff

---

## 10. MojoState Macro Design

### Goal
Make it dead simple for developers to define game state structs.

### Minimum Viable Macro (POC)

```rust
#[derive(MojoState)]
struct Bird {
    x: u64,
    y: u64,
}

// Should auto-generate:
// 1. #[repr(C)]
// 2. #[derive(Pod, Zeroable, Clone, Copy)]
// 3. Serialization helpers (to_bytes, from_bytes)
```

### Nice-to-Have (Post-POC)
- PDA derivation helpers
- Type registration for runtime lookups
- Validation hooks
- Custom seed attributes

### Implementation Note
- **Proc macro** work will be a learning experience
- Start simple, iterate based on usage

---

## 11. SDK Architecture - Layered Approach

### Layer 1: Low-Level Primitives
```rust
// Direct instruction building
mojo_sdk::instructions::create_world_ix(...)
mojo_sdk::instructions::write_state_ix(...)
```

### Layer 2: World Abstraction
```rust
let world = World::create("game123")?;
world.write_state(...)?;
world.read_state::<Bird>(...)?;
```

### Layer 3: Transaction Builder
```rust
// Automatically bundles instructions + MagicBlock CPIs
let tx = WorldTransaction::new(world)
    .write_state(player1, bird1)
    .write_state(player2, bird2)
    .commit()
    .build()?;
```

### Layer 4: High-Level Game Helpers
```rust
// Game-specific abstractions
let game = TurnBasedGame::new(world)?;
game.add_player(player1)?;
game.take_turn(player1, move_data)?;
```

### POC Scope (CONFIRMED)
- **Layer 4 is what end users/devs interact with**
- Layers 1-3 are the engine enabling Layer 4
- All layers needed for POC to demonstrate value

---

## 12. Testing Strategy

### Primary: LiteSVM (CONFIRMED)
- In-memory SVM for unit tests
- Fast iteration
- Current pattern in `tests/mod.rs` works well

### Secondary: Mock MagicBlock (Test Mode)
- SDK has "test mode" that skips actual MagicBlock calls
- Uses mock implementations for delegate/commit
- Enables testing without MagicBlock dependency

### Integration Testing
- Full integration with MagicBlock testnet later
- Not required for initial POC development

---

## 13. Edge Cases & Error Handling

### Known Issues to Address (Eventually)

#### A) Player Disconnects Mid-Game
- **Problem:** State stays delegated indefinitely
- **Potential Solutions:**
  - Timeout mechanism in World PDA
  - Allow other players to force commit after timeout
  - Session expiry logic

#### B) Commit Fails
- **Problem:** State updates lost? Rollback?
- **Potential Solutions:**
  - Retry logic in SDK
  - Transaction confirmation polling
  - Fallback to last committed state

#### C) Simultaneous Writes
- **Problem:** Two players update same state concurrently
- **Potential Solutions:**
  - Last write wins (simple, may be acceptable)
  - Optimistic locking with version numbers
  - Turn-based games avoid this naturally

### POC Stance
- **Document these issues** but don't solve them yet
- Focus on happy path for POC
- Revisit with real user feedback

---

## 14. Current Code Status

### Files Overview

**Program (mojo-program):**
- `src/lib.rs` - Entry point, instruction router
- `src/instructions/mod.rs` - Instruction enum (6 planned)
- `src/instructions/create_account.rs` - Account creation logic
- `src/state/gen_ix_handler.rs` - Generic instruction handler struct
- `src/tests/mod.rs` - LiteSVM tests with MyPosition example

**Key Observations:**
- Using **Pinocchio** for low-level Solana primitives
- Current `Initialize` instruction creates PDAs with custom state
- Test demonstrates creating a `MyPosition { x: u64, y: u64 }` state
- GenIxHandler handles seeds + size, actual state appended after

### What's Missing for POC

1. **MagicBlock Integration**
   - Add `ephemeral_rollups_sdk` dependency
   - Implement delegate/commit/undelegate CPIs

2. **SDK Crate**
   - Create `mojo-sdk` workspace member
   - Implement World abstraction
   - Build transaction helpers

3. **MojoState Macro**
   - Create `mojo-macros` workspace member
   - Proc macro for state structs

4. **Real Game Example**
   - Tic-tac-toe or similar as POC
   - Demonstrates full flow: create → play → commit

---

## 15. Implementation Roadmap

### Phase 1: Foundation (Current)
- [ ] Finalize architecture decisions (this document)
- [ ] Align on unclear design points
- [ ] Get comfortable with MagicBlock APIs

### Phase 2: Core Program
- [ ] Add MagicBlock SDK dependency
- [ ] Refactor instructions for world/state model
- [ ] Implement CreateWorld, WriteState, CommitWorld
- [ ] Add delegation CPIs to each instruction
- [ ] Update tests with new structure

### Phase 3: SDK Development
- [ ] Create mojo-sdk crate
- [ ] Implement Layer 1 (instruction builders)
- [ ] Implement Layer 2 (World abstraction)
- [ ] Implement Layer 3 (transaction builder)

### Phase 4: Macro Magic
- [ ] Create mojo-macros crate
- [ ] Implement basic MojoState derive
- [ ] Add serialization helpers
- [ ] Test with various struct types

### Phase 5: Example Game
- [ ] Build tic-tac-toe example
- [ ] Demonstrates full SDK usage
- [ ] Layer 4 helpers for turn-based games
- [ ] Documentation and README

### Phase 6: Polish & Testing
- [ ] Comprehensive test coverage
- [ ] Error handling improvements
- [ ] Documentation polish
- [ ] Developer guide

---

## 16. Open Questions & TODOs

### Critical (Need answers before coding)
- [ ] **Macro capabilities:** What exactly should MojoState generate? (To be discovered during building)
- [ ] **State type ID:** Use u8 identifier or another approach?
- [ ] **Payment model:** Who pays for ephemeral compute? (Needs research)

### Important (Can be decided during implementation)
- [ ] **World metadata:** What fields does World PDA need?
- [ ] **Error types:** What errors should SDK expose?
- [ ] **Sync function:** What does `sync()` actually do? (From teammate's pseudocode)

### Nice-to-Have (Post-POC)
- [ ] **Session management:** Track ephemeral rollup sessions
- [ ] **Multi-world support:** One player in multiple games
- [ ] **Observer pattern:** Watch state changes without writing
- [ ] **State history:** Track state transitions over time

---

## 17. Success Criteria for POC

### Must Have
1. Developer can create a world with `World::create()`
2. Developer can define custom state with `#[derive(MojoState)]`
3. Developer can write state with `world.write_state()`
4. Developer can read state with `world.read_state::<T>()`
5. MagicBlock delegation happens automatically
6. Full tic-tac-toe game works end-to-end
7. Tests pass with LiteSVM

### Nice to Have
1. Multiple state types per world
2. Multi-player state management
3. Commit/undelegate works smoothly
4. Good error messages
5. Documentation for SDK usage

### Out of Scope
1. Production-ready error handling
2. Advanced validation
3. State history/versioning
4. Multiple game types beyond POC
5. TypeScript SDK (pure Rust for now)

---

## 18. References & Resources

### MagicBlock
- GitHub: https://github.com/magicblock-labs
- SDK: https://github.com/magicblock-labs/ephemeral-rollups-sdk
- Examples: https://github.com/magicblock-labs/magicblock-engine-examples
- Docs: https://docs.magicblock.gg

### Solana Development
- Pinocchio: https://github.com/anza-xyz/pinocchio
- LiteSVM: https://github.com/LiteSVM/litesvm

### Current Project
- Scratchboard: https://gist.github.com/inspi-writer001/aa5020faafd44e320a0a0e0c5e71d344
- Moodboard: https://excalidraw.com/#room=a46b67cad46194a6070f,KQR06GWzcammufK6P9A7uQ

---

## 19. Glossary

- **World:** A container/namespace for game entities, represented as a PDA
- **State:** Individual game entities (GameBoard, Player, Bird, etc.)
- **Ephemeral Rollup:** MagicBlock's temporary high-performance execution environment
- **Delegate:** Transfer PDA control to ephemeral rollup for fast updates
- **Commit:** Sync final state from ephemeral rollup back to Solana mainnet
- **Undelegate:** Return PDA authority from ephemeral rollup to original owner
- **Mother Program:** Our mojo-program that owns all game state PDAs
- **MojoState:** Derive macro that makes structs compatible with Mojo engine
- **PDA:** Program Derived Address - deterministically generated Solana account

---

## Document Status

**Last Discussion:** 2025-11-03
**Participants:** Developer (Abimbola) 
**Next Steps:**
1. Take a break if feeling overwhelmed
2. Review this document when ready
3. Clarify remaining open questions
4. Begin implementation when aligned

**Note:** This is a living document. Update as we learn more during implementation.

// 0xAbim
