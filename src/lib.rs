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
use bevy::time::{Timer, TimerMode};

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
        infrastructure::bevy::font_service::FontPlugin,
        presentation::game_state::RpgStatePlugin,
        presentation::game_ui::GameUIPlugin,
        presentation::map_renderer::MapRendererPlugin,
        presentation::rendering::RenderingPlugin,
    ));

    // Add RPG-specific resources
    app.insert_resource(infrastructure::bevy::resources::PlayerResource::new())
        .insert_resource(infrastructure::bevy::resources::BaseResource::new())
        .insert_resource(infrastructure::bevy::resources::MapResource::new())
        .insert_resource(infrastructure::bevy::resources::GameStatsResource::new())
        .insert_resource(infrastructure::bevy::resources::GameTimerResource::new());

    // Add domain services as resources
    app.insert_resource(domain::services::TileMovementService::new())
        .insert_resource(domain::services::RestingService::new());

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
    movement_service: Res<domain::services::TileMovementService>,
    resting_service: Res<domain::services::RestingService>,
    mut pending_movement: Local<Option<domain::Position3D>>,
    mut rest_timer: Local<Option<Timer>>,
    time: Res<Time>,
) {
    if !player_resource.has_player() {
        return;
    }

    // Check if we're currently resting
    if let Some(ref mut timer) = *rest_timer {
        timer.tick(time.delta());
        if timer.just_finished() {
            info!("😴 Rest period complete, you can now move again");
            *rest_timer = None;
        } else {
            // Still resting, block all movement
            return;
        }
    }

    // Check if we need to retry a pending movement after resting
    let mut movement_attempted = false;
    let current_position = player_resource.player_position().unwrap_or_default();
    let mut target_position = current_position;

    // Handle pending movement first (after resting)
    if let Some(pending_pos) = pending_movement.take() {
        target_position = pending_pos;
        movement_attempted = true;
        info!(
            "🌅 Attempting previously failed movement after rest to {:?}",
            target_position
        );
    } else {
        // Handle new movement input

        if keyboard_input.just_pressed(KeyCode::ArrowUp)
            || keyboard_input.just_pressed(KeyCode::KeyW)
        {
            target_position.y -= 1; // Fixed: up key now moves up on screen (towards camera)
            movement_attempted = true;
        } else if keyboard_input.just_pressed(KeyCode::ArrowDown)
            || keyboard_input.just_pressed(KeyCode::KeyS)
        {
            target_position.y += 1; // Fixed: down key now moves down on screen (away from camera)
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
    }

    if movement_attempted {
        // Get player for dice calculations
        if let Some(player) = player_resource.get_player() {
            let player_level = player.level();

            // Get or generate map around player position
            let map = map_resource.get_or_create_map(current_position);

            // Attempt tile movement with dice roll
            match movement_service.attempt_movement(&player, target_position, &map, player_level) {
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

                            // Give small movement point recovery even for safe movement
                            // to prevent players from getting completely stuck
                            if let Some(player) = player_resource.get_player_mut() {
                                player.add_movement_points(2);
                                info!("🏃 Safe exploration grants 2 movement points");
                            }
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

                            // Check if player can't make any moves - trigger rest
                            if let Some(player) = player_resource.get_player() {
                                let current_pos = player.position();
                                let map = map_resource.get_or_create_map(*current_pos);

                                // Check if player can move to any adjacent tile
                                let adjacent_positions = [
                                    domain::Position3D::new(
                                        current_pos.x + 1,
                                        current_pos.y,
                                        current_pos.z,
                                    ),
                                    domain::Position3D::new(
                                        current_pos.x - 1,
                                        current_pos.y,
                                        current_pos.z,
                                    ),
                                    domain::Position3D::new(
                                        current_pos.x,
                                        current_pos.y + 1,
                                        current_pos.z,
                                    ),
                                    domain::Position3D::new(
                                        current_pos.x,
                                        current_pos.y - 1,
                                        current_pos.z,
                                    ),
                                ];

                                let can_move_anywhere = adjacent_positions.iter().any(|pos| {
                                    let movement_cost = map.movement_cost(pos);
                                    player.movement_points() >= movement_cost
                                });

                                if !can_move_anywhere {
                                    info!("🌙 You are exhausted and must rest for the night...");

                                    // Store the failed movement to retry after rest
                                    *pending_movement = Some(target_position);

                                    // Process rest cycle
                                    if let Some(player_mut) = player_resource.get_player_mut() {
                                        let current_pos = *player_mut.position();
                                        match resting_service
                                            .process_rest_cycle(player_mut, current_pos)
                                        {
                                            Ok(rest_result) => {
                                                info!("🌅 Dawn breaks after a night of rest");
                                                info!(
                                                    "🎲 Night Roll: {} - {}",
                                                    rest_result.dice_roll, rest_result.night_event
                                                );
                                                info!(
                                                    "😴 Rest Quality: {}",
                                                    rest_result.rest_outcome
                                                );
                                                info!("📖 {}", rest_result.description);

                                                if !rest_result.resources_gained.is_empty() {
                                                    info!(
                                                        "💰 Resources gained during rest: {}",
                                                        format_resource_summary(
                                                            &rest_result.resources_gained
                                                        )
                                                    );
                                                }

                                                // Set rest duration based on rest quality
                                                let rest_duration = match rest_result.rest_outcome {
                                                    domain::services::resting_service::RestOutcome::PoorRest => std::time::Duration::from_secs(6),      // Poor rest = longer time
                                                    domain::services::resting_service::RestOutcome::NormalRest => std::time::Duration::from_secs(4),   // Normal rest
                                                    domain::services::resting_service::RestOutcome::GoodRest => std::time::Duration::from_secs(3),     // Good rest = faster
                                                    domain::services::resting_service::RestOutcome::GreatRest => std::time::Duration::from_secs(2),    // Great rest = much faster
                                                    domain::services::resting_service::RestOutcome::ExceptionalRest => std::time::Duration::from_secs(1), // Exceptional = almost instant
                                                };

                                                *rest_timer = Some(Timer::new(
                                                    rest_duration,
                                                    TimerMode::Once,
                                                ));

                                                info!(
                                                    "🏃 Movement points restored: {} - resting for {} seconds...",
                                                    rest_result.movement_points_restored,
                                                    rest_duration.as_secs()
                                                );
                                                info!("💤 {} You feel drowsy and must rest before moving again",
                                                    match rest_result.rest_outcome {
                                                        domain::services::resting_service::RestOutcome::PoorRest => "😫",
                                                        domain::services::resting_service::RestOutcome::NormalRest => "😴",
                                                        domain::services::resting_service::RestOutcome::GoodRest => "😊",
                                                        domain::services::resting_service::RestOutcome::GreatRest => "😌",
                                                        domain::services::resting_service::RestOutcome::ExceptionalRest => "✨",
                                                    }
                                                );
                                                game_stats.record_experience_gain(10);
                                                // Base XP for surviving the night
                                            }
                                            Err(e) => {
                                                warn!("❌ Rest cycle failed: {}", e);
                                                // Fallback: just restore movement points with normal rest time
                                                player_mut.restore_points();
                                                *rest_timer = Some(Timer::new(
                                                    std::time::Duration::from_secs(4),
                                                    TimerMode::Once,
                                                ));
                                                info!(
                                                    "🏃 Emergency rest - movement points restored, resting for 4 seconds..."
                                                );
                                            }
                                        }
                                    }
                                }
                            }
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

/// Format resource collection for display
fn format_resource_summary(
    resources: &domain::value_objects::resources::ResourceCollection,
) -> String {
    let mut parts = Vec::new();

    for resource_amount in resources.amounts() {
        if resource_amount.amount > 0 {
            parts.push(format!(
                "{}: {}",
                resource_amount.resource_type, resource_amount.amount
            ));
        }
    }

    if parts.is_empty() {
        "None".to_string()
    } else {
        parts.join(", ")
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

    // Calculate movement point rewards based on dice roll success
    let movement_reward = match final_roll {
        20..=u8::MAX => 7, // Critical success - major reward
        17..=19 => 5,      // Great success - good reward
        13..=16 => 4,      // Success - moderate reward
        10..=12 => 3,      // Neutral - small reward
        7..=9 => 2,        // Mild failure - minimal reward
        4..=6 => 1,        // Failure - tiny reward
        _ => 0,            // Critical failure - no reward
    };

    // Apply movement point rewards for successful outcomes
    if movement_reward > 0 {
        if let Some(player) = player_resource.get_player_mut() {
            player.add_movement_points(movement_reward);
            info!(
                "🏃 Gained {} movement points from successful exploration!",
                movement_reward
            );
        }
    }

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
            let (damage, movement_bonus) = match final_roll {
                20..=u8::MAX => (0, 3), // Critical success - no damage, extra movement
                17..=19 => (0, 2),      // Great success - no damage, bonus movement
                13..=16 => (0, 1),      // Success - no damage, small bonus
                8..=12 => (5, 0),       // Neutral - minor damage
                4..=7 => (10, 0),       // Failure - moderate damage
                _ => (20, 0),           // Critical failure - major damage
            };

            if damage > 0 {
                info!("⚔️ Combat! Took {} damage", damage);
                // TODO: Implement actual damage system
            } else {
                info!("⚔️ Combat encounter successfully resolved!");
                if movement_bonus > 0 {
                    if let Some(player) = player_resource.get_player_mut() {
                        player.add_movement_points(movement_bonus);
                        info!(
                            "🏃 Combat victory! Gained {} extra movement points!",
                            movement_bonus
                        );
                    }
                }
                game_stats.record_experience_gain(20);
            }
        }

        EventType::Hazard => {
            let penalty = match final_roll {
                1..=3 => 2,  // Critical failure - lose movement points
                4..=7 => 1,  // Failure - lose movement point
                8..=12 => 0, // Neutral - no penalty
                _ => 0,      // Success+ - no penalty (already got reward above)
            };

            if penalty > 0 {
                if let Some(player) = player_resource.get_player_mut() {
                    // Safely subtract movement points (won't go below 0)
                    player.subtract_movement_points(penalty);
                    info!("⚠️ Environmental hazard! Lost {} movement points!", penalty);
                }
            } else if final_roll >= 13 {
                info!("⚠️ Successfully navigated environmental hazard!");
                game_stats.record_experience_gain(10);
            }
        }

        EventType::Trade => {
            if final_roll >= 13 {
                let mut resources = ResourceCollection::new();
                let data_amount = match final_roll {
                    20..=u8::MAX => 40, // Critical success
                    17..=19 => 30,      // Great success
                    _ => 20,            // Success
                };

                resources.set_amount(ResourceType::Data, data_amount);
                if let Some(player) = player_resource.get_player_mut() {
                    player.add_resources(&resources);
                    info!("🤝 Successful trade! Gained {} data", data_amount);
                    game_stats.record_experience_gain(15);
                }
            } else {
                info!("🤝 Trade opportunity, but couldn't reach an agreement");
            }
        }

        EventType::Boon => {
            let (xp_gain, extra_movement) = match final_roll {
                20..=u8::MAX => (100, 3), // Critical success - major boon
                17..=19 => (60, 2),       // Great success - good boon
                13..=16 => (30, 1),       // Success - moderate boon
                _ => (10, 0),             // Minor benefit
            };

            if let Some(player) = player_resource.get_player_mut() {
                if extra_movement > 0 {
                    player.add_movement_points(extra_movement);
                    info!("✨ Fortune smiles upon you! Gained {} experience and {} extra movement points!", xp_gain, extra_movement);
                } else {
                    info!("✨ Fortune smiles upon you! Gained {} experience", xp_gain);
                }
            }
            game_stats.record_experience_gain(xp_gain);
        }

        EventType::Mystery => {
            if final_roll >= 15 {
                let bonus_movement = if final_roll >= 18 { 2 } else { 1 };
                if let Some(player) = player_resource.get_player_mut() {
                    player.add_movement_points(bonus_movement);
                    info!("🔮 Mysterious phenomenon understood! Gained knowledge and {} movement points!", bonus_movement);
                }
                game_stats.record_experience_gain(40);
            } else {
                info!("🔮 A mysterious phenomenon occurs, but its meaning eludes you");
            }
        }

        EventType::Malfunction => {
            if final_roll <= 7 {
                // Equipment malfunction reduces movement points
                if let Some(player) = player_resource.get_player_mut() {
                    player.subtract_movement_points(1);
                    info!("🔧 Equipment malfunction! Lost 1 movement point due to efficiency reduction");
                }
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
