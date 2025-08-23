//! Audio Integration Module for Space Looter
//!
//! This module provides simplified audio integration using Bevy's built-in audio system.
//! It handles loading audio assets and playing sounds directly without complex service layers.

use crate::presentation::game_event_logger::{
    DiscoveryEvent, GameSystemEvent, MovementAttemptEvent, ResourceChangedEvent, RestCompletedEvent,
};
use bevy::prelude::*;

/// Plugin for audio event integration
pub struct AudioEventIntegrationPlugin;

impl Plugin for AudioEventIntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AudioAssets>()
            .add_systems(Startup, setup_audio_assets)
            .add_systems(
                Update,
                (
                    handle_movement_audio,
                    handle_discovery_audio,
                    handle_resource_audio,
                    handle_rest_audio,
                    handle_system_audio,
                    monitor_audio_status,
                ),
            );
    }
}

/// Resource containing loaded audio assets
#[derive(Resource, Default)]
pub struct AudioAssets {
    pub dice_roll: Option<Handle<AudioSource>>,
    pub menu_theme: Option<Handle<AudioSource>>,
    pub ambient_space: Option<Handle<AudioSource>>,
    pub movement_step: Option<Handle<AudioSource>>,
    pub discovery_chime: Option<Handle<AudioSource>>,
    pub ui_click: Option<Handle<AudioSource>>,
    pub resource_collect: Option<Handle<AudioSource>>,
    pub rest_complete: Option<Handle<AudioSource>>,
}

fn setup_audio_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("🎵 Loading audio assets...");

    let audio_assets = AudioAssets {
        dice_roll: Some(asset_server.load("audio/sfx/dice/dice_roll.wav")),
        menu_theme: Some(asset_server.load("audio/music/menu_theme.ogg")),
        ambient_space: Some(asset_server.load("audio/music/ambient_space.ogg")),
        movement_step: Some(asset_server.load("audio/sfx/movement/footstep_metal.wav")),
        discovery_chime: Some(asset_server.load("audio/sfx/events/crystal_chime.wav")),
        ui_click: Some(asset_server.load("audio/sfx/ui/button_click.wav")),
        resource_collect: Some(asset_server.load("audio/sfx/events/resource_found.wav")),
        rest_complete: Some(asset_server.load("audio/sfx/events/rest_complete.wav")),
    };

    commands.insert_resource(audio_assets);
    info!("✅ Audio assets loading initiated");
}

/// System to handle movement-related audio
fn handle_movement_audio(
    mut commands: Commands,
    mut movement_events: EventReader<MovementAttemptEvent>,
    audio_assets: Res<AudioAssets>,
    asset_server: Res<AssetServer>,
) {
    for event in movement_events.read() {
        info!(
            "🎵 Processing movement audio event: success={}",
            event.success
        );

        // First trigger the dice roll sound
        if let Some(_dice_result) = &event.dice_result {
            if let Some(dice_handle) = &audio_assets.dice_roll {
                let load_state = asset_server.load_state(dice_handle.id());
                info!("🎲 Playing dice roll sound (state: {:?})", load_state);
                commands.spawn(AudioPlayer::new(dice_handle.clone()));
            } else {
                warn!("🎲 No dice roll audio handle available!");
            }
        }

        // Play footstep sound for successful movement
        if event.success {
            if let Some(step_handle) = &audio_assets.movement_step {
                let load_state = asset_server.load_state(step_handle.id());
                info!("👟 Playing footstep sound (state: {:?})", load_state);
                commands.spawn(AudioPlayer::new(step_handle.clone()));
            } else {
                warn!("👟 No footstep audio handle available!");
            }
        }
    }
}

/// System to handle system event audio
fn handle_system_audio(
    mut commands: Commands,
    mut system_events: EventReader<GameSystemEvent>,
    audio_assets: Res<AudioAssets>,
    asset_server: Res<AssetServer>,
) {
    for event in system_events.read() {
        info!("🔔 Processing system audio event: {:?}", event.event_type);

        let audio_handle = match event.event_type {
            crate::presentation::game_event_logger::SystemEventType::RestPeriodEnded => {
                &audio_assets.rest_complete
            }
            _ => &audio_assets.ui_click,
        };

        if let Some(handle) = audio_handle {
            let load_state = asset_server.load_state(handle.id());
            info!("🔔 Playing system audio (state: {:?})", load_state);
            commands.spawn(AudioPlayer::new(handle.clone()));
        } else {
            warn!("🔔 No audio handle available for system event!");
        }
    }
}

/// System to handle discovery-related audio
fn handle_discovery_audio(
    mut commands: Commands,
    mut discovery_events: EventReader<DiscoveryEvent>,
    audio_assets: Res<AudioAssets>,
    asset_server: Res<AssetServer>,
) {
    for event in discovery_events.read() {
        info!(
            "🔍 Processing discovery audio event: {:?}",
            event.discovery_type
        );

        if let Some(discovery_handle) = &audio_assets.discovery_chime {
            let load_state = asset_server.load_state(discovery_handle.id());
            info!("🔍 Playing discovery chime (state: {:?})", load_state);
            commands.spawn(AudioPlayer::new(discovery_handle.clone()));
        } else {
            warn!("🔍 No discovery chime audio handle available!");
        }
    }
}

/// System to handle resource-related audio
fn handle_resource_audio(
    mut commands: Commands,
    mut resource_events: EventReader<ResourceChangedEvent>,
    audio_assets: Res<AudioAssets>,
    asset_server: Res<AssetServer>,
) {
    for event in resource_events.read() {
        info!(
            "💰 Processing resource audio event: {:?}",
            event.resource_type
        );

        if let Some(resource_handle) = &audio_assets.resource_collect {
            let load_state = asset_server.load_state(resource_handle.id());
            info!(
                "💰 Playing resource collect sound (state: {:?})",
                load_state
            );
            commands.spawn(AudioPlayer::new(resource_handle.clone()));
        } else {
            warn!("💰 No resource collect audio handle available!");
        }
    }
}

/// System to handle rest-related audio
fn handle_rest_audio(
    mut commands: Commands,
    mut rest_events: EventReader<RestCompletedEvent>,
    audio_assets: Res<AudioAssets>,
    asset_server: Res<AssetServer>,
) {
    for event in rest_events.read() {
        info!(
            "😴 Processing rest audio event: {:?}",
            event.result.rest_outcome
        );

        if let Some(rest_handle) = &audio_assets.rest_complete {
            let load_state = asset_server.load_state(rest_handle.id());
            info!("😴 Playing rest complete sound (state: {:?})", load_state);
            commands.spawn(AudioPlayer::new(rest_handle.clone()));
        } else {
            warn!("😴 No rest complete audio handle available!");
        }
    }
}

/// System to monitor audio asset loading status
fn monitor_audio_status(
    audio_assets: Res<AudioAssets>,
    asset_server: Res<AssetServer>,
    mut last_check: Local<f32>,
    time: Res<Time>,
) {
    // Check every 5 seconds
    if time.elapsed_secs() - *last_check > 5.0 {
        *last_check = time.elapsed_secs();

        let assets_to_check = [
            ("dice_roll", &audio_assets.dice_roll),
            ("menu_theme", &audio_assets.menu_theme),
            ("ambient_space", &audio_assets.ambient_space),
            ("movement_step", &audio_assets.movement_step),
            ("discovery_chime", &audio_assets.discovery_chime),
            ("ui_click", &audio_assets.ui_click),
            ("resource_collect", &audio_assets.resource_collect),
            ("rest_complete", &audio_assets.rest_complete),
        ];

        let mut loaded_count = 0;
        let mut failed_count = 0;
        let mut loading_count = 0;

        for (name, handle_opt) in assets_to_check.iter() {
            if let Some(handle) = handle_opt {
                match asset_server.load_state(handle.id()) {
                    bevy::asset::LoadState::Loaded => loaded_count += 1,
                    bevy::asset::LoadState::Loading => loading_count += 1,
                    bevy::asset::LoadState::Failed(_) => {
                        failed_count += 1;
                        error!("🚫 Audio asset '{}' failed to load!", name);
                    }
                    bevy::asset::LoadState::NotLoaded => {
                        warn!("❓ Audio asset '{}' not loaded yet", name);
                    }
                }
            }
        }

        info!(
            "🎵 Audio Status: {} loaded, {} loading, {} failed",
            loaded_count, loading_count, failed_count
        );

        if failed_count > 0 {
            error!(
                "❌ {} audio assets failed to load - check if files exist!",
                failed_count
            );
        }
    }
}

// Legacy exports for compatibility
#[derive(Resource, Default)]
pub struct AudioEventRegistry {
    pub assets_loaded: bool,
}
