//! Simple Game UI - Direct window overlay approach
//!
//! This module provides a very simple UI that renders directly as window overlays
//! without complex z-index layering, ensuring visibility over the 3D world.

use crate::domain::value_objects::terrain::TerrainType;
use crate::infrastructure::bevy::resources::{GameStatsResource, MapResource, PlayerResource};
use crate::presentation::RpgAppState;
use bevy::prelude::*;

/// Plugin for simple game UI functionality
pub struct GameUIPlugin;

impl Plugin for GameUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_simple_ui)
            .add_systems(Update, (update_ui_text, auto_start_exploration_system));
    }
}

/// Component markers for UI elements
#[derive(Component)]
pub struct UIMapText;

#[derive(Component)]
pub struct UIStatsText;

#[derive(Component)]
pub struct UIControlsText;

#[derive(Component)]
pub struct UIMapGrid;

#[derive(Component)]
pub struct UIMapTile {
    pub grid_x: i32,
    pub grid_y: i32,
}

/// Simple UI colors
struct UIColors;
impl UIColors {
    const BACKGROUND: Color = Color::srgba(0.0, 0.0, 0.0, 0.7);
    const TEXT_COLOR: Color = Color::WHITE;
    const ACCENT_COLOR: Color = Color::srgb(0.3, 0.8, 1.0);

    // Terrain colors for mini-map
    const PLAYER_COLOR: Color = Color::srgb(1.0, 1.0, 0.0); // Yellow
    const PLAINS_COLOR: Color = Color::srgb(0.4, 0.8, 0.2); // Green
    const FOREST_COLOR: Color = Color::srgb(0.2, 0.5, 0.1); // Dark Green
    const MOUNTAINS_COLOR: Color = Color::srgb(0.6, 0.6, 0.6); // Gray
    const DESERT_COLOR: Color = Color::srgb(0.9, 0.8, 0.4); // Sandy
    const OCEAN_COLOR: Color = Color::srgb(0.2, 0.4, 0.8); // Blue
    const TUNDRA_COLOR: Color = Color::srgb(0.8, 0.9, 1.0); // Light Blue
    const VOLCANIC_COLOR: Color = Color::srgb(0.9, 0.3, 0.1); // Red
    const CRYSTAL_COLOR: Color = Color::srgb(0.8, 0.4, 0.9); // Purple
    const CAVE_COLOR: Color = Color::srgb(0.3, 0.3, 0.3); // Dark Gray
    const CONSTRUCTED_COLOR: Color = Color::srgb(0.7, 0.7, 0.3); // Brown
    const SWAMP_COLOR: Color = Color::srgb(0.4, 0.6, 0.3); // Muddy Green
    const ANOMALY_COLOR: Color = Color::srgb(1.0, 0.2, 0.8); // Bright Pink
    const FOG_COLOR: Color = Color::srgb(0.5, 0.5, 0.5); // Medium Gray
    const UNKNOWN_COLOR: Color = Color::srgb(0.2, 0.2, 0.2); // Dark
}

/// Setup very simple UI overlay
fn setup_simple_ui(mut commands: Commands) {
    info!("Setting up simple direct UI overlay");

    // Root UI node that covers the entire screen
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        })
        .with_children(|parent| {
            // Top-left corner: Map container
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(10.0),
                    top: Val::Px(10.0),
                    width: Val::Px(300.0),
                    height: Val::Px(220.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BackgroundColor(UIColors::BACKGROUND),
            ))
            .with_children(|parent| {
                // Map title
                parent.spawn((
                    Text::new("MAP - Player at (0, 0)"),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(UIColors::TEXT_COLOR),
                    Node {
                        margin: UiRect::bottom(Val::Px(5.0)),
                        ..default()
                    },
                    UIMapText,
                ));

                // Mini-map grid (5x5 colored tiles)
                parent.spawn((
                    Node {
                        width: Val::Px(150.0),
                        height: Val::Px(150.0),
                        display: Display::Grid,
                        grid_template_columns: RepeatedGridTrack::flex(5, 1.0),
                        grid_template_rows: RepeatedGridTrack::flex(5, 1.0),
                        column_gap: Val::Px(2.0),
                        row_gap: Val::Px(2.0),
                        margin: UiRect::bottom(Val::Px(5.0)),
                        ..default()
                    },
                    UIMapGrid,
                ))
                .with_children(|parent| {
                    // Create 25 tiles (5x5 grid)
                    for y in 0..5 {
                        for x in 0..5 {
                            parent.spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                BackgroundColor(UIColors::FOG_COLOR),
                                UIMapTile {
                                    grid_x: x - 2,
                                    grid_y: y - 2
                                },
                            ));
                        }
                    }
                });

                // Legend
                parent.spawn((
                    Text::new("P=Player, G=Plains, F=Forest, M=Mountain\nD=Desert, O=Ocean, C=Crystal, ?=Fog"),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(UIColors::TEXT_COLOR),
                ));
            });

            // Top-right corner: Stats
            parent.spawn((
                Text::new("STATS: Loading..."),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(UIColors::ACCENT_COLOR),
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(10.0),
                    top: Val::Px(10.0),
                    width: Val::Px(200.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(UIColors::BACKGROUND),
                UIStatsText,
            ));

            // Bottom center: Controls
            parent.spawn((
                Text::new("CONTROLS: WASD=Move, SPACE=Dice, B=Base, Q=Quests, I=Inventory"),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(UIColors::TEXT_COLOR),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    bottom: Val::Px(10.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    justify_self: JustifySelf::Center,
                    ..default()
                },
                BackgroundColor(UIColors::BACKGROUND),
                UIControlsText,
            ));
        });

    info!("Simple UI overlay created");
}

/// Update all UI text content
fn update_ui_text(
    map_resource: Res<MapResource>,
    player_resource: Res<PlayerResource>,
    game_stats: Res<GameStatsResource>,
    mut map_query: Query<&mut Text, (With<UIMapText>, Without<UIStatsText>)>,
    mut stats_query: Query<&mut Text, (With<UIStatsText>, Without<UIMapText>)>,
    mut tile_query: Query<(&mut BackgroundColor, &UIMapTile)>,
) {
    // Update map title
    if let Ok(mut map_text) = map_query.single_mut() {
        if map_resource.has_map() && player_resource.has_player() {
            let player_pos = player_resource.player_position().unwrap_or_default();
            **map_text = format!("MAP - Player at ({}, {})", player_pos.x, player_pos.y);
        } else {
            **map_text = "MAP: Initializing...".to_string();
        }
    }

    // Update colored map tiles
    if map_resource.has_map() && player_resource.has_player() {
        let player_pos = player_resource.player_position().unwrap_or_default();
        let map = map_resource.current_map().unwrap();

        for (mut bg_color, tile_info) in tile_query.iter_mut() {
            let world_x = player_pos.x + tile_info.grid_x;
            let world_y = player_pos.y + tile_info.grid_y;

            if tile_info.grid_x == 0 && tile_info.grid_y == 0 {
                // Player tile - bright yellow
                bg_color.0 = UIColors::PLAYER_COLOR;
            } else {
                let tile_coord = crate::domain::value_objects::TileCoordinate::new(
                    world_x,
                    world_y,
                    player_pos.z,
                );

                if let Some(tile) = map.get_tile(&tile_coord) {
                    if tile.is_explored() {
                        bg_color.0 = get_terrain_color(tile.terrain_type);
                    } else {
                        bg_color.0 = UIColors::FOG_COLOR; // Fog
                    }
                } else {
                    bg_color.0 = UIColors::UNKNOWN_COLOR; // Unknown
                }
            }
        }
    }

    // Update stats text
    if let Ok(mut stats_text) = stats_query.single_mut() {
        if player_resource.has_player() {
            let player = player_resource.get_player().unwrap();
            let stats_display = format!(
                "PLAYER STATS\nLevel: {}\nMove: {}/{}\nAct: {}/{}\n\nGAME PROGRESS\nExplored: {}\nDice Rolls: {}\nSuccess: {:.0}%",
                player.level(),
                player.movement_points(),
                player.max_movement_points(),
                player.action_points(),
                player.max_action_points(),
                game_stats.tiles_explored,
                game_stats.dice_rolls_made,
                game_stats.success_rate() * 100.0
            );
            **stats_text = stats_display;
        } else {
            **stats_text = "STATS: Loading...".to_string();
        }
    }
}

/// Auto-start exploration mode
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
            info!("Auto-starting exploration mode!");
            *timer = 0.0;
        }
    } else if *current_state != RpgAppState::MainMenu {
        *timer = 0.0;
    }
}

/// Get color for terrain type
fn get_terrain_color(terrain: TerrainType) -> Color {
    match terrain {
        TerrainType::Plains => UIColors::PLAINS_COLOR,
        TerrainType::Forest => UIColors::FOREST_COLOR,
        TerrainType::Mountains => UIColors::MOUNTAINS_COLOR,
        TerrainType::Desert => UIColors::DESERT_COLOR,
        TerrainType::Ocean => UIColors::OCEAN_COLOR,
        TerrainType::Tundra => UIColors::TUNDRA_COLOR,
        TerrainType::Volcanic => UIColors::VOLCANIC_COLOR,
        TerrainType::Crystal => UIColors::CRYSTAL_COLOR,
        TerrainType::Cave => UIColors::CAVE_COLOR,
        TerrainType::Constructed => UIColors::CONSTRUCTED_COLOR,
        TerrainType::Swamp => UIColors::SWAMP_COLOR,
        TerrainType::Anomaly => UIColors::ANOMALY_COLOR,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terrain_colors_are_valid() {
        assert_ne!(
            get_terrain_color(TerrainType::Plains),
            get_terrain_color(TerrainType::Forest)
        );
        assert_ne!(
            get_terrain_color(TerrainType::Ocean),
            get_terrain_color(TerrainType::Desert)
        );
    }

    #[test]
    fn ui_colors_defined() {
        assert_ne!(UIColors::BACKGROUND, UIColors::TEXT_COLOR);
        assert_ne!(UIColors::TEXT_COLOR, UIColors::ACCENT_COLOR);
    }
}
