//! Space Looter - 3D Isometric Dice RPG
//!
//! A 3D isometric RPG with dice-based mechanics, exploration, base building,
//! and procedural content generation. Built with Domain-Driven Design principles
//! and clean architecture.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// Module declarations following DDD architecture
pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presentation;

use bevy::prelude::*;

/// Creates and configures the main RPG application
pub fn create_app() -> App {
    let mut app = App::new();

    // Configure Bevy plugins with web-optimized settings
    #[cfg(target_arch = "wasm32")]
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Space Looter - 3D Isometric RPG".into(),
            canvas: Some("#bevy".into()),
            fit_canvas_to_parent: true,
            prevent_default_event_handling: false,
            ..default()
        }),
        ..default()
    }));

    // Configure for native with RPG-appropriate resolution
    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Space Looter - 3D Isometric RPG".into(),
            resolution: (1200.0, 800.0).into(), // Wider for RPG UI
            ..default()
        }),
        ..default()
    }));

    // Initialize RPG state management
    app.init_state::<presentation::RpgAppState>();

    // Set RPG-appropriate background color (dark space theme)
    app.insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)));

    // Add core RPG systems
    app.add_plugins((
        presentation::game_state::RpgStatePlugin,
        presentation::game_ui::GameUIPlugin,
        presentation::map_renderer::MapRendererPlugin,
    ));

    // Add RPG-specific resources
    app.insert_resource(infrastructure::bevy::resources::PlayerResource::new())
        .insert_resource(infrastructure::bevy::resources::BaseResource::new())
        .insert_resource(infrastructure::bevy::resources::MapResource::new())
        .insert_resource(infrastructure::bevy::resources::GameStatsResource::new())
        .insert_resource(infrastructure::bevy::resources::GameTimerResource::new());

    // Initialize empty RpgGameSession - will be populated when game starts
    let dummy_player = domain::Player::create_new_character(
        "Demo Player".to_string(),
        domain::Position3D::origin(),
    )
    .unwrap();
    let dummy_base = domain::Base::new(
        domain::EntityId::generate(),
        "Demo Base".to_string(),
        domain::Position3D::origin(),
    )
    .unwrap();
    let rpg_session = presentation::game_state::RpgGameSession::new(dummy_player, dummy_base);
    app.insert_resource(rpg_session);

    // Add startup systems for RPG initialization
    app.add_systems(
        Startup,
        (setup_rpg_camera_system, initialize_rpg_world_system),
    );

    // Add core RPG update systems
    app.add_systems(
        Update,
        (
            // Core gameplay systems
            rpg_turn_management_system,
            rpg_exploration_system,
            rpg_dice_mechanics_system,
            // UI and presentation systems
            handle_window_resize_system,
            rpg_state_transition_system,
        ),
    );

    // Add RPG-specific system sets for better organization
    app.configure_sets(
        Update,
        (
            RpgSystemSet::Input,
            RpgSystemSet::Logic,
            RpgSystemSet::Dice,
            RpgSystemSet::UI,
        )
            .chain(),
    );

    info!("Space Looter RPG initialized successfully");
    app
}

/// RPG system organization sets
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum RpgSystemSet {
    /// Input processing for RPG controls
    Input,
    /// Core RPG game logic
    Logic,
    /// Dice mechanics and random events
    Dice,
    /// UI updates and rendering
    UI,
}

/// Setup cameras for RPG (2D tile view)
fn setup_rpg_camera_system(_commands: Commands) {
    info!("Setting up 2D tile-based RPG camera");

    // Note: Camera will be created by the game UI plugin
    // This system is kept for initialization logging

    info!("🎲 Controls: WASD/Arrows=Move, SPACE=Roll Dice, B=Base, Q=Quests, I=Inventory");
}

/// Initialize the RPG world with starting state
fn initialize_rpg_world_system(
    mut player_resource: ResMut<infrastructure::bevy::resources::PlayerResource>,
    mut base_resource: ResMut<infrastructure::bevy::resources::BaseResource>,
    mut map_resource: ResMut<infrastructure::bevy::resources::MapResource>,
    mut game_stats: ResMut<infrastructure::bevy::resources::GameStatsResource>,
) {
    info!("Initializing RPG world state");

    // Create starting player
    let starting_position = domain::Position3D::origin();
    let starting_stats = domain::PlayerStats::new(12, 10, 8, 14, 11, 9)
        .expect("Failed to create starting player stats");

    if let Err(e) = player_resource.create_player(
        "player_001".to_string(),
        "Space Looter".to_string(),
        starting_position,
        starting_stats,
    ) {
        error!("Failed to create starting player: {}", e);
    }

    // Create starting base
    let base_position = domain::Position3D::new(0, 0, 0);
    if let Err(e) = base_resource.create_base("Central Command".to_string(), base_position) {
        error!("Failed to create starting base: {}", e);
    }

    // Initialize game statistics
    game_stats.reset();

    // Force initial map generation around starting position
    if let Some(player_position) = player_resource.player_position() {
        let _initial_map = map_resource.get_or_create_map(player_position);
        info!(
            "🗺️ Initial map generated for position: {:?}",
            player_position
        );
    }

    info!("RPG world initialization complete");
}

/// Core RPG turn management system
fn rpg_turn_management_system(
    mut game_timer: ResMut<infrastructure::bevy::resources::GameTimerResource>,
    time: Res<Time>,
) {
    game_timer.update(time.delta_secs());
}

/// RPG tile-based exploration with dice roll events
fn rpg_exploration_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_resource: ResMut<infrastructure::bevy::resources::PlayerResource>,
    mut map_resource: ResMut<infrastructure::bevy::resources::MapResource>,
    mut game_stats: ResMut<infrastructure::bevy::resources::GameStatsResource>,
) {
    if !player_resource.has_player() {
        return;
    }

    // Create tile movement service
    let tile_movement_service = domain::services::TileMovementService::new();

    // Handle tile-based movement with dice rolls
    let mut movement_attempted = false;
    let current_position = player_resource.player_position().unwrap_or_default();
    let mut target_position = current_position;

    if keyboard_input.just_pressed(KeyCode::ArrowUp) || keyboard_input.just_pressed(KeyCode::KeyW) {
        target_position.y += 1;
        movement_attempted = true;
    } else if keyboard_input.just_pressed(KeyCode::ArrowDown)
        || keyboard_input.just_pressed(KeyCode::KeyS)
    {
        target_position.y -= 1;
        movement_attempted = true;
    } else if keyboard_input.just_pressed(KeyCode::ArrowLeft)
        || keyboard_input.just_pressed(KeyCode::KeyA)
    {
        target_position.x -= 1;
        movement_attempted = true;
    } else if keyboard_input.just_pressed(KeyCode::ArrowRight)
        || keyboard_input.just_pressed(KeyCode::KeyD)
    {
        target_position.x += 1;
        movement_attempted = true;
    }

    if movement_attempted {
        // Get player for dice calculations
        if let Some(player) = player_resource.get_player() {
            let player_level = player.level();

            // Get or generate map around player position
            let map = map_resource.get_or_create_map(current_position);

            // Attempt tile movement with dice roll
            match tile_movement_service.attempt_movement(
                &player,
                target_position,
                &map,
                player_level,
            ) {
                Ok(movement_result) => {
                    // Log dice roll result
                    info!("🎲 {}", movement_result.dice_result.description());
                    info!(
                        "📍 Outcome: {}",
                        movement_result.dice_result.outcome_category()
                    );

                    // Execute the movement
                    if let Ok(()) =
                        player_resource.move_player(target_position, movement_result.movement_cost)
                    {
                        game_stats.record_tile_explored();
                        info!("✅ Player moved to position: {:?}", target_position);

                        // Handle triggered event
                        if let Some(event) = movement_result.triggered_event {
                            info!(
                                "🎭 Event Triggered: {} - {}",
                                event.title(),
                                event.description()
                            );

                            // Process event outcomes based on type and dice result
                            process_movement_event(
                                &event,
                                &movement_result.dice_result,
                                &mut player_resource,
                                &mut game_stats,
                            );
                        } else {
                            info!("🚶 Safe movement - no events triggered");
                        }
                    }
                }
                Err(e) => {
                    warn!("❌ Movement failed: {}", e);
                    match e {
                        domain::DomainError::InvalidMapCoordinates(..) => {
                            info!("🚫 Can only move to adjacent tiles!");
                        }
                        domain::DomainError::TileNotAccessible(..) => {
                            info!("🚫 That tile is not passable!");
                        }
                        domain::DomainError::InsufficientResources(_) => {
                            info!("⚡ Not enough movement points!");
                        }
                        _ => {
                            info!("🚫 Movement not possible right now");
                        }
                    }
                }
            }
        }
    }
}

/// Process events triggered by tile movement
fn process_movement_event(
    event: &domain::entities::Event,
    dice_result: &domain::services::tile_movement::MovementDiceResult,
    player_resource: &mut infrastructure::bevy::resources::PlayerResource,
    game_stats: &mut infrastructure::bevy::resources::GameStatsResource,
) {
    use domain::entities::EventType;
    use domain::value_objects::resources::ResourceCollection;
    use domain::value_objects::ResourceType;

    let final_roll = dice_result.final_result;

    match event.event_type() {
        EventType::ResourceDiscovery => {
            let mut resources = ResourceCollection::new();
            let amount = match final_roll {
                20..=u8::MAX => 50, // Critical success - lots of resources
                17..=19 => 30,      // Great success
                13..=16 => 15,      // Success
                _ => 5,             // Minimal find
            };

            resources.set_amount(ResourceType::Metal, amount);
            if let Some(player) = player_resource.get_player_mut() {
                player.add_resources(&resources);
                info!("💰 Found {} metal!", amount);
                game_stats.record_experience_gain(amount as u32);
            }
        }

        EventType::Combat => {
            let damage = match final_roll {
                1..=3 => 20, // Critical failure - major damage
                4..=7 => 10, // Failure - moderate damage
                8..=12 => 5, // Neutral - minor damage
                _ => 0,      // Success+ - avoided or won
            };

            if damage > 0 {
                info!("⚔️ Combat! Took {} damage", damage);
                // TODO: Implement actual damage system
            } else {
                info!("⚔️ Combat encounter successfully resolved!");
                game_stats.record_experience_gain(20);
            }
        }

        EventType::Hazard => {
            let penalty = match final_roll {
                1..=3 => 15, // Critical failure - major hazard
                4..=7 => 8,  // Failure - moderate hazard
                8..=12 => 3, // Neutral - minor hazard
                _ => 0,      // Success+ - avoided hazard
            };

            if penalty > 0 {
                info!(
                    "⚠️ Environmental hazard! Lost {} movement points worth of energy",
                    penalty
                );
                // TODO: Implement actual penalty system
            } else {
                info!("⚠️ Successfully navigated environmental hazard!");
                game_stats.record_experience_gain(10);
            }
        }

        EventType::Trade => {
            if final_roll >= 13 {
                let mut resources = ResourceCollection::new();
                resources.set_amount(ResourceType::Data, 20);
                if let Some(player) = player_resource.get_player_mut() {
                    player.add_resources(&resources);
                    info!("🤝 Successful trade! Gained 20 data");
                    game_stats.record_experience_gain(15);
                }
            } else {
                info!("🤝 Trade opportunity, but couldn't reach an agreement");
            }
        }

        EventType::Boon => {
            let xp_gain = match final_roll {
                20..=u8::MAX => 100, // Critical success
                17..=19 => 60,       // Great success
                13..=16 => 30,       // Success
                _ => 10,             // Minor benefit
            };

            game_stats.record_experience_gain(xp_gain);
            info!("✨ Fortune smiles upon you! Gained {} experience", xp_gain);
        }

        EventType::Mystery => {
            if final_roll >= 15 {
                game_stats.record_experience_gain(40);
                info!("🔮 Mysterious phenomenon understood! Gained knowledge");
            } else {
                info!("🔮 A mysterious phenomenon occurs, but its meaning eludes you");
            }
        }

        EventType::Malfunction => {
            if final_roll <= 7 {
                info!("🔧 Equipment malfunction! Efficiency reduced");
                // TODO: Implement equipment durability system
            } else {
                info!("🔧 Equipment issue detected but quickly resolved");
                game_stats.record_experience_gain(5);
            }
        }

        EventType::Narrative => {
            game_stats.record_experience_gain(5);
            info!("📖 {}", event.description());
        }

        EventType::BaseEvent => {
            info!("🏠 Base-related event: {}", event.description());
            game_stats.record_experience_gain(10);
        }
    }
}

/// RPG dice mechanics and random events system
fn rpg_dice_mechanics_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut game_stats: ResMut<infrastructure::bevy::resources::GameStatsResource>,
) {
    // Roll dice on space bar press
    if keyboard_input.just_pressed(KeyCode::Space) {
        // Create a basic dice roll for demonstration
        if let Ok(dice_roll) = domain::DiceRoll::new(
            1,
            domain::DiceType::D20,
            domain::value_objects::dice::DiceModifier::none(),
        ) {
            game_stats.record_dice_roll(&dice_roll, 10);
            let total = dice_roll.total();

            match total {
                18..=20 => {
                    info!("🎲 Critical Success! ({})", total);
                    game_stats.record_experience_gain(50);
                }
                15..=17 => {
                    info!("🎲 Great Success! ({})", total);
                    game_stats.record_experience_gain(25);
                }
                10..=14 => {
                    info!("🎲 Success! ({})", total);
                    game_stats.record_experience_gain(10);
                }
                6..=9 => {
                    info!("🎲 Partial Success ({})", total);
                    game_stats.record_experience_gain(5);
                }
                _ => {
                    info!("🎲 Failed Roll ({})", total);
                }
            }
        }
    }
}

/// Handle RPG state transitions
fn rpg_state_transition_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<presentation::RpgAppState>>,
    mut next_state: ResMut<NextState<presentation::RpgAppState>>,
) {
    match current_state.get() {
        presentation::RpgAppState::Loading => {
            // Auto-transition to main menu after initialization
            next_state.set(presentation::RpgAppState::MainMenu);
        }
        presentation::RpgAppState::MainMenu => {
            if keyboard_input.just_pressed(KeyCode::Enter) {
                next_state.set(presentation::RpgAppState::Exploration);
                info!("🚀 Starting RPG exploration mode!");
            }
        }
        presentation::RpgAppState::Exploration => {
            if keyboard_input.just_pressed(KeyCode::KeyB) {
                next_state.set(presentation::RpgAppState::BaseManagement);
                info!("Entering base management mode");
            } else if keyboard_input.just_pressed(KeyCode::KeyQ) {
                next_state.set(presentation::RpgAppState::QuestLog);
                info!("Opening quest log");
            } else if keyboard_input.just_pressed(KeyCode::KeyI) {
                next_state.set(presentation::RpgAppState::Inventory);
                info!("Opening inventory");
            } else if keyboard_input.just_pressed(KeyCode::Escape) {
                next_state.set(presentation::RpgAppState::Paused);
            }
        }
        presentation::RpgAppState::BaseManagement
        | presentation::RpgAppState::QuestLog
        | presentation::RpgAppState::Inventory => {
            if keyboard_input.just_pressed(KeyCode::Escape) {
                next_state.set(presentation::RpgAppState::Exploration);
                info!("Returning to exploration");
            }
        }
        presentation::RpgAppState::Paused => {
            if keyboard_input.just_pressed(KeyCode::Escape) {
                next_state.set(presentation::RpgAppState::Exploration);
                info!("Resuming game");
            }
        }
        _ => {}
    }
}

/// System to handle window resize events for better responsive RPG UI
fn handle_window_resize_system() {
    // Window resize handling will be added later - focus on core mechanics first
}

/// WebAssembly entry point for RPG
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn main() {
    // Set up panic hook for better error messages in browser console
    #[cfg(feature = "web")]
    console_error_panic_hook::set_once();

    // Initialize logging for web
    #[cfg(feature = "web")]
    wasm_logger::init(wasm_logger::Config::default());

    info!("🎮 Starting Space Looter 3D Isometric RPG (WASM)");
    info!("🎲 Use WASD/Arrow keys to explore, SPACE to roll dice");
    info!("📋 Press B for base, Q for quests, I for inventory");

    // Create and run the RPG app
    let mut app = create_app();
    app.run();
}

/// Native entry point for RPG (when used as a library)
#[cfg(not(target_arch = "wasm32"))]
pub fn run() {
    info!("🎮 Starting Space Looter 3D Isometric RPG (Native)");
    info!("🎲 Use WASD/Arrow keys to explore, SPACE to roll dice");
    info!("📋 Press B for base, Q for quests, I for inventory");
    info!("🚀 Press ENTER in menu to start exploring!");

    let mut app = create_app();
    app.run();
}
