# Space Looter - Domain-Driven Design Architecture

This document outlines the Domain-Driven Design (DDD) architecture for Space Looter, a 2D space shooter game built with Bevy engine.

## 🏗️ Architecture Overview

The codebase follows DDD principles with clear separation between domain logic, application services, infrastructure, and presentation layers.

```
src/
├── README.md           # This file - Architecture documentation
├── main.rs             # Native application entry point
├── lib.rs              # WASM library entry point & app configuration
├── domain/             # Core business logic (entities, value objects)
│   ├── mod.rs          # Domain module exports
│   ├── entities/       # Game entities with business rules
│   │   ├── mod.rs      # Entity module exports
│   │   ├── player.rs   # Player entity with movement rules
│   │   ├── enemy.rs    # Enemy entity with behavior rules
│   │   └── game.rs     # Game session entity
│   ├── value_objects/  # Immutable value types
│   │   ├── mod.rs      # Value object exports
│   │   ├── position.rs # 2D position coordinates
│   │   ├── velocity.rs # Movement velocity vector
│   │   └── score.rs    # Game scoring value object
│   └── services/       # Domain services for complex business logic
│       ├── mod.rs      # Domain service exports
│       ├── collision.rs # Collision detection logic
│       └── spawning.rs # Enemy spawning rules
├── application/        # Use cases and application services
│   ├── mod.rs          # Application module exports
│   ├── use_cases/      # Game use cases
│   │   ├── mod.rs      # Use case exports
│   │   ├── move_player.rs    # Player movement use case
│   │   ├── spawn_enemies.rs  # Enemy spawning use case
│   │   ├── handle_collision.rs # Collision handling use case
│   │   └── update_score.rs   # Score updating use case
│   └── services/       # Application services
│       ├── mod.rs      # Application service exports
│       ├── game_session.rs # Game session management
│       └── input_handler.rs # Input processing service
├── infrastructure/     # External concerns (I/O, frameworks)
│   ├── mod.rs          # Infrastructure module exports
│   ├── bevy/          # Bevy engine integration
│   │   ├── mod.rs      # Bevy integration exports
│   │   ├── components.rs # Bevy ECS components
│   │   ├── resources.rs  # Bevy ECS resources
│   │   └── systems.rs    # Bevy ECS systems
│   ├── random/         # Random number generation
│   │   ├── mod.rs      # Random module exports
│   │   └── generator.rs # Random number implementation
│   └── web/           # Web-specific infrastructure
│       ├── mod.rs      # Web module exports
│       └── bindings.rs # WASM bindings and web integration
└── presentation/      # User interface and input handling
    ├── mod.rs          # Presentation module exports
    ├── game_state.rs   # Game state management
    ├── input.rs        # Input mapping and processing
    └── rendering.rs    # Rendering coordination
```

## 🎯 Domain Layer

### Core Principles
- **Pure Business Logic**: No dependencies on frameworks or external libraries
- **Rich Domain Model**: Entities encapsulate business rules and behavior
- **Immutable Value Objects**: Data structures that represent concepts
- **Domain Services**: Complex business operations that don't belong to entities

### Entities
```rust
// Player entity with movement capabilities
pub struct Player {
    id: PlayerId,
    position: Position,
    velocity: Velocity,
    speed: f32,
}

// Enemy entity with autonomous behavior
pub struct Enemy {
    id: EnemyId,
    position: Position,
    velocity: Velocity,
    enemy_type: EnemyType,
}

// Game session managing overall state
pub struct GameSession {
    id: GameSessionId,
    score: Score,
    state: GameState,
    start_time: Instant,
}
```

### Value Objects
```rust
// Immutable position coordinates
#[derive(Clone, Copy, PartialEq)]
pub struct Position {
    x: f32,
    y: f32,
}

// Movement velocity vector
#[derive(Clone, Copy)]
pub struct Velocity {
    dx: f32,
    dy: f32,
}

// Game score with validation
pub struct Score {
    value: u32,
}
```

## 🔧 Application Layer

### Use Cases
Each use case represents a specific game operation:

```rust
// Player movement use case
pub struct MovePlayerUseCase {
    collision_service: Arc<dyn CollisionService>,
}

impl MovePlayerUseCase {
    pub fn execute(&self, input: MovePlayerInput) -> Result<MovePlayerOutput, GameError> {
        // 1. Validate input
        // 2. Apply business rules
        // 3. Update player position
        // 4. Check boundaries
        // 5. Return result
    }
}
```

### Application Services
Coordinate between use cases and manage application state:

```rust
pub struct GameSessionService {
    session: GameSession,
    player_use_case: MovePlayerUseCase,
    enemy_use_case: SpawnEnemiesUseCase,
}
```

## 🏗️ Infrastructure Layer

### Bevy Integration
Maps domain concepts to Bevy ECS:

```rust
// Bevy components wrapping domain entities
#[derive(Component)]
pub struct PlayerComponent(pub Player);

#[derive(Component)]
pub struct EnemyComponent(pub Enemy);

// Bevy systems implementing use cases
pub fn player_movement_system(
    input: Res<ButtonInput<KeyCode>>,
    mut players: Query<&mut PlayerComponent>,
    move_player_use_case: Res<MovePlayerUseCase>,
) {
    // Execute use case through infrastructure
}
```

### External Services
```rust
pub trait RandomNumberGenerator {
    fn random_f32(&self) -> f32;
    fn random_range(&self, min: f32, max: f32) -> f32;
}

pub struct WebRandomGenerator;
impl RandomNumberGenerator for WebRandomGenerator {
    // Web-specific implementation
}
```

## 📱 Presentation Layer

### Game State Management
```rust
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Loading,
    Menu,
    Playing,
    Paused,
    GameOver,
}
```

### Input Handling
```rust
pub struct InputMapper {
    key_bindings: HashMap<KeyCode, GameAction>,
}

pub enum GameAction {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Pause,
}
```

## 🔄 Data Flow

1. **Input** → Presentation Layer captures user input
2. **Translation** → Input mapper converts to game actions
3. **Use Case** → Application layer executes business logic
4. **Domain** → Entities and services apply business rules
5. **Infrastructure** → Bevy systems update ECS components
6. **Rendering** → Presentation layer updates display

## 📋 Implementation Guidelines

### Domain Layer Rules
- ❌ No `use bevy::*` imports
- ❌ No infrastructure dependencies
- ✅ Pure Rust with standard library only
- ✅ Rich business logic and validation
- ✅ Comprehensive error handling

### Application Layer Rules
- ❌ No direct Bevy component access
- ✅ Coordinate between domain and infrastructure
- ✅ Handle use case orchestration
- ✅ Manage application state transitions

### Infrastructure Layer Rules
- ✅ Bevy integration allowed here
- ✅ External library dependencies
- ✅ Platform-specific implementations
- ✅ Adapter pattern for external services

### Testing Strategy
```rust
// Domain layer - Unit tests with pure logic
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_moves_within_boundaries() {
        let mut player = Player::new(/* ... */);
        let result = player.move_to(Position::new(100.0, 50.0));
        assert!(result.is_ok());
    }
}

// Application layer - Integration tests with mocks
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[test]
    fn move_player_use_case_updates_position() {
        // Test use case with mocked dependencies
    }
}

// Infrastructure layer - Bevy system tests
#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    #[test]
    fn player_movement_system_responds_to_input() {
        // Test Bevy integration
    }
}
```

## 🚀 Benefits of This Architecture

### 🎯 **Testability**
- Domain logic easily unit tested
- Clear dependency injection points
- Mockable external services

### 🔧 **Maintainability**  
- Clear separation of concerns
- Easy to modify business rules
- Framework-agnostic core logic

### 📈 **Scalability**
- Easy to add new game features
- Pluggable infrastructure components
- Clear extension points

### 🌐 **Cross-Platform**
- Domain logic works everywhere
- Platform-specific code isolated
- Easy to add new platforms

## 📝 Next Steps

1. **Refactor Current Code** into DDD layers
2. **Implement Domain Entities** with business rules
3. **Create Use Cases** for each game operation
4. **Build Infrastructure Adapters** for Bevy integration
5. **Add Comprehensive Tests** for each layer
6. **Document Domain Model** with business rules

## 🤝 Contributing

When adding new features:

1. **Start with Domain**: Define entities and business rules
2. **Create Use Cases**: Define application operations
3. **Build Infrastructure**: Implement Bevy integration
4. **Update Presentation**: Add UI/input handling
5. **Write Tests**: Cover all layers
6. **Update Documentation**: Keep this README current

---

This architecture ensures Space Looter remains maintainable, testable, and extensible as it grows in complexity.