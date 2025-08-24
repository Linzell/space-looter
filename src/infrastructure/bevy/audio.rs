//! Simplified Bevy audio system integration for Space Looter

use bevy::prelude::*;
use std::collections::HashMap;

use crate::domain::entities::audio::{AudioId, PlaybackId, Volume as DomainVolume};
use crate::domain::services::audio_service::{AudioServiceError, AudioSystemAdapter};

/// Plugin for Bevy audio system integration
pub struct SpaceLooterAudioPlugin;

impl Plugin for SpaceLooterAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AudioPlaybacks>()
            .add_systems(Update, cleanup_finished_audio);
    }
}

/// Resource tracking active audio playbacks
#[derive(Resource, Default)]
pub struct AudioPlaybacks {
    pub sinks: HashMap<PlaybackId, AudioSink>,
}

/// Bevy implementation of the audio system adapter
pub struct BevyAudioAdapter {
    playbacks: HashMap<PlaybackId, AudioSink>,
    asset_server: Handle<AudioSource>,
}

impl BevyAudioAdapter {
    pub fn new() -> Self {
        Self {
            playbacks: HashMap::new(),
            asset_server: Handle::default(),
        }
    }

    pub fn with_asset_server(mut self, asset_server: Handle<AudioSource>) -> Self {
        self.asset_server = asset_server;
        self
    }
}

impl AudioSystemAdapter for BevyAudioAdapter {
    fn start_playback(
        &mut self,
        playback_id: &PlaybackId,
        asset_id: &AudioId,
        volume: DomainVolume,
        looping: bool,
    ) -> Result<(), AudioServiceError> {
        // In a real implementation, you would:
        // 1. Load the audio asset using asset_id
        // 2. Create an AudioBundle with the loaded asset
        // 3. Configure volume and looping
        // 4. Spawn the audio entity
        // 5. Store the AudioSink for control

        info!(
            "Starting playback {} for asset {} (volume: {}, looping: {})",
            playback_id,
            asset_id,
            volume.value(),
            looping
        );

        Ok(())
    }

    fn stop_playback(&mut self, playback_id: &PlaybackId) {
        if let Some(sink) = self.playbacks.remove(playback_id) {
            sink.stop();
            info!("Stopped playback {}", playback_id);
        }
    }

    fn update_volume(&mut self, playback_id: &PlaybackId, volume: DomainVolume) {
        if let Some(sink) = self.playbacks.get_mut(playback_id) {
            sink.set_volume(bevy::audio::Volume::Linear(volume.value()));
            info!(
                "Updated volume for playback {} to {}",
                playback_id,
                volume.value()
            );
        }
    }

    fn cleanup_playback(&mut self, playback_id: &PlaybackId) {
        self.playbacks.remove(playback_id);
        info!("Cleaned up playback {}", playback_id);
    }

    fn update(&mut self) {
        // Update logic for frame-based processing
        // In a real implementation, this would handle:
        // - Fade effects
        // - Position updates for spatial audio
        // - Playback state monitoring
    }
}

/// System to clean up finished audio playbacks
pub fn cleanup_finished_audio(mut playbacks: ResMut<AudioPlaybacks>) {
    playbacks.sinks.retain(|id, sink| {
        if sink.empty() {
            info!("Removing finished audio playback: {}", id);
            false
        } else {
            true
        }
    });
}

/// Helper system to setup the audio system
pub fn setup_audio_system() {
    info!("Audio system initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapter_creation() {
        let adapter = BevyAudioAdapter::new();
        assert_eq!(adapter.playbacks.len(), 0);
    }

    #[test]
    fn volume_conversion() {
        let domain_volume = DomainVolume::new(0.5).unwrap();
        assert_eq!(domain_volume.value(), 0.5);
    }
}
