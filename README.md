# 🎲 Space Looter - 3D Isometric Dice RPG

A modern 3D isometric RPG with dice-based mechanics, built with Rust and Bevy engine, featuring Domain-Driven Design (DDD) architecture and compiled to WebAssembly for cross-browser web deployment.

## 🎮 Game Overview

Space Looter is a 3D isometric space exploration RPG where dice mechanics determine the outcomes of your actions. Explore procedurally generated worlds, build and upgrade your base, complete quests, and engage in turn-based combat encounters. The game showcases modern game development practices with a complete DDD architecture implementation.

### ✨ RPG Features
- **🎲 Dice-Based Mechanics**: All actions resolved through strategic dice rolling
- **🏗️ Base Building**: Construct and upgrade your space station
- **🗺️ Procedural Exploration**: Discover new locations and resources
- **⚔️ Turn-Based Combat**: Strategic encounters with dice-based resolution
- **📈 Character Progression**: Level up stats, gain experience, and unlock abilities
- **💰 Resource Management**: Gather and manage various space resources
- **📜 Quest System**: Complete missions and advance storylines
- **🎒 Inventory Management**: Collect and equip items and equipment

### 🏛️ Architecture Features
- **🏗️ Domain-Driven Design**: Clean architecture with clear separation of concerns
- **🌐 Cross-Browser Compatible**: Works on Chrome, Firefox, Safari, Edge, and mobile browsers
- **⚡ Real-time 3D**: Smooth isometric 3D rendering powered by Bevy ECS
- **🎮 RPG Controls**: Intuitive keyboard controls for exploration and management
- **📱 Web-Optimized**: Uses WebGL2 for maximum browser compatibility
- **🧪 Test Coverage**: Comprehensive unit and integration tests across all layers

## 🎯 How to Play

### 🎮 Controls
- **Movement**: WASD or Arrow keys to explore the world
- **Dice Rolling**: SPACE to roll dice for actions and events
- **Base Management**: B to access your base
- **Quest Log**: Q to view active and completed quests
- **Inventory**: I to manage items and equipment
- **Pause**: ESC to pause/resume the game
- **Start Game**: ENTER to begin from the main menu

### 🎲 Game Mechanics
- **Exploration**: Move around the 3D isometric world to discover new locations
- **Dice Actions**: Use dice rolls to determine success of exploration, combat, and resource gathering
- **Base Development**: Upgrade your facilities to unlock new capabilities
- **Resource Collection**: Gather materials to craft items and upgrade your base
- **Quest Completion**: Accept and complete various missions for rewards
- **Character Growth**: Gain experience and improve your character's abilities

## 🛠️ Development Setup

### Prerequisites
- Rust (latest stable version)
- wasm-pack
- Modern web browser

### Installation
```bash
# Clone the repository
git clone <your-repo-url>
cd space-looter

# Install Rust WASM target and tools
make setup
```

## 🌐 Building for Web

### Quick Build
```bash
# Build the web version
make web

# Build and serve locally
make serve
```

### Manual Build
```bash
# Build WASM package
wasm-pack build --target web --out-dir pkg --features web

# Serve with any HTTP server
python -m http.server 8000
# Or use Node.js
npx http-server .
```

## 🚀 Running Natively

```bash
# Run the native version
cargo run

# Run in release mode for better performance
cargo run --release
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test domain::
```

## 📁 Project Structure

Following Domain-Driven Design principles:

```
space-looter/
├── src/                 # Source code with DDD architecture
│   ├── domain/          # Pure business logic (entities, value objects, services)
│   │   ├── entities/    # Core RPG entities (Player, Base, Quest, Map)
│   │   ├── value_objects/ # Immutable value types (Position3D, Dice, Resources)
│   │   └── services/    # Domain services (Collision, Exploration)
│   ├── application/     # Use cases and application services
│   │   ├── use_cases/   # RPG operations (MovePlayer, RollDice, ManageBase)
│   │   └── services/    # Application coordination services
│   ├── infrastructure/ # External concerns (Bevy integration, web APIs)
│   │   ├── bevy/        # Bevy ECS integration (components, systems, resources)
│   │   ├── random/      # Dice rolling and RNG services
│   │   └── web/         # Web-specific implementations
│   ├── presentation/   # User interface and state management
│   │   ├── game_state.rs # RPG state management
│   │   ├── input.rs     # Input handling for RPG controls
│   │   └── rendering.rs # 3D isometric rendering coordination
│   ├── lib.rs          # Library entry point with RPG setup
│   └── main.rs         # Native executable entry point
├── web/                # Web deployment files
│   ├── index.html      # Main web page
│   └── style.css       # Web styling
├── Cargo.toml          # Rust dependencies and configuration
└── README.md           # This file
```

## 🎯 Core Systems

### 🏛️ Domain Layer (Business Logic)
- **🎲 Dice Mechanics**: Core dice rolling system with modifiers and critical success/failure
- **👤 Player Entity**: Character stats, experience, inventory, and progression
- **🏠 Base Entity**: Base building, upgrades, and resource storage
- **🗺️ Map Entity**: Procedural world generation and exploration
- **📜 Quest Entity**: Mission system with objectives and rewards
- **💎 Resource Value Objects**: Materials, currency, and crafting components

### 🔧 Application Layer (Use Cases)
- **🏃 Move Player**: Exploration with movement point costs and dice-based events
- **🎲 Roll Dice**: Action resolution system for all RPG mechanics  
- **🏗️ Manage Base**: Building construction and upgrades
- **📜 Quest Management**: Accept, progress, and complete missions
- **⚔️ Handle Encounters**: Combat and event resolution

### 🖥️ Infrastructure Layer (Technical)
- **🎮 Bevy Integration**: 3D isometric rendering and ECS systems
- **🎰 Random Services**: Dice rolling and procedural generation
- **🌐 Web Support**: WASM compilation and browser compatibility

### 🎨 Presentation Layer (UI/UX)
- **🎮 RPG State Management**: Game states (exploration, combat, base management)
- **⌨️ Input Handling**: Context-sensitive controls for different game modes
- **📺 3D Rendering**: Isometric view coordination and visual presentation

## 🎮 Game States

The RPG supports multiple game states:
- **📋 Main Menu**: Start your adventure
- **🎨 Character Creation**: Customize your space explorer
- **🗺️ Exploration**: Move around and discover the world
- **⚔️ Combat**: Turn-based encounters with dice resolution
- **🏠 Base Management**: Upgrade facilities and manage resources
- **📜 Quest Log**: Track your missions and objectives
- **🎒 Inventory**: Manage equipment and items
- **⚙️ Settings**: Configure game preferences

## 🔧 Configuration

The game includes configurable settings:
- Dice mechanics (critical thresholds, modifier ranges)
- Resource generation rates
- Base upgrade costs and requirements
- Experience progression curves
- Procedural generation parameters

## 🌐 Deployment

### Web Deployment
- Supports deployment to any static hosting service
- Optimized WASM bundle for fast loading
- Progressive Web App (PWA) support
- Mobile-friendly responsive design

### Native Distribution
- Cross-platform native executables
- Optimized release builds
- Standalone distribution without dependencies

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Follow the DDD architecture patterns
4. Add tests for new functionality
5. Ensure all tests pass (`cargo test`)
6. Commit your changes (`git commit -m 'Add amazing feature'`)
7. Push to the branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Bevy Engine**: For the powerful ECS game engine
- **Rust Community**: For the amazing ecosystem and tools
- **Domain-Driven Design**: For the architectural principles
- **WebAssembly**: For enabling high-performance web games

---

**Ready to explore the cosmos with dice in hand? Clone the repo and start your space adventure today!** 🚀🎲