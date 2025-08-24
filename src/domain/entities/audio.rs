use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use uuid::Uuid;

pub type AudioId = String;
pub type PlaybackId = String;

#[derive(Debug, Clone, thiserror::Error, Serialize, Deserialize)]
pub enum AudioError {
    #[error("Asset not found: {asset_id}")]
    AssetNotFound { asset_id: AudioId },
    #[error("Invalid configuration: {message}")]
    InvalidConfiguration { message: String },
    #[error("Playback failed: {reason}")]
    PlaybackFailed { reason: String },
    #[error("Invalid volume: {volume}")]
    InvalidVolume { volume: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioAsset {
    pub id: AudioId,
    pub name: String,
    pub file_path: String,
    pub asset_type: AudioAssetType,
    pub default_volume: Volume,
    pub is_looping: bool,
    pub created_at: SystemTime,
}

impl AudioAsset {
    pub fn new(name: String, file_path: String, asset_type: AudioAssetType) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            file_path,
            asset_type,
            default_volume: Volume::default(),
            is_looping: false,
            created_at: SystemTime::now(),
        }
    }

    pub fn with_volume(mut self, volume: Volume) -> Self {
        self.default_volume = volume;
        self
    }

    pub fn with_looping(mut self, looping: bool) -> Self {
        self.is_looping = looping;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioAssetType {
    /// Background ambient music (always playing)
    Ambient,
    /// Random playlist music
    Music,
    /// Sound effects
    SoundEffect,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Volume {
    value: f32,
}

impl Volume {
    pub fn new(value: f32) -> Result<Self, AudioError> {
        if !(0.0..=1.0).contains(&value) {
            return Err(AudioError::InvalidVolume { volume: value });
        }
        Ok(Self { value })
    }

    pub fn silent() -> Self {
        Self { value: 0.0 }
    }

    pub fn max() -> Self {
        Self { value: 1.0 }
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn multiply(&self, other: Volume) -> Self {
        Self {
            value: (self.value * other.value).clamp(0.0, 1.0),
        }
    }
}

impl Default for Volume {
    fn default() -> Self {
        Self {
            value: crate::domain::constants::DEFAULT_MASTER_VOLUME,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioPlayback {
    pub id: PlaybackId,
    pub asset_id: AudioId,
    pub volume: Volume,
    pub state: PlaybackState,
    pub started_at: SystemTime,
}

impl AudioPlayback {
    pub fn new(asset_id: AudioId, volume: Volume) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            asset_id,
            volume,
            state: PlaybackState::Playing,
            started_at: SystemTime::now(),
        }
    }

    pub fn pause(&mut self) {
        self.state = PlaybackState::Paused;
    }

    pub fn resume(&mut self) {
        self.state = PlaybackState::Playing;
    }

    pub fn stop(&mut self) {
        self.state = PlaybackState::Stopped;
    }

    pub fn set_volume(&mut self, volume: Volume) {
        self.volume = volume;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
}

/// Simplified music system with ambient + random playlist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicSystem {
    /// Always playing ambient music
    pub ambient_track: Option<AudioId>,
    /// Random playlist of music tracks
    pub music_playlist: Vec<AudioId>,
    /// Currently playing music track (from playlist)
    pub current_music: Option<AudioId>,
    /// Master volume for all music
    pub music_volume: Volume,
    /// Volume for ambient track
    pub ambient_volume: Volume,
    /// Last played track index to avoid immediate repeats
    pub last_track_index: Option<usize>,
}

impl MusicSystem {
    pub fn new() -> Self {
        Self {
            ambient_track: None,
            music_playlist: Vec::new(),
            current_music: None,
            music_volume: Volume::new(crate::domain::constants::DEFAULT_MUSIC_VOLUME).unwrap(),
            ambient_volume: Volume::new(crate::domain::constants::DEFAULT_AMBIENT_VOLUME).unwrap(),
            last_track_index: None,
        }
    }

    pub fn set_ambient_track(&mut self, track_id: AudioId) {
        self.ambient_track = Some(track_id);
    }

    pub fn add_music_track(&mut self, track_id: AudioId) {
        self.music_playlist.push(track_id);
    }

    pub fn get_next_random_track(&mut self) -> Option<AudioId> {
        if self.music_playlist.is_empty() {
            return None;
        }

        if self.music_playlist.len() == 1 {
            self.last_track_index = Some(0);
            return Some(self.music_playlist[0].clone());
        }

        // Simple random selection avoiding immediate repeats
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut index = rng.gen_range(0..self.music_playlist.len());

        // Avoid repeating the last track if possible
        if let Some(last_index) = self.last_track_index {
            if index == last_index && self.music_playlist.len() > 1 {
                index = (index + 1) % self.music_playlist.len();
            }
        }

        self.last_track_index = Some(index);
        self.current_music = Some(self.music_playlist[index].clone());
        Some(self.music_playlist[index].clone())
    }

    pub fn set_music_volume(&mut self, volume: Volume) {
        self.music_volume = volume;
    }

    pub fn set_ambient_volume(&mut self, volume: Volume) {
        self.ambient_volume = volume;
    }
}

impl Default for MusicSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Progression-based music adaptation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicProgression {
    pub danger_level: f32,         // 0.0 to 1.0
    pub exploration_progress: f32, // 0.0 to 1.0
    pub current_area_type: AreaType,
    pub current_terrain_type: Option<TerrainType>, // For terrain-specific ambient
    pub music_system: MusicSystem,
}

impl MusicProgression {
    pub fn new() -> Self {
        Self {
            danger_level: 0.0,
            exploration_progress: 0.0,
            current_area_type: AreaType::Space,
            current_terrain_type: None,
            music_system: MusicSystem::new(),
        }
    }

    /// Call this during progression to adapt music
    pub fn adapt_to_progression(&mut self, danger: f32, exploration: f32, area: AreaType) {
        self.danger_level = danger.clamp(0.0, 1.0);
        self.exploration_progress = exploration.clamp(0.0, 1.0);
        self.current_area_type = area;

        // Adapt volumes based on progression
        let base_music_volume = crate::domain::constants::DEFAULT_MUSIC_VOLUME;
        let tension_modifier = self.danger_level * 0.3; // Increase volume with danger
        let new_music_volume = (base_music_volume + tension_modifier).clamp(0.0, 1.0);

        self.music_system
            .set_music_volume(Volume::new(new_music_volume).unwrap());

        // Ambient volume stays more consistent but can be slightly affected
        let ambient_modifier = self.danger_level * 0.1;
        let new_ambient_volume =
            (crate::domain::constants::DEFAULT_AMBIENT_VOLUME + ambient_modifier).clamp(0.0, 1.0);
        self.music_system
            .set_ambient_volume(Volume::new(new_ambient_volume).unwrap());
    }

    /// Update current terrain for ambient audio adaptation
    pub fn update_terrain(&mut self, terrain: TerrainType) {
        self.current_terrain_type = Some(terrain);

        // Adjust ambient volume based on terrain characteristics
        let terrain_volume_modifier = crate::domain::constants::DEFAULT_AMBIENT_VOLUME
            * match terrain {
                TerrainType::Ocean | TerrainType::Swamp => 1.33, // Louder natural sounds
                TerrainType::Desert | TerrainType::Tundra => 0.67, // Quieter, sparse sounds
                TerrainType::Volcanic | TerrainType::Anomaly => 1.67, // Prominent atmospheric sounds
                TerrainType::Cave | TerrainType::Crystal => 1.17,     // Echoing, resonant sounds
                _ => 1.0,                                             // Default ambient volume
            };

        let danger_modifier = self.danger_level * 0.1;
        let final_ambient_volume = (terrain_volume_modifier + danger_modifier).clamp(0.0, 1.0);

        self.music_system
            .set_ambient_volume(Volume::new(final_ambient_volume).unwrap());
    }

    pub fn should_change_music(&self) -> bool {
        // Simple logic: change music based on danger level changes
        // This could be expanded with more sophisticated logic
        true
    }
}

impl Default for MusicProgression {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AreaType {
    Space,
    Asteroid,
    Station,
    Nebula,
    Anomaly,
}

/// Re-export TerrainType for audio system compatibility
pub use crate::domain::value_objects::terrain::TerrainType;

/// Sound effects trigger system (kept simple)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundTrigger {
    pub trigger_type: TriggerType,
    pub sound_id: AudioId,
    pub volume: Volume,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum TriggerType {
    // Movement
    MovementSuccess,
    MovementFailure,

    // Dice
    DiceRoll,
    DiceCriticalSuccess,
    DiceCriticalFailure,

    // Resources
    ResourceFound,
    ResourceGain,

    // Combat
    CombatHit,
    EnemyDefeated,

    // UI
    ButtonClick,
    MenuOpen,
    Achievement,

    // Environment
    Environmental,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_asset_creation() {
        let asset = AudioAsset::new(
            "test_music".to_string(),
            "assets/music/test.ogg".to_string(),
            AudioAssetType::Music,
        );

        assert_eq!(asset.name, "test_music");
        assert_eq!(asset.file_path, "assets/music/test.ogg");
        assert_eq!(
            asset.default_volume.value(),
            crate::domain::constants::DEFAULT_MASTER_VOLUME
        );
    }

    #[test]
    fn music_system_random_selection() {
        let mut system = MusicSystem::new();
        system.add_music_track("track1".to_string());
        system.add_music_track("track2".to_string());
        system.add_music_track("track3".to_string());

        let track = system.get_next_random_track();
        assert!(track.is_some());
        assert!(system.music_playlist.contains(&track.unwrap()));
    }

    #[test]
    fn volume_validation() {
        assert!(Volume::new(-0.1).is_err());
        assert!(Volume::new(1.1).is_err());
        assert!(Volume::new(0.5).is_ok());
    }

    #[test]
    fn progression_adaptation() {
        let mut progression = MusicProgression::new();
        progression.adapt_to_progression(0.8, 0.5, AreaType::Asteroid);

        assert_eq!(progression.danger_level, 0.8);
        assert!(
            progression.music_system.music_volume.value()
                > crate::domain::constants::DEFAULT_MUSIC_VOLUME
        );
    }
}
