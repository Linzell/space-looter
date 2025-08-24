//! Map Service - Handles procedural map generation and management
//!
//! This service encapsulates all map generation logic, including terrain
//! generation, resource node placement, and tile cache management.
//! It follows DDD principles by keeping generation logic separate from
//! the Map entity itself.

use crate::domain::{
    constants,
    entities::map::{Map, MapTile, ResourceNode},
    value_objects::{
        resources::ResourceType,
        terrain::{Elevation, TerrainType},
        EntityId, Position3D, TileCoordinate,
    },
    DomainResult,
};

/// Service responsible for map generation and management operations
#[derive(Debug, Clone)]
pub struct MapService {
    /// Random seed for consistent generation
    seed: u64,
}

/// Biome types that group terrain logically
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BiomeType {
    Temperate,   // Plains, Forest
    Arid,        // Desert, Volcanic
    Cold,        // Tundra, Mountains
    Wetlands,    // Swamp, Ocean
    Underground, // Cave, Crystal
    Artificial,  // Constructed, Anomaly
}

/// Biome configuration with terrain probabilities
#[derive(Debug, Clone)]
pub struct BiomeConfig {
    pub primary_terrain: TerrainType,
    pub secondary_terrain: TerrainType,
    pub rare_terrain: Option<TerrainType>,
    pub primary_weight: u32,
    pub secondary_weight: u32,
    pub rare_weight: u32,
}

impl MapService {
    /// Create a new map service with a random seed
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// Generate tiles around a player position for movement
    /// This ensures the player always has passable tiles nearby
    pub fn generate_tiles_around_player(
        &self,
        map: &mut Map,
        player_position: Position3D,
    ) -> DomainResult<Vec<TileCoordinate>> {
        let mut generated_tiles = Vec::new();
        let generation_radius =
            constants::FOGGED_VISIBLE_RADIUS + constants::TILE_GENERATION_BUFFER;

        // Generate tiles in buffer zone around player
        let needed_positions = player_position.positions_within_distance(generation_radius);

        for pos in needed_positions {
            let coord = TileCoordinate::from(pos);
            if !map.tiles().contains_key(&coord) {
                let tile = self.generate_single_tile(pos)?;
                map.set_tile(coord, tile);
                generated_tiles.push(coord);

                // Generate resource node if conditions are met
                if self.should_place_resource_node(&pos) {
                    if let Some(resource_node) = self.generate_resource_node(&pos)? {
                        map.add_resource_node(pos, resource_node);
                    }
                }
            }
        }

        Ok(generated_tiles)
    }

    /// Generate a chunk of tiles around a center position
    pub fn generate_chunk(
        &self,
        map: &mut Map,
        chunk_center: Position3D,
        chunk_size: i32,
    ) -> DomainResult<()> {
        for x in -chunk_size / 2..=chunk_size / 2 {
            for y in -chunk_size / 2..=chunk_size / 2 {
                let pos = Position3D::new(chunk_center.x + x, chunk_center.y + y, chunk_center.z);
                let coord = TileCoordinate::from(pos);

                // Only generate if tile doesn't exist
                if !map.tiles().contains_key(&coord) {
                    let tile = self.generate_single_tile(pos)?;
                    map.set_tile(coord, tile);

                    // Generate resource nodes
                    if self.should_place_resource_node(&pos) {
                        if let Some(resource_node) = self.generate_resource_node(&pos)? {
                            map.add_resource_node(pos, resource_node);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Generate a single tile at a specific position
    fn generate_single_tile(&self, position: Position3D) -> DomainResult<MapTile> {
        let terrain = self.generate_terrain_type(position);
        let elevation = self.generate_elevation(position);

        Ok(MapTile::new(terrain, elevation, false))
    }

    /// Generate terrain type based on position and biome zones
    fn generate_terrain_type(&self, position: Position3D) -> TerrainType {
        let biome = self.determine_biome(position);
        let config = self.get_biome_config(biome);
        let hash = self.hash_position(position);

        let total_weight = config.primary_weight + config.secondary_weight + config.rare_weight;
        let roll = hash % (total_weight as u64);

        if roll < config.primary_weight as u64 {
            config.primary_terrain
        } else if roll < (config.primary_weight + config.secondary_weight) as u64 {
            config.secondary_terrain
        } else if let Some(rare) = config.rare_terrain {
            rare
        } else {
            config.primary_terrain
        }
    }

    /// Determine biome type based on position
    fn determine_biome(&self, position: Position3D) -> BiomeType {
        let _hash = self.hash_position(position);

        // Create biome zones based on position
        let biome_x = (position.x as f64 / constants::MAP_CHUNK_SIZE as f64).floor() as i32;
        let biome_y = (position.y as f64 / constants::MAP_CHUNK_SIZE as f64).floor() as i32;

        let biome_hash = self.hash_biome_coordinate(biome_x, biome_y);

        // Generate larger biome zones (16x16 tiles each)
        match biome_hash % 100 {
            0..=35 => BiomeType::Temperate, // 35% - Most common, good for gameplay
            36..=55 => BiomeType::Arid,     // 20% - Desert regions
            56..=70 => BiomeType::Cold,     // 15% - Mountain/tundra regions
            71..=80 => BiomeType::Wetlands, // 10% - Swamps and water
            81..=90 => BiomeType::Underground, // 10% - Caves and crystals
            _ => BiomeType::Artificial,     // 10% - Constructed/anomaly
        }
    }

    /// Get biome configuration for terrain generation
    fn get_biome_config(&self, biome: BiomeType) -> BiomeConfig {
        match biome {
            BiomeType::Temperate => BiomeConfig {
                primary_terrain: TerrainType::Plains,
                secondary_terrain: TerrainType::Forest,
                rare_terrain: Some(TerrainType::Constructed),
                primary_weight: 60,   // 60% plains
                secondary_weight: 35, // 35% forest
                rare_weight: 5,       // 5% constructed
            },
            BiomeType::Arid => BiomeConfig {
                primary_terrain: TerrainType::Desert,
                secondary_terrain: TerrainType::Plains,
                rare_terrain: Some(TerrainType::Volcanic),
                primary_weight: 70,   // 70% desert
                secondary_weight: 25, // 25% plains (oases)
                rare_weight: 5,       // 5% volcanic
            },
            BiomeType::Cold => BiomeConfig {
                primary_terrain: TerrainType::Tundra,
                secondary_terrain: TerrainType::Mountains,
                rare_terrain: Some(TerrainType::Crystal),
                primary_weight: 60,   // 60% tundra
                secondary_weight: 35, // 35% mountains
                rare_weight: 5,       // 5% crystal formations
            },
            BiomeType::Wetlands => BiomeConfig {
                primary_terrain: TerrainType::Swamp,
                secondary_terrain: TerrainType::Plains,
                rare_terrain: Some(TerrainType::Ocean),
                primary_weight: 65,   // 65% swamp
                secondary_weight: 30, // 30% plains (dry areas)
                rare_weight: 5,       // 5% ocean (small lakes)
            },
            BiomeType::Underground => BiomeConfig {
                primary_terrain: TerrainType::Cave,
                secondary_terrain: TerrainType::Mountains,
                rare_terrain: Some(TerrainType::Crystal),
                primary_weight: 70,   // 70% caves
                secondary_weight: 20, // 20% mountains (surface)
                rare_weight: 10,      // 10% crystal caves
            },
            BiomeType::Artificial => BiomeConfig {
                primary_terrain: TerrainType::Constructed,
                secondary_terrain: TerrainType::Plains,
                rare_terrain: Some(TerrainType::Anomaly),
                primary_weight: 50,   // 50% constructed
                secondary_weight: 40, // 40% plains (ruins/abandoned)
                rare_weight: 10,      // 10% anomalies
            },
        }
    }

    /// Hash biome coordinates for consistent biome zones
    fn hash_biome_coordinate(&self, biome_x: i32, biome_y: i32) -> u64 {
        let mut hash = self.seed;
        hash = hash.wrapping_mul(73).wrapping_add(biome_x as u64);
        hash = hash.wrapping_mul(79).wrapping_add(biome_y as u64);
        hash
    }

    /// Generate elevation based on position
    fn generate_elevation(&self, position: Position3D) -> Elevation {
        // Simple elevation based on distance from origin with some variation
        let base_elevation = (position.x.abs() + position.y.abs()) / 10;
        let variation = (self.hash_position(position) % 5) as i32 - 2;
        let final_elevation = (base_elevation + variation).max(0);

        Elevation::new(final_elevation).unwrap_or(Elevation::sea_level())
    }

    /// Check if a resource node should be placed at this position
    fn should_place_resource_node(&self, position: &Position3D) -> bool {
        // Place resource nodes at specific intervals with some randomness
        let hash = self.hash_position(*position);
        (position.x + position.y) % 7 == 0 && hash % 3 == 0
    }

    /// Generate a resource node at a position
    fn generate_resource_node(&self, position: &Position3D) -> DomainResult<Option<ResourceNode>> {
        let hash = self.hash_position(*position);

        // Determine resource type based on position hash
        let resource_type = match hash % 4 {
            0 => ResourceType::Metal,
            1 => ResourceType::Energy,
            2 => ResourceType::Food,
            _ => ResourceType::Technology,
        };

        // Generate terrain at this position to get appropriate resource properties
        let terrain = self.generate_terrain_type(*position);

        if let Some(node_props) = terrain.generate_resource_node(resource_type) {
            let capacity = 50 + (hash % 100) as u32; // 50-149 capacity
            let current_amount = capacity; // Start full

            let node =
                ResourceNode::new(EntityId::generate(), node_props, capacity, current_amount);

            Ok(Some(node))
        } else {
            Ok(None)
        }
    }

    /// Hash a position for pseudo-random generation
    fn hash_position(&self, position: Position3D) -> u64 {
        // Simple hash combining position coordinates with seed
        let mut hash = self.seed;
        hash = hash.wrapping_mul(31).wrapping_add(position.x as u64);
        hash = hash.wrapping_mul(31).wrapping_add(position.y as u64);
        hash = hash.wrapping_mul(31).wrapping_add(position.z as u64);
        hash
    }

    /// Validate that generated terrain maintains connectivity
    pub fn validate_connectivity(
        &self,
        map: &Map,
        center: Position3D,
        radius: u32,
    ) -> DomainResult<bool> {
        // Check that there are sufficient passable tiles around the center
        let positions = center.positions_within_distance(radius);
        let passable_count = positions.iter().filter(|pos| map.is_passable(pos)).count();

        let total_count = positions.len();
        let passable_ratio = passable_count as f32 / total_count as f32;

        // Require at least 80% passable tiles for good connectivity
        Ok(passable_ratio >= 0.8)
    }

    /// Get generation statistics for debugging
    pub fn get_generation_stats(
        &self,
        map: &Map,
        area: Position3D,
        radius: u32,
    ) -> GenerationStats {
        let positions = area.positions_within_distance(radius);
        let mut stats = GenerationStats::default();

        for pos in positions {
            let coord = TileCoordinate::from(pos);
            if let Some(tile) = map.get_tile(&coord) {
                stats.total_tiles += 1;

                match tile.terrain_type {
                    TerrainType::Plains => stats.plains += 1,
                    TerrainType::Forest => stats.forest += 1,
                    TerrainType::Desert => stats.desert += 1,
                    TerrainType::Mountains => stats.mountains += 1,
                    TerrainType::Tundra => stats.tundra += 1,
                    TerrainType::Swamp => stats.swamp += 1,
                    TerrainType::Ocean => stats.ocean += 1,
                    TerrainType::Cave => stats.cave += 1,
                    TerrainType::Crystal => stats.crystal += 1,
                    TerrainType::Constructed => stats.constructed += 1,
                    TerrainType::Volcanic => stats.volcanic += 1,
                    TerrainType::Anomaly => stats.anomaly += 1,
                }

                if tile.terrain_type.is_passable() {
                    stats.passable += 1;
                }
            }

            if map.get_resource_node(&pos).is_some() {
                stats.resource_nodes += 1;
            }
        }

        stats
    }

    /// Get biome statistics for an area
    pub fn get_biome_stats(&self, area: Position3D, radius: u32) -> BiomeStats {
        let positions = area.positions_within_distance(radius);
        let mut stats = BiomeStats::default();

        for pos in positions {
            let biome = self.determine_biome(pos);
            stats.total_tiles += 1;

            match biome {
                BiomeType::Temperate => stats.temperate += 1,
                BiomeType::Arid => stats.arid += 1,
                BiomeType::Cold => stats.cold += 1,
                BiomeType::Wetlands => stats.wetlands += 1,
                BiomeType::Underground => stats.underground += 1,
                BiomeType::Artificial => stats.artificial += 1,
            }
        }

        stats
    }
}

impl Default for MapService {
    fn default() -> Self {
        Self::new(12345) // Default seed
    }
}

/// Statistics about generated map content
#[derive(Debug, Clone, Default)]
pub struct GenerationStats {
    pub total_tiles: u32,
    pub passable: u32,
    pub plains: u32,
    pub forest: u32,
    pub desert: u32,
    pub mountains: u32,
    pub tundra: u32,
    pub swamp: u32,
    pub ocean: u32,
    pub cave: u32,
    pub crystal: u32,
    pub constructed: u32,
    pub volcanic: u32,
    pub anomaly: u32,
    pub resource_nodes: u32,
}

/// Statistics about biome distribution
#[derive(Debug, Clone, Default)]
pub struct BiomeStats {
    pub total_tiles: u32,
    pub temperate: u32,
    pub arid: u32,
    pub cold: u32,
    pub wetlands: u32,
    pub underground: u32,
    pub artificial: u32,
}

impl BiomeStats {
    /// Get biome diversity index (0.0 = single biome, 1.0 = perfectly diverse)
    pub fn diversity_index(&self) -> f32 {
        if self.total_tiles == 0 {
            return 0.0;
        }

        let biomes = [
            self.temperate,
            self.arid,
            self.cold,
            self.wetlands,
            self.underground,
            self.artificial,
        ];

        let non_zero_biomes = biomes.iter().filter(|&&count| count > 0).count();

        if non_zero_biomes <= 1 {
            0.0
        } else {
            (non_zero_biomes - 1) as f32 / 5.0 // 6 total biomes - 1 = max diversity
        }
    }
}

impl GenerationStats {
    /// Get the percentage of passable tiles
    pub fn passable_percentage(&self) -> f32 {
        if self.total_tiles == 0 {
            0.0
        } else {
            (self.passable as f32 / self.total_tiles as f32) * 100.0
        }
    }

    /// Get the resource node density (nodes per 100 tiles)
    pub fn resource_density(&self) -> f32 {
        if self.total_tiles == 0 {
            0.0
        } else {
            (self.resource_nodes as f32 / self.total_tiles as f32) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_map() -> Map {
        Map::new(EntityId::generate(), "Test".to_string(), 12345).unwrap()
    }

    #[test]
    fn map_service_creation() {
        let service = MapService::new(12345);
        assert_eq!(service.seed, 12345);
    }

    #[test]
    fn generate_tiles_around_player() {
        let service = MapService::new(12345);
        let mut map = create_test_map();
        let player_pos = Position3D::origin();

        let generated = service
            .generate_tiles_around_player(&mut map, player_pos)
            .unwrap();

        // Should generate tiles around player
        assert!(!generated.is_empty());

        // Check that player position has a tile
        let player_coord = TileCoordinate::from(player_pos);
        assert!(map.get_tile(&player_coord).is_some());
    }

    #[test]
    fn chunk_generation() {
        let service = MapService::new(12345);
        let mut map = create_test_map();
        let center = Position3D::origin();

        service.generate_chunk(&mut map, center, 4).unwrap();

        // Should have generated tiles
        assert!(!map.tiles().is_empty());
    }

    #[test]
    fn terrain_generation_is_mostly_passable() {
        let service = MapService::new(12345);
        let mut map = create_test_map();
        let center = Position3D::origin();

        service.generate_chunk(&mut map, center, 10).unwrap();

        let stats = service.get_generation_stats(&map, center, 5);

        // Should have high percentage of passable terrain (Ocean is only impassable)
        assert!(stats.passable_percentage() >= 90.0);
    }

    #[test]
    fn biome_generation_creates_zones() {
        let service = MapService::new(12345);
        let center = Position3D::origin();

        // Test that same biome coordinates produce consistent biomes
        let biome1 = service.determine_biome(Position3D::new(0, 0, 0));
        let biome2 = service.determine_biome(Position3D::new(1, 1, 0)); // Same chunk
        let biome3 = service.determine_biome(Position3D::new(20, 20, 0)); // Different chunk

        // Positions within same chunk should have same biome
        assert_eq!(biome1, biome2);

        // Can't guarantee different biomes, but biome generation should work
        let _different_biome = biome3; // Just test it doesn't panic
    }

    #[test]
    fn biome_stats_diversity() {
        let service = MapService::new(12345);
        let center = Position3D::origin();

        // Generate larger area to get biome diversity
        let biome_stats = service.get_biome_stats(center, 50);

        // Should have some diversity over a large area
        assert!(biome_stats.total_tiles > 0);
        assert!(biome_stats.diversity_index() >= 0.0);
        assert!(biome_stats.diversity_index() <= 1.0);
    }

    #[test]
    fn terrain_diversity_improved() {
        let service = MapService::new(12345);
        let mut map = create_test_map();
        let center = Position3D::origin();

        // Generate large area to get terrain variety
        service.generate_chunk(&mut map, center, 30).unwrap();

        let stats = service.get_generation_stats(&map, center, 15);

        // Should have multiple terrain types
        let terrain_variety = [
            stats.plains,
            stats.forest,
            stats.desert,
            stats.mountains,
            stats.tundra,
            stats.swamp,
            stats.cave,
            stats.crystal,
            stats.constructed,
            stats.volcanic,
            stats.anomaly,
        ]
        .iter()
        .filter(|&&count| count > 0)
        .count();

        assert!(
            terrain_variety >= 3,
            "Should have at least 3 different terrain types"
        );
    }

    #[test]
    fn connectivity_validation() {
        let service = MapService::new(12345);
        let mut map = create_test_map();
        let center = Position3D::origin();

        service.generate_chunk(&mut map, center, 6).unwrap();

        let is_connected = service.validate_connectivity(&map, center, 3).unwrap();
        assert!(is_connected);
    }

    #[test]
    fn resource_node_generation() {
        let service = MapService::new(12345);
        let mut map = create_test_map();
        let center = Position3D::origin();

        service.generate_chunk(&mut map, center, 10).unwrap();

        let stats = service.get_generation_stats(&map, center, 5);

        // Should have some resource nodes
        assert!(stats.resource_nodes > 0);

        // Resource density should be reasonable (not too high)
        assert!(stats.resource_density() < 30.0);
    }

    #[test]
    fn position_hashing_consistency() {
        let service = MapService::new(12345);
        let pos = Position3D::new(5, 10, 0);

        let hash1 = service.hash_position(pos);
        let hash2 = service.hash_position(pos);

        // Same position should always produce same hash
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn different_seeds_produce_different_results() {
        let service1 = MapService::new(12345);
        let service2 = MapService::new(54321);
        let pos = Position3D::new(5, 5, 0);

        let terrain1 = service1.generate_terrain_type(pos);
        let terrain2 = service2.generate_terrain_type(pos);

        // Different seeds might produce different terrain (not guaranteed but likely)
        // At minimum, they should use different hash values
        let hash1 = service1.hash_position(pos);
        let hash2 = service2.hash_position(pos);
        assert_ne!(hash1, hash2);
    }
}
