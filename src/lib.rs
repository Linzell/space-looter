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

use crate::domain::services::audio_service::AudioService;
use crate::domain::services::game_log_service::{GameLogService, GameLogType};
use crate::infrastructure::time::TimeService as InfraTimeService;

use bevy::asset::{AssetMetaCheck, AssetPlugin};
use bevy::prelude::*;
use bevy::time::{Timer, TimerMode};

/// Web-specific initialization for WASM
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn wasm_main() {
    // Set panic hook first for better error reporting
    console_error_panic_hook::set_once();

    // Initialize logging
    wasm_logger::init(wasm_logger::Config::default());

    // Initialize web infrastructure
    let web_infra = infrastructure::web::WebInfrastructure::default();
    if let Err(e) = web_infra.initialize() {
        web_sys::console::error_1(
            &format!("Failed to initialize web infrastructure: {}", e).into(),
        );
        return;
    }

    web_sys::console::log_1(&"🎮 Space Looter starting in WASM 3D mode".into());
    web_sys::console::log_1(&"🚀 Initializing full 3D isometric RPG experience".into());

    // Hide loading screen and show the game
    let _ = infrastructure::web::utils::set_loading_state(false);

    // Make sure game area is visible for 3D rendering
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(game_area) = document.get_element_by_id("game-area") {
                let class_list = game_area.class_list();
                let _ = class_list.remove_1("hidden");
            }
        }
    }

    // Create and run full 3D game
    web_sys::console::log_1(&"🎮 Initializing 3D game engine...".into());
    let mut app = create_app();
    app.run();
}

/// Creates a headless app without any rendering for web compatibility
#[cfg(target_arch = "wasm32")]
fn create_headless_app() -> App {
    let mut app = App::new();

    web_sys::console::log_1(&"🔧 Setting up headless game mode...".into());

    // Add only minimal plugins for basic functionality
    app.add_plugins(MinimalPlugins);

    // Add StatesPlugin for state management support
    app.add_plugins(bevy::state::app::StatesPlugin);

    // Configure RPG functionality without rendering
    configure_headless_rpg_app(&mut app);

    web_sys::console::log_1(&"✅ Headless app configured successfully".into());
    app
}

/// Configure headless RPG app without rendering components
#[cfg(target_arch = "wasm32")]
fn configure_headless_rpg_app(app: &mut App) {
    web_sys::console::log_1(&"⚙️ Configuring RPG systems...".into());

    // Initialize RPG state management
    app.init_state::<presentation::RpgAppState>();

    // Add domain services as resources (no rendering required)
    app.insert_resource(domain::services::TileMovementService::new())
        .insert_resource(domain::services::RestingService::new());

    // Add basic RPG resources without rendering dependencies
    app.insert_resource(infrastructure::bevy::resources::PlayerResource::new())
        .insert_resource(infrastructure::bevy::resources::BaseResource::new())
        .insert_resource(infrastructure::bevy::resources::MapResource::new())
        .insert_resource(infrastructure::bevy::resources::GameStatsResource::new())
        .insert_resource(infrastructure::bevy::resources::GameTimerResource::new());

    // Initialize a demo game session
    let demo_player = domain::Player::create_new_character(
        "Web Demo Player".to_string(),
        domain::Position3D::origin(),
    )
    .unwrap();
    let demo_base = domain::Base::new(
        domain::EntityId::generate(),
        "Web Demo Base".to_string(),
        domain::Position3D::origin(),
    )
    .unwrap();
    let rpg_session = presentation::game_state::RpgGameSession::new(demo_player, demo_base);
    app.insert_resource(rpg_session);

    // Add headless game systems that log to console (no input-dependent systems)
    app.add_systems(
        Update,
        (
            headless_game_tick_system,
            rpg_turn_management_system,
            rpg_dice_mechanics_system,
        ),
    );

    web_sys::console::log_1(&"⚙️ RPG configuration complete - game ready!".into());
}

/// System that runs the headless game and logs status to console
#[cfg(target_arch = "wasm32")]
fn headless_game_tick_system(time: Res<Time>) {
    // Log game status every 10 seconds
    static mut LAST_LOG: f32 = 0.0;
    let current_time = time.elapsed_secs();

    unsafe {
        if current_time - LAST_LOG >= 10.0 {
            web_sys::console::log_1(
                &format!(
                    "🎮 Game Status: Running for {:.1}s - RPG engine active, dice systems ready",
                    current_time
                )
                .into(),
            );
            LAST_LOG = current_time;
        }
    }
}

/// Creates and configures the main RPG application
pub fn create_app() -> App {
    let mut app = App::new();

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

    // For WASM, we use the web-compatible version with proper canvas setup and disabled meta files
    #[cfg(target_arch = "wasm32")]
    {
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Space Looter - 3D Isometric RPG".into(),
                        canvas: Some("#bevy".into()),
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
        );
    }

    configure_rpg_app(&mut app);
    app
}

/// Common RPG app configuration used by both native and web versions
fn configure_rpg_app(app: &mut App) {
    // Initialize RPG state management
    app.init_state::<presentation::RpgAppState>();

    // Set RPG-appropriate background color (dark space theme)
    app.insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)));

    // Add core RPG systems
    app.add_plugins((
        infrastructure::bevy::font_service::FontPlugin,
        presentation::game_state::RpgStatePlugin,
        presentation::game_ui::GameUIPlugin,
        presentation::game_log_integration::GameLogIntegrationPlugin,
        presentation::map_renderer::MapRendererPlugin,
        presentation::rendering::RenderingPlugin,
        presentation::audio_integration::AudioEventIntegrationPlugin,
        presentation::game_event_logger::GameEventLoggerPlugin,
    ));

    // Register audio events
    app.add_event::<presentation::game_event_logger::MovementAttemptEvent>()
        .add_event::<presentation::game_event_logger::RestCompletedEvent>()
        .add_event::<presentation::game_event_logger::ResourceChangedEvent>()
        .add_event::<presentation::game_event_logger::DiscoveryEvent>()
        .add_event::<presentation::game_event_logger::GameSystemEvent>();

    // Add RPG-specific resources
    app.insert_resource(infrastructure::bevy::resources::PlayerResource::new())
        .insert_resource(infrastructure::bevy::resources::BaseResource::new())
        .insert_resource(infrastructure::bevy::resources::MapResource::new())
        .insert_resource(infrastructure::bevy::resources::GameStatsResource::new())
        .insert_resource(infrastructure::bevy::resources::GameTimerResource::new())
        .insert_resource(MusicControl::default());

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
    // Add systems individually to avoid complex tuple signature issues
    app.add_systems(Update, rpg_turn_management_system);
    app.add_systems(Update, rpg_exploration_system);
    app.add_systems(Update, rpg_dice_mechanics_system);
    app.add_systems(Update, handle_window_resize_system);
    app.add_systems(Update, rpg_state_transition_system);

    // Add dice sound timer system
    app.add_systems(Update, dice_sound_timer_system);

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

/// Component for delayed dice sound playback
#[derive(Component)]
struct DiceSoundTimer {
    timer: Timer,
    audio_handle: Handle<AudioSource>,
}

/// System to handle delayed dice sound playback
fn dice_sound_timer_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut DiceSoundTimer)>,
) {
    for (entity, mut dice_timer) in query.iter_mut() {
        dice_timer.timer.tick(time.delta());

        if dice_timer.timer.just_finished() {
            // Play the dice sound
            commands.spawn(AudioPlayer::new(dice_timer.audio_handle.clone()));

            // Remove the timer component
            commands.entity(entity).despawn();
        }
    }
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
/// RPG exploration and movement system with dice mechanics
pub fn rpg_exploration_system(
    mut player_resource: ResMut<infrastructure::bevy::resources::PlayerResource>,
    mut map_resource: ResMut<infrastructure::bevy::resources::MapResource>,
    mut game_stats: ResMut<infrastructure::bevy::resources::GameStatsResource>,
    tile_movement_service: Res<domain::services::TileMovementService>,
    resting_service: Res<domain::services::RestingService>,
    mut movement_events: EventReader<crate::presentation::movement::ExecuteRpgMovement>,
    mut movement_completed_events: EventReader<crate::presentation::movement::MovementCompleted>,
    mut resting_events: EventReader<crate::presentation::movement::RestingTriggered>,
    mut player_query: Query<
        (&mut crate::presentation::movement::SmoothMovement, Entity),
        With<crate::presentation::map_renderer::PlayerMarker>,
    >,
    mut pending_movement: Local<Option<domain::Position3D>>,
    mut pending_rpg_results: Local<
        Vec<(
            domain::Position3D,
            domain::services::tile_movement::MovementResult,
        )>,
    >,
    mut rest_timer: Local<Option<Timer>>,
    time: Res<Time>,
    mut game_log: ResMut<GameLogService>,
    mut commands: Commands,
    audio_assets: Option<Res<presentation::audio_integration::AudioAssets>>,
) {
    if !player_resource.has_player() {
        return;
    }

    // Check if we're currently resting
    if let Some(ref mut timer) = *rest_timer {
        timer.tick(time.delta());
        if timer.just_finished() {
            info!("😴 Rest period complete, you can now move again");
            // Play rest complete audio
            if let Some(audio_assets) = &audio_assets {
                if let Some(rest_handle) = &audio_assets.rest_complete {
                    commands.spawn(AudioPlayer::new(rest_handle.clone()));
                }
            }
            *rest_timer = None;
        } else {
            // Still resting, block all movement
            return;
        }
    }

    // Handle resting events (when movement points reach zero)
    for resting_event in resting_events.read() {
        if let Some(mut player) = player_resource.get_player_mut() {
            info!(
                "😴 Processing automatic rest at {:?}",
                resting_event.player_position
            );

            // Process rest cycle using the resting service
            match resting_service.process_rest_cycle(&mut player, resting_event.player_position) {
                Ok(rest_result) => {
                    info!("🌅 Rest completed: {}", rest_result.description);
                    info!(
                        "⚡ Movement points restored: {} (was {})",
                        rest_result.movement_points_restored,
                        resting_event.remaining_movement_points
                    );

                    // Log the rest result
                    game_log.log_message(rest_result.description, crate::GameLogType::Rest);

                    // Add resources gained message if any
                    if !rest_result.resources_gained.is_empty() {
                        let resource_types: Vec<String> = rest_result
                            .resources_gained
                            .amounts()
                            .iter()
                            .map(|amount| format!("{} {}", amount.amount, amount.resource_type))
                            .collect();
                        let resource_msg =
                            format!("Resources gained: {}", resource_types.join(", "));
                        game_log.log_message(resource_msg, crate::GameLogType::Resources);
                    }

                    // Play rest complete audio
                    if let Some(audio_assets) = &audio_assets {
                        if let Some(rest_handle) = &audio_assets.rest_complete {
                            commands.spawn(AudioPlayer::new(rest_handle.clone()));
                        }
                    }

                    // Update game stats
                    if rest_result.resources_gained.total_value() > 0 {
                        // Record resource gathering for each resource type gained
                        for amount in rest_result.resources_gained.amounts() {
                            game_stats.record_resource_gather(amount.resource_type, amount.amount);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to process rest cycle: {:?}", e);
                    game_log.log_message(
                        "Rest failed - something went wrong during the night".to_string(),
                        crate::GameLogType::Warning,
                    );
                }
            }
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
        // Movement input is now handled by the smooth movement system
        // This system only processes movement commands from the smooth movement system
    }

    // Process movement commands from smooth movement system
    for movement_event in movement_events.read() {
        info!(
            "🎮 RPG System: Received movement event to {:?}",
            movement_event.target_position
        );
        target_position = movement_event.target_position;
        movement_attempted = true;
    }

    // Process completed movements from smooth movement system
    for completion_event in movement_completed_events.read() {
        // Find and apply the corresponding RPG movement result
        if let Some(index) = pending_rpg_results
            .iter()
            .position(|(pos, _)| *pos == completion_event.final_position)
        {
            let (final_pos, movement_result) = pending_rpg_results.remove(index);

            info!(
                "🎮 RPG System: Applying delayed movement result to {:?}",
                final_pos
            );

            // Now actually update the player position
            if let Err(e) = player_resource.move_player(final_pos, 1) {
                warn!("Failed to update player position after animation: {:?}", e);
            } else {
                // Apply the movement result effects
                apply_movement_result(
                    &movement_result,
                    &mut player_resource,
                    &mut game_stats,
                    &mut game_log,
                );
            }
        }
    }

    if movement_attempted {
        // Get player for dice calculations
        if let Some(player) = player_resource.get_player() {
            let player_level = player.level();

            // Get or generate map around player position
            let map = map_resource.get_or_create_map_mut(current_position);

            // Attempt tile movement with dice roll - but DON'T update player position yet
            match tile_movement_service.attempt_movement(
                &player,
                target_position,
                map,
                player_level,
            ) {
                Ok(movement_result) => {
                    // Schedule dice roll sound with delay
                    if let Some(audio_assets) = &audio_assets {
                        if let Some(dice_handle) = &audio_assets.dice_roll {
                            commands.spawn((DiceSoundTimer {
                                timer: Timer::from_seconds(
                                    crate::domain::constants::DICE_SOUND_DELAY_MS as f32 / 1000.0,
                                    TimerMode::Once,
                                ),
                                audio_handle: dice_handle.clone(),
                            },));
                        }
                    }
                    // Log dice roll result - console only (debug)
                    info!("🎲 {}", movement_result.dice_result.description());
                    info!(
                        "📍 Outcome: {}",
                        movement_result.dice_result.outcome_category()
                    );

                    info!(
                        "🚀 Movement validation successful to: {:?} (cost: {})",
                        target_position, movement_result.movement_cost
                    );

                    // Update animation duration based on movement cost and terrain type
                    if let Ok((mut smooth_movement, _)) = player_query.single_mut() {
                        use crate::domain::constants::*;

                        // Get terrain type from the target position
                        let terrain_multiplier = if let Some(map) = map_resource.current_map() {
                            let tile_coord = crate::domain::value_objects::TileCoordinate::new(
                                target_position.x,
                                target_position.y,
                                target_position.z,
                            );
                            if let Some(tile) = map.get_tile(&tile_coord) {
                                get_terrain_duration_multiplier(tile.terrain_type)
                            } else {
                                1.0 // Default multiplier if no tile found
                            }
                        } else {
                            1.0 // Default multiplier if no map
                        };

                        // Calculate base duration from movement cost
                        let base_duration_ms = BASE_MOVEMENT_ANIMATION_DURATION_MS
                            + (movement_result.movement_cost as f32
                                * DURATION_PER_MOVEMENT_POINT_MS);

                        // Apply terrain multiplier
                        let duration_ms = (base_duration_ms * terrain_multiplier)
                            .min(MAX_MOVEMENT_ANIMATION_DURATION_MS);

                        info!(
                            "🎮 Updating animation duration to {}ms (cost: {}, terrain multiplier: {:.1}x)",
                            duration_ms, movement_result.movement_cost, terrain_multiplier
                        );
                        smooth_movement.duration =
                            std::time::Duration::from_millis(duration_ms as u64);
                    }

                    // Store the movement result to be applied when animation completes
                    info!("🎮 RPG System: Storing movement result for delayed execution");
                    pending_rpg_results.push((target_position, movement_result));

                    // Don't update player position immediately - wait for animation to complete
                    info!("✅ Movement validation passed, waiting for animation to complete");

                    // All movement result processing is now delayed until animation completes
                }
                Err(e) => {
                    warn!("❌ Movement failed: {}", e);

                    // This should rarely happen now due to pre-validation
                    // Play blocked movement audio
                    if let Some(audio_assets) = &audio_assets {
                        if let Some(ui_handle) = &audio_assets.ui_click {
                            commands.spawn(AudioPlayer::new(ui_handle.clone()));
                        }
                    }

                    match e {
                        domain::DomainError::InvalidMapCoordinates(..) => {
                            info!("🚫 Can only move to adjacent tiles!");
                            game_log.log_message(
                                "Can only move to adjacent tiles".to_string(),
                                GameLogType::Warning,
                            );
                        }
                        domain::DomainError::TileNotAccessible(..) => {
                            info!("🚫 That tile is not passable!");
                            game_log.log_message(
                                "That tile is not passable".to_string(),
                                GameLogType::Warning,
                            );
                        }
                        domain::DomainError::InsufficientResources(_) => {
                            info!("⚡ Not enough movement points!");
                            game_log.log_message(
                                "Not enough movement points".to_string(),
                                GameLogType::Warning,
                            );

                            // Play exhausted audio for no movement points
                            if let Some(audio_assets) = &audio_assets {
                                if let Some(ui_handle) = &audio_assets.ui_click {
                                    commands.spawn(AudioPlayer::new(ui_handle.clone()));
                                }
                            }

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
                                    game_log.log_message(
                                        "You are exhausted and must rest for the night".to_string(),
                                        GameLogType::System,
                                    );

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
                                                game_log.log_message(
                                                    "Dawn breaks after a night of rest".to_string(),
                                                    GameLogType::System,
                                                );

                                                // Play rest start audio
                                                if let Some(audio_assets) = &audio_assets {
                                                    if let Some(ui_handle) = &audio_assets.ui_click
                                                    {
                                                        commands.spawn(AudioPlayer::new(
                                                            ui_handle.clone(),
                                                        ));
                                                    }
                                                }

                                                info!(
                                                    "🎲 Night Roll: {} - {}",
                                                    rest_result.dice_roll, rest_result.night_event
                                                );
                                                game_log.log_message(
                                                    format!(
                                                        "Night Roll: {} - {}",
                                                        rest_result.dice_roll,
                                                        rest_result.night_event
                                                    ),
                                                    GameLogType::Rest,
                                                );

                                                info!(
                                                    "😴 Rest Quality: {}",
                                                    rest_result.rest_outcome
                                                );
                                                game_log.log_message(
                                                    format!(
                                                        "Rest Quality: {}",
                                                        rest_result.rest_outcome
                                                    ),
                                                    GameLogType::Rest,
                                                );

                                                info!("📖 {}", rest_result.description);
                                                game_log.log_message(
                                                    rest_result.description.clone(),
                                                    GameLogType::Narrative,
                                                );

                                                if !rest_result.resources_gained.is_empty() {
                                                    let resources_text = format_resource_summary(
                                                        &rest_result.resources_gained,
                                                    );
                                                    info!(
                                                        "💰 Resources gained during rest: {}",
                                                        resources_text
                                                    );
                                                    game_log.log_message(
                                                        format!(
                                                            "Resources gained during rest: {}",
                                                            resources_text
                                                        ),
                                                        GameLogType::Resources,
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
                                                    player_mut.movement_points(),
                                                    rest_duration.as_secs()
                                                );
                                                game_log.log_message(
                                                    format!("Movement points restored: {} - resting for {} seconds",
                                                        player_mut.movement_points(),
                                                        rest_duration.as_secs()
                                                    ),
                                                    GameLogType::System
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

                                                // Play emergency rest audio
                                                if let Some(audio_assets) = &audio_assets {
                                                    if let Some(ui_handle) = &audio_assets.ui_click
                                                    {
                                                        commands.spawn(AudioPlayer::new(
                                                            ui_handle.clone(),
                                                        ));
                                                    }
                                                }
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
    game_log: &mut ResMut<GameLogService>,
    commands: &mut Commands,
    audio_assets: Option<&Res<presentation::audio_integration::AudioAssets>>,
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
            game_log.log_message(
                format!(
                    "Gained {} movement points from successful exploration!",
                    movement_reward
                ),
                GameLogType::Resources,
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

                // Play resource discovery audio
                if let Some(audio_assets) = audio_assets {
                    if let Some(resource_handle) = &audio_assets.resource_collect {
                        commands.spawn(AudioPlayer::new(resource_handle.clone()));
                    }
                }

                // Note: Single audio play for all resource finds - no duplicate for rare finds
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
                // Play damage/combat audio
                if let Some(audio_assets) = audio_assets {
                    if let Some(ui_handle) = &audio_assets.ui_click {
                        commands.spawn(AudioPlayer::new(ui_handle.clone()));
                    }
                }
                // TODO: Implement actual damage system
            } else {
                info!("⚔️ Combat encounter successfully resolved!");
                // Play victory audio
                if let Some(audio_assets) = audio_assets {
                    if let Some(ui_handle) = &audio_assets.ui_click {
                        commands.spawn(AudioPlayer::new(ui_handle.clone()));
                    }
                }
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

                    // Play hazard audio
                    if let Some(audio_assets) = audio_assets {
                        if let Some(ui_handle) = &audio_assets.ui_click {
                            commands.spawn(AudioPlayer::new(ui_handle.clone()));
                        }
                    }
                }
            } else if final_roll >= 13 {
                info!("⚠️ Successfully navigated environmental hazard!");
                game_stats.record_experience_gain(10);
                // Play success audio
                if let Some(audio_assets) = audio_assets {
                    if let Some(ui_handle) = &audio_assets.ui_click {
                        commands.spawn(AudioPlayer::new(ui_handle.clone()));
                    }
                }
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
                    info!("💾 Successful trade! Gained {} data!", data_amount);
                    game_stats.record_experience_gain(data_amount as u32 / 2);

                    // Play successful trade audio
                    if let Some(audio_assets) = audio_assets {
                        if let Some(ui_handle) = &audio_assets.ui_click {
                            commands.spawn(AudioPlayer::new(ui_handle.clone()));
                        }
                    }
                }
            } else {
                info!("💼 Trade failed - no resources gained");
                // Play trade failure audio
                if let Some(audio_assets) = audio_assets {
                    if let Some(ui_handle) = &audio_assets.ui_click {
                        commands.spawn(AudioPlayer::new(ui_handle.clone()));
                    }
                }
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
    keyboard_input: Option<Res<ButtonInput<KeyCode>>>,
    mut game_stats: ResMut<infrastructure::bevy::resources::GameStatsResource>,
    mut commands: Commands,
    audio_assets: Option<Res<presentation::audio_integration::AudioAssets>>,
    time: Res<Time>,
) {
    // Auto-roll for headless mode or manual roll with spacebar
    let has_keyboard = keyboard_input.is_some();
    let should_roll = if let Some(keyboard) = &keyboard_input {
        keyboard.just_pressed(KeyCode::Space)
    } else {
        // Auto-roll every 5 seconds in headless mode
        static mut LAST_ROLL_TIME: f32 = 0.0;
        let current_time = time.elapsed_secs();
        unsafe {
            if current_time - LAST_ROLL_TIME >= 5.0 {
                LAST_ROLL_TIME = current_time;
                true
            } else {
                false
            }
        }
    };

    if should_roll {
        // Create a basic dice roll for demonstration
        if let Ok(dice_roll) = domain::DiceRoll::new(
            1,
            domain::DiceType::D20,
            domain::value_objects::dice::DiceModifier::none(),
        ) {
            game_stats.record_dice_roll(&dice_roll, 10);
            let total = dice_roll.total();

            // Trigger dice roll audio
            info!("🎲 Playing dice roll audio for roll: {}", total);
            if let Some(audio_assets) = &audio_assets {
                if let Some(dice_handle) = &audio_assets.dice_roll {
                    commands.spawn(AudioPlayer::new(dice_handle.clone()));
                }
            }

            let roll_prefix = if has_keyboard { "" } else { "Auto-rolled " };

            match total {
                20 => {
                    info!("🎲 {}Critical Success! ({})", roll_prefix, total);
                    game_stats.record_experience_gain(50);
                    #[cfg(target_arch = "wasm32")]
                    web_sys::console::log_1(
                        &format!("🎲 {}Critical Success! Rolled: {}", roll_prefix, total).into(),
                    );
                }
                18..=19 => {
                    info!("🎲 {}Great Success! ({})", roll_prefix, total);
                    game_stats.record_experience_gain(25);
                    #[cfg(target_arch = "wasm32")]
                    web_sys::console::log_1(
                        &format!("🎲 {}Great Success! Rolled: {}", roll_prefix, total).into(),
                    );
                }
                15..=17 => {
                    info!("🎲 {}Good Success! ({})", roll_prefix, total);
                    game_stats.record_experience_gain(25);
                    #[cfg(target_arch = "wasm32")]
                    web_sys::console::log_1(
                        &format!("🎲 {}Good Success! Rolled: {}", roll_prefix, total).into(),
                    );
                }
                10..=14 => {
                    info!("🎲 {}Success! ({})", roll_prefix, total);
                    game_stats.record_experience_gain(10);
                    #[cfg(target_arch = "wasm32")]
                    web_sys::console::log_1(
                        &format!("🎲 {}Success! Rolled: {}", roll_prefix, total).into(),
                    );
                }
                6..=9 => {
                    info!("🎲 {}Partial Success ({})", roll_prefix, total);
                    game_stats.record_experience_gain(5);
                    #[cfg(target_arch = "wasm32")]
                    web_sys::console::log_1(
                        &format!("🎲 {}Partial Success! Rolled: {}", roll_prefix, total).into(),
                    );
                }
                2..=5 => {
                    info!("🎲 {}Failed Roll ({})", roll_prefix, total);
                    #[cfg(target_arch = "wasm32")]
                    web_sys::console::log_1(
                        &format!("🎲 {}Failed! Rolled: {}", roll_prefix, total).into(),
                    );
                }
                1 => {
                    info!("🎲 {}Critical Failure! ({})", roll_prefix, total);
                    #[cfg(target_arch = "wasm32")]
                    web_sys::console::log_1(
                        &format!("🎲 {}Critical Failure! Rolled: {}", roll_prefix, total).into(),
                    );
                }
                _ => {
                    info!("🎲 {}Failed Roll ({})", roll_prefix, total);
                    #[cfg(target_arch = "wasm32")]
                    web_sys::console::log_1(
                        &format!("🎲 {}Failed! Rolled: {}", roll_prefix, total).into(),
                    );
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

/// Music management system based on game state
/// Component to mark background music entities for tracking and cleanup
#[derive(Component)]
struct BackgroundMusic;

#[derive(Resource)]
struct MusicControl {
    enabled: bool,
    should_play: bool,
}

impl Default for MusicControl {
    fn default() -> Self {
        Self {
            enabled: true,
            should_play: true,
        }
    }
}

// Old music management systems removed - now using terrain-based ambient system
// in src/presentation/audio_integration.rs

/// System to handle window resize events for better responsive RPG UI
fn handle_window_resize_system() {
    // Window resize handling will be added later - focus on core mechanics first
}

/// Native main function (for binary builds)
#[cfg(not(target_arch = "wasm32"))]
pub fn main() {
    info!("🎮 Starting Space Looter 3D Isometric RPG (Native)");
    info!("🎲 Use WASD/Arrow keys to explore, SPACE to roll dice");
    info!("📋 Press B for base, Q for quests, I for inventory");
    info!("🚀 Press ENTER in menu to start exploring!");

    let mut app = create_app();
    app.run();
}

/// Entry point for RPG (works on both native and WASM)
pub fn run() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        info!("🎮 Starting Space Looter 3D Isometric RPG (Native)");
        info!("🎲 Use WASD/Arrow keys to explore, SPACE to roll dice");
        info!("📋 Press B for base, Q for quests, I for inventory");
        info!("🚀 Press ENTER in menu to start exploring!");

        let mut app = create_app();
        app.run();
    }
    #[cfg(target_arch = "wasm32")]
    {
        // On WASM, the entry point is handled by wasm_main()
        web_sys::console::log_1(&"Run called on WASM - use wasm_main() instead".into());
    }
}

// Global music control resource for WASM communication
#[cfg(target_arch = "wasm32")]
use std::sync::{Arc, Mutex};
#[cfg(target_arch = "wasm32")]
static MUSIC_CONTROL_STATE: once_cell::sync::Lazy<Arc<Mutex<MusicControlState>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(MusicControlState::default())));

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Debug)]
struct MusicControlState {
    should_play: bool,
    should_stop: bool,
    audio_enabled: bool,
}

#[cfg(target_arch = "wasm32")]
impl Default for MusicControlState {
    fn default() -> Self {
        Self {
            should_play: true,
            should_stop: false,
            audio_enabled: true,
        }
    }
}

// WASM bindings for HTML music controls
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn play_music() {
    web_sys::console::log_1(&"🎵 Music play requested from HTML".into());

    if let Ok(mut state) = MUSIC_CONTROL_STATE.lock() {
        // Force a clean state reset
        state.should_play = false; // Reset first
        state.should_stop = false;
        state.audio_enabled = true;

        // Then enable play (this will trigger the restart logic)
        state.should_play = true;
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn stop_music() {
    web_sys::console::log_1(&"🔇 Music stop requested from HTML".into());

    if let Ok(mut state) = MUSIC_CONTROL_STATE.lock() {
        state.should_stop = true;
        state.should_play = false;
    }
}

/// Apply movement result effects after animation completes
fn apply_movement_result(
    movement_result: &domain::services::tile_movement::MovementResult,
    player_resource: &mut ResMut<infrastructure::bevy::resources::PlayerResource>,
    game_stats: &mut ResMut<infrastructure::bevy::resources::GameStatsResource>,
    game_log: &mut ResMut<GameLogService>,
) {
    // Update game statistics
    game_stats.record_tile_explored();

    // Handle movement result based on what happened
    if let Some(event) = &movement_result.triggered_event {
        info!(
            "🎭 Event Triggered: {} - {}",
            event.title(),
            event.description()
        );

        game_log.log_message(
            format!(
                "Event Triggered: {} - {}",
                event.title(),
                event.description()
            ),
            GameLogType::Event,
        );

        // Add resources from event
        if let Some(mut player) = player_resource.get_player_mut() {
            // Add movement points from successful exploration
            player.add_movement_points(2);
        }

        info!("🏃 Gained 2 movement points from successful exploration!");

        // Log event description
        info!("📖 {}", event.description());
        game_log.log_message(event.description().to_string(), GameLogType::Narrative);
    } else {
        info!("🚶 Safe movement - no events triggered");

        // Give small movement point recovery even for safe movement
        if let Some(player) = player_resource.get_player_mut() {
            player.add_movement_points(2);
            info!("🏃 Safe exploration grants 2 movement points");
            game_log.log_message(
                "Safe exploration grants 2 movement points".to_string(),
                GameLogType::Resources,
            );
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn toggle_audio(enabled: bool) {
    web_sys::console::log_1(&format!("🔊 Audio toggle requested from HTML: {}", enabled).into());

    if let Ok(mut state) = MUSIC_CONTROL_STATE.lock() {
        state.audio_enabled = enabled;
        if !enabled {
            state.should_stop = true;
            state.should_play = false;
        } else {
            state.should_play = true;
            state.should_stop = false;
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_music_state() -> bool {
    if let Ok(state) = MUSIC_CONTROL_STATE.lock() {
        state.should_play && state.audio_enabled
    } else {
        true // Default to playing
    }
}
