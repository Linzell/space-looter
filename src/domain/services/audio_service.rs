//! Simplified audio service for Space Looter
//! Handles ambient music (always playing) and random music playlist

use std::collections::HashMap;
use std::time::SystemTime;

use crate::domain::entities::audio::{
    AreaType, AudioAsset, AudioError, AudioId, AudioPlayback, MusicProgression, MusicSystem,
    PlaybackId, PlaybackState, SoundTrigger, TerrainType, TriggerType, Volume,
};

#[derive(Debug, Clone, thiserror::Error)]
pub enum AudioServiceError {
    #[error("Asset not found: {asset_id}")]
    AssetNotFound { asset_id: AudioId },
    #[error("Playback not found: {playback_id}")]
    PlaybackNotFound { playback_id: PlaybackId },
    #[error("Service not initialized")]
    ServiceNotInitialized,
    #[error("Invalid configuration: {message}")]
    InvalidConfiguration { message: String },
    #[error("System error: {message}")]
    SystemError { message: String },
}

pub trait AudioService {
    /// Play a sound effect
    fn play_sound(
        &mut self,
        asset_id: &AudioId,
        volume: Option<Volume>,
    ) -> Result<PlaybackId, AudioServiceError>;

    /// Stop a specific playback
    fn stop_playback(&mut self, playback_id: &PlaybackId) -> Result<(), AudioServiceError>;

    /// Set master volume
    fn set_master_volume(&mut self, volume: Volume);

    /// Start ambient music (loops automatically)
    fn start_ambient_music(&mut self, asset_id: &AudioId) -> Result<(), AudioServiceError>;

    /// Play next random music track
    fn play_next_music(&mut self) -> Result<Option<PlaybackId>, AudioServiceError>;

    /// Stop current music (not ambient)
    fn stop_current_music(&mut self);

    /// Trigger sound effect by type
    fn trigger_sound(
        &mut self,
        trigger_type: TriggerType,
    ) -> Result<Option<PlaybackId>, AudioServiceError>;

    /// Adapt music to current progression
    fn adapt_to_progression(&mut self, danger: f32, exploration: f32, area: AreaType);

    /// Set terrain-specific ambient music
    fn set_terrain_ambient(&mut self, terrain: TerrainType) -> Result<(), AudioServiceError>;

    /// Get current terrain ambient
    fn get_current_terrain(&self) -> Option<TerrainType>;

    /// Update service (call each frame)
    fn update(&mut self);

    /// Check if music track has finished
    fn is_music_finished(&self) -> bool;
}

pub struct SimpleAudioService {
    /// All loaded audio assets
    assets: HashMap<AudioId, AudioAsset>,
    /// Active playbacks
    playbacks: HashMap<PlaybackId, AudioPlayback>,
    /// Sound effect triggers
    sound_triggers: HashMap<TriggerType, SoundTrigger>,
    /// Music system with ambient + playlist
    music_system: MusicSystem,
    /// Progression system for adaptive music
    progression: MusicProgression,
    /// Master volume
    master_volume: Volume,
    /// Current ambient playback
    ambient_playback: Option<PlaybackId>,
    /// Current music playback (from playlist)
    current_music_playback: Option<PlaybackId>,
    /// Current terrain for ambient sounds
    current_terrain: Option<TerrainType>,
    /// System adapter for actual audio playback
    adapter: Option<Box<dyn AudioSystemAdapter>>,
    /// Last music change time
    last_music_change: SystemTime,
}

impl SimpleAudioService {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
            playbacks: HashMap::new(),
            sound_triggers: HashMap::new(),
            music_system: MusicSystem::new(),
            progression: MusicProgression::new(),
            master_volume: Volume::default(),
            ambient_playback: None,
            current_music_playback: None,
            current_terrain: None,
            adapter: None,
            last_music_change: SystemTime::now(),
        }
    }

    pub fn set_adapter(&mut self, adapter: Box<dyn AudioSystemAdapter>) {
        self.adapter = Some(adapter);
    }

    pub fn add_asset(&mut self, asset: AudioAsset) {
        self.assets.insert(asset.id.clone(), asset);
    }

    pub fn add_ambient_track(&mut self, asset_id: AudioId) {
        self.music_system.set_ambient_track(asset_id);
    }

    pub fn add_music_track(&mut self, asset_id: AudioId) {
        self.music_system.add_music_track(asset_id);
    }

    pub fn add_sound_trigger(&mut self, trigger_type: TriggerType, trigger: SoundTrigger) {
        self.sound_triggers.insert(trigger_type, trigger);
    }

    fn get_final_volume(&self, base_volume: Volume) -> Volume {
        base_volume.multiply(self.master_volume)
    }

    fn cleanup_finished_playbacks(&mut self) {
        let finished: Vec<PlaybackId> = self
            .playbacks
            .iter()
            .filter(|(_, playback)| matches!(playback.state, PlaybackState::Stopped))
            .map(|(id, _)| id.clone())
            .collect();

        for id in finished {
            self.playbacks.remove(&id);
            if let Some(adapter) = &mut self.adapter {
                adapter.cleanup_playback(&id);
            }
        }
    }
}

impl AudioService for SimpleAudioService {
    fn play_sound(
        &mut self,
        asset_id: &AudioId,
        volume: Option<Volume>,
    ) -> Result<PlaybackId, AudioServiceError> {
        let asset = self
            .assets
            .get(asset_id)
            .ok_or_else(|| AudioServiceError::AssetNotFound {
                asset_id: asset_id.clone(),
            })?;

        let final_volume = self.get_final_volume(volume.unwrap_or(asset.default_volume));
        let playback = AudioPlayback::new(asset_id.clone(), final_volume);
        let playback_id = playback.id.clone();

        if let Some(adapter) = &mut self.adapter {
            adapter.start_playback(&playback_id, asset_id, final_volume, false)?;
        }

        self.playbacks.insert(playback_id.clone(), playback);
        Ok(playback_id)
    }

    fn stop_playback(&mut self, playback_id: &PlaybackId) -> Result<(), AudioServiceError> {
        let playback = self.playbacks.get_mut(playback_id).ok_or_else(|| {
            AudioServiceError::PlaybackNotFound {
                playback_id: playback_id.clone(),
            }
        })?;

        playback.stop();

        if let Some(adapter) = &mut self.adapter {
            adapter.stop_playback(playback_id);
        }

        Ok(())
    }

    fn set_master_volume(&mut self, volume: Volume) {
        self.master_volume = volume;

        // Update all active playbacks
        let playback_updates: Vec<(PlaybackId, Volume)> = self
            .playbacks
            .iter()
            .map(|(id, playback)| (id.clone(), self.get_final_volume(playback.volume)))
            .collect();

        if let Some(adapter) = &mut self.adapter {
            for (id, final_volume) in playback_updates {
                adapter.update_volume(&id, final_volume);
            }
        }
    }

    fn start_ambient_music(&mut self, asset_id: &AudioId) -> Result<(), AudioServiceError> {
        let asset = self
            .assets
            .get(asset_id)
            .ok_or_else(|| AudioServiceError::AssetNotFound {
                asset_id: asset_id.clone(),
            })?;

        // Stop current ambient if playing
        if let Some(ambient_id) = self.ambient_playback.clone() {
            self.stop_playback(&ambient_id)?;
        }

        let final_volume = self.get_final_volume(self.music_system.ambient_volume);
        let playback = AudioPlayback::new(asset_id.clone(), final_volume);
        let playback_id = playback.id.clone();

        if let Some(adapter) = &mut self.adapter {
            adapter.start_playback(&playback_id, asset_id, final_volume, true)?;
            // true for looping
        }

        self.playbacks.insert(playback_id.clone(), playback);
        self.ambient_playback = Some(playback_id);

        Ok(())
    }

    fn play_next_music(&mut self) -> Result<Option<PlaybackId>, AudioServiceError> {
        // Stop current music if playing
        if let Some(music_id) = self.current_music_playback.clone() {
            self.stop_playback(&music_id)?;
            self.current_music_playback = None;
        }

        // Get next random track
        if let Some(track_id) = self.music_system.get_next_random_track() {
            let asset =
                self.assets
                    .get(&track_id)
                    .ok_or_else(|| AudioServiceError::AssetNotFound {
                        asset_id: track_id.clone(),
                    })?;

            let final_volume = self.get_final_volume(self.music_system.music_volume);
            let playback = AudioPlayback::new(track_id.clone(), final_volume);
            let playback_id = playback.id.clone();

            if let Some(adapter) = &mut self.adapter {
                adapter.start_playback(&playback_id, &track_id, final_volume, false)?;
            }

            self.playbacks.insert(playback_id.clone(), playback);
            self.current_music_playback = Some(playback_id.clone());
            self.last_music_change = SystemTime::now();

            Ok(Some(playback_id))
        } else {
            Ok(None)
        }
    }

    fn stop_current_music(&mut self) {
        if let Some(music_id) = self.current_music_playback.clone() {
            let _ = self.stop_playback(&music_id);
            self.current_music_playback = None;
        }
    }

    fn trigger_sound(
        &mut self,
        trigger_type: TriggerType,
    ) -> Result<Option<PlaybackId>, AudioServiceError> {
        if let Some(trigger) = self.sound_triggers.get(&trigger_type).cloned() {
            let playback_id = self.play_sound(&trigger.sound_id, Some(trigger.volume))?;
            Ok(Some(playback_id))
        } else {
            Ok(None)
        }
    }

    fn adapt_to_progression(&mut self, danger: f32, exploration: f32, area: AreaType) {
        self.progression
            .adapt_to_progression(danger, exploration, area);

        // Update ambient volume immediately
        if let Some(ambient_id) = self.ambient_playback.clone() {
            let final_volume = self.get_final_volume(self.music_system.ambient_volume);
            if let Some(adapter) = &mut self.adapter {
                adapter.update_volume(&ambient_id, final_volume);
            }
        }

        // Update current music volume immediately
        if let Some(music_id) = self.current_music_playback.clone() {
            let final_volume = self.get_final_volume(self.music_system.music_volume);
            if let Some(adapter) = &mut self.adapter {
                adapter.update_volume(&music_id, final_volume);
            }
        }
    }

    fn update(&mut self) {
        self.cleanup_finished_playbacks();

        if let Some(adapter) = &mut self.adapter {
            adapter.update();
        }
    }

    fn is_music_finished(&self) -> bool {
        if let Some(music_id) = &self.current_music_playback {
            if let Some(playback) = self.playbacks.get(music_id) {
                matches!(playback.state, PlaybackState::Stopped)
            } else {
                true // Playback not found, consider it finished
            }
        } else {
            true // No music playing
        }
    }

    fn set_terrain_ambient(&mut self, terrain: TerrainType) -> Result<(), AudioServiceError> {
        use crate::domain::constants::get_ambient_sound_for_terrain;

        // Only change if terrain actually changed
        if let Some(current) = &self.current_terrain {
            if std::mem::discriminant(current) == std::mem::discriminant(&terrain) {
                return Ok(()); // Same terrain, no change needed
            }
        }

        // Get the ambient sound path for this terrain
        let ambient_path = get_ambient_sound_for_terrain(&terrain);
        let asset_id = ambient_path.to_string();

        // Update progression system with new terrain
        self.progression.update_terrain(terrain.clone());
        self.current_terrain = Some(terrain);

        // Start the terrain-specific ambient music
        self.start_ambient_music(&asset_id)
    }

    fn get_current_terrain(&self) -> Option<TerrainType> {
        self.current_terrain.clone()
    }
}

/// Adapter trait for actual audio system integration
pub trait AudioSystemAdapter {
    fn start_playback(
        &mut self,
        playback_id: &PlaybackId,
        asset_id: &AudioId,
        volume: Volume,
        looping: bool,
    ) -> Result<(), AudioServiceError>;
    fn stop_playback(&mut self, playback_id: &PlaybackId);
    fn update_volume(&mut self, playback_id: &PlaybackId, volume: Volume);
    fn cleanup_playback(&mut self, playback_id: &PlaybackId);
    fn update(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::audio::{AudioAsset, AudioAssetType};

    struct MockAdapter;

    impl AudioSystemAdapter for MockAdapter {
        fn start_playback(
            &mut self,
            _: &PlaybackId,
            _: &AudioId,
            _: Volume,
            _: bool,
        ) -> Result<(), AudioServiceError> {
            Ok(())
        }
        fn stop_playback(&mut self, _: &PlaybackId) {}
        fn update_volume(&mut self, _: &PlaybackId, _: Volume) {}
        fn cleanup_playback(&mut self, _: &PlaybackId) {}
        fn update(&mut self) {}
    }

    #[test]
    fn service_creation() {
        let service = SimpleAudioService::new();
        assert_eq!(service.assets.len(), 0);
        assert_eq!(service.playbacks.len(), 0);
    }

    #[test]
    fn asset_management() {
        let mut service = SimpleAudioService::new();

        let asset = AudioAsset::new(
            "test".to_string(),
            "test.ogg".to_string(),
            AudioAssetType::SoundEffect,
        );
        let asset_id = asset.id.clone();

        service.add_asset(asset);
        assert!(service.assets.contains_key(&asset_id));
    }

    #[test]
    fn music_system_integration() {
        let mut service = SimpleAudioService::new();

        service.add_music_track("track1".to_string());
        service.add_music_track("track2".to_string());

        assert_eq!(service.music_system.music_playlist.len(), 2);
    }
}
