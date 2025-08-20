//! Space Looter Game Library
//!
//! A 2D space shooter game built with Domain-Driven Design principles.
//! This library provides both native and WebAssembly entry points.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// Module declarations following DDD architecture
pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presentation;

use bevy::prelude::*;
use bevy::window::WindowResized;

/// Creates and configures the main Bevy application
pub fn create_app() -> App {
    let mut app = App::new();

    // Configure Bevy plugins with web-optimized settings
    #[cfg(target_arch = "wasm32")]
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Space Looter".into(),
            canvas: Some("#bevy".into()),
            fit_canvas_to_parent: true,
            prevent_default_event_handling: false,
            ..default()
        }),
        ..default()
    }));

    // Configure for native
    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Space Looter".into(),
            resolution: (800.0, 600.0).into(),
            ..default()
        }),
        ..default()
    }));

    // Initialize game state
    app.init_state::<presentation::AppState>();

    // Set background color
    app.insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.2)));

    // Add game plugin with all systems
    app.add_plugins(infrastructure::bevy::BevyGamePlugin);

    // Add additional update systems
    app.add_systems(
        Update,
        (
            infrastructure::bevy::systems::update_score_display_system,
            handle_window_resize_system,
        ),
    );

    app
}

/// System to handle window resize events for better responsive behavior
fn handle_window_resize_system(
    mut resize_events: EventReader<WindowResized>,
    mut windows: Query<&mut Window>,
) {
    for event in resize_events.read() {
        if let Ok(mut window) = windows.get_single_mut() {
            info!("Window resized to {}x{}", event.width, event.height);

            #[cfg(target_arch = "wasm32")]
            {
                // On web, ensure the canvas matches the new size
                // Bevy should handle this automatically with fit_canvas_to_parent
                // but we log it for debugging purposes
                info!("Web canvas should auto-resize to parent container");
            }
        }
    }
}

/// WebAssembly entry point
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn main() {
    // Set up panic hook for better error messages in browser console
    #[cfg(feature = "web")]
    console_error_panic_hook::set_once();

    // Initialize logging for web
    #[cfg(feature = "web")]
    wasm_logger::init(wasm_logger::Config::default());

    info!("Starting Space Looter WASM application");

    // Create and run the app
    let mut app = create_app();
    app.run();
}

/// Native entry point (when used as a library)
#[cfg(not(target_arch = "wasm32"))]
pub fn run() {
    info!("Starting Space Looter native application");

    let mut app = create_app();
    app.run();
}
