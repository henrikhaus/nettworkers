# Nettworkers

A Rust-based networked multiplayer 2D platformer game demonstrating advanced client-server netcode techniques including client-side prediction, server reconciliation, and interpolation.

## 🎮 Overview

Nettworkers is a real-time multiplayer platformer that showcases modern networking techniques used in competitive online games. Players can move around a 2D world with physics-based movement while experiencing smooth, responsive gameplay despite network latency.

### Key Features

- **Real-time Multiplayer**: Support for multiple players in a shared 2D world
- **Advanced Netcode**: Implementation of client-side prediction, server reconciliation, and interpolation
- **Physics Engine**: Custom 2D physics with gravity, friction, and AABB collision detection
- **Scene System**: JSON-based level loading with decorative and collidable objects
- **Modern UI**: Immediate-mode GUI with multiple screens (main menu, settings, pause menu)
- **Cross-platform**: Built with Rust for performance and portability

## 🏗️ Architecture

The project is structured as a Rust workspace with three main components:

```
nettworkers/
├── server/          # Authoritative game server
├── client/          # Game client with rendering and UI
├── shared/          # Common data structures and protocols
└── scenes/          # JSON level definitions
```

### Server (`server/`)

- **UDP-based networking** on port 9000
- **100ms tick rate** for consistent game state updates
- **Authoritative physics** simulation
- **Player management** with automatic ID assignment
- **State broadcasting** to all connected clients

### Client (`client/`)

- **Macroquad-based rendering** engine
- **Client-side prediction** for responsive input
- **Server reconciliation** for state consistency
- **Interpolation** for smooth movement of other players
- **Scene rendering** with parallax effects

### Shared (`shared/`)

- **FlatBuffers serialization** for efficient network packets
- **Common game state** structures
- **Physics engine** shared between client and server
- **Command system** for player input

## 🌐 Networking Features

### Client-Side Prediction

Players can move immediately when pressing keys, without waiting for server confirmation. The client predicts the outcome of their actions locally for responsive controls.

### Server Reconciliation

When the client receives authoritative updates from the server, it reconciles any differences between its predicted state and the server's state using sequence numbers.

### Interpolation

Other players' movements are smoothly interpolated between server updates to provide fluid visual movement despite the discrete nature of network updates.

### Network Simulation

- **Configurable delay**: 1000ms artificial delay for testing netcode robustness
- **Sequence numbering**: For reliable state reconciliation
- **Timestamp synchronization**: Using Unix epoch timestamps

## 🚀 Getting Started

### Prerequisites

- **Rust** (latest stable version)
- **FlatBuffers compiler** (`flatc`) for protocol generation

### Installation

1. **Clone the repository**:

   ```bash
   git clone https://github.com/henrikhaus/nettworkers
   cd nettworkers
   ```

2. **Generate FlatBuffers code**:

   ```bash
   make generate_fbs
   ```

3. **Build the project**:
   ```bash
   cargo build --workspace
   ```

### Running the Game

1. **Start the server**:

   ```bash
   cargo run --bin server
   ```

   The server will start on `127.0.0.1:9000`

2. **Start the client** (in a separate terminal):

   ```bash
   cargo run --bin client
   ```

3. **Multiple clients**: Run additional client instances to test multiplayer functionality

### Controls

- **WASD** or **Arrow Keys**: Move left/right and jump
- **ESC**: Pause menu
- **Settings**: Toggle prediction, reconciliation, and interpolation features

## 🔧 Technical Details

### Network Protocol

The game uses a custom UDP protocol with FlatBuffers serialization:

#### Client → Server (Player Commands)

```rust
table PlayerCommands {
    sequence: uint32;           // For reconciliation
    dt_micro: uint64;          // Frame delta time
    commands: [PlayerCommand]; // Input commands
    client_timestamp_micro: uint64; // For latency calculation
}
```

#### Server → Client (Game State)

```rust
table GameState {
    client_player: ClientPlayer; // Authoritative client state
    players: [Player];          // Other players' states
    sequence: uint32;           // Server sequence number
}
```

### Physics System

- **Gravity**: Constant downward acceleration
- **Friction**: Ground friction for realistic movement
- **AABB Collision**: Axis-aligned bounding box collision detection
- **Penetration Resolution**: Separates overlapping objects

### Performance Characteristics

- **Server Tick Rate**: 100ms (10 TPS)
- **Client Frame Rate**: Variable (typically 60+ FPS)
- **Network Packet Size**: ~100-500 bytes per packet
- **Memory Usage**: Minimal due to Rust's zero-cost abstractions

## 🧪 Testing

Run the test suite:

```bash
# Run all tests
cargo test --workspace

# Run server tests only
cargo test --package server

# Run with verbose output
cargo test --workspace --verbose
```

### Test Coverage

- **Server functionality**: Player management, packet handling, game state updates
- **Physics system**: Movement, collision detection, boundary conditions
- **Integration tests**: Client-server communication
- **CI/CD**: Automated testing on GitHub Actions

## 🎨 Game Content

### Scene Format

Levels are defined in JSON format with the following structure:

```json
{
  "width": 2000.0,
  "height": 600.0,
  "spawn_point": { "x": 100.0, "y": 450.0 },
  "background_color": { "r": 20, "g": 20, "b": 50, "a": 255 },
  "decorations": {
    /* Visual elements */
  },
  "collidables": {
    /* Solid platforms and obstacles */
  }
}
```

### Available Scenes

- **scene_1.json**: Main platformer level with multiple platforms and decorations
- **scene_2.json**: Alternative level layout

## 🛠️ Development

### Project Structure

```
client/src/
├── main.rs              # Client entry point and game loop
├── predictor.rs         # Client-side prediction logic
├── interpolator.rs      # Interpolation for other players
├── render.rs           # Rendering system
├── ui/                 # User interface components
└── game_logic/         # Game state management

server/src/
└── main.rs             # Server entry point and networking

shared/src/
├── state/              # Game state and physics
├── *.fbs              # FlatBuffers schema definitions
└── generated/         # Auto-generated FlatBuffers code
```

### Key Dependencies

- **flatbuffers**: Efficient binary serialization
- **macroquad**: Cross-platform game framework
- **serde/serde_json**: JSON parsing for scenes
- **Standard library**: Networking, threading, collections

### Adding Features

1. **New player commands**: Add to `player_commands.fbs` and regenerate
2. **Game mechanics**: Implement in `shared/src/state/`
3. **UI elements**: Add to `client/src/ui/`
4. **Scenes**: Create new JSON files in `scenes/`

## 🔍 Debugging

### Network Debugging

- **Packet inspection**: Server logs all received packets
- **Latency simulation**: Configurable delay for testing
- **Sequence tracking**: Monitor prediction/reconciliation cycles

### Performance Profiling

```bash
# Profile the server
cargo run --release --bin server

# Profile the client
cargo run --release --bin client
```

## 📚 Learning Resources

This project demonstrates several important networking concepts:

- **Client-Server Architecture**: Authoritative server with client prediction
- **State Synchronization**: Keeping multiple clients in sync
- **Latency Compensation**: Techniques for responsive gameplay
- **Binary Protocols**: Efficient serialization with FlatBuffers
- **Real-time Systems**: Managing timing and consistency

## 🎯 Future Enhancements

- **TCP reliability layer** for critical game events (for example if we introduce gun combat)
- **Advanced physics** (slopes, moving platforms)
- **Audio system** with spatial sound
- Better **reconciliation** optimizations
