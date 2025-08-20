# 🚀 Space Looter

A modern 2D space shooter game built with Rust and Bevy engine, featuring Domain-Driven Design (DDD) architecture and compiled to WebAssembly for cross-browser web deployment.

## 🎮 Game Overview

Space Looter is a clean, well-architected 2D space shooter where you control a green spaceship and collect red enemies to increase your score. The game showcases modern game development practices with a complete DDD architecture implementation.

### Features
- **🏗️ Domain-Driven Design**: Clean architecture with clear separation of concerns
- **🌐 Cross-Browser Compatible**: Works on Chrome, Firefox, Safari, Edge, and mobile browsers
- **⚡ Real-time Physics**: Smooth movement and collision detection powered by Bevy ECS
- **🎮 Responsive Controls**: WASD or Arrow key movement with configurable input mapping
- **🎯 Dynamic Enemies**: Randomly spawning enemies with collision-based scoring
- **📱 Web-Optimized**: Uses WebGL2 for maximum browser compatibility
- **🧪 Test Coverage**: Comprehensive unit and integration tests across all layers

## 🎯 How to Play

- **Movement**: Use WASD keys or Arrow keys to move your green spaceship
- **Objective**: Collect red enemies by flying into them
- **Scoring**: Each enemy collected gives you 10 points
- **Boundaries**: Stay within the screen boundaries
- **Survive**: Keep collecting enemies to increase your score!

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

### Manual Build Process
```bash
# Add WASM target
rustup target add wasm32-unknown-unknown

# Install wasm-pack
cargo install wasm-pack

# Build WASM package
wasm-pack build --target web --out-dir pkg --release --features web

# Copy files to dist directory
cp pkg/space_looter.js dist/
cp pkg/space_looter_bg.wasm dist/
```

<old_text line=54>
```
space-looter/
├── src/                 # Source code with DDD architecture
│   ├── domain/          # Pure business logic (entities, value objects, services)
│   │   ├── entities/    # Core business entities (Player, Enemy, GameSession)
│   │   ├── value_objects/ # Immutable value types (Position, Velocity, Score)
│   │   └── services/    # Domain services (CollisionService, SpawningService)
│   ├── application/     # Use cases and application services
│   │   ├── use_cases/   # Business operations (MovePlayer, SpawnEnemies, etc.)
│   │   └── services/    # Application coordination services
│   ├── infrastructure/ # External concerns (Bevy integration, web APIs)
│   │   ├── bevy/        # Bevy ECS integration (components, systems, resources)
│   │   ├── random/      # Cross-platform random number generation
│   │   └── web/         # WebAssembly and web-specific code
│   ├── presentation/    # UI and input handling
│   │   ├── game_state.rs # Application state management
│   │   ├── input.rs     # Input mapping and processing
│   │   └── rendering.rs # Rendering coordination
│   ├── main.rs          # Native entry point
│   ├── lib.rs           # WASM library entry point & app configuration
│   └── README.md        # Architecture documentation
├── web/
│   └── index.html       # Source HTML template
├── dist/                # Generated build files (gitignored)
│   ├── index.html       # Game HTML page
│   ├── space_looter.js  # Generated JavaScript bindings (~100KB)
│   ├── space_looter_bg.wasm # Game WebAssembly binary (~30MB)
│   ├── serve.py         # Python development server
│   └── serve.js         # Node.js development server
├── pkg/                 # WASM build artifacts (gitignored)
├── target/              # Rust build cache (gitignored)
├── Cargo.toml           # Rust dependencies and configuration
├── Makefile             # Build automation
├── build-web.sh         # Web build script
├── .gitignore           # Git ignore rules
└── README.md            # This file
```

## 🏗️ Architecture Overview

This project implements **Domain-Driven Design (DDD)** principles with clean architecture:

### **Domain Layer** 📦
- **Pure business logic** with no external dependencies
- **Entities**: `Player`, `Enemy`, `GameSession` with rich behavior
- **Value Objects**: `Position`, `Velocity`, `Score` with validation
- **Domain Services**: `CollisionService`, `SpawningService`

### **Application Layer** ⚙️
- **Use Cases**: Specific business operations
- **Application Services**: Coordinate domain operations
- **DTOs**: Data transfer between layers

### **Infrastructure Layer** 🔧
- **Bevy Integration**: ECS components, systems, and resources
- **Web Platform**: WebAssembly bindings and web APIs
- **Cross-platform**: Random generation, time, input services

### **Presentation Layer** 🖥️
- **State Management**: Game state transitions and UI coordination
- **Input Handling**: Configurable key mapping and action processing
- **Rendering**: Visual effects and display coordination

## 🚀 Deployment

### GitHub Actions + Netlify (Recommended)

This project uses **tag-based deployment** with GitHub Actions building and Netlify serving:

#### How it Works
1. **Development**: Work on `main` branch (source code only)
2. **Release**: Create version tags to trigger deployment
3. **Build**: GitHub Actions builds WASM and commits to `deploy` branch
4. **Deploy**: Netlify automatically serves from `deploy` branch

#### Deploy a New Version
```bash
# Create and push a version tag
git tag v1.0.0
git push origin v1.0.0

# GitHub Actions will:
# 1. Build WASM with Rust caching (fast builds)
# 2. Commit built files to 'deploy' branch
# 3. Netlify auto-deploys from 'deploy' branch
```

#### Setup Steps
1. **Push code to GitHub** (includes `.github/workflows/deploy.yml`)
2. **Create `deploy` branch**: `git checkout -b deploy && git push origin deploy`
3. **Configure Netlify**:
   - Connect your repository
   - **Branch**: `deploy` (not main!)
   - **Build command**: (leave empty)
   - **Publish directory**: `.`
4. **Deploy**: Create your first tag!

#### Benefits
- ✅ **No build timeouts** (GitHub Actions: 6 hours vs Netlify: 18 minutes)
- ✅ **Fast builds** with Rust dependency caching
- ✅ **Controlled releases** - you choose when to deploy
- ✅ **Instant deploys** - Netlify just serves static files
- ✅ **Easy rollbacks** - switch to previous tag on deploy branch

### Alternative: Manual Deployment

#### Option 1: Direct Build & Deploy
1. Run `make web` to build
2. Go to [netlify.com](https://netlify.com)
3. Drag the `dist/` folder to deploy

#### Option 2: Netlify CLI
```bash
npm install -g netlify-cli
make web
cd dist
netlify deploy --prod
```

### Other Hosting Options
- **GitHub Pages**: Can use the same GitHub Actions workflow
- **Vercel**: Connect to `deploy` branch or upload `dist/` folder
- **Firebase Hosting**: Use GitHub Actions to deploy to Firebase
- **Any Static Host**: Upload contents from `deploy` branch



## 🖥️ Local Development

### Simple Commands

```bash
make setup    # Install all dependencies and tools
make dev      # Start development (choose native/web)
make build    # Build for production (native + web)
make test     # Run all tests (format + clippy + test)
make clean    # Clean all build files
```

That's it! Just 5 commands for everything.

### Development Workflows

#### 🚀 First Time Setup
```bash
make setup    # Install Rust, wasm-pack, cargo-watch
make dev      # Choose development mode
```

#### 🎮 Development Modes
When you run `make dev`, you choose:

**1. Native Development**
- Hot reload on file changes
- Fast rebuilds
- Native performance
- Good for game logic development

**2. Web Development**
- Auto-rebuild on file changes
- Test in browser
- Slower rebuilds
- Good for final testing

#### 🏗️ Production Build
```bash
make build    # Creates both native and web builds
```

### Local Testing Servers

Web development mode automatically starts a local server:
- Python 3 (preferred)
- Python 2 (fallback)
- Node.js (alternative)

Or manually:
```bash
cd dist
python serve.py    # After running 'make build'
```

## 🌐 Browser Compatibility

| Browser | Version | Status |
|---------|---------|--------|
| Chrome | All versions with WebAssembly | ✅ Fully Supported |
| Firefox | 52+ | ✅ Fully Supported |
| Safari | 11+ | ✅ Fully Supported |
| Edge | 16+ | ✅ Fully Supported |
| Mobile Chrome | Latest | ✅ Fully Supported |
| Mobile Safari | iOS 11+ | ✅ Fully Supported |

## ⚡ Performance Optimization

### Build Optimizations
```bash
# Install wasm-opt for smaller binaries
npm install -g wasm-opt

# Use optimized build profile
cargo build --profile wasm-release --target wasm32-unknown-unknown

# Enable compression on your web server
# Gzip/Brotli can reduce WASM file size by ~60%
```

### Current File Sizes
- **WASM Binary**: ~30MB (can be optimized to ~10-15MB)
- **JavaScript**: ~100KB
- **HTML**: ~12KB
- **Total**: ~30MB initial download

## 🔧 Technical Architecture

### Built With
- **🦀 Rust**: Memory-safe systems programming language
- **⚡ Bevy 0.16.1**: Modern ECS-based game engine
- **🌐 WebAssembly**: High-performance web execution
- **🎨 WebGL2**: Cross-browser graphics acceleration
- **🔗 wasm-bindgen**: Rust-JavaScript interop
- **🔄 cargo-watch**: File watching and hot reload for development

### Domain-Driven Design Benefits
- **🧪 Testability**: Pure domain logic easily unit tested
- **🔧 Maintainability**: Clear separation of concerns
- **📈 Scalability**: Easy to add new features and platforms
- **🌐 Cross-Platform**: Domain logic works everywhere

### Core Game Systems (Bevy ECS)
- **🎮 Player Input System**: Configurable key mapping and movement
- **👾 Enemy Spawning System**: Timed enemy generation with random positioning
- **⚡ Movement System**: Physics-based position updates with boundary clamping
- **💥 Collision System**: Entity interaction detection and scoring
- **🧹 Cleanup System**: Automatic off-screen entity removal
- **📊 UI Update System**: Real-time score and game state display

### Domain Entities & Components
- **👤 Player Entity**: Health, speed, position, movement capabilities
- **👾 Enemy Entity**: Type-based behavior, movement patterns
- **🎯 GameSession Entity**: Score tracking, timing, state management
- **📍 Position Value Object**: 2D coordinates with validation
- **⚡ Velocity Value Object**: Movement vectors with physics
- **🎯 Score Value Object**: Game scoring with business rules

### Web Platform Integration
- **🔒 Security Headers**: Proper CORS and content security policies
- **📱 Responsive Design**: Adapts to different screen sizes
- **⌨️ Input Handling**: Keyboard and touch event processing
- **🎨 Canvas Integration**: Hardware-accelerated WebGL2 rendering
- **📊 Performance Monitoring**: Frame rate and memory tracking

## 🐛 Troubleshooting

### Common Issues

**WASM file not loading:**
- Ensure your web server serves `.wasm` files with correct MIME type
- Check CORS headers if loading from different domain

**Game not responding:**
- Verify WebAssembly support in browser
- Check browser console for JavaScript errors
- Ensure WebGL2 is enabled

**Performance issues:**
- Try smaller window size
- Check if hardware acceleration is enabled
- Consider using optimized build profile

### Development Issues

**Build failures:**
```bash
make clean    # Clean everything
make setup    # Reinstall tools
make build    # Try building again
```

**Missing tools:**
```bash
make setup    # This installs everything you need
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test both native and web builds
5. Submit a pull request

### Development Guidelines
- Use `make dev` and choose your development mode
- Run `make test` before committing (includes formatting and linting)
- Use `make build` to create production builds
- Ensure cross-browser compatibility

## 📄 License

This project is open source and available under the MIT License.

## ✅ Current Implementation Status

### Completed Features
- ✅ **Complete DDD Architecture**: All layers implemented with clear separation
- ✅ **Domain Layer**: Entities, value objects, and domain services
- ✅ **Application Layer**: Use cases and application services
- ✅ **Infrastructure Layer**: Bevy integration, web platform, random generation
- ✅ **Presentation Layer**: State management, input handling, rendering
- ✅ **Web Build System**: Full WebAssembly compilation and deployment
- ✅ **Cross-Browser Support**: Tested on modern browsers
- ✅ **Game Mechanics**: Player movement, enemy spawning, collision detection
- ✅ **UI Systems**: Score display, game state management
- ✅ **Test Framework**: Unit tests across all layers
- ✅ **Documentation**: Comprehensive README and inline docs

### Architecture Implementation
- ✅ **Clean Architecture**: Pure domain logic with no infrastructure dependencies
- ✅ **SOLID Principles**: Single responsibility, dependency inversion
- ✅ **Error Handling**: Comprehensive error types across all layers
- ✅ **Type Safety**: Strong typing with validation at boundaries
- ✅ **Modularity**: Clear module boundaries and interfaces

## 🚀 Future Roadmap

### Game Features
- [ ] Sound effects and background music
- [ ] Particle effects for explosions and trails
- [ ] Power-ups and special abilities
- [ ] Multiple enemy types with different behaviors
- [ ] Level progression with increasing difficulty
- [ ] High score persistence (localStorage/backend)
- [ ] Mobile touch controls and responsive UI

### Development Experience
- [x] Hot reload for native development (`make run-watch`)
- [x] Auto-rebuild for web development (`make serve-watch`)
- [x] Interactive development menu (`make start`)
- [x] Environment status checking (`make status`)
- [ ] Live reload for web browser (auto-refresh on changes)
- [ ] Development dashboard with build status
- [ ] Enhanced error reporting and debugging tools

### Technical Enhancements
- [ ] Performance optimizations (WASM size reduction)
- [ ] Advanced visual effects and animations
- [ ] Multiplayer functionality with WebRTC
- [ ] Progressive Web App (PWA) features
- [ ] Analytics and telemetry integration
- [ ] CI/CD pipeline with automated testing
- [ ] Docker containerization for deployment

## 📞 Support

If you encounter issues:
1. Check the [Troubleshooting](#-troubleshooting) section
2. Review browser console for errors
3. Test on different browsers
4. Open an issue with detailed information

## 📂 Build Output

The build process generates files in the `dist/` directory:
- **This directory is gitignored** and created during each build
- Contains all files needed for web deployment
- Can be directly uploaded to any static hosting service
- Includes optimized WASM binaries and JavaScript bindings

### Build Artifacts
```
dist/
├── index.html           # Main game page (copied from web/index.html)
├── space_looter.js      # JavaScript bindings (~100KB)
├── space_looter_bg.wasm # Game binary (~30MB, optimizable to ~10MB)
├── serve.py            # Local Python server
└── serve.js            # Local Node.js server
```

### Git Management
- **Source code**: `main` branch (development)
- **Built files**: `deploy` branch (auto-generated by GitHub Actions)
- **Deployment**: Netlify serves from `deploy` branch
- **Tags**: Create version tags (`v1.0.0`) to trigger deployments

#### Branch Structure
```
main branch (source code)
├── src/
├── web/
├── Cargo.toml
├── .github/workflows/
└── README.md

deploy branch (built files, auto-generated)
├── index.html
├── space_looter.js
├── space_looter_bg.wasm
└── DEPLOYMENT.md
```

## 🏆 Key Achievements

This project demonstrates:
- **🏗️ Clean Architecture**: Proper DDD implementation in Rust
- **🌐 Web Technology**: Modern WebAssembly game development
- **⚡ Performance**: Efficient ECS-based game engine usage
- **🧪 Quality**: Comprehensive testing and documentation
- **📦 Deployment**: Complete build and deployment pipeline
- **🔄 Developer Experience**: Hot reload, auto-rebuild, and comprehensive tooling

---

**🎮 Ready to Play**: The game is fully functional and deployable!
**🔧 Ready to Extend**: Clean architecture makes adding features straightforward!
**📚 Ready to Learn**: Comprehensive documentation and examples included!

Built with ❤️ using Rust, Bevy, and Domain-Driven Design principles.
Deploy anywhere, play everywhere! 🌐🎮
