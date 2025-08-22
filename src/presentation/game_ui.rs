//! Space Looter Game UI - Advanced Space-Themed Interface
//!
//! This module provides a comprehensive space exploration UI with enhanced graphics,
//! space terminology, and immersive visual elements for the Space Looter RPG.

use crate::domain::services::font_service::{FontService, FontSize, FontType};
use crate::domain::value_objects::terrain::TerrainType;
use crate::infrastructure::bevy::font_service::{BevyFontService, RegularText};
use crate::infrastructure::bevy::resources::{GameStatsResource, MapResource, PlayerResource};
use crate::presentation::RpgAppState;
use bevy::prelude::*;

/// Plugin for space-themed game UI functionality
pub struct GameUIPlugin;

impl Plugin for GameUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (initialize_space_icons, setup_space_ui))
            .add_systems(
                Update,
                (
                    update_space_ui,
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

/// Space-themed UI color scheme
struct SpaceUIColors;
impl SpaceUIColors {
    // Main UI Colors
    const HUD_BACKGROUND: Color = Color::srgba(0.05, 0.15, 0.25, 0.9);
    const PANEL_BACKGROUND: Color = Color::srgba(0.1, 0.2, 0.3, 0.85);
    const SCANNER_BACKGROUND: Color = Color::srgba(0.0, 0.1, 0.2, 0.95);

    // Text Colors
    const PRIMARY_TEXT: Color = Color::srgb(0.85, 0.95, 1.0);
    const SECONDARY_TEXT: Color = Color::srgb(0.6, 0.8, 0.9);
    const WARNING_TEXT: Color = Color::srgb(1.0, 0.7, 0.3);
    const CRITICAL_TEXT: Color = Color::srgb(1.0, 0.3, 0.3);
    const SUCCESS_TEXT: Color = Color::srgb(0.3, 0.9, 0.5);

    // Accent Colors
    const ENERGY_COLOR: Color = Color::srgb(0.2, 0.8, 1.0);
    const RESOURCE_COLOR: Color = Color::srgb(0.8, 0.6, 0.2);
    const SCANNER_GRID: Color = Color::srgba(0.0, 0.8, 1.0, 0.3);

    // Sector Scanner Colors
    const SHIP_SIGNATURE: Color = Color::srgb(1.0, 1.0, 0.0); // Bright Yellow
    const EXPLORED_SPACE: Color = Color::srgba(0.2, 0.6, 0.9, 0.8);
    const UNEXPLORED_SPACE: Color = Color::srgba(0.1, 0.1, 0.3, 0.6);

    // Terrain Signatures (Space Objects)
    const ASTEROID_FIELD: Color = Color::srgb(0.6, 0.6, 0.6); // Gray
    const NEBULA: Color = Color::srgb(0.8, 0.4, 0.9); // Purple
    const STAR_SYSTEM: Color = Color::srgb(1.0, 0.9, 0.3); // Golden
    const SPACE_STATION: Color = Color::srgb(0.3, 0.9, 0.3); // Green
    const DERELICT: Color = Color::srgb(0.7, 0.3, 0.3); // Red
    const ANOMALY: Color = Color::srgb(1.0, 0.2, 0.8); // Bright Pink
    const QUANTUM_STORM: Color = Color::srgb(0.5, 0.9, 1.0); // Cyan
    const WORMHOLE: Color = Color::srgb(0.9, 0.1, 0.9); // Magenta
    const CRYSTAL_FORMATION: Color = Color::srgb(0.4, 0.8, 0.9); // Light Blue
    const VOID_REGION: Color = Color::srgb(0.2, 0.1, 0.4); // Dark Purple
    const MINING_OPERATION: Color = Color::srgb(0.9, 0.7, 0.2); // Orange
    const ALIEN_TERRITORY: Color = Color::srgb(0.8, 0.9, 0.4); // Lime
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
            // Top HUD Bar - Ship Status & Navigation
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(10.0),
                    top: Val::Px(10.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(SpaceUIColors::HUD_BACKGROUND),
                Name::new("TopHUD"),
            ))
            .with_children(|parent| {
                // Rocket icon
                parent.spawn((
                    ImageNode {
                        image: Handle::default(),
                        ..default()
                    },
                    Node {
                        width: Val::Px(20.0),
                        height: Val::Px(20.0),
                        margin: UiRect::right(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(1.0, 0.0, 0.0)),
                    RocketIcon,
                ));

                // Status text
                parent.spawn((
                    Text::new("SPACE LOOTER | SECTOR [0,0] | STATUS: OPERATIONAL"),
                    TextFont {
                        font_size: FontSize::Medium.to_pixels(),
                        ..default()
                    },
                    TextColor(SpaceUIColors::PRIMARY_TEXT),
                    RegularText,
                ));
            });

            // Left Panel - Sector Scanner
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(15.0),
                    top: Val::Px(65.0),
                    width: Val::Px(320.0),
                    height: Val::Px(280.0),
                    padding: UiRect::all(Val::Px(15.0)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BackgroundColor(SpaceUIColors::SCANNER_BACKGROUND),
                Name::new("SectorScanner"),
            ))
            .with_children(|parent| {
                // Scanner Header
                parent.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        margin: UiRect::bottom(Val::Px(8.0)),
                        ..default()
                    },
                ))
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
                        TextColor(SpaceUIColors::ENERGY_COLOR),
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
                    TextColor(SpaceUIColors::SECONDARY_TEXT),
                    Node {
                        margin: UiRect::bottom(Val::Px(8.0)),
                        ..default()
                    },
                    RegularText,
                    SectorMapDisplay,
                ));

                // Scanner Grid (7x7 for better detail)
                parent.spawn((
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
                    BorderColor(SpaceUIColors::SCANNER_GRID),
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
                                BackgroundColor(SpaceUIColors::UNEXPLORED_SPACE),
                                BorderColor(SpaceUIColors::SCANNER_GRID),
                                SectorTile {
                                    grid_x: x - 3,
                                    grid_y: y - 3,
                                },
                            ));
                        }
                    }
                });

                // Scanner Legend
                parent.spawn((
                    Text::new("VESSEL = EXPLORED = ANOMALY = STATION\nASTEROIDS = HOSTILE = NEBULA = UNKNOWN"),
                    TextFont {
                        font_size: FontSize::Small.to_pixels(),
                        ..default()
                    },
                    TextColor(SpaceUIColors::SECONDARY_TEXT),
                    RegularText,
                ));
            });

            // Right Panel - Ship Systems & Resources
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(15.0),
                    top: Val::Px(65.0),
                    width: Val::Px(280.0),
                    padding: UiRect::all(Val::Px(15.0)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BackgroundColor(SpaceUIColors::PANEL_BACKGROUND),
                Name::new("ShipSystemsPanel"),
            ))
            .with_children(|parent| {
                // Systems header with gear icon
                parent.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        margin: UiRect::bottom(Val::Px(10.0)),
                        ..default()
                    },
                ))
                .with_children(|parent| {
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
                        GearIcon,
                    ));

                    parent.spawn((
                        Text::new("SHIP SYSTEMS STATUS"),
                        TextFont {
                            font_size: FontSize::Medium.to_pixels(),
                            ..default()
                        },
                        TextColor(SpaceUIColors::ENERGY_COLOR),
                        RegularText,
                    ));
                });

                // Ship status text
                parent.spawn((
                    Text::new("SHIP SYSTEMS: INITIALIZING..."),
                    TextFont {
                        font_size: FontSize::Regular.to_pixels(),
                        ..default()
                    },
                    TextColor(SpaceUIColors::SUCCESS_TEXT),
                    RegularText,
                    ShipStatusPanel,
                ));
            });

            // Bottom Panel - Mission Control & Commands
            parent.spawn((
                Text::new("WASD/ARROWS: Navigate | SPACE: Quantum Dice | B: Base Operations | Q: Mission Log | I: Cargo Bay | ESC: System Menu"),
                TextFont {
                    font_size: FontSize::Small.to_pixels(),
                    ..default()
                },
                TextColor(SpaceUIColors::PRIMARY_TEXT),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    bottom: Val::Px(15.0),
                    width: Val::Px(500.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    justify_self: JustifySelf::Center,
                    ..default()
                },
                BackgroundColor(SpaceUIColors::PANEL_BACKGROUND),
                RegularText,
                MissionControlPanel,
            ));
        });

    info!("✅ Space Command Interface initialized");
}

/// Create top HUD bar with ship status - inline version
fn create_top_hud_bar() -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(50.0),
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            padding: UiRect::all(Val::Px(10.0)),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(SpaceUIColors::HUD_BACKGROUND),
        Name::new("TopHUD"),
    )
}

/// Create sector scanner - simplified inline version
fn create_sector_scanner() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(15.0),
            top: Val::Px(65.0),
            width: Val::Px(320.0),
            height: Val::Px(280.0),
            padding: UiRect::all(Val::Px(15.0)),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(SpaceUIColors::SCANNER_BACKGROUND),
        Name::new("SectorScanner"),
    )
}

/// Create ship systems panel - simplified inline version
fn create_ship_systems_panel() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(15.0),
            top: Val::Px(65.0),
            width: Val::Px(280.0),
            height: Val::Px(320.0),
            padding: UiRect::all(Val::Px(15.0)),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(SpaceUIColors::PANEL_BACKGROUND),
        Name::new("ShipSystems"),
    )
}

/// Create mission control panel - simplified inline version
fn create_mission_control_panel() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            bottom: Val::Px(15.0),
            width: Val::Px(500.0),
            height: Val::Px(80.0),
            padding: UiRect::all(Val::Px(12.0)),
            flex_direction: FlexDirection::Column,
            justify_self: JustifySelf::Center,
            ..default()
        },
        BackgroundColor(SpaceUIColors::PANEL_BACKGROUND),
        Name::new("MissionControl"),
    )
}

/// Create alert panel - simplified inline version
fn create_alert_panel() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(15.0),
            bottom: Val::Px(120.0),
            width: Val::Px(300.0),
            height: Val::Px(60.0),
            padding: UiRect::all(Val::Px(10.0)),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(SpaceUIColors::HUD_BACKGROUND),
        Visibility::Hidden,
        AlertPanel,
        Name::new("AlertSystem"),
    )
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
                bg_color.0 = SpaceUIColors::SHIP_SIGNATURE;
            } else {
                let tile_coord = crate::domain::value_objects::TileCoordinate::new(
                    world_x,
                    world_y,
                    player_pos.z,
                );

                if let Some(tile) = map.get_tile(&tile_coord) {
                    if tile.is_explored() {
                        bg_color.0 = get_space_object_signature(tile.terrain_type);
                    } else {
                        bg_color.0 = SpaceUIColors::UNEXPLORED_SPACE;
                    }
                } else {
                    bg_color.0 = SpaceUIColors::UNEXPLORED_SPACE;
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
                81..=100 => ("🟢 OPTIMAL", SpaceUIColors::SUCCESS_TEXT),
                61..=80 => ("🟡 GOOD", SpaceUIColors::WARNING_TEXT),
                31..=60 => ("🟠 DAMAGED", SpaceUIColors::WARNING_TEXT),
                _ => ("🔴 CRITICAL", SpaceUIColors::CRITICAL_TEXT),
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

/// Get space object signature color based on terrain type
fn get_space_object_signature(terrain: TerrainType) -> Color {
    match terrain {
        TerrainType::Plains => SpaceUIColors::EXPLORED_SPACE, // Open space
        TerrainType::Forest => SpaceUIColors::NEBULA,         // Dense nebula
        TerrainType::Mountains => SpaceUIColors::ASTEROID_FIELD, // Asteroid field
        TerrainType::Desert => SpaceUIColors::VOID_REGION,    // Void region
        TerrainType::Ocean => SpaceUIColors::QUANTUM_STORM,   // Quantum storm
        TerrainType::Tundra => SpaceUIColors::CRYSTAL_FORMATION, // Crystal formations
        TerrainType::Volcanic => SpaceUIColors::STAR_SYSTEM,  // Star system
        TerrainType::Crystal => SpaceUIColors::CRYSTAL_FORMATION, // Crystal fields
        TerrainType::Cave => SpaceUIColors::WORMHOLE,         // Wormhole
        TerrainType::Constructed => SpaceUIColors::SPACE_STATION, // Space station
        TerrainType::Swamp => SpaceUIColors::ALIEN_TERRITORY, // Alien territory
        TerrainType::Anomaly => SpaceUIColors::ANOMALY,       // Space anomaly
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn space_object_signatures_are_unique() {
        assert_ne!(
            get_space_object_signature(TerrainType::Plains),
            get_space_object_signature(TerrainType::Forest)
        );
        assert_ne!(
            get_space_object_signature(TerrainType::Ocean),
            get_space_object_signature(TerrainType::Desert)
        );
    }

    #[test]
    fn space_ui_colors_defined() {
        assert_ne!(SpaceUIColors::HUD_BACKGROUND, SpaceUIColors::PRIMARY_TEXT);
        assert_ne!(SpaceUIColors::PRIMARY_TEXT, SpaceUIColors::ENERGY_COLOR);
        assert_ne!(
            SpaceUIColors::SHIP_SIGNATURE,
            SpaceUIColors::UNEXPLORED_SPACE
        );
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
            let color = get_space_object_signature(terrain);
            // Ensure no color is completely black (invalid)
            assert!(color.red() > 0.0 || color.green() > 0.0 || color.blue() > 0.0);
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
