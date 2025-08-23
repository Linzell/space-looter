//! Audio domain entities and value objects for Space Looter
//!
//! This module defines the core audio concepts, including sound effects,
//! music tracks, and audio events that drive the game's audio experience.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::value_objects::position::Position3D;

/// Unique identifier for audio entities
pub type AudioId = Uuid;

/// Errors that can occur in audio domain operations
#[derive(Error, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AudioError {
    #[error("Audio asset not found: {asset_id}")]
    AssetNotFound { asset_id: String },

    #[error("Invalid audio configuration: {message}")]
    InvalidConfiguration { message: String },

    #[error("Audio playback failed: {reason}")]
    PlaybackFailed { reason: String },

    #[error("Volume out of range: {volume} (must be 0.0-1.0)")]
    InvalidVolume { volume: f32 },

    #[error("Audio format not supported: {format}")]
    UnsupportedFormat { format: String },
}

/// Audio asset entity representing a sound file or music track
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioAsset {
    id: AudioId,
    name: String,
    file_path: String,
    asset_type: AudioAssetType,
    category: AudioCategory,
    duration: Option<Duration>,
    is_looping: bool,
    default_volume: Volume,
    priority: AudioPriority,
    metadata: AudioMetadata,
    created_at: DateTime<Utc>,
    version: u64,
}

impl AudioAsset {
    /// Create a new audio asset
    pub fn new(
        name: String,
        file_path: String,
        asset_type: AudioAssetType,
        category: AudioCategory,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            file_path,
            asset_type,
            category,
            duration: None,
            is_looping: false,
            default_volume: Volume::new(0.7).unwrap(),
            priority: AudioPriority::Normal,
            metadata: AudioMetadata::default(),
            created_at: Utc::now(),
            version: 1,
        }
    }

    /// Set duration for the audio asset
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self.version += 1;
        self
    }

    /// Set looping behavior
    pub fn with_looping(mut self, is_looping: bool) -> Self {
        self.is_looping = is_looping;
        self.version += 1;
        self
    }

    /// Set default volume
    pub fn with_volume(mut self, volume: Volume) -> Self {
        self.default_volume = volume;
        self.version += 1;
        self
    }

    /// Set audio priority
    pub fn with_priority(mut self, priority: AudioPriority) -> Self {
        self.priority = priority;
        self.version += 1;
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: AudioMetadata) -> Self {
        self.metadata = metadata;
        self.version += 1;
        self
    }

    // Getters
    pub fn id(&self) -> AudioId {
        self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn file_path(&self) -> &str {
        &self.file_path
    }
    pub fn asset_type(&self) -> &AudioAssetType {
        &self.asset_type
    }
    pub fn category(&self) -> &AudioCategory {
        &self.category
    }
    pub fn duration(&self) -> Option<Duration> {
        self.duration
    }
    pub fn is_looping(&self) -> bool {
        self.is_looping
    }
    pub fn default_volume(&self) -> &Volume {
        &self.default_volume
    }
    pub fn priority(&self) -> &AudioPriority {
        &self.priority
    }
    pub fn metadata(&self) -> &AudioMetadata {
        &self.metadata
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn version(&self) -> u64 {
        self.version
    }
}

/// Types of audio assets
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AudioAssetType {
    /// Music track (typically longer, looping)
    Music,
    /// Sound effect (typically short, one-shot)
    SoundEffect,
    /// Ambient sound (environmental, looping)
    Ambient,
    /// Voice/dialogue
    Voice,
}

/// Audio categories for organization and mixing
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AudioCategory {
    // Music categories
    Music,
    Ambient,

    // SFX categories
    Movement,
    Dice,
    Events,
    UI,
    Resources,
    Combat,
    Environmental,

    // Voice categories
    Dialogue,
    Narrator,
}

/// Audio priority for playback management
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AudioPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Volume value object with validation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Volume {
    value: f32,
}

impl Volume {
    /// Create a new volume (0.0 to 1.0)
    pub fn new(value: f32) -> Result<Self, AudioError> {
        if value < 0.0 || value > 1.0 {
            return Err(AudioError::InvalidVolume { volume: value });
        }
        Ok(Self { value })
    }

    /// Create silent volume
    pub fn silent() -> Self {
        Self { value: 0.0 }
    }

    /// Create maximum volume
    pub fn max() -> Self {
        Self { value: 1.0 }
    }

    /// Get the raw volume value
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Multiply by another volume (for mixing)
    pub fn multiply(&self, other: &Volume) -> Volume {
        Volume {
            value: (self.value * other.value).min(1.0),
        }
    }
}

impl Default for Volume {
    fn default() -> Self {
        Self { value: 1.0 }
    }
}

/// Audio metadata for additional information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioMetadata {
    pub tags: Vec<String>,
    pub description: String,
    pub author: Option<String>,
    pub license: Option<String>,
    pub sample_rate: Option<u32>,
    pub bit_depth: Option<u32>,
    pub channels: Option<u32>,
    pub file_size_bytes: Option<u64>,
}

impl Default for AudioMetadata {
    fn default() -> Self {
        Self {
            tags: Vec::new(),
            description: String::new(),
            author: None,
            license: None,
            sample_rate: None,
            bit_depth: None,
            channels: None,
            file_size_bytes: None,
        }
    }
}

/// Audio playback instance entity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioPlayback {
    id: AudioId,
    asset_id: AudioId,
    playback_type: PlaybackType,
    volume: Volume,
    position: Option<Position3D>,
    started_at: DateTime<Utc>,
    duration: Option<Duration>,
    fade_in: Option<FadeSettings>,
    fade_out: Option<FadeSettings>,
    state: PlaybackState,
    loop_count: u32,
    metadata: HashMap<String, String>,
    version: u64,
}

impl AudioPlayback {
    /// Create a new audio playback instance
    pub fn new(asset_id: AudioId, playback_type: PlaybackType, volume: Volume) -> Self {
        Self {
            id: Uuid::new_v4(),
            asset_id,
            playback_type,
            volume,
            position: None,
            started_at: Utc::now(),
            duration: None,
            fade_in: None,
            fade_out: None,
            state: PlaybackState::Playing,
            loop_count: 0,
            metadata: HashMap::new(),
            version: 1,
        }
    }

    /// Set spatial position for 3D audio
    pub fn with_position(mut self, position: Position3D) -> Self {
        self.position = Some(position);
        self.version += 1;
        self
    }

    /// Set fade in effect
    pub fn with_fade_in(mut self, fade: FadeSettings) -> Self {
        self.fade_in = Some(fade);
        self.version += 1;
        self
    }

    /// Set fade out effect
    pub fn with_fade_out(mut self, fade: FadeSettings) -> Self {
        self.fade_out = Some(fade);
        self.version += 1;
        self
    }

    /// Set playback duration (for limiting loops)
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self.version += 1;
        self
    }

    /// Pause playback
    pub fn pause(&mut self) {
        self.state = PlaybackState::Paused;
        self.version += 1;
    }

    /// Resume playback
    pub fn resume(&mut self) {
        self.state = PlaybackState::Playing;
        self.version += 1;
    }

    /// Stop playback
    pub fn stop(&mut self) {
        self.state = PlaybackState::Stopped;
        self.version += 1;
    }

    /// Increment loop count
    pub fn increment_loop(&mut self) {
        self.loop_count += 1;
        self.version += 1;
    }

    /// Update volume
    pub fn set_volume(&mut self, volume: Volume) {
        self.volume = volume;
        self.version += 1;
    }

    // Getters
    pub fn id(&self) -> AudioId {
        self.id
    }
    pub fn asset_id(&self) -> AudioId {
        self.asset_id
    }
    pub fn playback_type(&self) -> &PlaybackType {
        &self.playback_type
    }
    pub fn volume(&self) -> &Volume {
        &self.volume
    }
    pub fn position(&self) -> Option<Position3D> {
        self.position
    }
    pub fn started_at(&self) -> DateTime<Utc> {
        self.started_at
    }
    pub fn duration(&self) -> Option<Duration> {
        self.duration
    }
    pub fn fade_in(&self) -> Option<&FadeSettings> {
        self.fade_in.as_ref()
    }
    pub fn fade_out(&self) -> Option<&FadeSettings> {
        self.fade_out.as_ref()
    }
    pub fn state(&self) -> &PlaybackState {
        &self.state
    }
    pub fn loop_count(&self) -> u32 {
        self.loop_count
    }
    pub fn version(&self) -> u64 {
        self.version
    }
}

/// Types of audio playback
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PlaybackType {
    /// Play once and stop
    OneShot,
    /// Loop indefinitely
    Loop,
    /// Loop for a specific number of times
    LoopCount(u32),
    /// Play for a specific duration then stop
    TimedPlay(Duration),
}

/// Audio playback state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
    Fading,
}

/// Fade in/out settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FadeSettings {
    pub duration: Duration,
    pub curve: FadeCurve,
}

/// Fade curve types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FadeCurve {
    Linear,
    Exponential,
    Logarithmic,
    Sine,
}

/// Audio event for triggering sounds based on game events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioEvent {
    id: AudioId,
    name: String,
    trigger: AudioTrigger,
    actions: Vec<AudioAction>,
    conditions: Vec<AudioCondition>,
    cooldown: Option<Duration>,
    last_triggered: Option<DateTime<Utc>>,
    is_active: bool,
    priority: AudioPriority,
    version: u64,
}

impl AudioEvent {
    /// Create a new audio event
    pub fn new(name: String, trigger: AudioTrigger) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            trigger,
            actions: Vec::new(),
            conditions: Vec::new(),
            cooldown: None,
            last_triggered: None,
            is_active: true,
            priority: AudioPriority::Normal,
            version: 1,
        }
    }

    /// Add an audio action
    pub fn with_action(mut self, action: AudioAction) -> Self {
        self.actions.push(action);
        self.version += 1;
        self
    }

    /// Add a condition
    pub fn with_condition(mut self, condition: AudioCondition) -> Self {
        self.conditions.push(condition);
        self.version += 1;
        self
    }

    /// Set cooldown period
    pub fn with_cooldown(mut self, cooldown: Duration) -> Self {
        self.cooldown = Some(cooldown);
        self.version += 1;
        self
    }

    /// Check if event can be triggered (considering cooldown)
    pub fn can_trigger(&self) -> bool {
        if !self.is_active {
            return false;
        }

        if let (Some(cooldown), Some(last_triggered)) = (self.cooldown, self.last_triggered) {
            let elapsed = Utc::now().signed_duration_since(last_triggered);
            if elapsed.to_std().unwrap_or(Duration::ZERO) < cooldown {
                return false;
            }
        }

        true
    }

    /// Mark event as triggered
    pub fn mark_triggered(&mut self) {
        self.last_triggered = Some(Utc::now());
        self.version += 1;
    }

    /// Enable/disable event
    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
        self.version += 1;
    }

    // Getters
    pub fn id(&self) -> AudioId {
        self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn trigger(&self) -> &AudioTrigger {
        &self.trigger
    }
    pub fn actions(&self) -> &[AudioAction] {
        &self.actions
    }
    pub fn conditions(&self) -> &[AudioCondition] {
        &self.conditions
    }
    pub fn cooldown(&self) -> Option<Duration> {
        self.cooldown
    }
    pub fn last_triggered(&self) -> Option<DateTime<Utc>> {
        self.last_triggered
    }
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    pub fn priority(&self) -> &AudioPriority {
        &self.priority
    }
    pub fn version(&self) -> u64 {
        self.version
    }
}

/// Audio trigger conditions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AudioTrigger {
    /// Movement-related events
    MovementSuccess,
    MovementFailure,
    MovementBlocked,
    FootstepMetal,
    FootstepRock,
    FootstepSand,
    TeleportEnter,
    TeleportExit,

    /// Dice roll events
    DiceRoll,
    DiceCriticalSuccess,
    DiceCriticalFailure,
    DiceHighRoll,
    DiceLowRoll,

    /// Resource Discovery events
    ResourceDiscovery,
    ResourceFound,
    RareResourceFound,
    CrystalChime,
    MetalClank,
    OrganicSquelch,
    ResourceGain,
    ResourceLoss,
    InventoryFull,

    /// Combat & Encounter events
    EnemyApproach,
    CombatStart,
    CombatHit,
    CombatMiss,
    EnemyDefeated,
    PlayerDamage,

    /// Environmental events
    WindHowl,
    EnergyHum,
    MachineryWhir,
    CaveEcho,
    SpaceSilence,
    TerrainChange,
    WeatherChange,
    TimeOfDay,

    /// Rest & Recovery events
    RestStart,
    RestComplete,
    RestDisturbed,
    SleepDisturbed,
    HealthRestore,

    /// UI events
    ButtonClick,
    ButtonHover,
    MenuOpen,
    MenuClose,
    Notification,
    Warning,
    Error,
    Achievement,
    LevelUp,

    /// Resource Management events
    CraftSuccess,
    TradeComplete,

    /// Music layer events
    MusicAmbientSpace,
    MusicMenuTheme,
    MusicTensionDiscovery,
    MusicCombatEncounter,
    MusicPeacefulRest,
    MusicMysteryAmbient,
    MusicVictorySuccess,

    /// Adaptive Music Layers
    MusicBaseLayer,
    MusicTensionLayer,
    MusicDiscoveryLayer,
    MusicDangerLayer,

    /// Custom event by name
    Custom(String),
}

/// Actions to perform when audio event is triggered
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AudioAction {
    /// Play a specific audio asset
    PlayAsset {
        asset_id: AudioId,
        volume: Option<Volume>,
        position: Option<Position3D>,
        playback_type: PlaybackType,
    },

    /// Stop audio playback
    StopAsset { asset_id: AudioId },

    /// Stop all audio in a category
    StopCategory { category: AudioCategory },

    /// Change volume of playing audio
    ChangeVolume { asset_id: AudioId, volume: Volume },

    /// Change volume of entire category
    ChangeCategoryVolume {
        category: AudioCategory,
        volume: Volume,
    },

    /// Fade in audio
    FadeIn {
        asset_id: AudioId,
        fade: FadeSettings,
    },

    /// Fade out audio
    FadeOut {
        asset_id: AudioId,
        fade: FadeSettings,
    },

    /// Switch background music
    SwitchMusic {
        asset_id: AudioId,
        fade_duration: Option<Duration>,
    },
}

/// Conditions that must be met for audio event to trigger
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AudioCondition {
    /// Volume level condition
    VolumeAbove {
        category: AudioCategory,
        threshold: Volume,
    },
    VolumeBelow {
        category: AudioCategory,
        threshold: Volume,
    },

    /// Playback state condition
    IsPlaying {
        asset_id: AudioId,
    },
    IsNotPlaying {
        asset_id: AudioId,
    },

    /// Time-based condition
    TimeOfDay {
        start_hour: u8,
        end_hour: u8,
    },

    /// Position-based condition
    WithinDistance {
        position: Position3D,
        max_distance: f32,
    },

    /// Random chance condition
    RandomChance {
        probability: f32,
    }, // 0.0 to 1.0

    /// Custom condition by name
    Custom {
        name: String,
        parameters: HashMap<String, String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_asset_creation() {
        let asset = AudioAsset::new(
            "test_sound".to_string(),
            "audio/test.ogg".to_string(),
            AudioAssetType::SoundEffect,
            AudioCategory::UI,
        );

        assert_eq!(asset.name(), "test_sound");
        assert_eq!(asset.file_path(), "audio/test.ogg");
        assert_eq!(asset.asset_type(), &AudioAssetType::SoundEffect);
        assert_eq!(asset.category(), &AudioCategory::UI);
        assert!(!asset.is_looping());
        assert_eq!(asset.default_volume().value(), 0.7);
        assert_eq!(asset.version(), 1);
    }

    #[test]
    fn volume_validation() {
        assert!(Volume::new(0.5).is_ok());
        assert!(Volume::new(0.0).is_ok());
        assert!(Volume::new(1.0).is_ok());
        assert!(Volume::new(-0.1).is_err());
        assert!(Volume::new(1.1).is_err());
    }

    #[test]
    fn volume_multiplication() {
        let vol1 = Volume::new(0.8).unwrap();
        let vol2 = Volume::new(0.5).unwrap();
        let result = vol1.multiply(&vol2);
        assert_eq!(result.value(), 0.4);
    }

    #[test]
    fn audio_event_cooldown() {
        let mut event = AudioEvent::new("test_event".to_string(), AudioTrigger::DiceRoll)
            .with_cooldown(Duration::from_secs(5));

        assert!(event.can_trigger());

        event.mark_triggered();
        assert!(!event.can_trigger()); // Should be on cooldown
    }

    #[test]
    fn playback_state_management() {
        let mut playback = AudioPlayback::new(
            Uuid::new_v4(),
            PlaybackType::OneShot,
            Volume::new(0.8).unwrap(),
        );

        assert_eq!(playback.state(), &PlaybackState::Playing);

        playback.pause();
        assert_eq!(playback.state(), &PlaybackState::Paused);

        playback.resume();
        assert_eq!(playback.state(), &PlaybackState::Playing);

        playback.stop();
        assert_eq!(playback.state(), &PlaybackState::Stopped);
    }
}
