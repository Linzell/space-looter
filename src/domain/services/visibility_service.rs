//! Visibility Service - Manages fog of war and tile visibility
//!
//! This service determines which tiles are visible to the player based on the
//! fog of war rules. Implements a diamond pattern with two visibility zones:
//! - Fully visible tiles (no fog): player + 4 adjacent tiles
//! - Fogged visible tiles (with fog overlay): larger diamond pattern

use crate::domain::{
    constants::{FOGGED_VISIBLE_RADIUS, FOG_OF_WAR_DIAMOND_PATTERN, FULLY_VISIBLE_RADIUS},
    value_objects::{Position3D, TileCoordinate},
};

/// Visibility level for tiles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisibilityLevel {
    /// Tile is not visible at all
    Hidden,
    /// Tile is visible but with fog overlay
    Fogged,
    /// Tile is fully visible with no fog
    FullyVisible,
}

/// Service for managing tile visibility and fog of war
#[derive(Debug, Clone)]
pub struct VisibilityService;

impl VisibilityService {
    /// Create a new visibility service
    pub fn new() -> Self {
        Self
    }

    /// Get the visibility level for a tile from the player's position
    pub fn get_tile_visibility(
        &self,
        player_pos: Position3D,
        tile_coord: TileCoordinate,
    ) -> VisibilityLevel {
        let tile_pos = Position3D::from(tile_coord);

        if FOG_OF_WAR_DIAMOND_PATTERN {
            // Check if fully visible (diamond pattern)
            if self.is_in_diamond_pattern_fully_visible(player_pos, tile_pos) {
                return VisibilityLevel::FullyVisible;
            }

            // Check if in fogged diamond pattern
            if self.is_in_diamond_pattern(player_pos, tile_pos) {
                return VisibilityLevel::Fogged;
            }

            VisibilityLevel::Hidden
        } else {
            // Fallback: simple radius visibility
            let distance = player_pos.manhattan_distance_2d(&tile_pos);
            if distance <= FULLY_VISIBLE_RADIUS {
                VisibilityLevel::FullyVisible
            } else if distance <= FOGGED_VISIBLE_RADIUS {
                VisibilityLevel::Fogged
            } else {
                VisibilityLevel::Hidden
            }
        }
    }

    /// Check if a tile is visible from the player's position (any visibility level)
    pub fn is_tile_visible(&self, player_pos: Position3D, tile_coord: TileCoordinate) -> bool {
        matches!(
            self.get_tile_visibility(player_pos, tile_coord),
            VisibilityLevel::FullyVisible | VisibilityLevel::Fogged
        )
    }

    /// Get all fully visible tile coordinates (no fog)
    pub fn get_fully_visible_coordinates(&self, player_pos: Position3D) -> Vec<TileCoordinate> {
        let mut visible_coords = Vec::new();
        let radius = FULLY_VISIBLE_RADIUS as i32;

        // Generate diamond pattern within radius using Manhattan distance
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                let manhattan_distance = dx.abs() + dy.abs();
                if manhattan_distance <= radius {
                    visible_coords.push(TileCoordinate::from(Position3D::new(
                        player_pos.x + dx,
                        player_pos.y + dy,
                        player_pos.z,
                    )));
                }
            }
        }

        visible_coords
    }

    /// Get all fogged visible tile coordinates (shown with fog overlay)
    pub fn get_fogged_visible_coordinates(&self, player_pos: Position3D) -> Vec<TileCoordinate> {
        let mut fogged_tiles = Vec::new();

        // Generate diamond pattern coordinates
        let radius = FOGGED_VISIBLE_RADIUS as i32;
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                let tile_pos = Position3D::new(player_pos.x + dx, player_pos.y + dy, player_pos.z);
                let tile_coord = TileCoordinate::from(tile_pos);

                // Include if in diamond pattern but not in plus pattern
                if self.is_in_diamond_pattern(player_pos, tile_pos)
                    && !self.is_in_diamond_pattern_fully_visible(player_pos, tile_pos)
                {
                    fogged_tiles.push(tile_coord);
                }
            }
        }

        fogged_tiles
    }

    /// Get all visible tile coordinates (both fully visible and fogged)
    pub fn get_all_visible_coordinates(&self, player_pos: Position3D) -> Vec<TileCoordinate> {
        let mut all_tiles = self.get_fully_visible_coordinates(player_pos);
        all_tiles.extend(self.get_fogged_visible_coordinates(player_pos));
        all_tiles
    }

    /// Check if a tile position is in the diamond pattern around player (fully visible)
    fn is_in_diamond_pattern_fully_visible(
        &self,
        player_pos: Position3D,
        tile_pos: Position3D,
    ) -> bool {
        let dx = (player_pos.x - tile_pos.x).abs();
        let dy = (player_pos.y - tile_pos.y).abs();
        let dz = (player_pos.z - tile_pos.z).abs();

        // Same Z level and within fully visible radius using Manhattan distance (diamond pattern)
        let radius = FULLY_VISIBLE_RADIUS as i32;
        dz == 0 && (dx + dy) <= radius
    }

    /// Check if a tile position is in the diamond pattern around player (fogged visible)
    fn is_in_diamond_pattern(&self, player_pos: Position3D, tile_pos: Position3D) -> bool {
        let dx = (player_pos.x - tile_pos.x).abs();
        let dy = (player_pos.y - tile_pos.y).abs();
        let dz = (player_pos.z - tile_pos.z).abs();

        // Same Z level and within diamond pattern (Manhattan distance)
        dz == 0 && (dx + dy) <= FOGGED_VISIBLE_RADIUS as i32
    }
}

impl Default for VisibilityService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diamond_pattern_visibility() {
        let service = VisibilityService::new();
        let player_pos = Position3D::new(0, 0, 0);

        // Player tile should be fully visible
        assert_eq!(
            service.get_tile_visibility(player_pos, TileCoordinate::new(0, 0, 0)),
            VisibilityLevel::FullyVisible
        );

        // Adjacent tiles should be fully visible (N, S, E, W)
        assert_eq!(
            service.get_tile_visibility(player_pos, TileCoordinate::new(0, 1, 0)),
            VisibilityLevel::FullyVisible
        );
        assert_eq!(
            service.get_tile_visibility(player_pos, TileCoordinate::new(0, -1, 0)),
            VisibilityLevel::FullyVisible
        );
        assert_eq!(
            service.get_tile_visibility(player_pos, TileCoordinate::new(1, 0, 0)),
            VisibilityLevel::FullyVisible
        );
        assert_eq!(
            service.get_tile_visibility(player_pos, TileCoordinate::new(-1, 0, 0)),
            VisibilityLevel::FullyVisible
        );

        // Diagonal tiles within diamond should be fogged
        assert_eq!(
            service.get_tile_visibility(player_pos, TileCoordinate::new(1, 1, 0)),
            VisibilityLevel::Fogged
        );
        assert_eq!(
            service.get_tile_visibility(player_pos, TileCoordinate::new(-1, -1, 0)),
            VisibilityLevel::Fogged
        );

        // Distance 2 tiles should be fogged
        assert_eq!(
            service.get_tile_visibility(player_pos, TileCoordinate::new(2, 0, 0)),
            VisibilityLevel::Fogged
        );
        assert_eq!(
            service.get_tile_visibility(player_pos, TileCoordinate::new(0, 2, 0)),
            VisibilityLevel::Fogged
        );

        // Distant tiles should be hidden
        assert_eq!(
            service.get_tile_visibility(player_pos, TileCoordinate::new(4, 0, 0)),
            VisibilityLevel::Hidden
        );
        assert_eq!(
            service.get_tile_visibility(player_pos, TileCoordinate::new(0, 4, 0)),
            VisibilityLevel::Hidden
        );
    }

    #[test]
    fn test_get_fully_visible_coordinates() {
        let service = VisibilityService::new();
        let player_pos = Position3D::new(5, 5, 0);
        let fully_visible = service.get_fully_visible_coordinates(player_pos);

        // Should have exactly 5 tiles (player + 4 adjacent)
        assert_eq!(fully_visible.len(), 5);

        // Should include player position
        assert!(fully_visible.contains(&TileCoordinate::new(5, 5, 0)));

        // Should include all 4 adjacent positions
        assert!(fully_visible.contains(&TileCoordinate::new(5, 6, 0))); // North
        assert!(fully_visible.contains(&TileCoordinate::new(5, 4, 0))); // South
        assert!(fully_visible.contains(&TileCoordinate::new(6, 5, 0))); // East
        assert!(fully_visible.contains(&TileCoordinate::new(4, 5, 0))); // West
    }

    #[test]
    fn test_get_fogged_visible_coordinates() {
        let service = VisibilityService::new();
        let player_pos = Position3D::new(0, 0, 0);
        let fogged_coords = service.get_fogged_visible_coordinates(player_pos);

        // Should include diagonal tiles
        assert!(fogged_coords.contains(&TileCoordinate::new(1, 1, 0)));
        assert!(fogged_coords.contains(&TileCoordinate::new(-1, 1, 0)));
        assert!(fogged_coords.contains(&TileCoordinate::new(1, -1, 0)));
        assert!(fogged_coords.contains(&TileCoordinate::new(-1, -1, 0)));

        // Should include distance 2 tiles
        assert!(fogged_coords.contains(&TileCoordinate::new(2, 0, 0)));
        assert!(fogged_coords.contains(&TileCoordinate::new(0, 2, 0)));
        assert!(fogged_coords.contains(&TileCoordinate::new(-2, 0, 0)));
        assert!(fogged_coords.contains(&TileCoordinate::new(0, -2, 0)));

        // Should NOT include fully visible tiles (plus pattern)
        assert!(!fogged_coords.contains(&TileCoordinate::new(0, 0, 0)));
        assert!(!fogged_coords.contains(&TileCoordinate::new(1, 0, 0)));
        assert!(!fogged_coords.contains(&TileCoordinate::new(-1, 0, 0)));
        assert!(!fogged_coords.contains(&TileCoordinate::new(0, 1, 0)));
        assert!(!fogged_coords.contains(&TileCoordinate::new(0, -1, 0)));
    }

    #[test]
    fn test_different_z_levels() {
        let service = VisibilityService::new();
        let player_pos = Position3D::new(0, 0, 0);

        // Tiles on different Z levels should be hidden
        assert_eq!(
            service.get_tile_visibility(player_pos, TileCoordinate::new(0, 0, 1)),
            VisibilityLevel::Hidden
        );
        assert_eq!(
            service.get_tile_visibility(player_pos, TileCoordinate::new(0, 1, 1)),
            VisibilityLevel::Hidden
        );
    }

    #[test]
    fn test_diamond_pattern_bounds() {
        let service = VisibilityService::new();
        let player_pos = Position3D::new(0, 0, 0);

        // Test edge of diamond pattern (should be fogged)
        assert_eq!(
            service.get_tile_visibility(player_pos, TileCoordinate::new(3, 0, 0)),
            VisibilityLevel::Fogged
        );
        assert_eq!(
            service.get_tile_visibility(player_pos, TileCoordinate::new(0, 3, 0)),
            VisibilityLevel::Fogged
        );
        assert_eq!(
            service.get_tile_visibility(player_pos, TileCoordinate::new(2, 1, 0)),
            VisibilityLevel::Fogged
        );

        // Test outside diamond pattern (should be hidden)
        assert_eq!(
            service.get_tile_visibility(player_pos, TileCoordinate::new(4, 0, 0)),
            VisibilityLevel::Hidden
        );
        assert_eq!(
            service.get_tile_visibility(player_pos, TileCoordinate::new(3, 2, 0)),
            VisibilityLevel::Hidden
        );
    }
}
