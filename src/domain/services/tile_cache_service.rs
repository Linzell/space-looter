//! Tile Cache Service - Manages tile persistence and memory optimization
//!
//! This service handles caching of generated tiles to improve performance when
//! the player moves around. It saves tiles that are no longer visible and
//! restores them when the player returns to previously visited areas.

use crate::domain::{
    constants::{TILE_CACHE_SIZE, TILE_UNLOAD_DISTANCE},
    entities::map::MapTile,
    value_objects::{Position3D, TileCoordinate},
};
use std::collections::HashMap;

/// Service for managing tile caching and persistence
#[derive(Debug, Clone)]
pub struct TileCacheService {
    /// Cache of tiles that are not currently loaded but should be preserved
    tile_cache: HashMap<TileCoordinate, MapTile>,
    /// Player position history for determining which tiles to keep
    player_history: Vec<Position3D>,
    /// Maximum number of historical positions to track
    history_size: usize,
}

impl TileCacheService {
    /// Create a new tile cache service
    pub fn new() -> Self {
        Self {
            tile_cache: HashMap::new(),
            player_history: Vec::new(),
            history_size: 20, // Keep last 20 positions
        }
    }

    /// Create a new tile cache service with custom history size
    pub fn with_history_size(history_size: usize) -> Self {
        Self {
            tile_cache: HashMap::new(),
            player_history: Vec::new(),
            history_size,
        }
    }

    /// Update player position and manage cache accordingly
    pub fn update_player_position(&mut self, new_position: Position3D) {
        // Add new position to history
        self.player_history.push(new_position);

        // Keep only recent positions
        if self.player_history.len() > self.history_size {
            self.player_history.remove(0);
        }

        // Clean up distant tiles from cache
        self.cleanup_distant_tiles();
    }

    /// Cache a tile that's being unloaded from active memory
    pub fn cache_tile(&mut self, coordinate: TileCoordinate, tile: MapTile) {
        // Only cache if we have space or if this tile is more important
        if self.tile_cache.len() < TILE_CACHE_SIZE || self.should_replace_cached_tile(coordinate) {
            self.tile_cache.insert(coordinate, tile);

            // Remove least important tile if we're over capacity
            if self.tile_cache.len() > TILE_CACHE_SIZE {
                self.remove_least_important_tile();
            }
        }
    }

    /// Retrieve a cached tile if available
    pub fn get_cached_tile(&self, coordinate: TileCoordinate) -> Option<&MapTile> {
        self.tile_cache.get(&coordinate)
    }

    /// Remove a tile from cache and return it if it exists
    pub fn take_cached_tile(&mut self, coordinate: TileCoordinate) -> Option<MapTile> {
        self.tile_cache.remove(&coordinate)
    }

    /// Check if a tile is in the cache
    pub fn has_cached_tile(&self, coordinate: TileCoordinate) -> bool {
        self.tile_cache.contains_key(&coordinate)
    }

    /// Get tiles that should be loaded for the current player position
    pub fn get_tiles_to_load(
        &self,
        _player_pos: Position3D,
        required_coords: &[TileCoordinate],
    ) -> Vec<TileCoordinate> {
        required_coords
            .iter()
            .filter(|coord| !self.has_cached_tile(**coord))
            .copied()
            .collect()
    }

    /// Get tiles that should be cached (no longer needed in active memory)
    pub fn get_tiles_to_cache(
        &self,
        player_pos: Position3D,
        current_coords: &[TileCoordinate],
    ) -> Vec<TileCoordinate> {
        let player_coord = TileCoordinate::from(player_pos);

        current_coords
            .iter()
            .filter(|coord| {
                let dx = (player_coord.x - coord.x).abs();
                let dy = (player_coord.y - coord.y).abs();
                let distance = (dx + dy) as u32;
                distance > TILE_UNLOAD_DISTANCE
            })
            .copied()
            .collect()
    }

    /// Clear all cached tiles (useful for memory cleanup)
    pub fn clear_cache(&mut self) {
        self.tile_cache.clear();
    }

    /// Get cache statistics for debugging
    pub fn get_cache_stats(&self) -> CacheStats {
        CacheStats {
            cached_tiles: self.tile_cache.len(),
            cache_capacity: TILE_CACHE_SIZE,
            player_history_length: self.player_history.len(),
            cache_usage_percentage: (self.tile_cache.len() as f32 / TILE_CACHE_SIZE as f32 * 100.0),
        }
    }

    /// Remove tiles that are too far from player history
    fn cleanup_distant_tiles(&mut self) {
        if self.player_history.is_empty() {
            return;
        }

        let coords_to_remove: Vec<TileCoordinate> = self
            .tile_cache
            .keys()
            .filter(|coord| !self.is_tile_near_player_history(**coord))
            .copied()
            .collect();

        for coord in coords_to_remove {
            self.tile_cache.remove(&coord);
        }
    }

    /// Check if a tile is near any position in player history
    fn is_tile_near_player_history(&self, tile_coord: TileCoordinate) -> bool {
        self.player_history.iter().any(|pos| {
            let pos_coord = TileCoordinate::from(*pos);
            let dx = (pos_coord.x - tile_coord.x).abs();
            let dy = (pos_coord.y - tile_coord.y).abs();
            let distance = (dx + dy) as u32;
            distance <= TILE_UNLOAD_DISTANCE
        })
    }

    /// Determine if we should replace a cached tile with a new one
    fn should_replace_cached_tile(&self, new_coord: TileCoordinate) -> bool {
        // If cache isn't full, always add
        if self.tile_cache.len() < TILE_CACHE_SIZE {
            return true;
        }

        // Check if new tile is closer to recent player positions than existing tiles
        let new_tile_importance = self.calculate_tile_importance(new_coord);

        // Find least important cached tile
        if let Some(min_importance) = self
            .tile_cache
            .keys()
            .map(|coord| self.calculate_tile_importance(*coord))
            .min()
        {
            new_tile_importance > min_importance
        } else {
            false
        }
    }

    /// Remove the least important tile from cache
    fn remove_least_important_tile(&mut self) {
        let least_important = self
            .tile_cache
            .keys()
            .min_by_key(|coord| self.calculate_tile_importance(**coord))
            .copied();

        if let Some(coord) = least_important {
            self.tile_cache.remove(&coord);
        }
    }

    /// Calculate importance score for a tile (higher = more important)
    fn calculate_tile_importance(&self, tile_coord: TileCoordinate) -> u32 {
        self.player_history
            .iter()
            .enumerate()
            .map(|(index, pos)| {
                let pos_coord = TileCoordinate::from(*pos);
                let dx = (pos_coord.x - tile_coord.x).abs();
                let dy = (pos_coord.y - tile_coord.y).abs();
                let distance = (dx + dy) as u32;
                let recency_weight = (self.player_history.len() - index) as u32;

                // Closer tiles and more recent positions are more important
                if distance == 0 {
                    recency_weight * 100 // Very important if player was on this tile
                } else if distance <= TILE_UNLOAD_DISTANCE {
                    recency_weight * (TILE_UNLOAD_DISTANCE - distance + 1)
                } else {
                    0 // Not important if too far
                }
            })
            .sum()
    }
}

impl Default for TileCacheService {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics for debugging and monitoring
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub cached_tiles: usize,
    pub cache_capacity: usize,
    pub player_history_length: usize,
    pub cache_usage_percentage: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::terrain::{Elevation, TerrainType};

    fn create_test_tile(coord: TileCoordinate, terrain_type: TerrainType) -> MapTile {
        MapTile {
            terrain_type,
            elevation: Elevation::sea_level(),
            is_explored: false,
            last_visited: None,
        }
    }

    #[test]
    fn test_cache_creation() {
        let cache = TileCacheService::new();
        assert_eq!(cache.tile_cache.len(), 0);
        assert_eq!(cache.player_history.len(), 0);
    }

    #[test]
    fn test_player_position_tracking() {
        let mut cache = TileCacheService::with_history_size(3);

        cache.update_player_position(Position3D::new(0, 0, 0));
        cache.update_player_position(Position3D::new(1, 0, 0));
        cache.update_player_position(Position3D::new(2, 0, 0));

        assert_eq!(cache.player_history.len(), 3);

        // Should keep only last 3 positions
        cache.update_player_position(Position3D::new(3, 0, 0));
        assert_eq!(cache.player_history.len(), 3);
        assert_eq!(cache.player_history[0], Position3D::new(1, 0, 0));
        assert_eq!(cache.player_history[2], Position3D::new(3, 0, 0));
    }

    #[test]
    fn test_tile_caching() {
        let mut cache = TileCacheService::new();
        let coord = TileCoordinate::new(5, 5, 0);
        let tile = create_test_tile(coord, TerrainType::Plains);

        // Cache the tile
        cache.cache_tile(coord, tile.clone());

        assert!(cache.has_cached_tile(coord));
        assert_eq!(
            cache.get_cached_tile(coord).unwrap().terrain_type,
            TerrainType::Plains
        );
    }

    #[test]
    fn test_tile_retrieval() {
        let mut cache = TileCacheService::new();
        let coord = TileCoordinate::new(3, 3, 0);
        let tile = create_test_tile(coord, TerrainType::Forest);

        cache.cache_tile(coord, tile.clone());

        let retrieved = cache.take_cached_tile(coord);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().terrain_type, TerrainType::Forest);

        // Should no longer be in cache
        assert!(!cache.has_cached_tile(coord));
    }

    #[test]
    fn test_cache_size_limit() {
        let mut cache = TileCacheService::new();

        // Fill cache beyond capacity
        for i in 0..TILE_CACHE_SIZE + 10 {
            let coord = TileCoordinate::new(i as i32, 0, 0);
            let tile = create_test_tile(coord, TerrainType::Plains);
            cache.cache_tile(coord, tile);
        }

        // Should not exceed capacity
        assert!(cache.tile_cache.len() <= TILE_CACHE_SIZE);
    }

    #[test]
    fn test_tiles_to_load() {
        let mut cache = TileCacheService::new();

        // Cache some tiles
        let cached_coord = TileCoordinate::new(1, 1, 0);
        let tile = create_test_tile(cached_coord, TerrainType::Plains);
        cache.cache_tile(cached_coord, tile);

        let required_coords = vec![
            TileCoordinate::new(0, 0, 0),
            cached_coord,
            TileCoordinate::new(2, 2, 0),
        ];

        let to_load = cache.get_tiles_to_load(Position3D::new(0, 0, 0), &required_coords);

        // Should only include coords not in cache
        assert_eq!(to_load.len(), 2);
        assert!(!to_load.contains(&cached_coord));
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = TileCacheService::new();

        // Add some tiles and positions
        cache.update_player_position(Position3D::new(0, 0, 0));
        let coord = TileCoordinate::new(1, 1, 0);
        let tile = create_test_tile(coord, TerrainType::Plains);
        cache.cache_tile(coord, tile);

        let stats = cache.get_cache_stats();
        assert_eq!(stats.cached_tiles, 1);
        assert_eq!(stats.player_history_length, 1);
        assert!(stats.cache_usage_percentage > 0.0);
    }

    #[test]
    fn test_distant_tile_cleanup() {
        let mut cache = TileCacheService::new();

        // Start at origin
        cache.update_player_position(Position3D::new(0, 0, 0));

        // Cache nearby and distant tiles
        let nearby_coord = TileCoordinate::new(1, 1, 0);
        let distant_coord = TileCoordinate::new(50, 50, 0);
        let nearby_tile = create_test_tile(nearby_coord, TerrainType::Plains);
        let distant_tile = create_test_tile(distant_coord, TerrainType::Forest);

        cache.cache_tile(nearby_coord, nearby_tile);
        cache.cache_tile(distant_coord, distant_tile);

        assert_eq!(cache.tile_cache.len(), 2);

        // Move to a different area
        cache.update_player_position(Position3D::new(2, 2, 0));

        // Distant tile should be removed, nearby should remain
        assert!(cache.has_cached_tile(nearby_coord));
        assert!(!cache.has_cached_tile(distant_coord));
    }
}
