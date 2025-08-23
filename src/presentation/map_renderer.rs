//! Map Renderer - 3D Isometric visual representation for the tile-based world
//!
//! This module provides both 3D isometric visualization and console-based debugging
//! of the game world, showing the tile grid, terrain types, and player position.

use crate::domain::value_objects::terrain::TerrainType;
use crate::infrastructure::bevy::resources::{MapResource, PlayerResource};
use bevy::prelude::*;

/// Plugin for 3D isometric map rendering functionality
pub struct MapRendererPlugin;

impl Plugin for MapRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                setup_3d_camera_system,
                setup_terrain_materials_system,
                initial_map_render_system,
            ),
        )
        .add_systems(
            Update,
            (
                setup_camera_following_system,
                update_3d_map_system,
                update_player_position_system,
                update_console_map_system,
                mark_tiles_explored_system,
                render_on_map_generation_system,
                force_initial_render_system,
            ),
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
    pub coordinate: crate::domain::value_objects::TileCoordinate,
    pub terrain_type: TerrainType,
    pub is_explored: bool,
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
        }
    }
}

/// Tracks rendering state to avoid unnecessary updates
#[derive(Resource, Default)]
pub struct RenderState {
    pub last_player_position: Option<(i32, i32, i32)>,
    pub rendered_tiles: std::collections::HashSet<(i32, i32, i32)>,
    pub fog_material: Option<Handle<StandardMaterial>>,
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
    mut materials: ResMut<Assets<StandardMaterial>>,
    terrain_materials: Res<TerrainMaterials>,
    mut render_state: ResMut<RenderState>,
    map_resource: Res<MapResource>,
    player_resource: Res<PlayerResource>,
) {
    // Only initialize fog material once
    if render_state.fog_material.is_none() {
        let fog_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.2, 0.3), // Dark bluish for unexplored
            metallic: 0.0,
            perceptual_roughness: 0.9,
            ..default()
        });
        render_state.fog_material = Some(fog_material);
        info!("🌫️ Fog of war material initialized");
    }

    // Render map when both map and player exist and no tiles are rendered yet
    if map_resource.has_map()
        && player_resource.has_player()
        && render_state.rendered_tiles.is_empty()
    {
        let player_position = player_resource.player_position().unwrap_or_default();
        let fog_material = render_state.fog_material.as_ref().unwrap().clone();

        render_initial_map_around_player(
            &mut commands,
            &mut meshes,
            &terrain_materials,
            &fog_material,
            &map_resource,
            &mut render_state,
            player_position,
        );

        info!("🗺️ Initial 3D map rendered around starting position");
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
        if let Some(fog_material) = render_state.fog_material.clone() {
            let player_position = player_resource.player_position().unwrap_or_default();

            render_initial_map_around_player(
                &mut commands,
                &mut meshes,
                &terrain_materials,
                &fog_material,
                &map_resource,
                &mut render_state,
                player_position,
            );

            info!("🗺️ Map rendered immediately after generation");
        }
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
        && render_state.fog_material.is_some()
    {
        let player_position = player_resource.player_position().unwrap_or_default();
        let fog_material = render_state.fog_material.clone().unwrap();

        // Force render the initial map immediately
        render_initial_map_around_player(
            &mut commands,
            &mut meshes,
            &terrain_materials,
            &fog_material,
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
    map_resource: Res<MapResource>,
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
    let current_pos = (player_position.x, player_position.y, player_position.z);

    // Only update if player moved
    if let Some(last_pos) = render_state.last_player_position {
        if last_pos == current_pos {
            return;
        }
    }

    render_state.last_player_position = Some(current_pos);
    let map = map_resource.current_map().unwrap();

    // Update existing tiles' exploration status and materials
    for (entity, mut material, mut terrain_tile) in query.iter_mut() {
        let coord = terrain_tile.coordinate;
        if let Some(tile) = map.get_tile(&coord) {
            let is_explored = tile.is_explored();

            // Update exploration status if changed
            if terrain_tile.is_explored != is_explored {
                terrain_tile.is_explored = is_explored;

                if is_explored {
                    // Reveal the terrain
                    *material = MeshMaterial3d(get_terrain_material(
                        &terrain_materials,
                        terrain_tile.terrain_type,
                    ));
                } else if let Some(fog_mat) = &render_state.fog_material {
                    // Apply fog of war
                    *material = MeshMaterial3d(fog_mat.clone());
                }
            }

            // Check if tile is too far from player (fog distance = 6 tiles)
            let distance = ((coord.x - player_position.x)
                .abs()
                .max((coord.y - player_position.y).abs())) as u32;
            if distance > 6 {
                // Apply fog of war for distant tiles
                if let Some(fog_mat) = &render_state.fog_material {
                    *material = MeshMaterial3d(fog_mat.clone());
                }
            }
        }
    }

    // Render new tiles if needed
    render_new_tiles_around_player(
        &mut commands,
        &mut meshes,
        &terrain_materials,
        &map_resource,
        &mut render_state,
        player_position,
    );

    info!(
        "🗺️  Updated map around player position ({}, {}, {})",
        player_position.x, player_position.y, player_position.z
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
        // Smooth movement animation
        transform.translation = transform.translation.lerp(final_position, 0.1);
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
    fog_material: &Handle<StandardMaterial>,
    map_resource: &MapResource,
    render_state: &mut RenderState,
    player_position: crate::domain::value_objects::Position3D,
) {
    let map = map_resource.current_map().unwrap();
    let render_radius = 8;
    let cube_mesh = meshes.add(Mesh::from(Cuboid::new(1.0, 0.2, 1.0)));

    info!(
        "🗺️ Rendering initial map with fog of war around ({}, {})",
        player_position.x, player_position.y
    );

    for dy in -render_radius..=render_radius {
        for dx in -render_radius..=render_radius {
            let tile_x = player_position.x + dx;
            let tile_y = player_position.y + dy;
            let tile_z = player_position.z;

            let tile_coord =
                crate::domain::value_objects::TileCoordinate::new(tile_x, tile_y, tile_z);

            if let Some(tile) = map.get_tile(&tile_coord) {
                let world_pos = tile_to_world_position(tile_x, tile_y, tile_z);
                let height_offset = get_terrain_height_offset(tile.terrain_type);
                let final_position = Vec3::new(world_pos.x, height_offset, world_pos.z);

                // Apply fog of war: only show terrain for explored tiles within view distance
                let distance = (dx.abs().max(dy.abs())) as u32;
                let is_explored = tile.is_explored();
                let material = if is_explored && distance <= 6 {
                    get_terrain_material(terrain_materials, tile.terrain_type)
                } else {
                    fog_material.clone()
                };

                commands.spawn((
                    Mesh3d(cube_mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::from_translation(final_position),
                    TerrainTile {
                        coordinate: tile_coord,
                        terrain_type: tile.terrain_type,
                        is_explored,
                    },
                    Name::new(format!("Terrain_{}_{}", tile_x, tile_y)),
                ));

                render_state.rendered_tiles.insert((tile_x, tile_y, tile_z));
            }
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
    let render_radius = 8;
    let cube_mesh = meshes.add(Mesh::from(Cuboid::new(1.0, 0.2, 1.0)));

    // For now, just check if tile is already in rendered_tiles set
    // This prevents duplicate rendering

    for dy in -render_radius..=render_radius {
        for dx in -render_radius..=render_radius {
            let tile_x = player_position.x + dx;
            let tile_y = player_position.y + dy;
            let tile_z = player_position.z;

            // Skip if tile already exists
            if render_state_mut
                .rendered_tiles
                .contains(&(tile_x, tile_y, tile_z))
            {
                continue;
            }

            let tile_coord =
                crate::domain::value_objects::TileCoordinate::new(tile_x, tile_y, tile_z);

            if let Some(tile) = map.get_tile(&tile_coord) {
                let world_pos = tile_to_world_position(tile_x, tile_y, tile_z);
                let height_offset = get_terrain_height_offset(tile.terrain_type);
                let final_position = Vec3::new(world_pos.x, height_offset, world_pos.z);

                // Apply fog of war based on distance and exploration
                let distance = (dx.abs().max(dy.abs())) as u32;
                let is_explored = tile.is_explored();
                let material = if is_explored && distance <= 6 {
                    get_terrain_material(terrain_materials, tile.terrain_type)
                } else if let Some(fog_mat) = &render_state_mut.fog_material {
                    fog_mat.clone()
                } else {
                    get_terrain_material(terrain_materials, tile.terrain_type)
                };

                commands.spawn((
                    Mesh3d(cube_mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::from_translation(final_position),
                    TerrainTile {
                        coordinate: tile_coord,
                        terrain_type: tile.terrain_type,
                        is_explored,
                    },
                    Name::new(format!("Terrain_{}_{}", tile_x, tile_y)),
                ));

                render_state_mut
                    .rendered_tiles
                    .insert((tile_x, tile_y, tile_z));
            }
        }
    }
}

/// Convert tile coordinates to 3D world position
fn tile_to_world_position(tile_x: i32, tile_y: i32, tile_z: i32) -> Vec3 {
    // Convert tile grid to isometric-like 3D coordinates
    let x = (tile_x as f32) * 1.2; // Slight spacing between tiles
    let z = (tile_y as f32) * 1.2;
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
                if tile.is_explored() {
                    print!("{}", get_terrain_symbol(tile.terrain_type));
                } else {
                    print!("░░"); // Unexplored but exists
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

/// System to mark tiles as explored when player visits them
pub fn mark_tiles_explored_system(
    player_resource: Res<PlayerResource>,
    mut map_resource: ResMut<MapResource>,
) {
    if !player_resource.has_player() || !map_resource.has_map() {
        return;
    }

    let player_position = player_resource.player_position().unwrap_or_default();
    let tile_coord = crate::domain::value_objects::TileCoordinate::new(
        player_position.x,
        player_position.y,
        player_position.z,
    );

    if let Some(map) = map_resource.current_map_mut() {
        if let Some(tile) = map.get_tile(&tile_coord) {
            if !tile.is_explored() {
                let terrain_symbol = get_terrain_symbol(tile.terrain_type);
                let mut explored_tile = tile.clone();
                explored_tile.explore();
                map.set_tile(tile_coord, explored_tile);

                info!(
                    "🔍 Explored new tile at ({}, {}) - {}",
                    player_position.x, player_position.y, terrain_symbol
                );
            }
        }
    }
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
        assert_eq!(world_pos, Vec3::new(1.2, 0.3, 1.2));
    }

    #[test]
    fn terrain_height_offsets() {
        assert_eq!(get_terrain_height_offset(TerrainType::Plains), 0.0);
        assert!(get_terrain_height_offset(TerrainType::Mountains) > 0.0);
        assert!(get_terrain_height_offset(TerrainType::Ocean) < 0.0);
    }
}
