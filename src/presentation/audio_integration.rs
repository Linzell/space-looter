//! Audio Integration Module for Space Looter
//!
//! This module provides simplified audio integration using Bevy's built-in audio system.
//! It handles loading audio assets and playing sounds directly without complex service layers.

use crate::domain::constants::*;
use crate::domain::value_objects::terrain::TerrainType;
use crate::presentation::game_event_logger::{
    DiscoveryEvent, GameSystemEvent, MovementAttemptEvent, ResourceChangedEvent, RestCompletedEvent,
};
use bevy::prelude::*;

/// Event to trigger music adaptation to progression
#[derive(Event)]
pub struct MusicProgressionEvent {
    pub danger_level: f32,         // 0.0 to 1.0
    pub exploration_progress: f32, // 0.0 to 1.0
    pub area_type: AreaType,
}

/// Event to change terrain-specific ambient sound
#[derive(Event)]
pub struct TerrainChangeEvent {
    pub new_terrain: TerrainType,
    pub fade_duration: Option<f32>, // Seconds, None for instant
}

/// Types of areas that affect music selection
#[derive(Debug, Clone, PartialEq)]
pub enum AreaType {
    Space,
    Asteroid,
    Station,
    Nebula,
    Anomaly,
}

/// Plugin for audio event integration
pub struct AudioEventIntegrationPlugin;

impl Plugin for AudioEventIntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AudioAssets>()
            .init_resource::<MusicManager>()
            .add_event::<MusicProgressionEvent>()
            .add_event::<TerrainChangeEvent>()
            .add_systems(Startup, (setup_audio_assets, setup_initial_terrain))
            .add_systems(
                Update,
                (
                    handle_movement_audio,
                    handle_discovery_audio,
                    handle_resource_audio,
                    handle_rest_audio,
                    handle_system_audio,
                    monitor_audio_status,
                    manage_music_playlist,
                    handle_music_progression_events,
                    handle_terrain_change_events,
                    retry_ambient_music_loading,
                ),
            );
    }
}

/// Resource containing loaded audio assets
#[derive(Resource, Default)]
pub struct AudioAssets {
    pub dice_roll: Option<Handle<AudioSource>>,
    pub menu_theme: Option<Handle<AudioSource>>,
    pub movement_step: Option<Handle<AudioSource>>,
    pub discovery_chime: Option<Handle<AudioSource>>,
    pub ui_click: Option<Handle<AudioSource>>,
    pub resource_collect: Option<Handle<AudioSource>>,
    pub rest_complete: Option<Handle<AudioSource>>,
    // Random music playlist
    pub music_tracks: Vec<Handle<AudioSource>>,
    // Terrain-specific ambient sounds
    pub ambient_sounds: std::collections::HashMap<String, Handle<AudioSource>>,
}

/// Resource for managing music playback
#[derive(Resource)]
pub struct MusicManager {
    pub current_ambient: Option<Entity>,
    pub current_music: Option<Entity>,
    pub last_track_index: Option<usize>,
    pub music_change_timer: Timer,
    pub ambient_volume: f32,
    pub music_volume: f32,
    pub danger_level: f32,
    pub current_area: AreaType,
    pub current_terrain: Option<crate::domain::value_objects::terrain::TerrainType>,
    pub last_logged_status: String,
    pub last_ambient_retry_status: String,
}

impl Default for MusicManager {
    fn default() -> Self {
        use bevy::time::Timer;

        Self {
            current_ambient: None,
            current_music: None,
            last_track_index: None,
            music_change_timer: Timer::from_seconds(
                crate::domain::constants::MUSIC_CHANGE_INTERVAL_SECONDS,
                bevy::time::TimerMode::Repeating,
            ),
            ambient_volume: crate::domain::constants::DEFAULT_AMBIENT_VOLUME,
            music_volume: crate::domain::constants::DEFAULT_MUSIC_VOLUME,
            danger_level: 0.0,
            current_area: AreaType::Space,
            current_terrain: None,
            last_logged_status: String::new(),
            last_ambient_retry_status: String::new(),
        }
    }
}

fn setup_audio_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("🎵 Loading audio assets...");

    let mut ambient_sounds = std::collections::HashMap::new();

    // Load terrain-specific ambient sounds
    for terrain in &[
        TerrainType::Plains,
        TerrainType::Forest,
        TerrainType::Mountains,
        TerrainType::Desert,
        TerrainType::Tundra,
        TerrainType::Swamp,
        TerrainType::Ocean,
        TerrainType::Volcanic,
        TerrainType::Anomaly,
        TerrainType::Constructed,
        TerrainType::Cave,
        TerrainType::Crystal,
    ] {
        let path = get_ambient_sound_for_terrain(terrain);
        let key = get_terrain_name(terrain).to_string();
        ambient_sounds.insert(key, asset_server.load(path));
    }

    // Add Space as default/fallback
    ambient_sounds.insert("Space".to_string(), asset_server.load(AUDIO_AMBIENT_SPACE));

    let audio_assets = AudioAssets {
        dice_roll: Some(asset_server.load(AUDIO_DICE_ROLL)),
        menu_theme: Some(asset_server.load(AUDIO_MENU_THEME)),
        movement_step: Some(asset_server.load(AUDIO_MOVEMENT_STEP)),
        discovery_chime: Some(asset_server.load(AUDIO_DISCOVERY_CHIME)),
        ui_click: Some(asset_server.load(AUDIO_UI_CLICK)),
        resource_collect: Some(asset_server.load(AUDIO_RESOURCE_FOUND)),
        rest_complete: Some(asset_server.load(AUDIO_REST_COMPLETE)),
        // Load random music playlist using helper function
        music_tracks: load_music_playlist(&asset_server),
        // Terrain-specific ambient sounds
        ambient_sounds,
    };

    commands.insert_resource(audio_assets);
    info!("✅ Audio assets loading initiated");
}

/// Helper function to load music playlist - easy to expand
fn load_music_playlist(asset_server: &AssetServer) -> Vec<Handle<AudioSource>> {
    AUDIO_MUSIC_PLAYLIST
        .iter()
        .map(|file| asset_server.load(*file))
        .collect()
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
                commands.spawn((
                    AudioPlayer::new(dice_handle.clone()),
                    PlaybackSettings::ONCE.with_volume(bevy::audio::Volume::Linear(
                        crate::domain::constants::DEFAULT_SFX_VOLUME,
                    )),
                ));
            } else {
                warn!("🎲 No dice roll audio handle available!");
            }
        }

        // Play footstep sound for successful movement
        if event.success {
            if let Some(step_handle) = &audio_assets.movement_step {
                let load_state = asset_server.load_state(step_handle.id());
                info!("👟 Playing footstep sound (state: {:?})", load_state);
                commands.spawn((
                    AudioPlayer::new(step_handle.clone()),
                    PlaybackSettings::ONCE.with_volume(bevy::audio::Volume::Linear(
                        crate::domain::constants::DEFAULT_SFX_VOLUME,
                    )),
                ));
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
            commands.spawn((
                AudioPlayer::new(handle.clone()),
                PlaybackSettings::ONCE.with_volume(bevy::audio::Volume::Linear(
                    crate::domain::constants::DEFAULT_SFX_VOLUME,
                )),
            ));
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
            commands.spawn((
                AudioPlayer::new(discovery_handle.clone()),
                PlaybackSettings::ONCE.with_volume(bevy::audio::Volume::Linear(
                    crate::domain::constants::DEFAULT_SFX_VOLUME,
                )),
            ));
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
            commands.spawn((
                AudioPlayer::new(resource_handle.clone()),
                PlaybackSettings::ONCE.with_volume(bevy::audio::Volume::Linear(
                    crate::domain::constants::DEFAULT_SFX_VOLUME,
                )),
            ));
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
            commands.spawn((
                AudioPlayer::new(rest_handle.clone()),
                PlaybackSettings::ONCE.with_volume(bevy::audio::Volume::Linear(
                    crate::domain::constants::DEFAULT_SFX_VOLUME,
                )),
            ));
        } else {
            warn!("😴 No rest complete audio handle available!");
        }
    }
}

/// System to monitor audio asset loading status
fn monitor_audio_status(
    audio_assets: Res<AudioAssets>,
    asset_server: Res<AssetServer>,
    mut music_manager: ResMut<MusicManager>,
    mut last_check: Local<f32>,
    time: Res<Time>,
) {
    // Check every few seconds to avoid spam
    if time.elapsed_secs() - *last_check > AUDIO_STATUS_CHECK_INTERVAL_SECONDS as f32 {
        *last_check = time.elapsed_secs();

        let assets_to_check = [
            ("dice_roll", &audio_assets.dice_roll),
            ("menu_theme", &audio_assets.menu_theme),
            ("movement_step", &audio_assets.movement_step),
            ("discovery_chime", &audio_assets.discovery_chime),
            ("ui_click", &audio_assets.ui_click),
            ("resource_collect", &audio_assets.resource_collect),
            ("rest_complete", &audio_assets.rest_complete),
        ];

        let mut _loaded_count = 0;
        let mut failed_count = 0;
        let mut _loading_count = 0;

        for (name, handle_opt) in assets_to_check.iter() {
            if let Some(handle) = handle_opt {
                match asset_server.load_state(handle.id()) {
                    bevy::asset::LoadState::Loaded => _loaded_count += 1,
                    bevy::asset::LoadState::Loading => _loading_count += 1,
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

        // Also check music playlist
        let mut music_loaded = 0;
        let mut music_failed = 0;

        let track_names = AUDIO_TRACK_NAMES;

        for (i, handle) in audio_assets.music_tracks.iter().enumerate() {
            let track_name = track_names.get(i).unwrap_or(&"unknown");
            match asset_server.load_state(handle.id()) {
                bevy::asset::LoadState::Loaded => music_loaded += 1,
                bevy::asset::LoadState::Failed(error) => {
                    music_failed += 1;
                    error!(
                        "🚫 Music track {} ({}) failed to load: {:?}",
                        i, track_name, error
                    );
                }
                bevy::asset::LoadState::Loading => {
                    // Only log loading state occasionally to avoid spam
                    if *last_check as u32 % 10 == 0 {
                        info!("⏳ Music track {} ({}) still loading...", i, track_name);
                    }
                }
                bevy::asset::LoadState::NotLoaded => {
                    warn!("❓ Music track {} ({}) not loaded yet", i, track_name);
                }
            }
        }

        if failed_count > 0 {
            error!(
                "❌ {} audio assets failed to load - check if files exist!",
                failed_count
            );
        }

        // Generate status message and only log if it's different from last time
        let status_message = if music_failed > 0 {
            format!("❌ {} music tracks failed to load - check if audio files exist in assets/audio/music/", music_failed)
        } else if music_loaded > 0 && music_loaded == track_names.len() {
            format!("🎵 All {} music tracks loaded successfully", music_loaded)
        } else if music_loaded > 0 {
            format!(
                "🎵 {} out of {} music tracks loaded successfully",
                music_loaded,
                track_names.len()
            )
        } else {
            String::new()
        };

        // Only log if status changed
        if !status_message.is_empty() && status_message != music_manager.last_logged_status {
            if music_failed > 0 {
                error!("{}", status_message);
            } else {
                info!("{}", status_message);
            }
            music_manager.last_logged_status = status_message;
        }
    }
}

/// Retry loading ambient music until it succeeds
fn retry_ambient_music_loading(
    mut commands: Commands,
    audio_assets: Res<AudioAssets>,
    mut music_manager: ResMut<MusicManager>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
) {
    // Only try to start ambient music if we don't already have it playing
    if music_manager.current_ambient.is_some() {
        return;
    }

    // Only check periodically to avoid spam
    if (time.elapsed_secs() as u64) % AMBIENT_RETRY_INTERVAL_SECONDS != 0 {
        return;
    }

    // Try to load ambient for current terrain, fallback to space
    let (terrain_name, ambient_handle) = if let Some(terrain) = &music_manager.current_terrain {
        let terrain_name = get_terrain_name(terrain);
        let handle = audio_assets.ambient_sounds.get(terrain_name);
        (terrain_name, handle)
    } else {
        // No terrain set, use space ambient as default
        ("Space", audio_assets.ambient_sounds.get("Space"))
    };

    if let Some(handle) = ambient_handle {
        match asset_server.load_state(handle.id()) {
            bevy::asset::LoadState::Loaded => {
                info!("🎵 Starting {} ambient sound (loaded)", terrain_name);
                let entity = commands
                    .spawn((
                        AudioPlayer::new(handle.clone()),
                        PlaybackSettings::LOOP
                            .with_volume(bevy::audio::Volume::Linear(music_manager.ambient_volume)),
                    ))
                    .id();
                music_manager.current_ambient = Some(entity);
                // Clear retry status since we successfully loaded
                music_manager.last_ambient_retry_status.clear();
            }
            bevy::asset::LoadState::Loading => {
                let status = format!(
                    "⏳ {} ambient sound still loading, will retry...",
                    terrain_name
                );
                if status != music_manager.last_ambient_retry_status {
                    info!("{}", status);
                    music_manager.last_ambient_retry_status = status;
                }
            }
            bevy::asset::LoadState::Failed(error) => {
                let status = format!(
                    "🚫 {} ambient sound failed to load: {:?}",
                    terrain_name, error
                );
                if status != music_manager.last_ambient_retry_status {
                    error!("{}", status);
                    error!("   Check if audio file exists and is valid!");
                    music_manager.last_ambient_retry_status = status;
                }
            }
            bevy::asset::LoadState::NotLoaded => {
                let status = format!(
                    "❓ {} ambient sound not loaded yet, will retry...",
                    terrain_name
                );
                if status != music_manager.last_ambient_retry_status {
                    warn!("{}", status);
                    music_manager.last_ambient_retry_status = status;
                }
            }
        }
    } else {
        error!(
            "🚫 No ambient sound handle found for terrain: {}",
            terrain_name
        );
    }
}

/// Manage random music playlist
fn manage_music_playlist(
    mut commands: Commands,
    audio_assets: Res<AudioAssets>,
    mut music_manager: ResMut<MusicManager>,
    time: Res<Time>,
    audio_sinks: Query<&AudioSink>,
    asset_server: Res<AssetServer>,
) {
    music_manager.music_change_timer.tick(time.delta());

    // Early return if no music tracks are configured
    if audio_assets.music_tracks.is_empty() {
        return;
    }

    // Only try to play music if we have loaded tracks
    let loaded_tracks: Vec<_> = audio_assets
        .music_tracks
        .iter()
        .enumerate()
        .filter(|(_, handle)| {
            matches!(
                asset_server.load_state(handle.id()),
                bevy::asset::LoadState::Loaded
            )
        })
        .collect();

    // Check if current music has finished or it's time to change
    let should_change_music = if let Some(current_entity) = music_manager.current_music {
        if let Ok(sink) = audio_sinks.get(current_entity) {
            sink.empty() // Music finished
        } else {
            true // No music playing
        }
    } else {
        // Start first track immediately when tracks are loaded, not after timer
        !loaded_tracks.is_empty()
    };

    if should_change_music && !loaded_tracks.is_empty() {
        info!(
            "🎵 Attempting to change music - {} loaded tracks available",
            loaded_tracks.len()
        );
        // Stop current music if playing
        if let Some(current_entity) = music_manager.current_music {
            if let Ok(sink) = audio_sinks.get(current_entity) {
                sink.stop();
                info!("🛑 Stopped previous music track");
            }
        }

        // Pick random track from loaded tracks (avoid immediate repeats)
        let selected_track =
            get_random_loaded_track(&loaded_tracks, music_manager.last_track_index);

        if let Some((original_index, track_handle)) = selected_track {
            let track_names = AUDIO_TRACK_NAMES;
            let track_name = track_names.get(original_index).unwrap_or(&"unknown");
            info!(
                "🎵 Playing random music track {} ({})",
                original_index, track_name
            );
            let entity = commands
                .spawn((
                    AudioPlayer::new(track_handle.clone()),
                    PlaybackSettings::ONCE
                        .with_volume(bevy::audio::Volume::Linear(music_manager.music_volume)),
                ))
                .id();

            music_manager.current_music = Some(entity);
            music_manager.last_track_index = Some(original_index);
            music_manager.music_change_timer.reset();
        } else {
            // Only log occasionally to avoid spam when tracks are still loading
            if music_manager.music_change_timer.just_finished() {
                info!(
                    "🎵 No music tracks loaded yet, waiting... ({} tracks total)",
                    audio_assets.music_tracks.len()
                );
            }
        }
    }
}

/// Handle music progression events (triggered from anywhere in the game)
fn handle_music_progression_events(
    mut events: EventReader<MusicProgressionEvent>,
    mut music_manager: ResMut<MusicManager>,
    mut audio_sinks: Query<&mut AudioSink>,
) {
    for event in events.read() {
        info!(
            "🎵 Processing music progression: danger={:.2}, exploration={:.2}, area={:?}",
            event.danger_level, event.exploration_progress, event.area_type
        );

        let significant_change = (music_manager.danger_level - event.danger_level).abs() > 0.1
            || music_manager.current_area != event.area_type;

        if significant_change {
            music_manager.danger_level = event.danger_level;
            music_manager.current_area = event.area_type.clone();

            // Adapt volumes based on danger level and area
            let base_music_volume = crate::domain::constants::DEFAULT_MUSIC_VOLUME
                * match event.area_type {
                    AreaType::Space => 1.0,
                    AreaType::Asteroid => 1.17,
                    AreaType::Station => 0.83,
                    AreaType::Nebula => 1.33,
                    AreaType::Anomaly => 1.5,
                };

            let tension_modifier = event.danger_level * 0.4;
            music_manager.music_volume = (base_music_volume + tension_modifier).clamp(0.0, 1.0);

            let base_ambient_volume = crate::domain::constants::DEFAULT_AMBIENT_VOLUME;
            let ambient_modifier = event.danger_level * 0.15;
            music_manager.ambient_volume = (base_ambient_volume + ambient_modifier).clamp(0.0, 1.0);

            // Update current playing tracks
            if let Some(ambient_entity) = music_manager.current_ambient {
                if let Ok(mut sink) = audio_sinks.get_mut(ambient_entity) {
                    sink.set_volume(bevy::audio::Volume::Linear(music_manager.ambient_volume));
                }
            }

            if let Some(music_entity) = music_manager.current_music {
                if let Ok(mut sink) = audio_sinks.get_mut(music_entity) {
                    sink.set_volume(bevy::audio::Volume::Linear(music_manager.music_volume));
                }
            }

            info!(
                "🎵 Music adapted: area={:?}, danger={:.2}, music_vol={:.2}, ambient_vol={:.2}",
                event.area_type,
                event.danger_level,
                music_manager.music_volume,
                music_manager.ambient_volume
            );

            // If danger level is very high, consider switching to more intense music immediately
            if event.danger_level > 0.8 && music_manager.music_change_timer.elapsed_secs() > 10.0 {
                music_manager
                    .music_change_timer
                    .set_elapsed(std::time::Duration::from_secs_f32(25.0)); // Force music change soon
            }
        }
    }
}

/// Handle terrain change events for ambient sound switching
fn handle_terrain_change_events(
    mut events: EventReader<TerrainChangeEvent>,
    mut commands: Commands,
    audio_assets: Res<AudioAssets>,
    mut music_manager: ResMut<MusicManager>,
    asset_server: Res<AssetServer>,
    audio_sinks: Query<&AudioSink>,
) {
    for event in events.read() {
        use crate::domain::constants::{get_ambient_sound_for_terrain, get_terrain_name};

        let terrain_name = get_terrain_name(&event.new_terrain);

        // Only log if terrain actually changed
        let should_change = if let Some(current) = &music_manager.current_terrain {
            std::mem::discriminant(current) != std::mem::discriminant(&event.new_terrain)
        } else {
            true
        };

        if !should_change {
            return; // Same terrain, no change needed
        }

        info!(
            "🌍 Terrain changed to: {} - switching ambient sound",
            terrain_name
        );

        // Get the file path for this terrain's ambient sound
        let ambient_path = get_ambient_sound_for_terrain(&event.new_terrain);
        let ambient_handle = audio_assets.ambient_sounds.get(terrain_name);

        if let Some(handle) = ambient_handle {
            match asset_server.load_state(handle.id()) {
                bevy::asset::LoadState::Loaded => {
                    // Stop current ambient if playing
                    if let Some(current_ambient) = music_manager.current_ambient {
                        if let Ok(sink) = audio_sinks.get(current_ambient) {
                            sink.stop();
                            info!("🛑 Stopped previous ambient sound");
                        }
                    }

                    // Adjust volume based on terrain characteristics
                    let terrain_volume = crate::domain::constants::DEFAULT_AMBIENT_VOLUME
                        * match event.new_terrain {
                            crate::domain::value_objects::terrain::TerrainType::Ocean
                            | crate::domain::value_objects::terrain::TerrainType::Swamp => 1.33,
                            crate::domain::value_objects::terrain::TerrainType::Desert
                            | crate::domain::value_objects::terrain::TerrainType::Tundra => 0.67,
                            crate::domain::value_objects::terrain::TerrainType::Volcanic
                            | crate::domain::value_objects::terrain::TerrainType::Anomaly => 1.67,
                            crate::domain::value_objects::terrain::TerrainType::Cave
                            | crate::domain::value_objects::terrain::TerrainType::Crystal => 1.17,
                            _ => 1.0,
                        };

                    let danger_modifier = music_manager.danger_level * 0.1;
                    let final_volume = (terrain_volume + danger_modifier).clamp(0.0, 1.0);

                    // Start new terrain ambient
                    info!(
                        "🌍 Starting {} ambient sound (volume: {:.2})",
                        terrain_name, final_volume
                    );
                    let entity = commands
                        .spawn((
                            AudioPlayer::new(handle.clone()),
                            PlaybackSettings::LOOP
                                .with_volume(bevy::audio::Volume::Linear(final_volume)),
                        ))
                        .id();

                    music_manager.current_ambient = Some(entity);
                    music_manager.current_terrain = Some(event.new_terrain.clone());
                    music_manager.ambient_volume = final_volume;
                    music_manager.last_ambient_retry_status.clear();
                }
                bevy::asset::LoadState::Loading => {
                    if music_manager.last_ambient_retry_status != "loading" {
                        info!(
                            "⏳ {} ambient sound still loading, will retry...",
                            terrain_name
                        );
                        music_manager.last_ambient_retry_status = "loading".to_string();
                    }
                }
                bevy::asset::LoadState::Failed(error) => {
                    if !music_manager
                        .last_ambient_retry_status
                        .starts_with("failed")
                    {
                        error!(
                            "🚫 {} ambient sound failed to load: {:?}",
                            terrain_name, error
                        );
                        error!("   Path: {}", ambient_path);

                        // Try fallback to space ambient
                        if let Some(space_handle) = audio_assets.ambient_sounds.get("Space") {
                            if matches!(
                                asset_server.load_state(space_handle.id()),
                                bevy::asset::LoadState::Loaded
                            ) {
                                info!("🌌 Using space ambient as fallback");
                                let entity = commands
                                    .spawn((
                                        AudioPlayer::new(space_handle.clone()),
                                        PlaybackSettings::LOOP.with_volume(
                                            bevy::audio::Volume::Linear(
                                                crate::domain::constants::DEFAULT_MUSIC_VOLUME,
                                            ),
                                        ),
                                    ))
                                    .id();
                                music_manager.current_ambient = Some(entity);
                            }
                        }

                        music_manager.last_ambient_retry_status =
                            format!("failed: {}", terrain_name);
                    }
                }
                bevy::asset::LoadState::NotLoaded => {
                    if music_manager.last_ambient_retry_status != "not_loaded" {
                        warn!("❓ {} ambient sound not loaded yet", terrain_name);
                        music_manager.last_ambient_retry_status = "not_loaded".to_string();
                    }
                }
            }
        } else {
            if !music_manager
                .last_ambient_retry_status
                .starts_with("no_handle")
            {
                error!(
                    "🚫 No ambient sound handle found for terrain: {}",
                    terrain_name
                );
                error!("   Expected path: {}", ambient_path);
                music_manager.last_ambient_retry_status = format!("no_handle: {}", terrain_name);
            }
        }
    }
}

/// Helper function to get random loaded track avoiding immediate repeats
fn get_random_loaded_track(
    loaded_tracks: &[(usize, &Handle<AudioSource>)],
    last_index: Option<usize>,
) -> Option<(usize, Handle<AudioSource>)> {
    if loaded_tracks.is_empty() {
        return None;
    }

    if loaded_tracks.len() == 1 {
        let (original_index, handle) = loaded_tracks[0];
        return Some((original_index, handle.clone()));
    }

    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut selected_idx = rng.gen_range(0..loaded_tracks.len());

    // Avoid immediate repeat if possible
    if let Some(last) = last_index {
        let (current_original_index, _) = loaded_tracks[selected_idx];
        if current_original_index == last && loaded_tracks.len() > 1 {
            selected_idx = (selected_idx + 1) % loaded_tracks.len();
        }
    }

    let (original_index, handle) = loaded_tracks[selected_idx];
    Some((original_index, handle.clone()))
}

// Legacy exports for compatibility
#[derive(Resource, Default)]
pub struct AudioEventRegistry {
    pub assets_loaded: bool,
}

/// Helper function to send terrain change events from anywhere in your game
///
/// Usage example:
/// ```rust
/// // When player moves to new terrain
/// fn on_terrain_change(mut terrain_events: EventWriter<TerrainChangeEvent>) {
///     send_terrain_change(&mut terrain_events, TerrainType::Forest, Some(2.0));
/// }
/// ```
pub fn send_terrain_change(
    events: &mut EventWriter<TerrainChangeEvent>,
    new_terrain: crate::domain::value_objects::terrain::TerrainType,
    fade_duration: Option<f32>,
) {
    events.write(TerrainChangeEvent {
        new_terrain,
        fade_duration,
    });
}

/// Quick terrain change functions for common scenarios
pub fn change_terrain_instant(
    events: &mut EventWriter<TerrainChangeEvent>,
    terrain: crate::domain::value_objects::terrain::TerrainType,
) {
    send_terrain_change(events, terrain, None);
}

pub fn change_terrain_fade(
    events: &mut EventWriter<TerrainChangeEvent>,
    terrain: crate::domain::value_objects::terrain::TerrainType,
) {
    send_terrain_change(events, terrain, Some(2.0));
}

/// Setup initial terrain (Space) when game starts
fn setup_initial_terrain(mut terrain_events: EventWriter<TerrainChangeEvent>) {
    info!("🌍 Setting up initial terrain: Space");
    // Start with space terrain by default
    terrain_events.write(TerrainChangeEvent {
        new_terrain: crate::domain::value_objects::terrain::TerrainType::Plains, // Start with plains for testing
        fade_duration: None, // Instant, no fade on startup
    });
}

/// Helper function to send music progression events from anywhere in your game
///
/// Usage example:
/// ```rust
/// // In your game systems where progression changes occur
/// fn on_area_change(mut progression_events: EventWriter<MusicProgressionEvent>) {
///     progression_events.write(MusicProgressionEvent {
///         danger_level: 0.8,
///         exploration_progress: 0.6,
///         area_type: AreaType::Anomaly,
///     });
/// }
/// ```
pub fn send_music_progression(
    events: &mut EventWriter<MusicProgressionEvent>,
    danger_level: f32,
    exploration_progress: f32,
    area_type: AreaType,
) {
    events.write(MusicProgressionEvent {
        danger_level: danger_level.clamp(0.0, 1.0),
        exploration_progress: exploration_progress.clamp(0.0, 1.0),
        area_type,
    });
}
