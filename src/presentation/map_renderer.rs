//! Map Renderer - 3D Isometric visual representation for the tile-based world
//!
//! This module provides both 3D isometric visualization and console-based debugging
//! of the game world, showing the tile grid, terrain types, and player position.

use crate::domain::services::{MapService, TileCacheService, VisibilityLevel, VisibilityService};
use crate::domain::value_objects::terrain::TerrainType;
use crate::domain::value_objects::TileCoordinate;
use crate::infrastructure::bevy::resources::{MapResource, PlayerResource};
use bevy::prelude::*;

/// Plugin for 3D isometric map rendering functionality
pub struct MapRendererPlugin;

impl Plugin for MapRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
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
                update_console_map_system,
                render_on_map_generation_system,
                force_initial_render_system,
                initial_map_render_system,
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
                base_color: Color::srgb(0.4, 0.8, 0.3), // Green
                metallic: 0.0,
                perceptual_roughness: 0.8,
                ..default()
            }),
            forest: materials.add(StandardMaterial {
                base_color: Color::srgb(0.2, 0.6, 0.2), // Dark green
                metallic: 0.0,
                perceptual_roughness: 0.9,
                ..default()
            }),
            mountains: materials.add(StandardMaterial {
                base_color: Color::srgb(0.6, 0.5, 0.4), // Gray-brown
                metallic: 0.1,
                perceptual_roughness: 0.7,
                ..default()
            }),
            desert: materials.add(StandardMaterial {
                base_color: Color::srgb(0.9, 0.8, 0.5), // Sandy yellow
                metallic: 0.0,
                perceptual_roughness: 0.6,
                ..default()
            }),
            tundra: materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.9, 1.0), // Icy blue
                metallic: 0.2,
                perceptual_roughness: 0.3,
                ..default()
            }),
            ocean: materials.add(StandardMaterial {
                base_color: Color::srgb(0.2, 0.4, 0.8), // Deep blue
                metallic: 0.3,
                perceptual_roughness: 0.1,
                ..default()
            }),
            swamp: materials.add(StandardMaterial {
                base_color: Color::srgb(0.4, 0.5, 0.3), // Murky green
                metallic: 0.0,
                perceptual_roughness: 0.9,
                ..default()
            }),
            volcanic: materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.2, 0.1), // Red-orange
                metallic: 0.0,
                perceptual_roughness: 0.8,
                emissive: LinearRgba::new(0.3, 0.1, 0.0, 1.0),
                ..default()
            }),
            constructed: materials.add(StandardMaterial {
                base_color: Color::srgb(0.5, 0.5, 0.6), // Metallic gray
                metallic: 0.7,
                perceptual_roughness: 0.2,
                ..default()
            }),
            cave: materials.add(StandardMaterial {
                base_color: Color::srgb(0.3, 0.2, 0.2), // Dark brown
                metallic: 0.0,
                perceptual_roughness: 0.9,
                ..default()
            }),
            crystal: materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.6, 1.0), // Purple crystal
                metallic: 0.1,
                perceptual_roughness: 0.0,
                emissive: LinearRgba::new(0.2, 0.1, 0.3, 1.0),
                ..default()
            }),
            anomaly: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.0, 1.0), // Magenta
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
                perceptual_roughness: 0.9,
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
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            last_player_position: None,
            rendered_tiles: std::collections::HashSet::new(),
            tile_cache: TileCacheService::new(),
            initial_exploration_done: false,
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
    mut camera_query: Query<
        (&mut Transform, Entity),
        (With<IsometricCamera>, Without<PlayerMarker>),
    >,
    player_query: Query<(Entity, &Transform), (With<PlayerMarker>, Without<IsometricCamera>)>,
) {
    // Always try to find the current player and update camera
    if let Ok((player_entity, player_transform)) = player_query.single() {
        for (mut camera_transform, _camera_entity) in camera_query.iter_mut() {
            // Log camera following (only once by checking if we need to update position significantly)
            let current_pos = camera_transform.translation;
            let player_pos = player_transform.translation;
            let expected_pos = player_pos + Vec3::new(10.0, 15.0, 10.0);

            if (current_pos - expected_pos).length() > 5.0 {
                info!("📹 Camera now following player entity: {:?}", player_entity);
            }

            // Position camera at isometric angle relative to player for proper centering
            let camera_offset = Vec3::new(10.0, 15.0, 10.0);
            let target_camera_pos = player_pos + camera_offset;

            // Smooth camera movement with lower lerp factor to reduce trembling
            let distance = (target_camera_pos - current_pos).length();

            // Only update if distance is significant to prevent micro-movements
            if distance > 0.1 {
                camera_transform.translation = current_pos.lerp(target_camera_pos, 0.02);
                // Always look at the player to keep them centered
                camera_transform.look_at(player_pos, Vec3::Y);
            }
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
            MeshMaterial3d(match (tile.is_explored, visibility_level) {
                (true, VisibilityLevel::FullyVisible) => {
                    get_terrain_material(&terrain_materials, tile.terrain_type)
                }
                _ => terrain_materials.fog_overlay.clone(),
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
    mut player_query: Query<&mut Transform, With<PlayerMarker>>,
    existing_player: Query<Entity, With<PlayerMarker>>,
) {
    if !player_resource.has_player() {
        return;
    }

    let player_position = player_resource.player_position().unwrap_or_default();
    let world_pos = tile_to_world_position(player_position.x, player_position.y, player_position.z);
    let final_position = Vec3::new(world_pos.x, 1.0, world_pos.z); // Player above terrain

    // Update existing player or create new one
    if let Ok(mut transform) = player_query.single_mut() {
        // Immediate position update for tile synchronization
        transform.translation = final_position;
    } else {
        // Create player if doesn't exist
        for entity in existing_player.iter() {
            commands.entity(entity).despawn();
        }

        // Create player as a cylinder/capsule
        let player_mesh = meshes.add(Mesh::from(Cylinder::new(0.3, 1.5)));

        commands.spawn((
            Mesh3d(player_mesh),
            MeshMaterial3d(terrain_materials.player.clone()),
            Transform::from_translation(final_position),
            PlayerMarker,
            Name::new("Player"),
        ));
    }
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
            let material = match (is_explored, visibility_level) {
                (true, VisibilityLevel::FullyVisible) => {
                    get_terrain_material(terrain_materials, tile.terrain_type)
                }
                _ => terrain_materials.fog_overlay.clone(),
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
            let material = match (is_explored, visibility_level) {
                (true, VisibilityLevel::FullyVisible) => {
                    get_terrain_material(terrain_materials, tile.terrain_type)
                }
                _ => terrain_materials.fog_overlay.clone(),
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

/// Update console map display when player moves (kept for debugging)
fn update_console_map_system(map_resource: Res<MapResource>, player_resource: Res<PlayerResource>) {
    // Only render occasionally to avoid spam
    static mut LAST_POSITION: Option<(i32, i32)> = None;

    if !map_resource.has_map() || !player_resource.has_player() {
        return;
    }

    let player_position = player_resource.player_position().unwrap_or_default();
    let current_pos = (player_position.x, player_position.y);

    // Only render if player moved
    unsafe {
        if let Some(last_pos) = LAST_POSITION {
            if last_pos == current_pos {
                return;
            }
        }
        LAST_POSITION = Some(current_pos);
    }

    let map = map_resource.current_map().unwrap();

    // Render ASCII map around player (smaller for console)
    let view_radius = 3;
    println!(
        "\n🗺️ Debug Map View (Player at {}, {}):",
        player_position.x, player_position.y
    );
    println!("═══════════════════");

    for dy in -view_radius..=view_radius {
        print!("│");
        for dx in -view_radius..=view_radius {
            let grid_x = player_position.x + dx;
            let grid_y = player_position.y + dy;

            // Player position
            if dx == 0 && dy == 0 {
                print!("@");
                continue;
            }

            let tile_coord = crate::domain::value_objects::TileCoordinate::new(
                grid_x,
                grid_y,
                player_position.z,
            );

            if let Some(tile) = map.get_tile(&tile_coord) {
                let visibility_service = VisibilityService::new();
                let is_visible = visibility_service.is_tile_visible(player_position, tile_coord);

                if tile.is_explored() && is_visible {
                    print!("{}", get_terrain_symbol(tile.terrain_type));
                } else {
                    print!("░░"); // Unexplored or not visible
                }
            } else {
                print!("██"); // Unknown/ungenerated
            }
        }
        println!("│");
    }

    println!("═══════════════════");
    println!("@=Player | ^=Mountains | T=Forest | .=Plains | ~=Desert");
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
                    let terrain_symbol = get_terrain_symbol(tile.terrain_type);
                    let mut explored_tile = tile.clone();
                    explored_tile.explore();
                    map.set_tile(tile_coord, explored_tile);

                    info!(
                        "🔍 Explored new tile at ({}, {}) - {}",
                        tile_coord.x, tile_coord.y, terrain_symbol
                    );
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
