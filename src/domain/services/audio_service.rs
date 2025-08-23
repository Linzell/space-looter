//! Audio service for managing audio playback, events, and sound effects
//!
//! This service handles the business logic for audio management including
//! playing sounds, managing music, handling audio events, and coordinating
//! with the audio system infrastructure.

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::entities::audio::{
    AudioAction, AudioAsset, AudioCategory, AudioCondition, AudioEvent, AudioId, AudioPlayback,
    AudioPriority, AudioTrigger, FadeSettings, PlaybackState, PlaybackType, Volume,
};
use crate::domain::value_objects::position::Position3D;

/// Errors that can occur in audio service operations
#[derive(Error, Debug, Clone, PartialEq)]
pub enum AudioServiceError {
    #[error("Audio asset not found: {asset_id}")]
    AssetNotFound { asset_id: AudioId },

    #[error("Audio event not found: {event_id}")]
    EventNotFound { event_id: AudioId },

    #[error("Playback instance not found: {playback_id}")]
    PlaybackNotFound { playback_id: AudioId },

    #[error("Audio service not initialized")]
    ServiceNotInitialized,

    #[error("Maximum concurrent playbacks reached: {limit}")]
    PlaybackLimitReached { limit: usize },

    #[error("Audio condition not met: {condition}")]
    ConditionNotMet { condition: String },

    #[error("Invalid audio configuration: {message}")]
    InvalidConfiguration { message: String },

    #[error("Audio system error: {message}")]
    SystemError { message: String },
}

/// Audio service trait defining the audio management interface
pub trait AudioService: Send + Sync {
    /// Play an audio asset
    fn play_asset(
        &mut self,
        asset_id: AudioId,
        playback_type: PlaybackType,
        volume: Option<Volume>,
        position: Option<Position3D>,
    ) -> Result<AudioId, AudioServiceError>;

    /// Stop audio playback
    fn stop_playback(&mut self, playback_id: AudioId) -> Result<(), AudioServiceError>;

    /// Stop all audio in a category
    fn stop_category(&mut self, category: &AudioCategory) -> Result<(), AudioServiceError>;

    /// Pause audio playback
    fn pause_playback(&mut self, playback_id: AudioId) -> Result<(), AudioServiceError>;

    /// Resume audio playback
    fn resume_playback(&mut self, playback_id: AudioId) -> Result<(), AudioServiceError>;

    /// Change playback volume
    fn set_playback_volume(
        &mut self,
        playback_id: AudioId,
        volume: Volume,
    ) -> Result<(), AudioServiceError>;

    /// Change category volume (affects all sounds in category)
    fn set_category_volume(
        &mut self,
        category: &AudioCategory,
        volume: Volume,
    ) -> Result<(), AudioServiceError>;

    /// Fade in audio
    fn fade_in(
        &mut self,
        playback_id: AudioId,
        fade: FadeSettings,
    ) -> Result<(), AudioServiceError>;

    /// Fade out audio
    fn fade_out(
        &mut self,
        playback_id: AudioId,
        fade: FadeSettings,
    ) -> Result<(), AudioServiceError>;

    /// Switch background music with crossfade
    fn switch_music(
        &mut self,
        new_asset_id: AudioId,
        fade_duration: Option<Duration>,
    ) -> Result<AudioId, AudioServiceError>;

    /// Trigger an audio event
    fn trigger_event(&mut self, trigger: AudioTrigger) -> Result<(), AudioServiceError>;

    /// Register an audio event
    fn register_event(&mut self, event: AudioEvent) -> Result<(), AudioServiceError>;

    /// Unregister an audio event
    fn unregister_event(&mut self, event_id: AudioId) -> Result<(), AudioServiceError>;

    /// Get current playback instances
    fn get_active_playbacks(&self) -> Vec<&AudioPlayback>;

    /// Get playback by ID
    fn get_playback(&self, playback_id: AudioId) -> Option<&AudioPlayback>;

    /// Check if asset is currently playing
    fn is_asset_playing(&self, asset_id: AudioId) -> bool;

    /// Get category volume
    fn get_category_volume(&self, category: &AudioCategory) -> Volume;

    /// Update audio service (called each frame)
    fn update(&mut self, delta_time: Duration) -> Result<(), AudioServiceError>;

    /// Clean up finished playbacks
    fn cleanup(&mut self) -> Result<(), AudioServiceError>;
}

/// Default implementation of AudioService
pub struct DefaultAudioService {
    /// Audio assets by ID
    assets: HashMap<AudioId, AudioAsset>,

    /// Currently active playback instances
    playbacks: HashMap<AudioId, AudioPlayback>,

    /// Registered audio events
    events: HashMap<AudioId, AudioEvent>,

    /// Category volume settings
    category_volumes: HashMap<AudioCategory, Volume>,

    /// Maximum concurrent playbacks allowed
    max_concurrent_playbacks: usize,

    /// Current background music playback ID
    current_music: Option<AudioId>,

    /// Master volume control
    master_volume: Volume,

    /// Service initialization state
    is_initialized: bool,

    /// Audio system adapter
    system_adapter: Option<Arc<dyn AudioSystemAdapter>>,
}

impl DefaultAudioService {
    /// Create a new audio service
    pub fn new() -> Self {
        let mut category_volumes = HashMap::new();

        // Set default category volumes
        category_volumes.insert(AudioCategory::Music, Volume::new(0.7).unwrap());
        category_volumes.insert(AudioCategory::Ambient, Volume::new(0.5).unwrap());
        category_volumes.insert(AudioCategory::Movement, Volume::new(0.8).unwrap());
        category_volumes.insert(AudioCategory::Dice, Volume::new(0.8).unwrap());
        category_volumes.insert(AudioCategory::Events, Volume::new(0.8).unwrap());
        category_volumes.insert(AudioCategory::UI, Volume::new(0.6).unwrap());
        category_volumes.insert(AudioCategory::Resources, Volume::new(0.8).unwrap());
        category_volumes.insert(AudioCategory::Combat, Volume::new(0.9).unwrap());
        category_volumes.insert(AudioCategory::Environmental, Volume::new(0.6).unwrap());
        category_volumes.insert(AudioCategory::Dialogue, Volume::new(0.8).unwrap());
        category_volumes.insert(AudioCategory::Narrator, Volume::new(0.8).unwrap());

        Self {
            assets: HashMap::new(),
            playbacks: HashMap::new(),
            events: HashMap::new(),
            category_volumes,
            max_concurrent_playbacks: 32,
            current_music: None,
            master_volume: Volume::new(1.0).unwrap(),
            is_initialized: false,
            system_adapter: None,
        }
    }

    /// Initialize the service with an audio system adapter
    pub fn initialize(
        &mut self,
        system_adapter: Arc<dyn AudioSystemAdapter>,
    ) -> Result<(), AudioServiceError> {
        self.system_adapter = Some(system_adapter);
        self.is_initialized = true;
        Ok(())
    }

    /// Add an audio asset to the service
    pub fn add_asset(&mut self, asset: AudioAsset) {
        self.assets.insert(asset.id(), asset);
    }

    /// Remove an audio asset
    pub fn remove_asset(&mut self, asset_id: AudioId) -> Option<AudioAsset> {
        self.assets.remove(&asset_id)
    }

    /// Get an audio asset by ID
    pub fn get_asset(&self, asset_id: AudioId) -> Option<&AudioAsset> {
        self.assets.get(&asset_id)
    }

    /// Set master volume
    pub fn set_master_volume(&mut self, volume: Volume) {
        self.master_volume = volume;

        // Collect playback data before borrowing system adapter
        let playback_updates: Vec<(AudioId, Volume)> = self
            .playbacks
            .values()
            .map(|playback| (playback.id(), self.calculate_final_volume(playback)))
            .collect();

        // Update all active playbacks
        if let Some(adapter) = &self.system_adapter {
            for (playback_id, final_volume) in playback_updates {
                let _ = adapter.update_volume(playback_id, final_volume);
            }
        }
    }

    /// Get master volume
    pub fn master_volume(&self) -> &Volume {
        &self.master_volume
    }

    /// Calculate final volume for a playback (master * category * playback)
    fn calculate_final_volume(&self, playback: &AudioPlayback) -> Volume {
        let asset = match self.assets.get(&playback.asset_id()) {
            Some(asset) => asset,
            None => return Volume::silent(),
        };

        let default_volume = Volume::max();
        let category_volume = self
            .category_volumes
            .get(asset.category())
            .unwrap_or(&default_volume);

        self.master_volume
            .multiply(category_volume)
            .multiply(playback.volume())
    }

    /// Check if conditions are met for an audio event
    fn check_conditions(&self, conditions: &[AudioCondition]) -> bool {
        for condition in conditions {
            if !self.check_single_condition(condition) {
                return false;
            }
        }
        true
    }

    /// Check a single audio condition
    fn check_single_condition(&self, condition: &AudioCondition) -> bool {
        match condition {
            AudioCondition::VolumeAbove {
                category,
                threshold,
            } => {
                let current_volume = self.get_category_volume(category);
                current_volume.value() > threshold.value()
            }
            AudioCondition::VolumeBelow {
                category,
                threshold,
            } => {
                let current_volume = self.get_category_volume(category);
                current_volume.value() < threshold.value()
            }
            AudioCondition::IsPlaying { asset_id } => self.is_asset_playing(*asset_id),
            AudioCondition::IsNotPlaying { asset_id } => !self.is_asset_playing(*asset_id),
            AudioCondition::RandomChance { probability } => {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                rng.gen::<f32>() < *probability
            }
            AudioCondition::WithinDistance {
                position,
                max_distance,
            } => {
                // For now, always return true - would need game context for actual position checking
                // This would be implemented with player position service integration
                true
            }
            AudioCondition::TimeOfDay {
                start_hour,
                end_hour,
            } => {
                // For now, always return true - would need game time service integration
                true
            }
            AudioCondition::Custom { name, parameters } => {
                // Custom conditions would be handled by external condition evaluators
                // For now, always return true
                true
            }
        }
    }

    /// Execute audio actions
    fn execute_actions(&mut self, actions: &[AudioAction]) -> Result<(), AudioServiceError> {
        for action in actions {
            self.execute_single_action(action)?;
        }
        Ok(())
    }

    /// Execute a single audio action
    fn execute_single_action(&mut self, action: &AudioAction) -> Result<(), AudioServiceError> {
        match action {
            AudioAction::PlayAsset {
                asset_id,
                volume,
                position,
                playback_type,
            } => {
                let vol = volume.clone().unwrap_or_else(|| {
                    self.assets
                        .get(asset_id)
                        .map(|a| a.default_volume().clone())
                        .unwrap_or_else(|| Volume::max())
                });
                self.play_asset(*asset_id, playback_type.clone(), Some(vol), *position)?;
            }
            AudioAction::StopAsset { asset_id } => {
                // Find and stop all playbacks of this asset
                let playbacks_to_stop: Vec<AudioId> = self
                    .playbacks
                    .iter()
                    .filter(|(_, playback)| playback.asset_id() == *asset_id)
                    .map(|(id, _)| *id)
                    .collect();

                for playback_id in playbacks_to_stop {
                    self.stop_playback(playback_id)?;
                }
            }
            AudioAction::StopCategory { category } => {
                self.stop_category(category)?;
            }
            AudioAction::ChangeVolume { asset_id, volume } => {
                // Find playbacks of this asset and update volume
                let playbacks_to_update: Vec<AudioId> = self
                    .playbacks
                    .iter()
                    .filter(|(_, playback)| playback.asset_id() == *asset_id)
                    .map(|(id, _)| *id)
                    .collect();

                for playback_id in playbacks_to_update {
                    self.set_playback_volume(playback_id, volume.clone())?;
                }
            }
            AudioAction::ChangeCategoryVolume { category, volume } => {
                self.set_category_volume(category, volume.clone())?;
            }
            AudioAction::FadeIn { asset_id, fade } => {
                // Find playbacks of this asset and fade in
                let playbacks_to_fade: Vec<AudioId> = self
                    .playbacks
                    .iter()
                    .filter(|(_, playback)| playback.asset_id() == *asset_id)
                    .map(|(id, _)| *id)
                    .collect();

                for playback_id in playbacks_to_fade {
                    self.fade_in(playback_id, fade.clone())?;
                }
            }
            AudioAction::FadeOut { asset_id, fade } => {
                // Find playbacks of this asset and fade out
                let playbacks_to_fade: Vec<AudioId> = self
                    .playbacks
                    .iter()
                    .filter(|(_, playback)| playback.asset_id() == *asset_id)
                    .map(|(id, _)| *id)
                    .collect();

                for playback_id in playbacks_to_fade {
                    self.fade_out(playback_id, fade.clone())?;
                }
            }
            AudioAction::SwitchMusic {
                asset_id,
                fade_duration,
            } => {
                self.switch_music(*asset_id, *fade_duration)?;
            }
        }
        Ok(())
    }

    /// Cleanup finished or stopped playbacks
    fn cleanup_finished_playbacks(&mut self) {
        let finished_playbacks: Vec<AudioId> = self
            .playbacks
            .iter()
            .filter(|(_, playback)| playback.state() == &PlaybackState::Stopped)
            .map(|(id, _)| *id)
            .collect();

        for playback_id in finished_playbacks {
            self.playbacks.remove(&playback_id);
            if let Some(adapter) = &self.system_adapter {
                let _ = adapter.cleanup_playback(playback_id);
            }
        }
    }
}

impl AudioService for DefaultAudioService {
    fn play_asset(
        &mut self,
        asset_id: AudioId,
        playback_type: PlaybackType,
        volume: Option<Volume>,
        position: Option<Position3D>,
    ) -> Result<AudioId, AudioServiceError> {
        if !self.is_initialized {
            return Err(AudioServiceError::ServiceNotInitialized);
        }

        if self.playbacks.len() >= self.max_concurrent_playbacks {
            return Err(AudioServiceError::PlaybackLimitReached {
                limit: self.max_concurrent_playbacks,
            });
        }

        let asset = self
            .assets
            .get(&asset_id)
            .ok_or(AudioServiceError::AssetNotFound { asset_id })?;

        let volume = volume.unwrap_or_else(|| asset.default_volume().clone());
        let mut playback = AudioPlayback::new(asset_id, playback_type, volume);

        if let Some(pos) = position {
            playback = playback.with_position(pos);
        }

        let playback_id = playback.id();

        // Calculate final volume including master and category volumes
        let final_volume = self.calculate_final_volume(&playback);

        // Start playback through system adapter
        if let Some(adapter) = &self.system_adapter {
            adapter.start_playback(playback_id, asset, &playback, final_volume)?;
        }

        self.playbacks.insert(playback_id, playback);

        Ok(playback_id)
    }

    fn stop_playback(&mut self, playback_id: AudioId) -> Result<(), AudioServiceError> {
        let playback = self
            .playbacks
            .get_mut(&playback_id)
            .ok_or(AudioServiceError::PlaybackNotFound { playback_id })?;

        playback.stop();

        if let Some(adapter) = &self.system_adapter {
            adapter.stop_playback(playback_id)?;
        }

        Ok(())
    }

    fn stop_category(&mut self, category: &AudioCategory) -> Result<(), AudioServiceError> {
        let playbacks_to_stop: Vec<AudioId> = self
            .playbacks
            .iter()
            .filter(|(_, playback)| {
                self.assets
                    .get(&playback.asset_id())
                    .map(|asset| asset.category() == category)
                    .unwrap_or(false)
            })
            .map(|(id, _)| *id)
            .collect();

        for playback_id in playbacks_to_stop {
            self.stop_playback(playback_id)?;
        }

        Ok(())
    }

    fn pause_playback(&mut self, playback_id: AudioId) -> Result<(), AudioServiceError> {
        let playback = self
            .playbacks
            .get_mut(&playback_id)
            .ok_or(AudioServiceError::PlaybackNotFound { playback_id })?;

        playback.pause();

        if let Some(adapter) = &self.system_adapter {
            adapter.pause_playback(playback_id)?;
        }

        Ok(())
    }

    fn resume_playback(&mut self, playback_id: AudioId) -> Result<(), AudioServiceError> {
        let playback = self
            .playbacks
            .get_mut(&playback_id)
            .ok_or(AudioServiceError::PlaybackNotFound { playback_id })?;

        playback.resume();

        if let Some(adapter) = &self.system_adapter {
            adapter.resume_playback(playback_id)?;
        }

        Ok(())
    }

    fn set_playback_volume(
        &mut self,
        playback_id: AudioId,
        volume: Volume,
    ) -> Result<(), AudioServiceError> {
        {
            let playback = self
                .playbacks
                .get_mut(&playback_id)
                .ok_or(AudioServiceError::PlaybackNotFound { playback_id })?;

            playback.set_volume(volume);
        }

        // Calculate new final volume after updating the playback
        let final_volume = {
            let playback = self.playbacks.get(&playback_id).unwrap();
            self.calculate_final_volume(playback)
        };

        if let Some(adapter) = &self.system_adapter {
            adapter.update_volume(playback_id, final_volume)?;
        }

        Ok(())
    }

    fn set_category_volume(
        &mut self,
        category: &AudioCategory,
        volume: Volume,
    ) -> Result<(), AudioServiceError> {
        self.category_volumes.insert(category.clone(), volume);

        // Update all active playbacks in this category
        let playbacks_to_update: Vec<AudioId> = self
            .playbacks
            .iter()
            .filter(|(_, playback)| {
                self.assets
                    .get(&playback.asset_id())
                    .map(|asset| asset.category() == category)
                    .unwrap_or(false)
            })
            .map(|(id, _)| *id)
            .collect();

        for playback_id in playbacks_to_update {
            let playback = &self.playbacks[&playback_id];
            let final_volume = self.calculate_final_volume(playback);

            if let Some(adapter) = &self.system_adapter {
                adapter.update_volume(playback_id, final_volume)?;
            }
        }

        Ok(())
    }

    fn fade_in(
        &mut self,
        playback_id: AudioId,
        fade: FadeSettings,
    ) -> Result<(), AudioServiceError> {
        let _playback = self
            .playbacks
            .get_mut(&playback_id)
            .ok_or(AudioServiceError::PlaybackNotFound { playback_id })?;

        if let Some(adapter) = &self.system_adapter {
            adapter.fade_in(playback_id, fade)?;
        }

        Ok(())
    }

    fn fade_out(
        &mut self,
        playback_id: AudioId,
        fade: FadeSettings,
    ) -> Result<(), AudioServiceError> {
        let _playback = self
            .playbacks
            .get_mut(&playback_id)
            .ok_or(AudioServiceError::PlaybackNotFound { playback_id })?;

        if let Some(adapter) = &self.system_adapter {
            adapter.fade_out(playback_id, fade)?;
        }

        Ok(())
    }

    fn switch_music(
        &mut self,
        new_asset_id: AudioId,
        fade_duration: Option<Duration>,
    ) -> Result<AudioId, AudioServiceError> {
        // Stop current music if playing
        if let Some(current_music_id) = self.current_music {
            if let Some(fade_duration) = fade_duration {
                let fade = FadeSettings {
                    duration: fade_duration,
                    curve: crate::domain::entities::audio::FadeCurve::Linear,
                };
                self.fade_out(current_music_id, fade)?;
            } else {
                self.stop_playback(current_music_id)?;
            }
        }

        // Start new music
        let new_playback_id = self.play_asset(new_asset_id, PlaybackType::Loop, None, None)?;

        self.current_music = Some(new_playback_id);

        if let Some(fade_duration) = fade_duration {
            let fade = FadeSettings {
                duration: fade_duration,
                curve: crate::domain::entities::audio::FadeCurve::Linear,
            };
            self.fade_in(new_playback_id, fade)?;
        }

        Ok(new_playback_id)
    }

    fn trigger_event(&mut self, trigger: AudioTrigger) -> Result<(), AudioServiceError> {
        let matching_events: Vec<AudioEvent> = self
            .events
            .values()
            .filter(|event| event.trigger() == &trigger && event.can_trigger())
            .cloned()
            .collect();

        for mut event in matching_events {
            if self.check_conditions(event.conditions()) {
                self.execute_actions(event.actions())?;
                event.mark_triggered();
                self.events.insert(event.id(), event);
            }
        }

        Ok(())
    }

    fn register_event(&mut self, event: AudioEvent) -> Result<(), AudioServiceError> {
        self.events.insert(event.id(), event);
        Ok(())
    }

    fn unregister_event(&mut self, event_id: AudioId) -> Result<(), AudioServiceError> {
        self.events
            .remove(&event_id)
            .ok_or(AudioServiceError::EventNotFound { event_id })?;
        Ok(())
    }

    fn get_active_playbacks(&self) -> Vec<&AudioPlayback> {
        self.playbacks.values().collect()
    }

    fn get_playback(&self, playback_id: AudioId) -> Option<&AudioPlayback> {
        self.playbacks.get(&playback_id)
    }

    fn is_asset_playing(&self, asset_id: AudioId) -> bool {
        self.playbacks.values().any(|playback| {
            playback.asset_id() == asset_id && playback.state() == &PlaybackState::Playing
        })
    }

    fn get_category_volume(&self, category: &AudioCategory) -> Volume {
        self.category_volumes
            .get(category)
            .cloned()
            .unwrap_or_else(|| Volume::max())
    }

    fn update(&mut self, delta_time: Duration) -> Result<(), AudioServiceError> {
        // Update system adapter
        if let Some(adapter) = &self.system_adapter {
            adapter.update(delta_time)?;
        }

        // Clean up finished playbacks
        self.cleanup_finished_playbacks();

        Ok(())
    }

    fn cleanup(&mut self) -> Result<(), AudioServiceError> {
        self.cleanup_finished_playbacks();
        Ok(())
    }
}

/// Trait for audio system adapters (Bevy, WASM, etc.)
pub trait AudioSystemAdapter: Send + Sync {
    fn start_playback(
        &self,
        playback_id: AudioId,
        asset: &AudioAsset,
        playback: &AudioPlayback,
        final_volume: Volume,
    ) -> Result<(), AudioServiceError>;

    fn stop_playback(&self, playback_id: AudioId) -> Result<(), AudioServiceError>;

    fn pause_playback(&self, playback_id: AudioId) -> Result<(), AudioServiceError>;

    fn resume_playback(&self, playback_id: AudioId) -> Result<(), AudioServiceError>;

    fn update_volume(&self, playback_id: AudioId, volume: Volume) -> Result<(), AudioServiceError>;

    fn fade_in(&self, playback_id: AudioId, fade: FadeSettings) -> Result<(), AudioServiceError>;

    fn fade_out(&self, playback_id: AudioId, fade: FadeSettings) -> Result<(), AudioServiceError>;

    fn cleanup_playback(&self, playback_id: AudioId) -> Result<(), AudioServiceError>;

    fn update(&self, delta_time: Duration) -> Result<(), AudioServiceError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::audio::{AudioAsset, AudioAssetType};

    #[test]
    fn service_creation() {
        let service = DefaultAudioService::new();
        assert!(!service.is_initialized);
        assert_eq!(service.playbacks.len(), 0);
        assert_eq!(service.events.len(), 0);
    }

    #[test]
    fn asset_management() {
        let mut service = DefaultAudioService::new();
        let asset = AudioAsset::new(
            "test".to_string(),
            "test.ogg".to_string(),
            AudioAssetType::SoundEffect,
            AudioCategory::UI,
        );
        let asset_id = asset.id();

        service.add_asset(asset);
        assert!(service.get_asset(asset_id).is_some());

        let removed = service.remove_asset(asset_id);
        assert!(removed.is_some());
        assert!(service.get_asset(asset_id).is_none());
    }

    #[test]
    fn volume_calculation() {
        let mut service = DefaultAudioService::new();
        service.set_master_volume(Volume::new(0.8).unwrap());

        let asset = AudioAsset::new(
            "test".to_string(),
            "test.ogg".to_string(),
            AudioAssetType::SoundEffect,
            AudioCategory::UI,
        )
        .with_volume(Volume::new(0.5).unwrap());

        let playback =
            AudioPlayback::new(asset.id(), PlaybackType::OneShot, Volume::new(0.9).unwrap());

        service.add_asset(asset);
        let final_volume = service.calculate_final_volume(&playback);

        // Master (0.8) * Category UI (0.6) * Playback (0.9) = 0.432
        assert!((final_volume.value() - 0.432).abs() < 0.001);
    }
}
