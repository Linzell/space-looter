//! Map Entity - Represents the 3D isometric game world
//!
//! This entity manages the game world including terrain, resources,
//! and procedural generation of map chunks.

use crate::domain::value_objects::{
    resources::ResourceNodeProperties,
    terrain::{Elevation, TerrainType},
    EntityId, Position3D, ResourceType, TileCoordinate,
};
use crate::domain::{constants, DomainError, DomainResult};
use chrono::{DateTime, Utc};
use std::collections::{HashMap, VecDeque};

/// The game world map entity
#[derive(Debug, Clone, PartialEq)]
pub struct Map {
    id: EntityId,
    name: String,
    seed: u64,
    tiles: HashMap<TileCoordinate, MapTile>,
    resource_nodes: HashMap<Position3D, ResourceNode>,
    created_at: DateTime<Utc>,
    last_updated: DateTime<Utc>,
    version: u64,
    player_history: VecDeque<Position3D>,
    cache_dir: String,
}

impl Map {
    /// Create a new map
    pub fn new(id: EntityId, name: String, seed: u64) -> DomainResult<Self> {
        if name.is_empty() || name.len() > 100 {
            return Err(DomainError::ValidationError(
                "Map name must be between 1 and 100 characters".to_string(),
            ));
        }

        let now = Utc::now();
        let cache_dir = format!("cache/maps/{}", id);

        Ok(Self {
            id,
            name,
            seed,
            tiles: HashMap::new(),
            resource_nodes: HashMap::new(),
            created_at: now,
            last_updated: now,
            version: 1,
            player_history: VecDeque::with_capacity(constants::PLAYER_HISTORY_SIZE),
            cache_dir,
        })
    }

    /// Get map ID
    pub fn id(&self) -> &EntityId {
        &self.id
    }

    /// Get map name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get map seed
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Get tile at coordinate (loads from cache if needed)
    pub fn get_tile(&self, coordinate: &TileCoordinate) -> Option<&MapTile> {
        if let Some(tile) = self.tiles.get(coordinate) {
            Some(tile)
        } else {
            // Try to load from cache
            None // Simplified - cache loading would be async
        }
    }

    /// Set tile at coordinate
    pub fn set_tile(&mut self, coordinate: TileCoordinate, tile: MapTile) {
        self.tiles.insert(coordinate, tile);
        self.last_updated = Utc::now();
        self.version += 1;
    }

    /// Get all loaded tiles
    pub fn tiles(&self) -> &HashMap<TileCoordinate, MapTile> {
        &self.tiles
    }

    /// Update player position and manage tile cache
    pub fn update_player_position(&mut self, position: Position3D) {
        // Add to history
        self.player_history.push_back(position);
        if self.player_history.len() > constants::PLAYER_HISTORY_SIZE {
            self.player_history.pop_front();
        }

        // Note: Tile generation is now handled by MapService
        // This method only manages the cache - generation should be done
        // via MapService.generate_tiles_around_player() before calling this method

        // Calculate visible tiles around player
        let visible_coords: std::collections::HashSet<TileCoordinate> = position
            .positions_within_distance(constants::VISIBLE_TILE_RADIUS)
            .into_iter()
            .chain(
                self.player_history
                    .iter()
                    .flat_map(|pos| pos.positions_within_distance(constants::HISTORY_TILE_RADIUS)),
            )
            .map(TileCoordinate::from)
            .collect();

        // Remove tiles not in visible set or history
        let tiles_to_cache: Vec<TileCoordinate> = self
            .tiles
            .keys()
            .filter(|coord| !visible_coords.contains(coord))
            .cloned()
            .collect();

        // Cache old tiles (simplified - would be async)
        for coord in tiles_to_cache {
            if let Some(_tile) = self.tiles.remove(&coord) {
                // Cache tile to file system
            }
        }
    }

    /// Get resource node at position
    pub fn get_resource_node(&self, position: &Position3D) -> Option<&ResourceNode> {
        self.resource_nodes.get(position)
    }

    /// Add resource node at position
    pub fn add_resource_node(&mut self, position: Position3D, node: ResourceNode) {
        self.resource_nodes.insert(position, node);
        self.last_updated = Utc::now();
        self.version += 1;
    }

    /// Get all resource nodes
    pub fn resource_nodes(&self) -> &HashMap<Position3D, ResourceNode> {
        &self.resource_nodes
    }

    /// Check if position is passable
    pub fn is_passable(&self, position: &Position3D) -> bool {
        let tile_coord = TileCoordinate::from(*position);
        if let Some(tile) = self.get_tile(&tile_coord) {
            tile.terrain_type.is_passable()
        } else {
            false // Unknown terrain is not passable
        }
    }

    /// Get movement cost for position
    pub fn movement_cost(&self, position: &Position3D) -> u8 {
        let tile_coord = TileCoordinate::from(*position);
        if let Some(tile) = self.get_tile(&tile_coord) {
            tile.terrain_type.movement_cost()
        } else {
            10 // High cost for unknown terrain
        }
    }

    /// Get danger level at position
    pub fn danger_level(&self, position: &Position3D) -> u8 {
        let tile_coord = TileCoordinate::from(*position);
        if let Some(tile) = self.get_tile(&tile_coord) {
            tile.terrain_type.danger_level()
        } else {
            5 // Medium danger for unknown terrain
        }
    }

    /// Get tiles in a radius around position
    pub fn get_tiles_in_radius(
        &self,
        center: &Position3D,
        radius: u32,
    ) -> Vec<(TileCoordinate, &MapTile)> {
        let positions = center.positions_within_distance(radius);
        positions
            .iter()
            .filter_map(|pos| {
                let coord = TileCoordinate::from(*pos);
                self.get_tile(&coord).map(|tile| (coord, tile))
            })
            .collect()
    }

    /// Generate procedural content for a chunk
    /// Note: This method is deprecated - use MapService.generate_chunk() instead
    /// Kept for backward compatibility during transition
    pub fn generate_chunk(
        &mut self,
        chunk_center: Position3D,
        chunk_size: i32,
    ) -> DomainResult<()> {
        use crate::domain::services::MapService;

        let map_service = MapService::new(self.seed);
        map_service.generate_chunk(self, chunk_center, chunk_size)
    }
}

/// A single tile in the map
#[derive(Debug, Clone, PartialEq)]
pub struct MapTile {
    pub terrain_type: TerrainType,
    pub elevation: Elevation,
    pub is_explored: bool,
    pub last_visited: Option<DateTime<Utc>>,
}

impl MapTile {
    /// Create a new map tile
    pub fn new(terrain_type: TerrainType, elevation: Elevation, is_explored: bool) -> Self {
        Self {
            terrain_type,
            elevation,
            is_explored,
            last_visited: None,
        }
    }

    /// Mark tile as explored
    pub fn explore(&mut self) {
        self.is_explored = true;
        self.last_visited = Some(Utc::now());
    }

    /// Check if tile has been explored
    pub fn is_explored(&self) -> bool {
        self.is_explored
    }

    /// Get visibility modifier for this tile
    pub fn visibility_modifier(&self) -> f32 {
        self.terrain_type.visibility_modifier()
    }

    /// Get event probability modifier for this tile
    pub fn event_probability_modifier(&self) -> f32 {
        self.terrain_type.event_probability_modifier()
    }
}

/// A resource node on the map
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceNode {
    id: EntityId,
    properties: ResourceNodeProperties,
    max_capacity: u32,
    current_amount: u32,
    last_harvested: Option<DateTime<Utc>>,
    total_harvested: u32,
    created_at: DateTime<Utc>,
}

impl ResourceNode {
    /// Create a new resource node
    pub fn new(
        id: EntityId,
        properties: ResourceNodeProperties,
        max_capacity: u32,
        current_amount: u32,
    ) -> Self {
        Self {
            id,
            properties,
            max_capacity,
            current_amount: current_amount.min(max_capacity),
            last_harvested: None,
            total_harvested: 0,
            created_at: Utc::now(),
        }
    }

    /// Get node ID
    pub fn id(&self) -> &EntityId {
        &self.id
    }

    /// Get node properties
    pub fn properties(&self) -> &ResourceNodeProperties {
        &self.properties
    }

    /// Get current amount available
    pub fn current_amount(&self) -> u32 {
        self.current_amount
    }

    /// Get maximum capacity
    pub fn max_capacity(&self) -> u32 {
        self.max_capacity
    }

    /// Check if node is depleted
    pub fn is_depleted(&self) -> bool {
        self.current_amount == 0
    }

    /// Check if node is full
    pub fn is_full(&self) -> bool {
        self.current_amount >= self.max_capacity
    }

    /// Harvest resources from this node
    pub fn harvest(&mut self, amount: u32) -> u32 {
        let harvested = amount.min(self.current_amount);
        self.current_amount -= harvested;
        self.total_harvested += harvested;
        self.last_harvested = Some(Utc::now());
        harvested
    }

    /// Regenerate resources based on regeneration rate
    pub fn regenerate(&mut self, current_time: DateTime<Utc>) -> u32 {
        if let Some(last_harvested) = self.last_harvested {
            if let Some(interval) = self
                .properties
                .regeneration_rate
                .regeneration_interval_minutes()
            {
                let minutes_elapsed = current_time
                    .signed_duration_since(last_harvested)
                    .num_minutes() as u32;

                if minutes_elapsed >= interval {
                    let regen_percentage =
                        self.properties.regeneration_rate.regeneration_percentage();
                    let regen_amount = (self.max_capacity as f32 * regen_percentage) as u32;
                    let old_amount = self.current_amount;
                    self.current_amount =
                        (self.current_amount + regen_amount).min(self.max_capacity);
                    return self.current_amount - old_amount;
                }
            }
        }
        0
    }

    /// Get total amount harvested from this node
    pub fn total_harvested(&self) -> u32 {
        self.total_harvested
    }

    /// Get when this node was last harvested
    pub fn last_harvested(&self) -> Option<DateTime<Utc>> {
        self.last_harvested
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::resources::ResourceRichness;

    #[test]
    fn map_creation() {
        let id = EntityId::generate();
        let name = "Test Map".to_string();
        let seed = 12345;

        let map = Map::new(id, name.clone(), seed).unwrap();
        assert_eq!(map.name(), &name);
        assert_eq!(map.seed(), seed);
        assert!(map.tiles().is_empty());
    }

    #[test]
    fn map_tile_operations() {
        let mut map = Map::new(EntityId::generate(), "Test".to_string(), 123).unwrap();

        let coord = TileCoordinate::new(0, 0, 0);
        let tile = MapTile::new(TerrainType::Plains, Elevation::sea_level(), false);

        map.set_tile(coord, tile.clone());

        let retrieved = map.get_tile(&coord);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().terrain_type, TerrainType::Plains);
    }

    #[test]
    fn map_tile_exploration() {
        let mut tile = MapTile::new(TerrainType::Forest, Elevation::sea_level(), false);
        assert!(!tile.is_explored());

        tile.explore();
        assert!(tile.is_explored());
        assert!(tile.last_visited.is_some());
    }

    #[test]
    fn resource_node_operations() {
        let props = ResourceNodeProperties::new(
            ResourceType::Metal,
            ResourceRichness::Average,
            crate::domain::value_objects::resources::ResourceAccessibility::Easy,
            crate::domain::value_objects::resources::RegenerationRate::None,
        );

        let mut node = ResourceNode::new(EntityId::generate(), props, 100, 100);

        assert_eq!(node.current_amount(), 100);
        assert!(!node.is_depleted());
        assert!(node.is_full());

        let harvested = node.harvest(30);
        assert_eq!(harvested, 30);
        assert_eq!(node.current_amount(), 70);
        assert_eq!(node.total_harvested(), 30);
    }

    #[test]
    fn map_passability() {
        let mut map = Map::new(EntityId::generate(), "Test".to_string(), 123).unwrap();

        let coord = TileCoordinate::new(0, 0, 0);
        let plains_tile = MapTile::new(TerrainType::Plains, Elevation::sea_level(), false);
        let ocean_tile = MapTile::new(TerrainType::Ocean, Elevation::sea_level(), false);

        map.set_tile(coord, plains_tile);
        assert!(map.is_passable(&Position3D::new(0, 0, 0)));

        map.set_tile(coord, ocean_tile);
        assert!(!map.is_passable(&Position3D::new(0, 0, 0)));
    }

    #[test]
    fn chunk_generation() {
        let mut map = Map::new(EntityId::generate(), "Test".to_string(), 123).unwrap();

        let center = Position3D::new(0, 0, 0);
        map.generate_chunk(center, 4).unwrap();

        // Should have generated some tiles
        assert!(!map.tiles().is_empty());

        // Should have generated some resource nodes
        assert!(!map.resource_nodes().is_empty());
    }

    #[test]
    fn tiles_in_radius() {
        let mut map = Map::new(EntityId::generate(), "Test".to_string(), 123).unwrap();

        // Add some tiles
        for x in -2..=2 {
            for y in -2..=2 {
                let coord = TileCoordinate::new(x, y, 0);
                let tile = MapTile::new(TerrainType::Plains, Elevation::sea_level(), false);
                map.set_tile(coord, tile);
            }
        }

        let center = Position3D::new(0, 0, 0);
        let tiles = map.get_tiles_in_radius(&center, 1);

        // Should include center and adjacent tiles (5 total in Manhattan distance 1)
        assert_eq!(tiles.len(), 5);
    }
}
