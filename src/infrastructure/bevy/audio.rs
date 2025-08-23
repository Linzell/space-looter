//! Bevy audio system integration for Space Looter
//!
//! This module provides the Bevy-specific implementation of the audio system,
//! including components, resources, systems, and the audio system adapter.

use bevy::audio::{AudioSink, SpatialAudioSink, Volume as BevyVolume};
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::domain::entities::audio::{
    AudioAsset, AudioId, AudioPlayback, FadeSettings, PlaybackState, Volume as DomainVolume,
};
use crate::domain::services::audio_service::{AudioService, AudioServiceError, AudioSystemAdapter};
use crate::domain::value_objects::position::Position3D;

/// Plugin for Bevy audio system integration
pub struct SpaceLooterAudioPlugin;

impl Plugin for SpaceLooterAudioPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add audio resources
            .init_resource::<AudioAssets>()
            .init_resource::<AudioPlaybacks>()
            .init_resource::<AudioSettings>()
            // Add audio systems
            .add_systems(Startup, setup_audio_system)
            .add_systems(
                Update,
                (
                    update_audio_playbacks,
                    handle_spatial_audio,
                    process_fade_effects,
                    cleanup_finished_audio,
                )
                    .chain(),
            )
            // Add event handlers
            .add_event::<AudioEvent>()
            .add_event::<PlayAudioRequest>()
            .add_event::<StopAudioRequest>()
            .add_event::<VolumeChangeRequest>();
    }
}

/// Resource containing loaded audio assets
#[derive(Resource, Default)]
pub struct AudioAssets {
    pub handles: HashMap<AudioId, Handle<AudioSource>>,
    pub metadata: HashMap<AudioId, AudioAsset>,
}

impl AudioAssets {
    pub fn add_asset(&mut self, asset: AudioAsset, handle: Handle<AudioSource>) {
        let id = asset.id();
        self.metadata.insert(id, asset);
        self.handles.insert(id, handle);
    }

    pub fn get_handle(&self, asset_id: AudioId) -> Option<&Handle<AudioSource>> {
        self.handles.get(&asset_id)
    }

    pub fn get_asset(&self, asset_id: AudioId) -> Option<&AudioAsset> {
        self.metadata.get(&asset_id)
    }
}

/// Resource tracking active audio playbacks
#[derive(Resource, Default)]
pub struct AudioPlaybacks {
    pub playbacks: HashMap<AudioId, AudioPlayback>,
    pub volumes: HashMap<AudioId, f32>,
}

/// Audio system settings
#[derive(Resource)]
pub struct AudioSettings {
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub ui_volume: f32,
    pub spatial_audio_enabled: bool,
    pub max_concurrent_sounds: usize,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 0.7,
            sfx_volume: 0.8,
            ui_volume: 0.6,
            spatial_audio_enabled: true,
            max_concurrent_sounds: 32,
        }
    }
}

/// Component for spatial audio sources
#[derive(Component)]
pub struct SpatialAudioSource {
    pub asset_id: AudioId,
    pub position: Position3D,
    pub max_distance: f32,
    pub rolloff_factor: f32,
}

impl SpatialAudioSource {
    pub fn new(asset_id: AudioId, position: Position3D) -> Self {
        Self {
            asset_id,
            position,
            max_distance: 100.0,
            rolloff_factor: 1.0,
        }
    }

    pub fn with_distance(mut self, max_distance: f32) -> Self {
        self.max_distance = max_distance;
        self
    }

    pub fn with_rolloff(mut self, rolloff_factor: f32) -> Self {
        self.rolloff_factor = rolloff_factor;
        self
    }
}

/// Component for fade effects
#[derive(Component)]
pub struct AudioFade {
    pub playback_id: AudioId,
    pub target_volume: f32,
    pub duration: Duration,
    pub elapsed: Duration,
    pub curve: FadeCurve,
    pub fade_type: FadeType,
}

#[derive(Clone, Copy)]
pub enum FadeType {
    In,
    Out,
}

#[derive(Clone, Copy)]
pub enum FadeCurve {
    Linear,
    Exponential,
    Logarithmic,
    Sine,
}

impl AudioFade {
    pub fn fade_in(playback_id: AudioId, target_volume: f32, duration: Duration) -> Self {
        Self {
            playback_id,
            target_volume,
            duration,
            elapsed: Duration::ZERO,
            curve: FadeCurve::Linear,
            fade_type: FadeType::In,
        }
    }

    pub fn fade_out(playback_id: AudioId, duration: Duration) -> Self {
        Self {
            playback_id,
            target_volume: 0.0,
            duration,
            elapsed: Duration::ZERO,
            curve: FadeCurve::Linear,
            fade_type: FadeType::Out,
        }
    }

    pub fn calculate_volume(&self) -> f32 {
        let progress = (self.elapsed.as_secs_f32() / self.duration.as_secs_f32()).clamp(0.0, 1.0);

        let curve_value = match self.curve {
            FadeCurve::Linear => progress,
            FadeCurve::Exponential => progress * progress,
            FadeCurve::Logarithmic => (progress * std::f32::consts::E).ln().max(0.0),
            FadeCurve::Sine => (progress * std::f32::consts::PI / 2.0).sin(),
        };

        match self.fade_type {
            FadeType::In => curve_value * self.target_volume,
            FadeType::Out => (1.0 - curve_value) * self.target_volume,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.elapsed >= self.duration
    }
}

/// Events for audio system communication
#[derive(Event)]
pub struct AudioEvent {
    pub event_type: AudioEventType,
}

#[derive(Debug, Clone)]
pub enum AudioEventType {
    AssetLoaded { asset_id: AudioId },
    PlaybackStarted { playback_id: AudioId },
    PlaybackFinished { playback_id: AudioId },
    PlaybackError { error: String },
}

#[derive(Event)]
pub struct PlayAudioRequest {
    pub asset_id: AudioId,
    pub position: Option<Position3D>,
    pub volume: Option<f32>,
    pub looping: bool,
}

#[derive(Event)]
pub struct StopAudioRequest {
    pub playback_id: AudioId,
}

#[derive(Event)]
pub struct VolumeChangeRequest {
    pub playback_id: AudioId,
    pub volume: f32,
}

/// Setup system for audio initialization
fn setup_audio_system(_commands: Commands) {
    info!("Initializing Space Looter audio system");

    // The audio system is automatically initialized by Bevy
    // We can add any additional setup here if needed
}

/// System to update audio playbacks
fn update_audio_playbacks(
    _time: Res<Time>,
    mut playbacks: ResMut<AudioPlaybacks>,
    mut audio_events: EventWriter<AudioEvent>,
) {
    // For now, just keep the playbacks
    // In a real implementation, we would check which playbacks have finished
    // and clean them up accordingly
}

/// System to handle spatial audio positioning
fn handle_spatial_audio(
    spatial_sources: Query<&SpatialAudioSource>,
    _playbacks: ResMut<AudioPlaybacks>,
) {
    // Update spatial audio positions based on game entities
    // This would integrate with the player position system
    for _spatial_source in spatial_sources.iter() {
        // Position updates would be handled here
        // For now, we'll keep the existing position
    }
}

/// System to process fade effects
fn process_fade_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut fade_query: Query<(Entity, &mut AudioFade)>,
    mut playbacks: ResMut<AudioPlaybacks>,
) {
    for (entity, mut fade) in fade_query.iter_mut() {
        fade.elapsed += time.delta();

        let volume = fade.calculate_volume();

        // Store volume for the playback
        playbacks.volumes.insert(fade.playback_id, volume);

        // Remove fade component when complete
        if fade.is_complete() {
            commands.entity(entity).remove::<AudioFade>();
        }
    }
}

/// System to clean up finished audio
fn cleanup_finished_audio(mut playbacks: ResMut<AudioPlaybacks>) {
    // Remove any playbacks that are in stopped state
    playbacks
        .playbacks
        .retain(|_, playback| playback.state() != &PlaybackState::Stopped);
}

/// Bevy audio adapter implementation
pub struct BevyAudioAdapter {
    assets: Arc<Mutex<HashMap<AudioId, Handle<AudioSource>>>>,
}

impl BevyAudioAdapter {
    pub fn new() -> Self {
        Self {
            assets: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_asset(&self, asset_id: AudioId, handle: Handle<AudioSource>) {
        if let Ok(mut assets) = self.assets.lock() {
            assets.insert(asset_id, handle);
        }
    }

    fn convert_volume(volume: &DomainVolume) -> f32 {
        volume.value()
    }

    fn convert_fade_settings(fade: &FadeSettings) -> (Duration, FadeCurve) {
        let curve = match fade.curve {
            crate::domain::entities::audio::FadeCurve::Linear => FadeCurve::Linear,
            crate::domain::entities::audio::FadeCurve::Exponential => FadeCurve::Exponential,
            crate::domain::entities::audio::FadeCurve::Logarithmic => FadeCurve::Logarithmic,
            crate::domain::entities::audio::FadeCurve::Sine => FadeCurve::Sine,
        };
        (fade.duration, curve)
    }
}

impl AudioSystemAdapter for BevyAudioAdapter {
    fn start_playback(
        &self,
        playback_id: AudioId,
        asset: &AudioAsset,
        playback: &AudioPlayback,
        final_volume: DomainVolume,
    ) -> Result<(), AudioServiceError> {
        // This would need access to Bevy's audio system to create the actual AudioSink
        // In a real implementation, this would be integrated with Bevy's ECS
        // For now, we'll return Ok to satisfy the trait

        info!(
            "Starting playback {} for asset {} with volume {}",
            playback_id,
            asset.name(),
            final_volume.value()
        );

        Ok(())
    }

    fn stop_playback(&self, _playback_id: AudioId) -> Result<(), AudioServiceError> {
        // In a real implementation, we would stop the actual audio sink
        info!("Stopping playback {}", _playback_id);
        Ok(())
    }

    fn pause_playback(&self, _playback_id: AudioId) -> Result<(), AudioServiceError> {
        // In a real implementation, we would pause the actual audio sink
        info!("Pausing playback {}", _playback_id);
        Ok(())
    }

    fn resume_playback(&self, _playback_id: AudioId) -> Result<(), AudioServiceError> {
        // In a real implementation, we would resume the actual audio sink
        info!("Resuming playback {}", _playback_id);
        Ok(())
    }

    fn update_volume(
        &self,
        _playback_id: AudioId,
        volume: DomainVolume,
    ) -> Result<(), AudioServiceError> {
        // In a real implementation, we would update the actual audio sink volume
        info!(
            "Updating volume for playback {} to {}",
            _playback_id,
            Self::convert_volume(&volume)
        );
        Ok(())
    }

    fn fade_in(&self, playback_id: AudioId, _fade: FadeSettings) -> Result<(), AudioServiceError> {
        // In a real implementation, this would create a fade component
        // and add it to the ECS world
        info!("Starting fade in for playback {}", playback_id);
        Ok(())
    }

    fn fade_out(&self, playback_id: AudioId, _fade: FadeSettings) -> Result<(), AudioServiceError> {
        // In a real implementation, this would create a fade component
        // and add it to the ECS world
        info!("Starting fade out for playback {}", playback_id);
        Ok(())
    }

    fn cleanup_playback(&self, _playback_id: AudioId) -> Result<(), AudioServiceError> {
        // In a real implementation, we would clean up the actual audio sink
        info!("Cleaning up playback {}", _playback_id);
        Ok(())
    }

    fn update(&self, delta_time: Duration) -> Result<(), AudioServiceError> {
        // Update logic for the adapter (if needed)
        Ok(())
    }
}

/// Helper function to convert Position3D to Bevy's Vec3
pub fn position3d_to_vec3(pos: Position3D) -> Vec3 {
    Vec3::new(pos.x() as f32, pos.y() as f32, pos.z() as f32)
}

/// Helper function to load audio assets from the file system
pub fn load_audio_assets(
    asset_server: &AssetServer,
    audio_assets: &mut AudioAssets,
    asset_list: &[(AudioId, &str, AudioAsset)],
) {
    for (_asset_id, file_path, asset) in asset_list {
        let handle: Handle<AudioSource> = asset_server.load(*file_path);
        audio_assets.add_asset(asset.clone(), handle);

        info!("Loaded audio asset: {} from {}", asset.name(), file_path);
    }
}

/// Macro for easy audio asset registration
#[macro_export]
macro_rules! register_audio_assets {
    ($audio_assets:expr, $asset_server:expr, [
        $(($id:expr, $path:expr, $asset:expr)),* $(,)?
    ]) => {
        {
            use $crate::infrastructure::bevy::audio::load_audio_assets;
            let asset_list = vec![$(($id, $path, $asset)),*];
            load_audio_assets($asset_server, $audio_assets, &asset_list);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::audio::{AudioAssetType, AudioCategory};

    #[test]
    fn audio_fade_calculation() {
        let mut fade = AudioFade::fade_in(uuid::Uuid::new_v4(), 1.0, Duration::from_secs(2));

        // At start
        assert_eq!(fade.calculate_volume(), 0.0);

        // At halfway point
        fade.elapsed = Duration::from_secs(1);
        assert_eq!(fade.calculate_volume(), 0.5);

        // At end
        fade.elapsed = Duration::from_secs(2);
        assert_eq!(fade.calculate_volume(), 1.0);
    }

    #[test]
    fn spatial_audio_source_creation() {
        let asset_id = uuid::Uuid::new_v4();
        let position = Position3D::new(1, 2, 3);

        let source = SpatialAudioSource::new(asset_id, position)
            .with_distance(50.0)
            .with_rolloff(0.8);

        assert_eq!(source.asset_id, asset_id);
        assert_eq!(source.position, position);
        assert_eq!(source.max_distance, 50.0);
        assert_eq!(source.rolloff_factor, 0.8);
    }

    #[test]
    fn position_conversion() {
        let pos3d = Position3D::new(1, 2, 3);
        let vec3 = position3d_to_vec3(pos3d);

        assert_eq!(vec3, Vec3::new(1.0, 2.0, 3.0));
    }
}
