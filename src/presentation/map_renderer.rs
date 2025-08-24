//! Map Renderer - 3D Isometric visual representation for the tile-based world
//!
//! This module provides both 3D isometric visualization and console-based debugging
//! of the game world, showing the tile grid, terrain types, and player position.

use crate::domain::constants::get_terrain_render_color;
use crate::domain::services::{MapService, TileCacheService, VisibilityLevel, VisibilityService};
use crate::domain::value_objects::terrain::TerrainType;
use crate::domain::value_objects::TileCoordinate;
use crate::infrastructure::bevy::resources::{MapResource, PlayerResource};
use crate::presentation::audio_integration::TerrainChangeEvent;
use crate::presentation::movement::{CameraFollowsMovement, SmoothMovement, SmoothMovementPlugin};
use bevy::prelude::*;

/// Plugin for 3D isometric map rendering functionality
pub struct MapRendererPlugin;

impl Plugin for MapRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SmoothMovementPlugin)
            .add_systems(
                Startup,
                (setup_3d_camera_system, setup_terrain_materials_system),
            )
            .add_systems(
                Update,
                (
                    setup_camera_following_system,
                    mark_tiles_explored_system,
                    update_3d_map_system,
                    update_player_position_system,
                    render_on_map_generation_system,
                    force_initial_render_system,
                    initial_map_render_system,
                    detect_player_terrain_changes,
                )
                    .chain()
                    .after(crate::rpg_exploration_system),
            )
            .init_resource::<TerrainMaterials>()
            .init_resource::<RenderState>();
    }
}

/// 3D Camera setup for isometric view
#[derive(Component)]
pub struct IsometricCamera;

/// Player representation in 3D world
#[derive(Component)]
pub struct PlayerMarker;

/// Terrain tile representation in 3D world
#[derive(Component)]
pub struct TerrainTile {
    pub coordinate: TileCoordinate,
    pub terrain_type: TerrainType,
    pub is_explored: bool,
    pub visibility_level: VisibilityLevel,
}

/// Materials for different terrain types
#[derive(Resource)]
pub struct TerrainMaterials {
    pub plains: Handle<StandardMaterial>,
    pub forest: Handle<StandardMaterial>,
    pub mountains: Handle<StandardMaterial>,
    pub desert: Handle<StandardMaterial>,
    pub tundra: Handle<StandardMaterial>,
    pub ocean: Handle<StandardMaterial>,
    pub swamp: Handle<StandardMaterial>,
    pub volcanic: Handle<StandardMaterial>,
    pub constructed: Handle<StandardMaterial>,
    pub cave: Handle<StandardMaterial>,
    pub crystal: Handle<StandardMaterial>,
    pub anomaly: Handle<StandardMaterial>,
    pub player: Handle<StandardMaterial>,
    pub fog_overlay: Handle<StandardMaterial>,
}

impl FromWorld for TerrainMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();

        Self {
            plains: materials.add(StandardMaterial {
                base_color: get_terrain_render_color(TerrainType::Plains),
                metallic: 0.0,
                perceptual_roughness: 0.8,
                ..default()
            }),
            forest: materials.add(StandardMaterial {
                base_color: get_terrain_render_color(TerrainType::Forest),
                metallic: 0.0,
                perceptual_roughness: 0.9,
                ..default()
            }),
            mountains: materials.add(StandardMaterial {
                base_color: get_terrain_render_color(TerrainType::Mountains),
                metallic: 0.1,
                perceptual_roughness: 0.7,
                ..default()
            }),
            desert: materials.add(StandardMaterial {
                base_color: get_terrain_render_color(TerrainType::Desert),
                metallic: 0.0,
                perceptual_roughness: 0.6,
                ..default()
            }),
            tundra: materials.add(StandardMaterial {
                base_color: get_terrain_render_color(TerrainType::Tundra),
                metallic: 0.2,
                perceptual_roughness: 0.3,
                ..default()
            }),
            ocean: materials.add(StandardMaterial {
                base_color: get_terrain_render_color(TerrainType::Ocean),
                metallic: 0.3,
                perceptual_roughness: 0.1,
                ..default()
            }),
            swamp: materials.add(StandardMaterial {
                base_color: get_terrain_render_color(TerrainType::Swamp),
                metallic: 0.0,
                perceptual_roughness: 0.9,
                ..default()
            }),
            volcanic: materials.add(StandardMaterial {
                base_color: get_terrain_render_color(TerrainType::Volcanic),
                metallic: 0.0,
                perceptual_roughness: 0.8,
                emissive: LinearRgba::new(0.3, 0.1, 0.0, 1.0),
                ..default()
            }),
            constructed: materials.add(StandardMaterial {
                base_color: get_terrain_render_color(TerrainType::Constructed),
                metallic: 0.7,
                perceptual_roughness: 0.2,
                ..default()
            }),
            cave: materials.add(StandardMaterial {
                base_color: get_terrain_render_color(TerrainType::Cave),
                metallic: 0.0,
                perceptual_roughness: 0.9,
                ..default()
            }),
            crystal: materials.add(StandardMaterial {
                base_color: get_terrain_render_color(TerrainType::Crystal),
                metallic: 0.1,
                perceptual_roughness: 0.0,
                emissive: LinearRgba::new(0.2, 0.1, 0.3, 1.0),
                ..default()
            }),
            anomaly: materials.add(StandardMaterial {
                base_color: get_terrain_render_color(TerrainType::Anomaly),
                metallic: 0.5,
                perceptual_roughness: 0.3,
                emissive: LinearRgba::new(0.5, 0.0, 0.5, 1.0),
                ..default()
            }),
            player: materials.add(StandardMaterial {
                base_color: Color::srgb(0.0, 0.8, 1.0), // Cyan
                metallic: 0.3,
                perceptual_roughness: 0.4,
                emissive: LinearRgba::new(0.0, 0.2, 0.3, 1.0),
                ..default()
            }),
            fog_overlay: materials.add(StandardMaterial {
                base_color: Color::srgba(0.2, 0.2, 0.3, 0.7), // Semi-transparent dark blue
                alpha_mode: AlphaMode::Blend,
                metallic: 0.0,
                perceptual_roughness: 1.0,
                ..default()
            }),
        }
    }
}

/// Tracks rendering state to avoid unnecessary updates
#[derive(Resource)]
pub struct RenderState {
    pub last_player_position: Option<crate::domain::value_objects::Position3D>,
    pub rendered_tiles: std::collections::HashSet<(i32, i32, i32)>,
    pub tile_cache: TileCacheService,
    pub initial_exploration_done: bool,
    pub last_terrain_type: Option<TerrainType>,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            last_player_position: None,
            rendered_tiles: std::collections::HashSet::new(),
            tile_cache: TileCacheService::new(),
            initial_exploration_done: false,
            last_terrain_type: None,
        }
    }
}

/// Setup 3D isometric camera
fn setup_3d_camera_system(mut commands: Commands) {
    info!("🎨 Setting up 3D isometric camera and lighting");

    // Isometric camera position
    // Position it at an angle to create the isometric view
    let camera_position = Vec3::new(10.0, 15.0, 10.0);
    let look_at = Vec3::new(0.0, 0.0, 0.0);

    // Create 3D camera with isometric perspective - render first (order -1)
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(camera_position).looking_at(look_at, Vec3::Y),
        Projection::Perspective(PerspectiveProjection {
            fov: std::f32::consts::PI / 6.0, // 30 degrees for more isometric feel
            ..default()
        }),
        Camera {
            order: -1, // Render before UI
            ..default()
        },
        IsometricCamera,
        Name::new("Isometric Camera"),
    ));

    // Add directional light for better 3D visualization
    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            std::f32::consts::PI * -0.15,
            std::f32::consts::PI * -0.25,
        )),
    ));

    // Add ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
        affects_lightmapped_meshes: true,
    });

    info!("✅ 3D isometric camera and lighting setup complete");
}

/// System to setup and maintain camera following - runs continuously to handle player respawn
fn setup_camera_following_system(
    mut commands: Commands,
    mut camera_query: Query<
        (&mut Transform, Entity),
        (With<IsometricCamera>, Without<PlayerMarker>),
    >,
    player_query: Query<(Entity, &SmoothMovement), With<PlayerMarker>>,
) {
    // Always try to find the current player and setup camera following
    if let Ok((player_entity, smooth_movement)) = player_query.single() {
        for (mut camera_transform, camera_entity) in camera_query.iter_mut() {
            // Check if camera already has CameraFollowsMovement component
            commands
                .entity(camera_entity)
                .insert(CameraFollowsMovement {
                    target_entity: Some(player_entity),
                    offset: Vec3::new(10.0, 15.0, 10.0),
                    follow_speed: 2.0,
                });

            // Set initial camera position
            let player_pos = smooth_movement.current_position;
            let camera_offset = Vec3::new(10.0, 15.0, 10.0);
            let target_camera_pos = player_pos + camera_offset;

            camera_transform.translation = target_camera_pos;
            camera_transform.look_at(player_pos, Vec3::Y);
        }
    }
}

/// Setup terrain materials (handled by FromWorld trait)
fn setup_terrain_materials_system() {
    info!("🎨 Terrain materials loaded");
}

/// Initial map render system - renders map immediately when game starts
fn initial_map_render_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    terrain_materials: Res<TerrainMaterials>,
    mut render_state: ResMut<RenderState>,
    mut map_resource: ResMut<MapResource>,
    player_resource: Res<PlayerResource>,
) {
    // Render map when both map and player exist and no tiles are rendered yet
    if map_resource.has_map()
        && player_resource.has_player()
        && render_state.rendered_tiles.is_empty()
    {
        let player_position = player_resource.player_position().unwrap_or_default();

        // Explore plus pattern around starting position immediately
        let visibility_service = VisibilityService::new();
        let visible_coords = visibility_service.get_all_visible_coordinates(player_position);

        if let Some(map) = map_resource.current_map_mut() {
            for tile_coord in visible_coords {
                if let Some(tile) = map.get_tile(&tile_coord) {
                    if !tile.is_explored() {
                        let mut explored_tile = tile.clone();
                        explored_tile.explore();
                        map.set_tile(tile_coord, explored_tile);
                        info!(
                            "🔍 Initially explored tile at ({}, {})",
                            tile_coord.x, tile_coord.y
                        );
                    }
                }
            }
        }

        render_state.initial_exploration_done = true;
        info!("🗺️ Initial plus pattern explored around player");

        render_initial_map_around_player(
            &mut commands,
            &mut meshes,
            &terrain_materials,
            &map_resource,
            &mut render_state,
            player_position,
        );

        info!("🚀 FIRST TIME 3D map render with fog of war - map now visible!");
    }
}

/// Render map immediately when a new map is generated
fn render_on_map_generation_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    terrain_materials: Res<TerrainMaterials>,
    mut render_state: ResMut<RenderState>,
    map_resource: Res<MapResource>,
    player_resource: Res<PlayerResource>,
) {
    if !map_resource.has_map() || !player_resource.has_player() {
        return;
    }

    // Only render if we haven't rendered anything yet and the map was just generated
    if render_state.rendered_tiles.is_empty() {
        let player_position = player_resource.player_position().unwrap_or_default();

        render_initial_map_around_player(
            &mut commands,
            &mut meshes,
            &terrain_materials,
            &map_resource,
            &mut render_state,
            player_position,
        );

        info!("🗺️ Map rendered immediately after generation");
    }
}

/// Force initial render system - ensures map shows immediately when available
fn force_initial_render_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    terrain_materials: Res<TerrainMaterials>,
    mut render_state: ResMut<RenderState>,
    map_resource: Res<MapResource>,
    player_resource: Res<PlayerResource>,
) {
    // Force render if map exists but nothing is rendered yet
    if map_resource.has_map()
        && player_resource.has_player()
        && render_state.rendered_tiles.is_empty()
    {
        let player_position = player_resource.player_position().unwrap_or_default();

        // Force render the initial map immediately
        render_initial_map_around_player(
            &mut commands,
            &mut meshes,
            &terrain_materials,
            &map_resource,
            &mut render_state,
            player_position,
        );

        info!("🚀 FORCED initial 3D map render - map now visible!");
    }
}

/// Update 3D map representation
fn update_3d_map_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    terrain_materials: Res<TerrainMaterials>,
    mut render_state: ResMut<RenderState>,
    mut map_resource: ResMut<MapResource>,
    player_resource: Res<PlayerResource>,
    mut query: Query<(
        Entity,
        &mut MeshMaterial3d<StandardMaterial>,
        &mut TerrainTile,
    )>,
) {
    if !map_resource.has_map() || !player_resource.has_player() {
        return;
    }

    let player_position = player_resource.player_position().unwrap_or_default();

    // Only update if player moved or this is the first run
    if let Some(last_pos) = render_state.last_player_position {
        if last_pos == player_position {
            return; // Player hasn't moved, no need to update
        }
    }

    info!(
        "🎯 3D MAP RENDERER: Player moved from {:?} to {:?} - updating diamond pattern",
        render_state.last_player_position, player_position
    );

    // Update last position to prevent re-entry
    render_state.last_player_position = Some(player_position);

    // Generate missing tiles before rendering diamond pattern
    let seed = map_resource.current_map().unwrap().seed();
    let map_service = MapService::new(seed);

    // Generate tiles in mutable scope
    {
        if let Some(map) = map_resource.current_map_mut() {
            if let Err(e) = map_service.generate_tiles_around_player(map, player_position) {
                error!("Failed to generate tiles around player: {:?}", e);
            } else {
                info!("✅ Generated missing tiles around player for diamond pattern");
            }
        }
    }

    let map = map_resource.current_map().unwrap();

    // Update tile cache with new player position
    render_state
        .tile_cache
        .update_player_position(player_position);

    // Get all visible tiles (both fully visible and fogged)
    let visibility_service = VisibilityService::new();
    let all_visible_coords = visibility_service.get_all_visible_coordinates(player_position);
    let visible_set: std::collections::HashSet<_> = all_visible_coords.iter().collect();

    info!(
        "👁️ Diamond pattern: {} tiles around player at ({}, {})",
        all_visible_coords.len(),
        player_position.x,
        player_position.y
    );

    // Debug: Log first few visible coordinates to verify diamond pattern
    let visible_sample: Vec<String> = all_visible_coords
        .iter()
        .take(8)
        .map(|coord| format!("({}, {})", coord.x, coord.y))
        .collect();
    info!("🔍 Sample visible coords: [{}]", visible_sample.join(", "));

    // Debug: Show first few visible coordinates
    let sample_coords: Vec<String> = all_visible_coords
        .iter()
        .take(10)
        .map(|coord| format!("({}, {})", coord.x, coord.y))
        .collect();
    info!(
        "👁️ Diamond pattern: {} tiles around player at ({}, {})",
        all_visible_coords.len(),
        player_position.x,
        player_position.y
    );

    // Clear all existing tiles completely to prevent mismatch
    let all_tile_entities: Vec<Entity> = query.iter().map(|(entity, _, _)| entity).collect();

    for entity in all_tile_entities {
        commands.entity(entity).despawn();
    }

    // Reset rendered tiles tracking
    render_state.rendered_tiles.clear();

    // Force map generation for allRender all visible tiles fresh
    let mut new_tiles_count = 0;
    let mut skipped_tiles = Vec::new();

    for coord in all_visible_coords {
        let coord_tuple = (coord.x, coord.y, coord.z);

        // Get tile from map
        let tile = if let Some(existing_tile) = map.get_tile(&coord) {
            existing_tile
        } else {
            skipped_tiles.push(format!("({}, {})", coord.x, coord.y));
            continue;
        };

        new_tiles_count += 1;

        // Get visibility level for this tile
        let visibility_level = visibility_service.get_tile_visibility(player_position, coord);

        // Create 3D representation
        let world_pos = tile_to_world_position(coord.x, coord.y, coord.z);
        let height_offset = get_terrain_height_offset(tile.terrain_type);

        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(2.0, 0.2, 2.0))),
            MeshMaterial3d(if tile.is_explored {
                // Show terrain for all explored tiles (no fog on explored areas)
                get_terrain_material(&terrain_materials, tile.terrain_type)
            } else {
                // Show fog for unexplored tiles
                terrain_materials.fog_overlay.clone()
            }),
            Transform::from_translation(Vec3::new(world_pos.x, height_offset, world_pos.z)),
            TerrainTile {
                coordinate: coord,
                terrain_type: tile.terrain_type,
                is_explored: tile.is_explored,
                visibility_level,
            },
            Name::new(format!("Tile_{}_{}_{}", coord.x, coord.y, coord.z)),
        ));

        render_state.rendered_tiles.insert(coord_tuple);
    }

    info!(
        "✨ Rendered {} tiles for fresh diamond pattern",
        new_tiles_count
    );

    if !skipped_tiles.is_empty() {
        info!(
            "⚠️ Skipped {} tiles (not in map): [{}]",
            skipped_tiles.len(),
            skipped_tiles.join(", ")
        );
    }

    // Debug: Log rendered tile coordinates
    let rendered_sample: Vec<String> = render_state
        .rendered_tiles
        .iter()
        .take(8)
        .map(|(x, y, _z)| format!("({}, {})", x, y))
        .collect();
    info!("🏗️ Sample rendered tiles: [{}]", rendered_sample.join(", "));

    info!(
        "🗺️ Map update complete: {} total rendered tiles around player ({}, {}, {})",
        render_state.rendered_tiles.len(),
        player_position.x,
        player_position.y,
        player_position.z
    );
}

/// Update 3D player position
fn update_player_position_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    terrain_materials: Res<TerrainMaterials>,
    player_resource: Res<PlayerResource>,
    player_query: Query<(Entity, &SmoothMovement), With<PlayerMarker>>,
    existing_player: Query<Entity, With<PlayerMarker>>,
) {
    if !player_resource.has_player() {
        return;
    }

    // Only create player entity if it doesn't exist
    // NEVER update position - smooth movement system handles all position updates
    if player_query.is_empty() {
        let player_position = player_resource.player_position().unwrap_or_default();
        let position_3d = crate::domain::value_objects::position::Position3D::new(
            player_position.x,
            player_position.y,
            player_position.z,
        );

        // Create player if doesn't exist
        for entity in existing_player.iter() {
            commands.entity(entity).despawn();
        }

        // Create player as a cylinder/capsule with smooth movement
        let player_mesh = meshes.add(Mesh::from(Cylinder::new(0.3, 1.5)));
        let smooth_movement = SmoothMovement::new(position_3d);
        let world_pos = crate::presentation::movement::tile_to_world_position(position_3d);
        let final_position = Vec3::new(world_pos.x, 1.0, world_pos.z); // Player above terrain

        commands.spawn((
            Mesh3d(player_mesh),
            MeshMaterial3d(terrain_materials.player.clone()),
            Transform::from_translation(final_position),
            smooth_movement,
            PlayerMarker,
            Name::new("Player"),
        ));
    }
    // If player already exists, do NOTHING - smooth movement system controls position
}

/// Render initial map around player
fn render_initial_map_around_player(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    terrain_materials: &TerrainMaterials,
    map_resource: &MapResource,
    render_state: &mut RenderState,
    player_position: crate::domain::value_objects::Position3D,
) {
    let map = map_resource.current_map().unwrap();
    let cube_mesh = meshes.add(Mesh::from(Cuboid::new(1.0, 0.2, 1.0)));

    info!(
        "🗺️ Rendering initial map with fog of war around ({}, {})",
        player_position.x, player_position.y
    );

    // Debug: Log diamond pattern coordinates for initial render
    let visibility_service = VisibilityService::new();
    let all_visible_coords = visibility_service.get_all_visible_coordinates(player_position);
    let fully_visible_coords = visibility_service.get_fully_visible_coordinates(player_position);
    let fogged_coords = visibility_service.get_fogged_visible_coordinates(player_position);

    let visible_coords_str: Vec<String> = all_visible_coords
        .iter()
        .map(|coord| format!("({}, {})", coord.x, coord.y))
        .collect();
    info!(
        "🔍 INITIAL RENDER: Diamond pattern coordinates: [{}]",
        visible_coords_str.join(", ")
    );

    let fully_visible_str: Vec<String> = fully_visible_coords
        .iter()
        .map(|coord| format!("({}, {})", coord.x, coord.y))
        .collect();
    info!(
        "✨ INITIAL RENDER: Fully visible (green) tiles: [{}]",
        fully_visible_str.join(", ")
    );

    let fogged_str: Vec<String> = fogged_coords
        .iter()
        .map(|coord| format!("({}, {})", coord.x, coord.y))
        .collect();
    info!(
        "👁️ INITIAL RENDER: Fogged visible (gray) tiles: [{}]",
        fogged_str.join(", ")
    );

    // Only render tiles that are actually visible according to visibility service
    let visibility_service = VisibilityService::new();
    let visible_coords = visibility_service.get_all_visible_coordinates(player_position);

    for tile_coord in visible_coords {
        if let Some(tile) = map.get_tile(&tile_coord) {
            let world_pos = tile_to_world_position(tile_coord.x, tile_coord.y, tile_coord.z);
            let height_offset = get_terrain_height_offset(tile.terrain_type);
            let final_position = Vec3::new(world_pos.x, height_offset, world_pos.z);

            // Apply fog of war: only show terrain for explored tiles that are visible
            let is_explored = tile.is_explored;
            let visibility_level =
                visibility_service.get_tile_visibility(player_position, tile_coord);
            let material = if is_explored {
                // Show terrain for all explored tiles (no fog on explored areas)
                get_terrain_material(&terrain_materials, tile.terrain_type)
            } else {
                // Show fog for unexplored tiles
                terrain_materials.fog_overlay.clone()
            };

            commands.spawn((
                Mesh3d(cube_mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(final_position),
                TerrainTile {
                    coordinate: tile_coord,
                    terrain_type: tile.terrain_type,
                    is_explored: tile.is_explored,
                    visibility_level,
                },
                Name::new(format!("Terrain_{}_{}", tile_coord.x, tile_coord.y)),
            ));

            render_state
                .rendered_tiles
                .insert((tile_coord.x, tile_coord.y, tile_coord.z));
        }
    }

    info!(
        "🗺️ Initial map render complete - {} tiles with fog of war",
        render_state.rendered_tiles.len()
    );
}

/// Render new tiles around player if needed
fn render_new_tiles_around_player(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    terrain_materials: &TerrainMaterials,
    map_resource: &MapResource,
    render_state_mut: &mut RenderState,
    player_position: crate::domain::value_objects::Position3D,
) {
    let map = map_resource.current_map().unwrap();
    let cube_mesh = meshes.add(Mesh::from(Cuboid::new(1.0, 0.2, 1.0)));

    // Only render tiles that are actually visible according to visibility service
    let visibility_service = VisibilityService::new();
    let visible_coords = visibility_service.get_all_visible_coordinates(player_position);

    for tile_coord in visible_coords {
        // Skip if tile already exists
        if render_state_mut
            .rendered_tiles
            .contains(&(tile_coord.x, tile_coord.y, tile_coord.z))
        {
            continue;
        }

        if let Some(tile) = map.get_tile(&tile_coord) {
            let world_pos = tile_to_world_position(tile_coord.x, tile_coord.y, tile_coord.z);
            let height_offset = get_terrain_height_offset(tile.terrain_type);
            let final_position = Vec3::new(world_pos.x, height_offset, world_pos.z);

            // Apply visibility and fog based on exploration status
            let is_explored = tile.is_explored;
            let visibility_level =
                visibility_service.get_tile_visibility(player_position, tile_coord);
            let material = if is_explored {
                // Show terrain for all explored tiles (no fog on explored areas)
                get_terrain_material(&terrain_materials, tile.terrain_type)
            } else {
                // Show fog for unexplored tiles
                terrain_materials.fog_overlay.clone()
            };

            commands.spawn((
                Mesh3d(cube_mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(final_position),
                TerrainTile {
                    coordinate: tile_coord,
                    terrain_type: tile.terrain_type,
                    is_explored: tile.is_explored,
                    visibility_level,
                },
                Name::new(format!("Terrain_{}_{}", tile_coord.x, tile_coord.y)),
            ));

            render_state_mut
                .rendered_tiles
                .insert((tile_coord.x, tile_coord.y, tile_coord.z));
        }
    }
}

/// Convert tile coordinates to 3D world position
fn tile_to_world_position(tile_x: i32, tile_y: i32, tile_z: i32) -> Vec3 {
    // Convert tile grid to isometric-like 3D coordinates
    let x = (tile_x as f32) * 2.2; // Proper spacing for 2.0 unit wide tiles
    let z = (tile_y as f32) * 2.2;
    let y = (tile_z as f32) * 0.3; // Layer height

    Vec3::new(x, y, z)
}

/// Get material for terrain type
fn get_terrain_material(
    materials: &TerrainMaterials,
    terrain_type: TerrainType,
) -> Handle<StandardMaterial> {
    match terrain_type {
        TerrainType::Plains => materials.plains.clone(),
        TerrainType::Forest => materials.forest.clone(),
        TerrainType::Mountains => materials.mountains.clone(),
        TerrainType::Desert => materials.desert.clone(),
        TerrainType::Tundra => materials.tundra.clone(),
        TerrainType::Ocean => materials.ocean.clone(),
        TerrainType::Swamp => materials.swamp.clone(),
        TerrainType::Volcanic => materials.volcanic.clone(),
        TerrainType::Constructed => materials.constructed.clone(),
        TerrainType::Cave => materials.cave.clone(),
        TerrainType::Crystal => materials.crystal.clone(),
        TerrainType::Anomaly => materials.anomaly.clone(),
    }
}

/// Get height offset for different terrain types
fn get_terrain_height_offset(terrain_type: TerrainType) -> f32 {
    match terrain_type {
        TerrainType::Plains => 0.0,
        TerrainType::Forest => 0.1,
        TerrainType::Mountains => 0.8,
        TerrainType::Desert => -0.1,
        TerrainType::Tundra => 0.0,
        TerrainType::Ocean => -0.5,
        TerrainType::Swamp => -0.2,
        TerrainType::Volcanic => 0.3,
        TerrainType::Constructed => 0.2,
        TerrainType::Cave => -0.3,
        TerrainType::Crystal => 0.4,
        TerrainType::Anomaly => 0.6,
    }
}

/// Get symbol for terrain type (console debug only)
fn get_terrain_symbol(terrain_type: TerrainType) -> &'static str {
    match terrain_type {
        TerrainType::Plains => ".",
        TerrainType::Forest => "T",
        TerrainType::Mountains => "^",
        TerrainType::Desert => "~",
        TerrainType::Tundra => "i",
        TerrainType::Swamp => "s",
        TerrainType::Ocean => "O",
        TerrainType::Volcanic => "V",
        TerrainType::Anomaly => "!",
        TerrainType::Constructed => "#",
        TerrainType::Cave => "c",
        TerrainType::Crystal => "*",
    }
}

/// System to mark tiles as explored when player visits them (plus pattern visibility)
pub fn mark_tiles_explored_system(
    player_resource: Res<PlayerResource>,
    mut map_resource: ResMut<MapResource>,
    mut render_state: ResMut<RenderState>,
) {
    if !player_resource.has_player() || !map_resource.has_map() {
        return;
    }

    let player_position = player_resource.player_position().unwrap_or_default();

    // Only explore if player has moved to a new position
    let should_explore = match render_state.last_player_position {
        Some(last_pos) => last_pos != player_position,
        None => true, // First time
    };

    if !should_explore {
        return;
    }

    let visibility_service = VisibilityService::new();
    let visible_coords = visibility_service.get_all_visible_coordinates(player_position);

    if let Some(map) = map_resource.current_map_mut() {
        for tile_coord in visible_coords {
            if let Some(tile) = map.get_tile(&tile_coord) {
                if !tile.is_explored() {
                    let mut explored_tile = tile.clone();
                    explored_tile.explore();
                    map.set_tile(tile_coord, explored_tile);
                }
            }
        }
    }

    // Note: last_player_position is managed by update_3d_map_system to prevent conflicts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terrain_symbols_are_different() {
        let plains_symbol = get_terrain_symbol(TerrainType::Plains);
        let ocean_symbol = get_terrain_symbol(TerrainType::Ocean);
        let mountain_symbol = get_terrain_symbol(TerrainType::Mountains);

        // Symbols should be different
        assert_ne!(plains_symbol, ocean_symbol);
        assert_ne!(plains_symbol, mountain_symbol);
        assert_ne!(ocean_symbol, mountain_symbol);
    }

    #[test]
    fn all_terrain_types_have_symbols() {
        // Test that all terrain types have symbols
        assert!(!get_terrain_symbol(TerrainType::Plains).is_empty());
        assert!(!get_terrain_symbol(TerrainType::Forest).is_empty());
        assert!(!get_terrain_symbol(TerrainType::Mountains).is_empty());
        assert!(!get_terrain_symbol(TerrainType::Desert).is_empty());
    }

    #[test]
    fn tile_to_world_position_conversion() {
        let world_pos = tile_to_world_position(0, 0, 0);
        assert_eq!(world_pos, Vec3::new(0.0, 0.0, 0.0));

        let world_pos = tile_to_world_position(1, 1, 1);
        assert_eq!(world_pos, Vec3::new(2.2, 0.3, 2.2));
    }

    #[test]
    fn terrain_height_offsets() {
        assert_eq!(get_terrain_height_offset(TerrainType::Plains), 0.0);
        assert!(get_terrain_height_offset(TerrainType::Mountains) > 0.0);
        assert!(get_terrain_height_offset(TerrainType::Ocean) < 0.0);
    }
}

/// System to detect terrain changes and trigger ambient audio updates
fn detect_player_terrain_changes(
    map_resource: Res<MapResource>,
    player_resource: Res<PlayerResource>,
    mut render_state: ResMut<RenderState>,
    mut terrain_events: EventWriter<TerrainChangeEvent>,
) {
    // Only check if we have both map and player data
    if !player_resource.has_player() || !map_resource.has_map() {
        return;
    }

    let current_position = player_resource.player_position().unwrap_or_default();
    let tile_coord =
        TileCoordinate::new(current_position.x, current_position.y, current_position.z);

    // Get the current terrain type
    if let Some(map) = &map_resource.current_map {
        if let Some(tile) = map.get_tile(&tile_coord) {
            let current_terrain = tile.terrain_type;

            // Check if terrain has changed
            let terrain_changed = match render_state.last_terrain_type {
                Some(last_terrain) => {
                    std::mem::discriminant(&last_terrain)
                        != std::mem::discriminant(&current_terrain)
                }
                None => true, // First time detection
            };

            if terrain_changed {
                info!(
                    "🌍 Player moved to {} terrain at ({}, {}, {})",
                    crate::domain::constants::get_terrain_name(&current_terrain),
                    current_position.x,
                    current_position.y,
                    current_position.z
                );

                // Send terrain change event for ambient audio
                terrain_events.write(TerrainChangeEvent {
                    new_terrain: current_terrain,
                    fade_duration: Some(1.5), // Smooth 1.5 second transition
                });

                // Update our tracked terrain type
                render_state.last_terrain_type = Some(current_terrain);
            }
        }
    }
}
