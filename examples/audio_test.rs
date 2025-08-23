use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_audio_test)
        .add_systems(Update, test_audio_input)
        .run();
}

fn setup_audio_test(mut commands: Commands, asset_server: Res<AssetServer>) {
    println!("🔊 Audio Test Starting...");
    println!("Press SPACE to test dice roll sound");
    println!("Press M to test menu music");
    println!("Press ESC to quit");

    // Load audio assets
    let dice_sound: Handle<AudioSource> = asset_server.load("audio/sfx/dice/dice_roll.wav");
    let menu_music: Handle<AudioSource> = asset_server.load("audio/music/menu_theme.ogg");

    commands.insert_resource(AudioTestResources {
        dice_sound,
        menu_music,
    });

    println!("✅ Audio assets loaded");
}

#[derive(Resource)]
struct AudioTestResources {
    dice_sound: Handle<AudioSource>,
    menu_music: Handle<AudioSource>,
}

fn test_audio_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    audio_resources: Option<Res<AudioTestResources>>,
    mut exit: EventWriter<AppExit>,
) {
    if let Some(audio_res) = audio_resources {
        if keyboard_input.just_pressed(KeyCode::Space) {
            println!("🎲 Playing dice roll sound...");
            commands.spawn(AudioPlayer::new(audio_res.dice_sound.clone()));
        }

        if keyboard_input.just_pressed(KeyCode::KeyM) {
            println!("🎵 Playing menu music...");
            commands.spawn((
                AudioPlayer::new(audio_res.menu_music.clone()),
                PlaybackSettings::LOOP,
            ));
        }
    }

    if keyboard_input.just_pressed(KeyCode::Escape) {
        println!("👋 Exiting audio test");
        exit.send(AppExit::Success);
    }
}
