//! 3D Position and Coordinate System for Isometric World
//!
//! This module provides coordinate systems for the 3D isometric game world.
//! It includes both world coordinates (for game logic) and screen coordinates
//! (for rendering in isometric view).

use crate::domain::{DomainError, DomainResult};
use serde::{Deserialize, Serialize};
use std::fmt;

/// 3D position in world space using integer coordinates
///
/// Uses integer coordinates to avoid floating-point precision issues
/// and to align with tile-based world generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position3D {
    pub x: i32, // East-West axis
    pub y: i32, // North-South axis
    pub z: i32, // Elevation/height
}

impl Position3D {
    /// Create a new 3D position
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Create a position at ground level (z = 0)
    pub fn ground(x: i32, y: i32) -> Self {
        Self::new(x, y, 0)
    }

    /// Origin position (0, 0, 0)
    pub fn origin() -> Self {
        Self::new(0, 0, 0)
    }

    /// Calculate Manhattan distance to another position (ignoring elevation)
    pub fn manhattan_distance_2d(&self, other: &Position3D) -> u32 {
        ((self.x - other.x).abs() + (self.y - other.y).abs()) as u32
    }

    /// Calculate 3D Manhattan distance including elevation
    pub fn manhattan_distance_3d(&self, other: &Position3D) -> u32 {
        ((self.x - other.x).abs() + (self.y - other.y).abs() + (self.z - other.z).abs()) as u32
    }

    /// Get x coordinate
    pub fn x(&self) -> i32 {
        self.x
    }

    /// Get y coordinate
    pub fn y(&self) -> i32 {
        self.y
    }

    /// Get z coordinate
    pub fn z(&self) -> i32 {
        self.z
    }

    /// Calculate Euclidean distance to another position (2D)
    pub fn euclidean_distance_2d(&self, other: &Position3D) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dy = (self.y - other.y) as f32;
        (dx * dx + dy * dy).sqrt()
    }

    /// Calculate 3D Euclidean distance
    pub fn euclidean_distance_3d(&self, other: &Position3D) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dy = (self.y - other.y) as f32;
        let dz = (self.z - other.z) as f32;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Calculate distance to another position (alias for euclidean_distance_3d)
    pub fn distance_to(&self, other: &Position3D) -> f32 {
        self.euclidean_distance_3d(other)
    }

    /// Move by offset
    pub fn offset(&self, dx: i32, dy: i32, dz: i32) -> Self {
        Self::new(self.x + dx, self.y + dy, self.z + dz)
    }

    /// Move in cardinal direction
    pub fn move_direction(&self, direction: Direction, distance: i32) -> Self {
        match direction {
            Direction::North => self.offset(0, distance, 0),
            Direction::South => self.offset(0, -distance, 0),
            Direction::East => self.offset(distance, 0, 0),
            Direction::West => self.offset(-distance, 0, 0),
            Direction::Up => self.offset(0, 0, distance),
            Direction::Down => self.offset(0, 0, -distance),
        }
    }

    /// Get all adjacent positions (6-directional for 3D)
    pub fn adjacent_positions(&self) -> Vec<Position3D> {
        vec![
            self.move_direction(Direction::North, 1),
            self.move_direction(Direction::South, 1),
            self.move_direction(Direction::East, 1),
            self.move_direction(Direction::West, 1),
            self.move_direction(Direction::Up, 1),
            self.move_direction(Direction::Down, 1),
        ]
    }

    /// Get all positions within Manhattan distance (2D)
    pub fn positions_within_distance(&self, distance: u32) -> Vec<Position3D> {
        let mut positions = Vec::new();
        let dist = distance as i32;

        for dx in -dist..=dist {
            for dy in -dist..=dist {
                if (dx.abs() + dy.abs()) <= dist {
                    positions.push(self.offset(dx, dy, 0));
                }
            }
        }

        positions
    }

    /// Convert to isometric screen coordinates
    /// Uses standard isometric projection: screen_x = (x - y), screen_y = (x + y) / 2 - z
    pub fn to_isometric_screen(
        &self,
        tile_width: f32,
        tile_height: f32,
    ) -> IsometricScreenPosition {
        let iso_x = (self.x - self.y) as f32 * tile_width / 2.0;
        let iso_y =
            (self.x + self.y) as f32 * tile_height / 4.0 - (self.z as f32 * tile_height / 2.0);

        IsometricScreenPosition::new(iso_x, iso_y)
    }

    /// Check if position is at ground level
    pub fn is_ground_level(&self) -> bool {
        self.z == 0
    }

    /// Check if position is above ground
    pub fn is_elevated(&self) -> bool {
        self.z > 0
    }

    /// Check if position is below ground (underground)
    pub fn is_underground(&self) -> bool {
        self.z < 0
    }

    /// Get the position directly below this one
    pub fn below(&self) -> Position3D {
        self.offset(0, 0, -1)
    }

    /// Get the position directly above this one
    pub fn above(&self) -> Position3D {
        self.offset(0, 0, 1)
    }

    /// Convert to tile coordinate
    pub fn to_tile_coordinate(&self) -> TileCoordinate {
        TileCoordinate::new(self.x, self.y, self.z)
    }
}

impl fmt::Display for Position3D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl Default for Position3D {
    fn default() -> Self {
        Self::origin()
    }
}

/// Cardinal and vertical directions for movement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    North,
    South,
    East,
    West,
    Up,
    Down,
}

impl Direction {
    /// Get all horizontal directions
    pub fn horizontal() -> Vec<Direction> {
        vec![
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
        ]
    }

    /// Get all directions including vertical
    pub fn all() -> Vec<Direction> {
        vec![
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
            Direction::Up,
            Direction::Down,
        ]
    }

    /// Get opposite direction
    pub fn opposite(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
        }
    }

    /// Check if this is a horizontal direction
    pub fn is_horizontal(&self) -> bool {
        matches!(
            self,
            Direction::North | Direction::South | Direction::East | Direction::West
        )
    }

    /// Check if this is a vertical direction
    pub fn is_vertical(&self) -> bool {
        matches!(self, Direction::Up | Direction::Down)
    }

    /// Get unit offset vector for this direction
    pub fn offset(&self) -> (i32, i32, i32) {
        match self {
            Direction::North => (0, 1, 0),
            Direction::South => (0, -1, 0),
            Direction::East => (1, 0, 0),
            Direction::West => (-1, 0, 0),
            Direction::Up => (0, 0, 1),
            Direction::Down => (0, 0, -1),
        }
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Direction::North => write!(f, "North"),
            Direction::South => write!(f, "South"),
            Direction::East => write!(f, "East"),
            Direction::West => write!(f, "West"),
            Direction::Up => write!(f, "Up"),
            Direction::Down => write!(f, "Down"),
        }
    }
}

/// Tile coordinate that represents a discrete world position
///
/// This is essentially the same as Position3D but used in contexts
/// where we want to emphasize we're working with discrete tile positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileCoordinate {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl TileCoordinate {
    /// Create a new tile coordinate
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Origin tile (0, 0, 0)
    pub fn origin() -> Self {
        Self::new(0, 0, 0)
    }

    /// Convert to world position
    pub fn to_position(&self) -> Position3D {
        Position3D::new(self.x, self.y, self.z)
    }

    /// Get chunk coordinate this tile belongs to
    pub fn to_chunk_coordinate(&self, chunk_size: i32) -> ChunkCoordinate {
        ChunkCoordinate::new(
            self.x.div_euclid(chunk_size),
            self.y.div_euclid(chunk_size),
            self.z.div_euclid(chunk_size),
        )
    }

    /// Get local coordinate within chunk
    pub fn local_in_chunk(&self, chunk_size: i32) -> TileCoordinate {
        TileCoordinate::new(
            self.x.rem_euclid(chunk_size),
            self.y.rem_euclid(chunk_size),
            self.z.rem_euclid(chunk_size),
        )
    }

    /// Check if this tile is in the same chunk as another
    pub fn same_chunk(&self, other: &TileCoordinate, chunk_size: i32) -> bool {
        self.to_chunk_coordinate(chunk_size) == other.to_chunk_coordinate(chunk_size)
    }
}

impl fmt::Display for TileCoordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tile({}, {}, {})", self.x, self.y, self.z)
    }
}

impl From<Position3D> for TileCoordinate {
    fn from(pos: Position3D) -> Self {
        TileCoordinate::new(pos.x, pos.y, pos.z)
    }
}

impl From<TileCoordinate> for Position3D {
    fn from(tile: TileCoordinate) -> Self {
        Position3D::new(tile.x, tile.y, tile.z)
    }
}

/// Chunk coordinate for world partitioning and procedural generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChunkCoordinate {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkCoordinate {
    /// Create a new chunk coordinate
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Origin chunk (0, 0, 0)
    pub fn origin() -> Self {
        Self::new(0, 0, 0)
    }

    /// Get the world position of the chunk's origin tile
    pub fn to_world_origin(&self, chunk_size: i32) -> Position3D {
        Position3D::new(
            self.x * chunk_size,
            self.y * chunk_size,
            self.z * chunk_size,
        )
    }

    /// Get all tile coordinates within this chunk
    pub fn tile_coordinates(&self, chunk_size: i32) -> Vec<TileCoordinate> {
        let mut tiles = Vec::new();
        let origin = self.to_world_origin(chunk_size);

        for x in 0..chunk_size {
            for y in 0..chunk_size {
                for z in 0..chunk_size {
                    tiles.push(TileCoordinate::new(
                        origin.x + x,
                        origin.y + y,
                        origin.z + z,
                    ));
                }
            }
        }

        tiles
    }

    /// Get adjacent chunk coordinates
    pub fn adjacent_chunks(&self) -> Vec<ChunkCoordinate> {
        vec![
            ChunkCoordinate::new(self.x + 1, self.y, self.z),
            ChunkCoordinate::new(self.x - 1, self.y, self.z),
            ChunkCoordinate::new(self.x, self.y + 1, self.z),
            ChunkCoordinate::new(self.x, self.y - 1, self.z),
            ChunkCoordinate::new(self.x, self.y, self.z + 1),
            ChunkCoordinate::new(self.x, self.y, self.z - 1),
        ]
    }

    /// Manhattan distance to another chunk
    pub fn manhattan_distance(&self, other: &ChunkCoordinate) -> u32 {
        ((self.x - other.x).abs() + (self.y - other.y).abs() + (self.z - other.z).abs()) as u32
    }
}

impl fmt::Display for ChunkCoordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Chunk({}, {}, {})", self.x, self.y, self.z)
    }
}

/// Screen position for isometric rendering
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct IsometricScreenPosition {
    pub x: f32,
    pub y: f32,
}

impl IsometricScreenPosition {
    /// Create a new screen position
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Origin screen position
    pub fn origin() -> Self {
        Self::new(0.0, 0.0)
    }

    /// Convert back to world coordinates (approximate)
    /// This is the inverse of the isometric projection
    pub fn to_world_position(&self, tile_width: f32, tile_height: f32, z: i32) -> Position3D {
        let world_x = ((self.x / (tile_width / 2.0))
            + (self.y + z as f32 * tile_height / 2.0) / (tile_height / 4.0))
            / 2.0;
        let world_y = ((self.y + z as f32 * tile_height / 2.0) / (tile_height / 4.0)
            - (self.x / (tile_width / 2.0)))
            / 2.0;

        Position3D::new(world_x.round() as i32, world_y.round() as i32, z)
    }

    /// Offset screen position
    pub fn offset(&self, dx: f32, dy: f32) -> Self {
        Self::new(self.x + dx, self.y + dy)
    }

    /// Calculate screen distance to another position
    pub fn distance(&self, other: &IsometricScreenPosition) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

impl fmt::Display for IsometricScreenPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Screen({:.1}, {:.1})", self.x, self.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_3d_creation() {
        let pos = Position3D::new(5, -3, 2);
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, -3);
        assert_eq!(pos.z, 2);
    }

    #[test]
    fn position_3d_ground_level() {
        let pos = Position3D::ground(10, 20);
        assert_eq!(pos, Position3D::new(10, 20, 0));
        assert!(pos.is_ground_level());
        assert!(!pos.is_elevated());
        assert!(!pos.is_underground());
    }

    #[test]
    fn position_3d_distances() {
        let pos1 = Position3D::new(0, 0, 0);
        let pos2 = Position3D::new(3, 4, 0);

        assert_eq!(pos1.manhattan_distance_2d(&pos2), 7);
        assert_eq!(pos1.euclidean_distance_2d(&pos2), 5.0);
    }

    #[test]
    fn position_3d_movement() {
        let pos = Position3D::origin();
        let north = pos.move_direction(Direction::North, 1);
        assert_eq!(north, Position3D::new(0, 1, 0));

        let up = pos.move_direction(Direction::Up, 2);
        assert_eq!(up, Position3D::new(0, 0, 2));
    }

    #[test]
    fn position_3d_adjacent() {
        let pos = Position3D::origin();
        let adjacent = pos.adjacent_positions();
        assert_eq!(adjacent.len(), 6);
        assert!(adjacent.contains(&Position3D::new(0, 1, 0))); // North
        assert!(adjacent.contains(&Position3D::new(1, 0, 0))); // East
        assert!(adjacent.contains(&Position3D::new(0, 0, 1))); // Up
    }

    #[test]
    fn direction_properties() {
        assert_eq!(Direction::North.opposite(), Direction::South);
        assert!(Direction::North.is_horizontal());
        assert!(!Direction::North.is_vertical());
        assert!(Direction::Up.is_vertical());
        assert!(!Direction::Up.is_horizontal());
    }

    #[test]
    fn direction_offset() {
        assert_eq!(Direction::North.offset(), (0, 1, 0));
        assert_eq!(Direction::East.offset(), (1, 0, 0));
        assert_eq!(Direction::Up.offset(), (0, 0, 1));
    }

    #[test]
    fn tile_coordinate_conversion() {
        let pos = Position3D::new(5, 10, 2);
        let tile = pos.to_tile_coordinate();
        assert_eq!(tile.x, 5);
        assert_eq!(tile.y, 10);
        assert_eq!(tile.z, 2);

        let back_to_pos = tile.to_position();
        assert_eq!(pos, back_to_pos);
    }

    #[test]
    fn chunk_coordinate_calculation() {
        let tile = TileCoordinate::new(17, 25, 3);
        let chunk = tile.to_chunk_coordinate(16);
        assert_eq!(chunk, ChunkCoordinate::new(1, 1, 0));

        let local = tile.local_in_chunk(16);
        assert_eq!(local, TileCoordinate::new(1, 9, 3));
    }

    #[test]
    fn chunk_coordinate_tiles() {
        let chunk = ChunkCoordinate::new(0, 0, 0);
        let tiles = chunk.tile_coordinates(2);
        assert_eq!(tiles.len(), 8); // 2x2x2 = 8 tiles
        assert!(tiles.contains(&TileCoordinate::new(0, 0, 0)));
        assert!(tiles.contains(&TileCoordinate::new(1, 1, 1)));
    }

    #[test]
    fn isometric_screen_conversion() {
        let pos = Position3D::new(4, 2, 1);
        let screen = pos.to_isometric_screen(64.0, 32.0);

        // For position (4, 2, 1) with 64x32 tiles:
        // iso_x = (4 - 2) * 32 = 64
        // iso_y = (4 + 2) * 8 - 1 * 16 = 48 - 16 = 32
        assert_eq!(screen.x, 64.0);
        assert_eq!(screen.y, 32.0);
    }

    #[test]
    fn positions_within_distance() {
        let pos = Position3D::origin();
        let nearby = pos.positions_within_distance(1);

        // Should include center (0,0,0) and 4 adjacent positions in 2D
        assert_eq!(nearby.len(), 5);
        assert!(nearby.contains(&Position3D::new(0, 0, 0)));
        assert!(nearby.contains(&Position3D::new(1, 0, 0)));
        assert!(nearby.contains(&Position3D::new(0, 1, 0)));
        assert!(nearby.contains(&Position3D::new(-1, 0, 0)));
        assert!(nearby.contains(&Position3D::new(0, -1, 0)));
    }

    #[test]
    fn position_elevation_checks() {
        let ground = Position3D::ground(5, 5);
        let elevated = Position3D::new(5, 5, 3);
        let underground = Position3D::new(5, 5, -2);

        assert!(ground.is_ground_level());
        assert!(elevated.is_elevated());
        assert!(underground.is_underground());

        assert_eq!(ground.above(), elevated.offset(0, 0, -2));
        assert_eq!(elevated.below(), ground.offset(0, 0, 2));
    }
}
