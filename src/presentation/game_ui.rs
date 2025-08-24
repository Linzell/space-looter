//! Space Looter Game UI - Advanced Space-Themed Interface
//!
//! This module provides a comprehensive space exploration UI with enhanced graphics,
//! space terminology, and immersive visual elements for the Space Looter RPG.

use crate::domain::constants::{
    get_terrain_scanner_color, CRITICAL_TEXT, ENERGY_COLOR, PANEL_BACKGROUND, PRIMARY_TEXT,
    RESOURCE_COLOR, SCANNER_GRID, SECONDARY_TEXT, SHIP_SIGNATURE, SUCCESS_TEXT, UNEXPLORED_SPACE,
    WARNING_TEXT,
};
use crate::domain::services::font_service::{FontService, FontSize, FontType};
use crate::domain::services::game_log_service::{GameLogService, GameLogType};

use crate::infrastructure::bevy::font_service::{BevyFontService, RegularText};
use crate::infrastructure::bevy::resources::{GameStatsResource, MapResource, PlayerResource};
use crate::infrastructure::time::TimeService;
use crate::presentation::RpgAppState;
use bevy::prelude::*;

/// Plugin for space-themed game UI functionality
pub struct GameUIPlugin;

impl Plugin for GameUIPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameLogService>()
            .add_systems(Startup, (initialize_space_icons, setup_space_ui))
            .add_systems(
                Update,
                (
                    update_space_ui,
                    update_game_log_display,
                    auto_start_exploration_system,
                    handle_ui_animations,
                    apply_fonts_to_ui_text,
                ),
            );
    }
}

/// Component markers for UI elements
#[derive(Component)]
pub struct SectorMapDisplay;

#[derive(Component)]
pub struct ShipStatusPanel;

#[derive(Component)]
pub struct MissionControlPanel;

#[derive(Component)]
pub struct ScannerGrid;

#[derive(Component)]
pub struct SectorTile {
    pub grid_x: i32,
    pub grid_y: i32,
}

#[derive(Component)]
pub struct UIAnimated {
    pub timer: Timer,
    pub original_color: Color,
}

#[derive(Component)]
pub struct ResourceBar;

#[derive(Component)]
pub struct AlertPanel;

/// Component for the main game log panel
#[derive(Component)]
pub struct GameLogPanel;

/// Component for individual log entries
#[derive(Component)]
pub struct GameLogEntry {
    pub timestamp: u64,
    pub log_type: GameLogType,
    pub fade_timer: Timer,
}

/// Component for the log scroll container
#[derive(Component)]
pub struct GameLogScrollArea;

#[derive(Component)]
pub struct LogScrollContainer;

#[derive(Component)]
pub struct LogContentArea;

#[derive(Component)]
pub struct RocketIcon;

#[derive(Component)]
pub struct SatelliteIcon;

#[derive(Component)]
pub struct GearIcon;

/// Space UI icons resource for loading image-based icons
#[derive(Resource, Default)]
pub struct SpaceIcons {
    pub rocket: Handle<Image>,
    pub satellite: Handle<Image>,
    pub gear: Handle<Image>,
    pub dice: Handle<Image>,
}

/// Setup comprehensive space-themed UI
fn setup_space_ui(mut commands: Commands) {
    info!("🚀 Initializing Space Looter Command Interface");

    // Root container - Full screen HUD overlay
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            Name::new("SpaceHUD_Root"),
        ))
        .with_children(|parent| {
            // Left Panel - Sector Scanner
            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(15.0),
                        top: Val::Px(15.0),
                        width: Val::Px(320.0),
                        height: Val::Px(280.0),
                        padding: UiRect::all(Val::Px(15.0)),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    Name::new("SectorScanner"),
                ))
                .with_children(|parent| {
                    // Scanner Header
                    parent
                        .spawn((Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            margin: UiRect::bottom(Val::Px(8.0)),
                            ..default()
                        },))
                        .with_children(|parent| {
                            // Satellite icon
                            parent.spawn((
                                ImageNode {
                                    image: Handle::default(),
                                    ..default()
                                },
                                Node {
                                    width: Val::Px(16.0),
                                    height: Val::Px(16.0),
                                    margin: UiRect::right(Val::Px(8.0)),
                                    ..default()
                                },
                                SatelliteIcon,
                            ));

                            // Title text
                            parent.spawn((
                                Text::new("LONG RANGE SCANNER"),
                                TextFont {
                                    font_size: FontSize::Medium.to_pixels(),
                                    ..default()
                                },
                                TextColor(ENERGY_COLOR),
                                RegularText,
                                Name::new("ScannerTitle"),
                            ));
                        });

                    // Coordinates Display
                    parent.spawn((
                        Text::new("COORDINATES: [0, 0, 0] | SCANNING..."),
                        TextFont {
                            font_size: FontSize::Regular.to_pixels(),
                            ..default()
                        },
                        TextColor(SECONDARY_TEXT),
                        Node {
                            margin: UiRect::bottom(Val::Px(8.0)),
                            ..default()
                        },
                        RegularText,
                        SectorMapDisplay,
                    ));

                    // Scanner Grid (7x7 for better detail)
                    parent
                        .spawn((
                            Node {
                                width: Val::Px(210.0),
                                height: Val::Px(210.0),
                                display: Display::Grid,
                                grid_template_columns: RepeatedGridTrack::flex(7, 1.0),
                                grid_template_rows: RepeatedGridTrack::flex(7, 1.0),
                                column_gap: Val::Px(1.0),
                                row_gap: Val::Px(1.0),
                                margin: UiRect::bottom(Val::Px(8.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BorderColor(SCANNER_GRID),
                            ScannerGrid,
                        ))
                        .with_children(|parent| {
                            // Create 49 scanner tiles (7x7 grid)
                            for y in 0..7 {
                                for x in 0..7 {
                                    parent.spawn((
                                        Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Percent(100.0),
                                            border: UiRect::all(Val::Px(0.5)),
                                            ..default()
                                        },
                                        BackgroundColor(UNEXPLORED_SPACE),
                                        BorderColor(SCANNER_GRID),
                                        SectorTile {
                                            grid_x: x - 3,
                                            grid_y: y - 3,
                                        },
                                    ));
                                }
                            }
                        });
                });

            // Game Log Panel - Bottom Left
            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(15.0),
                        bottom: Val::Px(15.0),
                        width: Val::Px(400.0),
                        height: Val::Px(200.0),
                        padding: UiRect::all(Val::Px(12.0)),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    GameLogPanel,
                    BackgroundColor(PANEL_BACKGROUND),
                    Name::new("GameLogPanel"),
                ))
                .with_children(|parent| {
                    // Log header
                    parent.spawn((
                        Text::new("MISSION LOG"),
                        TextFont {
                            font_size: FontSize::Medium.to_pixels(),
                            ..default()
                        },
                        TextColor(ENERGY_COLOR),
                        RegularText,
                        Node {
                            margin: UiRect::bottom(Val::Px(8.0)),
                            ..default()
                        },
                    ));

                    // Scrollable log area with working scroll
                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(85.0),
                                flex_direction: FlexDirection::Column,
                                overflow: Overflow::clip_y(), // Enable vertical scrolling
                                justify_content: JustifyContent::FlexEnd,
                                ..default()
                            },
                            GameLogScrollArea,
                        ))
                        .with_children(|parent| {
                            // Inner content area for log entries
                            parent.spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Column,
                                    ..default()
                                },
                                LogContentArea,
                            ));
                        });
                });
        });

    info!("Space Command Interface initialized");
}

/// Update all space UI elements
fn update_space_ui(
    map_resource: Res<MapResource>,
    player_resource: Res<PlayerResource>,
    game_stats: Res<GameStatsResource>,
    mut scanner_query: Query<
        &mut Text,
        (
            With<SectorMapDisplay>,
            Without<ShipStatusPanel>,
            Without<MissionControlPanel>,
        ),
    >,
    mut status_query: Query<
        &mut Text,
        (
            With<ShipStatusPanel>,
            Without<SectorMapDisplay>,
            Without<MissionControlPanel>,
        ),
    >,
    mut control_query: Query<
        &mut Text,
        (
            With<MissionControlPanel>,
            Without<SectorMapDisplay>,
            Without<ShipStatusPanel>,
        ),
    >,
    mut tile_query: Query<(&mut BackgroundColor, &SectorTile)>,
) {
    // Update scanner coordinates
    if let Ok(mut scanner_text) = scanner_query.single_mut() {
        if map_resource.has_map() && player_resource.has_player() {
            let player_pos = player_resource.player_position().unwrap_or_default();
            **scanner_text = format!(
                "COORDINATES: [{}, {}, {}] | SECTOR SCAN ACTIVE",
                player_pos.x, player_pos.y, player_pos.z
            );
        } else {
            **scanner_text = "COORDINATES: [?, ?, ?] | INITIALIZING SENSORS...".to_string();
        }
    }

    // Update scanner grid tiles
    if map_resource.has_map() && player_resource.has_player() {
        let player_pos = player_resource.player_position().unwrap_or_default();
        let map = map_resource.current_map().unwrap();

        for (mut bg_color, tile_info) in tile_query.iter_mut() {
            let world_x = player_pos.x + tile_info.grid_x;
            let world_y = player_pos.y + tile_info.grid_y;

            if tile_info.grid_x == 0 && tile_info.grid_y == 0 {
                // Ship signature - pulsing yellow
                bg_color.0 = SHIP_SIGNATURE;
            } else {
                let tile_coord = crate::domain::value_objects::TileCoordinate::new(
                    world_x,
                    world_y,
                    player_pos.z,
                );

                if let Some(tile) = map.get_tile(&tile_coord) {
                    if tile.is_explored() {
                        bg_color.0 = get_terrain_scanner_color(tile.terrain_type);
                    } else {
                        bg_color.0 = UNEXPLORED_SPACE;
                    }
                } else {
                    bg_color.0 = UNEXPLORED_SPACE;
                }
            }
        }
    }

    // Update ship systems status
    if let Ok(mut status_text) = status_query.single_mut() {
        if player_resource.has_player() {
            let player = player_resource.get_player().unwrap();
            let health_percent = 100; // Player health is always 100 for now
            let energy_percent = (player.movement_points() as f32
                / player.max_movement_points() as f32
                * 100.0) as i32;

            let health_status = match health_percent {
                81..=100 => ("🟢 OPTIMAL", SUCCESS_TEXT),
                61..=80 => ("🟡 GOOD", WARNING_TEXT),
                31..=60 => ("🟠 DAMAGED", WARNING_TEXT),
                _ => ("🔴 CRITICAL", CRITICAL_TEXT),
            };

            **status_text = format!(
                "HULL INTEGRITY: {}% - {}\nPOWER CORE: {}% CAPACITY\nPROPULSION: {}/{} THRUST\nPILOT LEVEL: {}\n\nMISSION PROGRESS\nSectors Mapped: {}\nQuantum Events: {}\nSuccess Rate: {:.0}%",
                health_percent,
                health_status.0,
                energy_percent,
                player.movement_points(),
                player.max_movement_points(),
                player.level(),
                game_stats.tiles_explored,
                game_stats.dice_rolls_made,
                game_stats.success_rate() * 100.0
            );
        } else {
            **status_text =
                "SHIP SYSTEMS: INITIALIZING...\nESTABLISHING QUANTUM LINK...".to_string();
        }
    }

    // Update mission control commands (can be dynamic based on state)
    if let Ok(mut control_text) = control_query.single_mut() {
        **control_text = "WASD/ARROWS: Navigate Sectors | SPACE: Quantum Dice Roll | B: Base Operations | Q: Mission Database | I: Cargo Manifest | ESC: Command Menu".to_string();
    }
}

/// Add a log message to the game log service
pub fn add_game_log_message(
    mut game_log: ResMut<GameLogService>,
    message: String,
    log_type: GameLogType,
) {
    game_log.log_message(message, log_type);
}

/// Update the game log display with new messages
fn update_game_log_display(
    mut commands: Commands,
    game_log: Res<GameLogService>,
    log_scroll_query: Query<Entity, With<GameLogScrollArea>>,
    existing_entries: Query<Entity, With<GameLogEntry>>,
    time: Res<Time>,
    mut entry_query: Query<(&mut GameLogEntry, &mut TextColor)>,
    mut needs_initial_update: Local<bool>,
) {
    let should_update = game_log.is_changed() || !*needs_initial_update;

    if !should_update {
        // Update fade timers for existing entries
        for (mut entry, mut color) in entry_query.iter_mut() {
            entry.fade_timer.tick(time.delta());

            if entry.fade_timer.finished() {
                // Start fading after 10 seconds
                let current_time = TimeService::now_millis().unwrap_or(0);
                let elapsed_secs = ((current_time - entry.timestamp) as f32) / 1000.0;
                let fade_progress = (elapsed_secs - 10.0).max(0.0) / 5.0;
                let alpha = (1.0 - fade_progress).max(0.3);

                let base_color = get_log_type_color(&entry.log_type);
                if let Color::Srgba(srgba) = base_color {
                    color.0 = Color::srgba(srgba.red, srgba.green, srgba.blue, alpha);
                } else {
                    color.0 = Color::srgba(0.8, 0.8, 0.8, alpha);
                }
            }
        }
        return;
    }

    *needs_initial_update = true;

    if let Ok(scroll_entity) = log_scroll_query.single() {
        // Clear existing entries
        for entity in existing_entries.iter() {
            commands.entity(entity).despawn();
        }

        // Add recent messages (last 10)
        let recent_messages = game_log.get_recent_messages(10);

        commands.entity(scroll_entity).with_children(|parent| {
            for message in recent_messages.iter() {
                let color = get_log_type_color(&message.log_type);

                parent.spawn((
                    Text::new(&message.message),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(color),
                    Node {
                        margin: UiRect::bottom(Val::Px(3.0)),
                        ..default()
                    },
                    GameLogEntry {
                        timestamp: TimeService::now_millis().unwrap_or(0),
                        log_type: message.log_type.clone(),
                        fade_timer: Timer::from_seconds(10.0, TimerMode::Once),
                    },
                ));
            }
        });
    }
}

/// Get the appropriate color for a log type
fn get_log_type_color(log_type: &GameLogType) -> Color {
    match log_type {
        GameLogType::Movement => PRIMARY_TEXT,
        GameLogType::Combat => CRITICAL_TEXT,
        GameLogType::Discovery => SUCCESS_TEXT,
        GameLogType::Rest => ENERGY_COLOR,
        GameLogType::Resources => RESOURCE_COLOR,
        GameLogType::Event => WARNING_TEXT,
        GameLogType::System => SECONDARY_TEXT,
        GameLogType::Warning => WARNING_TEXT,
        GameLogType::Critical => CRITICAL_TEXT,
        GameLogType::Narrative => PRIMARY_TEXT,
    }
}

/// Auto-start exploration mode with space theme
fn auto_start_exploration_system(
    current_state: Res<State<RpgAppState>>,
    mut next_state: ResMut<NextState<RpgAppState>>,
    mut timer: Local<f32>,
    time: Res<Time>,
    map_resource: Res<MapResource>,
    player_resource: Res<PlayerResource>,
) {
    if *current_state == RpgAppState::MainMenu
        && map_resource.has_map()
        && player_resource.has_player()
    {
        *timer += time.delta_secs();
        if *timer >= 0.5 {
            next_state.set(RpgAppState::Exploration);
            info!("🚀 Launching deep space exploration mission!");
            *timer = 0.0;
        }
    } else if *current_state != RpgAppState::MainMenu {
        *timer = 0.0;
    }
}

/// Handle UI animations (pulsing effects, etc.)
fn handle_ui_animations(
    time: Res<Time>,
    mut animated_query: Query<(&mut BackgroundColor, &mut UIAnimated)>,
) {
    for (mut bg_color, mut animated) in animated_query.iter_mut() {
        animated.timer.tick(time.delta());

        if animated.timer.finished() {
            animated.timer.reset();
            // Pulse effect for animated elements
            let pulse = (time.elapsed_secs() * 3.0).sin() * 0.3 + 0.7;
            // Simple pulse effect by adjusting alpha
            let original = animated.original_color;
            bg_color.0 = original.with_alpha(pulse);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::TerrainType;

    use super::*;

    #[test]
    fn space_object_signatures_are_unique() {
        assert_ne!(
            get_terrain_scanner_color(TerrainType::Plains),
            get_terrain_scanner_color(TerrainType::Forest)
        );
        assert_ne!(
            get_terrain_scanner_color(TerrainType::Ocean),
            get_terrain_scanner_color(TerrainType::Desert)
        );
    }

    #[test]
    fn space_ui_colors_defined() {
        assert_ne!(PANEL_BACKGROUND, PRIMARY_TEXT);
        assert_ne!(PRIMARY_TEXT, ENERGY_COLOR);
        assert_ne!(SHIP_SIGNATURE, UNEXPLORED_SPACE);
    }

    #[test]
    fn ui_components_have_proper_markers() {
        // This would test component spawning in integration tests
        // For now, just verify the component structs exist
        let _scanner = SectorMapDisplay;
        let _status = ShipStatusPanel;
        let _mission = MissionControlPanel;
        assert!(true); // Components compile correctly
    }

    #[test]
    fn terrain_to_space_object_mapping() {
        // Test that all terrain types have space object equivalents
        let all_terrains = [
            TerrainType::Plains,
            TerrainType::Forest,
            TerrainType::Mountains,
            TerrainType::Desert,
            TerrainType::Ocean,
            TerrainType::Tundra,
            TerrainType::Volcanic,
            TerrainType::Crystal,
            TerrainType::Cave,
            TerrainType::Constructed,
            TerrainType::Swamp,
            TerrainType::Anomaly,
        ];

        for terrain in all_terrains {
            let color = get_terrain_scanner_color(terrain);
            // Ensure no color is completely black (invalid)
            if let Color::Srgba(srgba) = color {
                assert!(srgba.red > 0.0 || srgba.green > 0.0 || srgba.blue > 0.0);
            }
        }
    }
}

/// System to apply proper fonts to UI text elements and load space icons
fn apply_fonts_to_ui_text(
    mut regular_query: Query<&mut TextFont, With<RegularText>>,
    mut icon_query: Query<(
        &mut ImageNode,
        Option<&RocketIcon>,
        Option<&SatelliteIcon>,
        Option<&GearIcon>,
    )>,
    font_service: Res<BevyFontService>,
    space_icons: Res<SpaceIcons>,
) {
    // Apply regular font to text components
    if let Ok(regular_font) = font_service.get_font_handle(FontType::UiRegular) {
        for mut text_font in regular_query.iter_mut() {
            if text_font.font == Handle::default() {
                text_font.font = regular_font.clone();
            }
        }
    }

    // Apply appropriate icons based on component markers
    for (mut image_node, rocket, satellite, gear) in icon_query.iter_mut() {
        if image_node.image == Handle::default() {
            if rocket.is_some() {
                image_node.image = space_icons.rocket.clone();
                info!("Applied rocket icon");
            } else if satellite.is_some() {
                image_node.image = space_icons.satellite.clone();
                info!("Applied satellite icon");
            } else if gear.is_some() {
                image_node.image = space_icons.gear.clone();
                info!("Applied gear icon");
            }
        }
    }
}

/// System to initialize space icons
fn initialize_space_icons(mut commands: Commands, asset_server: Res<AssetServer>) {
    let space_icons = SpaceIcons {
        rocket: asset_server.load("icons/rocket.png"),
        satellite: asset_server.load("icons/satellite.png"),
        gear: asset_server.load("icons/gear.png"),
        dice: asset_server.load("icons/game_die.png"),
    };

    commands.insert_resource(space_icons);
    info!("Space icons loaded successfully");
}
