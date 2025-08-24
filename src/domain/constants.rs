//! Domain Constants - Core game constants and configuration values
//!
//! This module contains all the fundamental constants that define game mechanics,
//! balance, and behavior. These values are used throughout the domain layer
//! to ensure consistent game rules and progression.

use crate::domain::value_objects::{DiceType, ResourceType, TerrainType};
use bevy::prelude::Color;

// =============================================================================
// DICE MECHANICS CONSTANTS
// =============================================================================

/// Default movement dice type
pub const MOVEMENT_DICE: DiceType = DiceType::D6;

/// Default action dice type
pub const ACTION_DICE: DiceType = DiceType::D20;

/// Default resource gathering dice type
pub const RESOURCE_DICE: DiceType = DiceType::D10;

/// Base event chance for automatic events (0.0 to 1.0)
pub const BASE_EVENT_CHANCE: f32 = 0.3;

/// Maximum modifier that can be applied to dice rolls
pub const MAX_DICE_MODIFIER: i8 = 10;

/// Minimum modifier that can be applied to dice rolls
pub const MIN_DICE_MODIFIER: i8 = -10;

/// Critical success threshold
pub const CRITICAL_SUCCESS_THRESHOLD: u8 = 18;

/// Critical failure threshold
pub const CRITICAL_FAILURE_THRESHOLD: u8 = 2;

// =============================================================================
// PLAYER PROGRESSION CONSTANTS
// =============================================================================

/// Maximum player level achievable
pub const MAX_PLAYER_LEVEL: u32 = 50;

/// Base experience requirement for level 2
pub const BASE_EXPERIENCE_REQUIREMENT: u32 = 100;

/// Multiplier for experience requirements (each level requires more XP)
pub const EXPERIENCE_MULTIPLIER: f32 = 1.5;

/// Maximum stat value for any player stat
pub const MAX_STAT_VALUE: u8 = 20;

/// Minimum stat value for any player stat
pub const MIN_STAT_VALUE: u8 = 1;

/// Starting stat value for new characters
pub const DEFAULT_STAT_VALUE: u8 = 10;

/// Stat points gained per level up
pub const STAT_POINTS_PER_LEVEL: u8 = 2;

// =============================================================================
// MOVEMENT AND ACTION POINTS CONSTANTS
// =============================================================================

/// Base movement points per turn
pub const BASE_MOVEMENT_POINTS: u8 = 3;

/// Base action points per turn
pub const BASE_ACTION_POINTS: u8 = 2;

/// Maximum movement points a player can have
pub const MAX_MOVEMENT_POINTS: u32 = 20;

/// Movement points regenerated per turn
pub const MOVEMENT_REGEN_PER_TURN: u32 = 3;

/// Base movement cost for normal terrain
pub const BASE_MOVEMENT_COST: u8 = 1;

/// Maximum distance for valid movement (Manhattan distance)
pub const MAX_MOVEMENT_DISTANCE: u32 = 1;

// =============================================================================
// MAP AND VISIBILITY CONSTANTS
// =============================================================================

/// Map chunk size (tiles per chunk)
pub const MAP_CHUNK_SIZE: u32 = 16;

/// Maximum map exploration radius from spawn
pub const MAX_EXPLORATION_RADIUS: u32 = 100;

/// Radius for fully visible tiles (no fog) - plus pattern around player
pub const FULLY_VISIBLE_RADIUS: u32 = 2;

/// Radius for fogged tiles (visible but with fog overlay) - diamond pattern
pub const FOGGED_VISIBLE_RADIUS: u32 = 5;

/// Extra buffer radius for tile generation (ensures tiles exist beyond visible area)
pub const TILE_GENERATION_BUFFER: u32 = 2;

/// Enable diamond pattern fog of war (true = diamond, false = circular)
pub const FOG_OF_WAR_DIAMOND_PATTERN: bool = true;

// =============================================================================
// TILE CACHING CONSTANTS
// =============================================================================

/// Number of player movement positions to keep in history
pub const PLAYER_HISTORY_SIZE: usize = 10;

/// Radius around historical positions to keep tiles loaded
pub const HISTORY_TILE_RADIUS: u32 = 1;

/// Maximum number of tiles to keep in memory
pub const MAX_LOADED_TILES: usize = 200;

/// Tile cache size for performance (keep more tiles in memory for returning)
pub const TILE_CACHE_SIZE: usize = 500;

/// Distance threshold for unloading tiles from memory
pub const TILE_UNLOAD_DISTANCE: u32 = 8;

/// Cache directory name for map tiles
pub const TILE_CACHE_DIR: &str = "cache/tiles";

// =============================================================================
// VISIBILITY CALCULATIONS
// =============================================================================

/// Maximum tiles in fully visible zone (player + 4 adjacent)
pub const MAX_FULLY_VISIBLE_TILES: usize = 5;

/// Maximum tiles in fogged visible zone (diamond pattern minus center)
pub const MAX_FOGGED_VISIBLE_TILES: usize = 20;

/// Total maximum visible tiles (fully visible + fogged)
pub const MAX_TOTAL_VISIBLE_TILES: usize = MAX_FULLY_VISIBLE_TILES + MAX_FOGGED_VISIBLE_TILES;

// =============================================================================
// RESOURCE CONSTANTS
// =============================================================================

/// Starting amount of basic resources for new players
pub const STARTING_METAL: u32 = 50;
pub const STARTING_ENERGY: u32 = 30;
pub const STARTING_FOOD: u32 = 40;

/// Maximum amount of any single resource type
pub const MAX_RESOURCE_AMOUNT: u32 = 999_999;

/// Starting resources for new bases
pub const STARTING_RESOURCES: [(ResourceType, i32); 4] = [
    (ResourceType::Metal, 50),
    (ResourceType::Energy, 30),
    (ResourceType::Food, 20),
    (ResourceType::Technology, 5),
];

/// Default storage capacity for new bases
pub const BASE_STORAGE_CAPACITY: u32 = 1000;

/// Resource regeneration rates (units per minute)
pub const SLOW_REGEN_RATE: u32 = 1;
pub const MODERATE_REGEN_RATE: u32 = 3;
pub const FAST_REGEN_RATE: u32 = 5;

/// Resource gathering time in seconds
pub const RESOURCE_GATHERING_TIME: u32 = 10;

// =============================================================================
// EVENT SYSTEM CONSTANTS
// =============================================================================

/// Event check interval in seconds
pub const EVENT_CHECK_INTERVAL: u32 = 30;

/// Probability ranges for different event types
pub const EVENT_PROBABILITY_RESOURCE: f32 = 0.25;
pub const EVENT_PROBABILITY_COMBAT: f32 = 0.15;
pub const EVENT_PROBABILITY_TRADE: f32 = 0.10;
pub const EVENT_PROBABILITY_HAZARD: f32 = 0.20;
pub const EVENT_PROBABILITY_MYSTERY: f32 = 0.05;
pub const EVENT_PROBABILITY_MALFUNCTION: f32 = 0.10;
pub const EVENT_PROBABILITY_BOON: f32 = 0.08;
pub const EVENT_PROBABILITY_NARRATIVE: f32 = 0.05;
pub const EVENT_PROBABILITY_BASE_EVENT: f32 = 0.02;

/// Event cooldown times (in game turns)
pub const EVENT_COOLDOWN_SHORT: u32 = 3;
pub const EVENT_COOLDOWN_MEDIUM: u32 = 10;
pub const EVENT_COOLDOWN_LONG: u32 = 30;

/// Maximum number of active events at once
pub const MAX_ACTIVE_EVENTS: usize = 5;

// =============================================================================
// COMBAT CONSTANTS
// =============================================================================

/// Base health for new players
pub const BASE_PLAYER_HEALTH: u32 = 100;

/// Maximum health a player can have
pub const MAX_PLAYER_HEALTH: u32 = 500;

/// Base damage for unarmed combat
pub const BASE_UNARMED_DAMAGE: u32 = 5;

/// Critical hit threshold (dice roll required)
pub const CRITICAL_HIT_THRESHOLD: u8 = 18;

/// Critical hit damage multiplier
pub const CRITICAL_HIT_MULTIPLIER: f32 = 2.0;

/// Collision detection radius for interactions (in world units)
pub const COLLISION_RADIUS: f32 = 32.0;

// =============================================================================
// LEGACY BEVY SYSTEM CONSTANTS (FOR COMPATIBILITY)
// =============================================================================

/// Player sprite size (width, height) for Bevy systems
pub const PLAYER_SIZE: (f32, f32) = (32.0, 32.0);

/// Enemy sprite size (width, height) for Bevy systems
pub const ENEMY_SIZE: (f32, f32) = (32.0, 32.0);

/// Default enemy speed for legacy systems
pub const DEFAULT_ENEMY_SPEED: f32 = 150.0;

/// Points awarded per enemy destroyed
pub const POINTS_PER_ENEMY: u32 = 10;

// =============================================================================
// BASE BUILDING CONSTANTS
// =============================================================================

/// Maximum number of buildings per base level
pub const MAX_BUILDINGS_PER_LEVEL: u8 = 3;

/// Base upgrade time (in minutes)
pub const BASE_UPGRADE_TIME: u32 = 30;

/// Resource cost multiplier for each building level
pub const BUILDING_COST_MULTIPLIER: f32 = 2.0;

/// Maximum building level
pub const MAX_BUILDING_LEVEL: u8 = 10;

// =============================================================================
// GAME TIME CONSTANTS
// =============================================================================

/// Seconds per game minute (for time scaling)
pub const SECONDS_PER_GAME_MINUTE: u32 = 60;

/// Game minutes per real minute (time acceleration)
pub const GAME_TIME_ACCELERATION: f32 = 1.0;

/// Maximum game session time (in minutes)
pub const MAX_SESSION_TIME: u32 = 480; // 8 hours

/// Enemy spawn interval in seconds
pub const ENEMY_SPAWN_INTERVAL: f32 = 2.0;

// =============================================================================
// DIFFICULTY SCALING CONSTANTS
// =============================================================================

/// Base difficulty multiplier
pub const BASE_DIFFICULTY: f32 = 1.0;

/// Difficulty increase per player level
pub const DIFFICULTY_PER_LEVEL: f32 = 0.1;

/// Maximum difficulty multiplier
pub const MAX_DIFFICULTY: f32 = 5.0;

/// Danger level thresholds
pub const LOW_DANGER_THRESHOLD: u8 = 3;
pub const MEDIUM_DANGER_THRESHOLD: u8 = 6;
pub const HIGH_DANGER_THRESHOLD: u8 = 8;

// =============================================================================
// TERRAIN RENDERING CONSTANTS
// =============================================================================

/// Terrain colors for consistent rendering across 3D map and UI components
/// These colors match the 3D map renderer materials for visual consistency
pub const TERRAIN_COLOR_PLAINS: Color = Color::srgb(0.4, 0.8, 0.3); // Green
pub const TERRAIN_COLOR_FOREST: Color = Color::srgb(0.2, 0.6, 0.2); // Dark green
pub const TERRAIN_COLOR_MOUNTAINS: Color = Color::srgb(0.6, 0.5, 0.4); // Gray-brown
pub const TERRAIN_COLOR_DESERT: Color = Color::srgb(0.9, 0.8, 0.5); // Sandy yellow
pub const TERRAIN_COLOR_TUNDRA: Color = Color::srgb(0.8, 0.9, 1.0); // Icy blue
pub const TERRAIN_COLOR_OCEAN: Color = Color::srgb(0.2, 0.4, 0.8); // Deep blue
pub const TERRAIN_COLOR_SWAMP: Color = Color::srgb(0.4, 0.5, 0.3); // Murky green
pub const TERRAIN_COLOR_VOLCANIC: Color = Color::srgb(0.8, 0.2, 0.1); // Red-orange
pub const TERRAIN_COLOR_CONSTRUCTED: Color = Color::srgb(0.5, 0.5, 0.6); // Metallic gray
pub const TERRAIN_COLOR_CAVE: Color = Color::srgb(0.3, 0.2, 0.2); // Dark brown
pub const TERRAIN_COLOR_CRYSTAL: Color = Color::srgb(0.8, 0.6, 1.0); // Purple crystal
pub const TERRAIN_COLOR_ANOMALY: Color = Color::srgb(1.0, 0.0, 1.0); // Magenta

// =============================================================================
// UI COLOR CONSTANTS
// =============================================================================

/// Main UI background colors for space-themed interface
pub const HUD_BACKGROUND: Color = Color::srgba(0.05, 0.15, 0.25, 0.9);
pub const PANEL_BACKGROUND: Color = Color::srgba(0.1, 0.2, 0.3, 0.85);
pub const SCANNER_BACKGROUND: Color = Color::srgba(0.0, 0.1, 0.2, 0.95);

/// Text colors for UI elements
pub const PRIMARY_TEXT: Color = Color::srgb(0.85, 0.95, 1.0);
pub const SECONDARY_TEXT: Color = Color::srgb(0.6, 0.8, 0.9);
pub const WARNING_TEXT: Color = Color::srgb(1.0, 0.7, 0.3);
pub const CRITICAL_TEXT: Color = Color::srgb(1.0, 0.3, 0.3);
pub const SUCCESS_TEXT: Color = Color::srgb(0.3, 0.9, 0.5);

/// Accent colors for UI elements
pub const ENERGY_COLOR: Color = Color::srgb(0.2, 0.8, 1.0);
pub const RESOURCE_COLOR: Color = Color::srgb(0.8, 0.6, 0.2);
pub const SCANNER_GRID: Color = Color::srgba(0.0, 0.8, 1.0, 0.3);

/// Sector scanner visualization colors
pub const SHIP_SIGNATURE: Color = Color::srgb(1.0, 1.0, 0.0); // Bright Yellow
pub const EXPLORED_SPACE: Color = Color::srgba(0.2, 0.6, 0.9, 0.8);
pub const UNEXPLORED_SPACE: Color = Color::srgba(0.1, 0.1, 0.3, 0.6);

/// Space object signature colors for scanner display
pub const ASTEROID_FIELD: Color = Color::srgb(0.6, 0.6, 0.6); // Gray
pub const NEBULA: Color = Color::srgb(0.8, 0.4, 0.9); // Purple
pub const STAR_SYSTEM: Color = Color::srgb(1.0, 0.9, 0.3); // Golden
pub const SPACE_STATION: Color = Color::srgb(0.3, 0.9, 0.3); // Green
pub const DERELICT: Color = Color::srgb(0.7, 0.3, 0.3); // Red
pub const ANOMALY_SIGNATURE: Color = Color::srgb(1.0, 0.2, 0.8); // Bright Pink
pub const QUANTUM_STORM: Color = Color::srgb(0.5, 0.9, 1.0); // Cyan
pub const WORMHOLE: Color = Color::srgb(0.9, 0.1, 0.9); // Magenta
pub const CRYSTAL_FORMATION: Color = Color::srgb(0.4, 0.8, 0.9); // Light Blue
pub const VOID_REGION: Color = Color::srgb(0.2, 0.1, 0.4); // Dark Purple
pub const MINING_OPERATION: Color = Color::srgb(0.9, 0.7, 0.2); // Orange
pub const ALIEN_TERRITORY: Color = Color::srgb(0.8, 0.9, 0.4); // Lime

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

/// Calculate experience required for a specific level
pub fn experience_for_level(level: u32) -> u32 {
    if level <= 1 {
        return 0;
    }

    let mut total = 0;
    let mut requirement = BASE_EXPERIENCE_REQUIREMENT;

    for _ in 2..=level {
        total += requirement;
        requirement = (requirement as f32 * EXPERIENCE_MULTIPLIER) as u32;
    }

    total
}

/// Get resource type display priority (for UI sorting)
pub fn resource_display_priority(resource_type: ResourceType) -> u8 {
    match resource_type {
        ResourceType::Metal => 1,
        ResourceType::Energy => 2,
        ResourceType::Food => 3,
        ResourceType::Technology => 4,
        ResourceType::Alloys => 5,
        ResourceType::Data => 6,
        ResourceType::Organics => 7,
        ResourceType::ExoticMatter => 8,
    }
}

/// Calculate movement cost based on terrain and conditions
pub fn calculate_movement_cost(base_cost: u8, terrain_modifier: f32, danger_level: u8) -> u8 {
    let modified_cost = (base_cost as f32) * terrain_modifier;
    let danger_penalty = (danger_level as f32) * 0.1;
    let final_cost = modified_cost + danger_penalty;

    (final_cost.ceil() as u8).max(1)
}

/// Check if a dice roll is a critical success
pub fn is_critical_success(roll: u8, dice_type: DiceType) -> bool {
    let max_value = match dice_type {
        DiceType::D4 => 4,
        DiceType::D6 => 6,
        DiceType::D8 => 8,
        DiceType::D10 => 10,
        DiceType::D12 => 12,
        DiceType::D20 => 20,
        DiceType::D100 => 100,
    };

    roll == max_value
}

/// Check if a dice roll is a critical failure
pub fn is_critical_failure(roll: u8) -> bool {
    roll == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_experience_calculation() {
        assert_eq!(experience_for_level(1), 0);
        assert_eq!(experience_for_level(2), BASE_EXPERIENCE_REQUIREMENT);
        assert!(experience_for_level(3) > experience_for_level(2));
    }

    #[test]
    fn test_movement_cost_calculation() {
        let base_cost = calculate_movement_cost(1, 1.0, 0);
        assert_eq!(base_cost, 1);

        let difficult_terrain = calculate_movement_cost(1, 2.0, 5);
        assert!(difficult_terrain > base_cost);
    }

    #[test]
    fn test_critical_rolls() {
        assert!(is_critical_success(20, DiceType::D20));
        assert!(!is_critical_success(19, DiceType::D20));

        assert!(is_critical_failure(1));
        assert!(!is_critical_failure(2));
    }

    #[test]
    fn test_resource_display_priority() {
        assert_eq!(resource_display_priority(ResourceType::Metal), 1);
        assert!(
            resource_display_priority(ResourceType::ExoticMatter)
                > resource_display_priority(ResourceType::Metal)
        );
    }

    #[test]
    fn test_constants_validity() {
        // Ensure constants are within reasonable ranges
        assert!(MAX_PLAYER_LEVEL > 1);
        assert!(BASE_EXPERIENCE_REQUIREMENT > 0);
        assert!(EXPERIENCE_MULTIPLIER > 1.0);
        assert!(BASE_EVENT_CHANCE >= 0.0 && BASE_EVENT_CHANCE <= 1.0);

        // Visibility constants
        assert!(FOGGED_VISIBLE_RADIUS > FULLY_VISIBLE_RADIUS);
        assert!(TILE_GENERATION_BUFFER > 0);

        // Movement constants
        assert!(MAX_MOVEMENT_POINTS >= BASE_MOVEMENT_POINTS as u32);
        assert!(BASE_MOVEMENT_COST > 0);

        // Resource constants
        assert!(MAX_RESOURCE_AMOUNT > 0);
        assert!(BASE_STORAGE_CAPACITY > 0);
    }

    #[test]
    fn test_fog_of_war_calculations() {
        // Test that visibility calculations make sense
        assert_eq!(
            MAX_TOTAL_VISIBLE_TILES,
            MAX_FULLY_VISIBLE_TILES + MAX_FOGGED_VISIBLE_TILES
        );
        assert!(MAX_FOGGED_VISIBLE_TILES > MAX_FULLY_VISIBLE_TILES);
    }

    #[test]
    fn test_terrain_colors_defined() {
        // Ensure all terrain colors are defined and valid
        assert_ne!(TERRAIN_COLOR_PLAINS, TERRAIN_COLOR_FOREST);
        assert_ne!(TERRAIN_COLOR_DESERT, TERRAIN_COLOR_OCEAN);

        // Test that colors are in valid range
        if let Color::Srgba(srgba) = TERRAIN_COLOR_PLAINS {
            assert!(srgba.red >= 0.0 && srgba.red <= 1.0);
            assert!(srgba.green >= 0.0 && srgba.green <= 1.0);
            assert!(srgba.blue >= 0.0 && srgba.blue <= 1.0);
        }
    }

    #[test]
    fn test_scanner_color_simulation() {
        // Test that material-aware scanner colors are different from base colors
        let base_plains = get_terrain_render_color(TerrainType::Plains);
        let scanner_plains = get_terrain_scanner_color(TerrainType::Plains);

        // Scanner colors should be adjusted for material properties
        // Plains has high roughness (0.8) so should appear darker
        let base_linear = base_plains.to_linear();
        let scanner_linear = scanner_plains.to_linear();
        assert!(scanner_linear.red <= base_linear.red);
        assert!(scanner_linear.green <= base_linear.green);
        assert!(scanner_linear.blue <= base_linear.blue);

        // Test emissive materials (Volcanic should be brighter)
        let base_volcanic = get_terrain_render_color(TerrainType::Volcanic);
        let scanner_volcanic = get_terrain_scanner_color(TerrainType::Volcanic);

        // Should have some red component from emissive
        let base_vol_linear = base_volcanic.to_linear();
        let scanner_vol_linear = scanner_volcanic.to_linear();
        assert!(scanner_vol_linear.red >= base_vol_linear.red * 0.5);

        // Test metallic materials (Constructed should be darker)
        let base_constructed = get_terrain_render_color(TerrainType::Constructed);
        let scanner_constructed = get_terrain_scanner_color(TerrainType::Constructed);

        // High metallic (0.7) should make it darker
        let base_const_linear = base_constructed.to_linear();
        let scanner_const_linear = scanner_constructed.to_linear();
        assert!(scanner_const_linear.red < base_const_linear.red);

        // All colors should be in valid range
        let plains_linear = scanner_plains.to_linear();
        let volcanic_linear = scanner_volcanic.to_linear();
        let constructed_linear = scanner_constructed.to_linear();
        assert!(plains_linear.red >= 0.0 && plains_linear.red <= 1.0);
        assert!(volcanic_linear.green >= 0.0 && volcanic_linear.green <= 1.0);
        assert!(constructed_linear.blue >= 0.0 && constructed_linear.blue <= 1.0);
    }
}

/// Get the rendering color for a terrain type
/// This ensures consistent colors between 3D map and UI components
pub fn get_terrain_render_color(terrain_type: TerrainType) -> Color {
    match terrain_type {
        TerrainType::Plains => TERRAIN_COLOR_PLAINS,
        TerrainType::Forest => TERRAIN_COLOR_FOREST,
        TerrainType::Mountains => TERRAIN_COLOR_MOUNTAINS,
        TerrainType::Desert => TERRAIN_COLOR_DESERT,
        TerrainType::Tundra => TERRAIN_COLOR_TUNDRA,
        TerrainType::Ocean => TERRAIN_COLOR_OCEAN,
        TerrainType::Swamp => TERRAIN_COLOR_SWAMP,
        TerrainType::Volcanic => TERRAIN_COLOR_VOLCANIC,
        TerrainType::Constructed => TERRAIN_COLOR_CONSTRUCTED,
        TerrainType::Cave => TERRAIN_COLOR_CAVE,
        TerrainType::Crystal => TERRAIN_COLOR_CRYSTAL,
        TerrainType::Anomaly => TERRAIN_COLOR_ANOMALY,
    }
}

/// Get the scanner grid color that matches the bright 3D appearance
/// Uses colors that directly match what's visible in the 3D rendered tiles
pub fn get_terrain_scanner_color(terrain_type: TerrainType) -> Color {
    // Use bright colors that match the actual 3D appearance
    match terrain_type {
        TerrainType::Plains => Color::srgb(0.5, 0.9, 0.4), // Bright green like in 3D
        TerrainType::Forest => Color::srgb(0.3, 0.7, 0.3), // Bright forest green
        TerrainType::Mountains => Color::srgb(0.7, 0.6, 0.5), // Light gray-brown
        TerrainType::Desert => Color::srgb(1.0, 0.9, 0.6), // Very bright sandy yellow like in 3D
        TerrainType::Tundra => Color::srgb(0.9, 1.0, 1.0), // Very bright icy blue
        TerrainType::Ocean => Color::srgb(0.3, 0.5, 0.9),  // Bright blue
        TerrainType::Swamp => Color::srgb(0.5, 0.6, 0.4),  // Brighter murky green
        TerrainType::Volcanic => Color::srgb(0.9, 0.3, 0.2), // Bright red-orange
        TerrainType::Constructed => Color::srgb(0.6, 0.6, 0.7), // Light metallic gray
        TerrainType::Cave => Color::srgb(0.4, 0.3, 0.3),   // Lighter brown
        TerrainType::Crystal => Color::srgb(0.9, 0.7, 1.0), // Bright purple crystal
        TerrainType::Anomaly => Color::srgb(1.0, 0.2, 1.0), // Bright magenta
    }
}
