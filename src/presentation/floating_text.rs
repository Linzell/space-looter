//! Floating Text System - Simple visual feedback for game events
//!
//! This system creates floating text messages that appear above tiles to show
//! dice results, resource gains, and other important game feedback. Uses simple
//! 2D text rendering that works reliably with Bevy.

use crate::domain::entities::EventType;
use crate::domain::services::resource_rewards::RewardTier;
use crate::domain::value_objects::{Position3D, ResourceCollection, ResourceType};
use bevy::prelude::*;

/// Plugin for floating text system
pub struct FloatingTextPlugin;

impl Plugin for FloatingTextPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FloatingTextSettings>()
            .add_event::<FloatingTextEvent>()
            .add_systems(
                Update,
                (
                    handle_floating_text_events,
                    update_floating_text,
                    cleanup_expired_text,
                ),
            )
            .add_systems(Startup, setup_floating_text_font);
    }
}

/// Settings for floating text behavior
#[derive(Resource)]
pub struct FloatingTextSettings {
    pub enabled: bool,
    pub duration: f32,
    pub rise_speed: f32,
    pub max_texts: usize,
}

impl Default for FloatingTextSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            duration: 2.0,   // 2 seconds visible
            rise_speed: 0.5, // How fast text floats up
            max_texts: 8,    // Max floating texts on screen
        }
    }
}

/// Event to create floating text
#[derive(Event)]
pub struct FloatingTextEvent {
    pub text_type: FloatingTextType,
    pub position: Position3D,
    pub message: String,
}

/// Types of floating text with different colors and sizes
#[derive(Debug, Clone)]
pub enum FloatingTextType {
    DiceRoll { result: u8, modifier: i8 },
    ResourceGain { total_value: u32 },
    ExperienceGain { amount: u32 },
    Success,
    Failure,
    CriticalSuccess,
    CriticalFailure,
    EventTrigger { event_type: EventType },
}

/// Component for floating text entities
#[derive(Component)]
pub struct FloatingText {
    pub text_type: FloatingTextType,
    pub created_at: f64,
    pub initial_position: Vec3,
}

/// Component for text animation
#[derive(Component)]
pub struct TextAnimation {
    pub timer: Timer,
    pub rise_distance: f32,
}

/// Font handle resource
#[derive(Resource)]
struct FloatingTextFont(Handle<Font>);

/// Setup font for floating text
fn setup_floating_text_font(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Try to load a font, fallback to default if not found
    let font_handle = asset_server.load("fonts/FiraSans-Bold.ttf");
    commands.insert_resource(FloatingTextFont(font_handle));

    info!("Floating text system initialized");
}

/// Handle floating text events
fn handle_floating_text_events(
    mut commands: Commands,
    mut text_events: EventReader<FloatingTextEvent>,
    settings: Res<FloatingTextSettings>,
    font: Option<Res<FloatingTextFont>>,
    time: Res<Time>,
    existing_texts: Query<&FloatingText>,
) {
    if !settings.enabled {
        text_events.clear();
        return;
    }

    // Check if we have too many texts
    if existing_texts.iter().count() >= settings.max_texts {
        text_events.clear();
        return;
    }

    let font_handle = match font {
        Some(font) => font.0.clone(),
        None => return, // Font not loaded yet
    };

    for event in text_events.read() {
        create_floating_text(
            &mut commands,
            event,
            font_handle.clone(),
            time.elapsed_secs_f64(),
            &settings,
        );
    }
}

/// Create a floating text entity
fn create_floating_text(
    commands: &mut Commands,
    event: &FloatingTextEvent,
    font: Handle<Font>,
    current_time: f64,
    settings: &FloatingTextSettings,
) {
    let (color, size) = get_text_style(&event.text_type);

    // Convert tile position to world position
    let world_pos = Vec3::new(
        event.position.x as f32 * 32.0, // Assuming 32 pixel tiles
        event.position.y as f32 * 32.0,
        100.0, // High Z to appear on top
    );

    commands
        .spawn((
            Text2d::new(event.message.clone()),
            TextFont {
                font,
                font_size: size,
                ..default()
            },
            TextColor(color),
            Transform::from_translation(world_pos),
        ))
        .insert(FloatingText {
            text_type: event.text_type.clone(),
            created_at: current_time,
            initial_position: world_pos,
        })
        .insert(TextAnimation {
            timer: Timer::from_seconds(settings.duration, TimerMode::Once),
            rise_distance: 0.0,
        });

    info!("Created floating text: {}", event.message);
}

/// Update floating text animations
fn update_floating_text(
    mut texts: Query<(&mut Transform, &mut TextAnimation, &FloatingText)>,
    settings: Res<FloatingTextSettings>,
    time: Res<Time>,
) {
    for (mut transform, mut animation, floating_text) in texts.iter_mut() {
        animation.timer.tick(time.delta());

        // Move text upward
        let rise_amount = settings.rise_speed * time.delta_secs();
        animation.rise_distance += rise_amount;
        transform.translation.y = floating_text.initial_position.y + animation.rise_distance;

        // Fade out in the last 0.5 seconds
        let remaining = animation.timer.remaining_secs();
        if remaining < 0.5 {
            let alpha = remaining / 0.5;
            // Note: In a full implementation, you'd modify the text color alpha here
            transform.scale = Vec3::splat(alpha.max(0.1));
        }
    }
}

/// Cleanup expired floating text
fn cleanup_expired_text(
    mut commands: Commands,
    texts: Query<(Entity, &TextAnimation), With<FloatingText>>,
) {
    for (entity, animation) in texts.iter() {
        if animation.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// Get text color and size based on type
fn get_text_style(text_type: &FloatingTextType) -> (Color, f32) {
    match text_type {
        FloatingTextType::DiceRoll { result, .. } => {
            if *result >= 18 {
                (Color::srgb(1.0, 1.0, 0.0), 20.0) // High roll
            } else if *result <= 3 {
                (Color::srgb(1.0, 0.0, 0.0), 18.0) // Low roll
            } else {
                (Color::WHITE, 16.0) // Normal roll
            }
        }
        FloatingTextType::ResourceGain { total_value } => {
            if *total_value > 50 {
                (Color::srgb(0.0, 1.0, 0.0), 18.0) // Big gain
            } else {
                (Color::srgb(0.0, 0.8, 0.0), 16.0) // Normal gain
            }
        }
        FloatingTextType::ExperienceGain { amount } => {
            if *amount > 50 {
                (Color::srgb(0.0, 1.0, 1.0), 18.0) // Big XP gain
            } else {
                (Color::srgb(0.5, 0.8, 1.0), 16.0) // Normal XP
            }
        }
        FloatingTextType::CriticalSuccess => (Color::srgb(1.0, 0.8, 0.0), 24.0),
        FloatingTextType::Success => (Color::srgb(0.0, 0.8, 0.0), 18.0),
        FloatingTextType::Failure => (Color::srgb(1.0, 0.3, 0.0), 18.0),
        FloatingTextType::CriticalFailure => (Color::srgb(1.0, 0.0, 0.0), 24.0),
        FloatingTextType::EventTrigger { event_type } => {
            let color = match event_type {
                EventType::Combat => Color::srgb(1.0, 0.0, 0.0),
                EventType::ResourceDiscovery => Color::srgb(0.0, 1.0, 0.0),
                EventType::Trade => Color::srgb(0.0, 0.0, 1.0),
                EventType::Boon => Color::srgb(1.0, 1.0, 0.0),
                EventType::Mystery => Color::srgb(0.8, 0.0, 0.8),
                EventType::Hazard => Color::srgb(1.0, 0.5, 0.0),
                _ => Color::WHITE,
            };
            (color, 16.0)
        }
    }
}

/// Toggle floating text with 'T' key
pub fn toggle_floating_text_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<FloatingTextSettings>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyT) {
        settings.enabled = !settings.enabled;
        info!(
            "Floating text {}",
            if settings.enabled {
                "enabled"
            } else {
                "disabled"
            }
        );
    }
}

/// Helper functions for creating floating text events
impl FloatingTextEvent {
    /// Create a dice roll text
    pub fn dice_roll(position: Position3D, result: u8, modifier: i8) -> Self {
        let message = if modifier != 0 {
            let base_roll = if modifier > 0 {
                result.saturating_sub(modifier as u8)
            } else {
                result + (-modifier) as u8
            };
            format!("d20: {} ({:+}) = {}", base_roll, modifier, result)
        } else {
            format!("d20: {}", result)
        };

        Self {
            text_type: FloatingTextType::DiceRoll { result, modifier },
            position,
            message,
        }
    }

    /// Create a resource gain text
    pub fn resource_gain(position: Position3D, resources: &ResourceCollection) -> Self {
        let total_value = resources.total_value();
        let message = if total_value == 0 {
            "No resources".to_string()
        } else {
            // Show the most significant resources
            let mut parts = Vec::new();
            for resource_type in ResourceType::all() {
                let amount = resources.get_amount(resource_type);
                if amount > 0 {
                    let symbol = match resource_type {
                        ResourceType::Metal => "Metal",
                        ResourceType::Energy => "Energy",
                        ResourceType::Food => "Food",
                        ResourceType::Technology => "Tech",
                        ResourceType::ExoticMatter => "Exotic",
                        ResourceType::Alloys => "Alloys",
                        ResourceType::Data => "Data",
                        ResourceType::Organics => "Bio",
                    };
                    parts.push(format!("{}: {}", symbol, amount));
                }
            }

            if parts.len() > 2 {
                format!("Gained {} resources", total_value)
            } else {
                format!("Gained {}", parts.join(", "))
            }
        };

        Self {
            text_type: FloatingTextType::ResourceGain { total_value },
            position,
            message,
        }
    }

    /// Create an experience gain text
    pub fn experience(position: Position3D, amount: u32) -> Self {
        Self {
            text_type: FloatingTextType::ExperienceGain { amount },
            position,
            message: format!("+{} XP", amount),
        }
    }

    /// Create a success tier text
    pub fn success_tier(position: Position3D, tier: RewardTier) -> Self {
        let (text_type, message) = match tier {
            RewardTier::CriticalSuccess => (
                FloatingTextType::CriticalSuccess,
                "CRITICAL SUCCESS!".to_string(),
            ),
            RewardTier::GreatSuccess => (FloatingTextType::Success, "Great Success!".to_string()),
            RewardTier::Success => (FloatingTextType::Success, "Success!".to_string()),
            RewardTier::Neutral => (FloatingTextType::Success, "OK".to_string()),
            RewardTier::Failure => (FloatingTextType::Failure, "Failed".to_string()),
            RewardTier::CriticalFailure => (
                FloatingTextType::CriticalFailure,
                "CRITICAL FAILURE!".to_string(),
            ),
        };

        Self {
            text_type,
            position,
            message,
        }
    }

    /// Create an event trigger text
    pub fn event_trigger(position: Position3D, event_type: EventType, title: &str) -> Self {
        let message = match event_type {
            EventType::Combat => "Combat!".to_string(),
            EventType::ResourceDiscovery => "Resources Found!".to_string(),
            EventType::Trade => "Trading Opportunity".to_string(),
            EventType::Boon => "Lucky Find!".to_string(),
            EventType::Mystery => "Strange Phenomenon".to_string(),
            EventType::Hazard => "Hazard!".to_string(),
            EventType::Malfunction => "Malfunction".to_string(),
            EventType::Narrative => title.to_string(),
            EventType::BaseEvent => "Base Event".to_string(),
        };

        Self {
            text_type: FloatingTextType::EventTrigger { event_type },
            position,
            message,
        }
    }
}
