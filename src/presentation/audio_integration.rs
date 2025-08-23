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
        movement_step: Some(asset_server.load("audio/sfx/movement/step.wav")),
        discovery_chime: Some(asset_server.load("audio/sfx/events/discovery.wav")),
        ui_click: Some(asset_server.load("audio/sfx/ui/click.wav")),
        resource_collect: Some(asset_server.load("audio/sfx/resources/collect.wav")),
        rest_complete: Some(asset_server.load("audio/sfx/events/rest_complete.wav")),
    };

    commands.insert_resource(audio_assets);
    info!("✅ Audio assets loaded successfully");
}

/// System to handle movement-related audio
fn handle_movement_audio(
    mut commands: Commands,
    mut movement_events: EventReader<MovementAttemptEvent>,
    audio_assets: Res<AudioAssets>,
) {
    for event in movement_events.read() {
        info!(
            "🎵 Processing movement audio event: success={}",
            event.success
        );

        // First trigger the dice roll sound
        if let Some(_dice_result) = &event.dice_result {
            if let Some(dice_handle) = &audio_assets.dice_roll {
                info!("🎲 Playing dice roll sound");
                commands.spawn(AudioPlayer::new(dice_handle.clone()));
            }
        }

        // Play footstep sound for successful movement
        if event.success {
            if let Some(step_handle) = &audio_assets.movement_step {
                info!("👟 Playing footstep sound");
                commands.spawn(AudioPlayer::new(step_handle.clone()));
            }
        }
    }
}

/// System to handle system event audio
fn handle_system_audio(
    mut commands: Commands,
    mut system_events: EventReader<GameSystemEvent>,
    audio_assets: Res<AudioAssets>,
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
            commands.spawn(AudioPlayer::new(handle.clone()));
        }
    }
}

/// System to handle discovery-related audio
fn handle_discovery_audio(
    mut commands: Commands,
    mut discovery_events: EventReader<DiscoveryEvent>,
    audio_assets: Res<AudioAssets>,
) {
    for event in discovery_events.read() {
        info!(
            "🔍 Processing discovery audio event: {:?}",
            event.discovery_type
        );

        if let Some(discovery_handle) = &audio_assets.discovery_chime {
            commands.spawn(AudioPlayer::new(discovery_handle.clone()));
        }
    }
}

/// System to handle resource-related audio
fn handle_resource_audio(
    mut commands: Commands,
    mut resource_events: EventReader<ResourceChangedEvent>,
    audio_assets: Res<AudioAssets>,
) {
    for event in resource_events.read() {
        info!(
            "💰 Processing resource audio event: {:?}",
            event.resource_type
        );

        if let Some(resource_handle) = &audio_assets.resource_collect {
            commands.spawn(AudioPlayer::new(resource_handle.clone()));
        }
    }
}

/// System to handle rest-related audio
fn handle_rest_audio(
    mut commands: Commands,
    mut rest_events: EventReader<RestCompletedEvent>,
    audio_assets: Res<AudioAssets>,
) {
    for event in rest_events.read() {
        info!(
            "😴 Processing rest audio event: {:?}",
            event.result.rest_outcome
        );

        if let Some(rest_handle) = &audio_assets.rest_complete {
            commands.spawn(AudioPlayer::new(rest_handle.clone()));
        }
    }
}

// Legacy exports for compatibility - these are no longer used
#[derive(Resource, Default)]
pub struct AudioEventRegistry {
    pub assets_loaded: bool,
}
