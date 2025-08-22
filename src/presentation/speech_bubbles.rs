//! Speech Bubble System - Visual feedback for RPG events
//!
//! This system provides RPG-style speech bubbles that appear above tiles to show
//! dice results, events, and resource rewards in real-time. The bubbles use
//! SVG icons and are color-coded for better visual clarity.

use crate::domain::entities::EventType;
use crate::domain::services::resource_rewards::RewardTier;
use crate::domain::value_objects::{Position3D, ResourceType};
use bevy::prelude::*;
use std::collections::HashMap;

/// Plugin for speech bubble system
pub struct SpeechBubblePlugin;

impl Plugin for SpeechBubblePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpeechBubbleSettings>()
            .init_resource::<SpeechBubbleQueue>()
            .add_event::<SpeechBubbleEvent>()
            .add_systems(
                Update,
                (
                    handle_speech_bubble_events,
                    update_speech_bubbles,
                    cleanup_expired_bubbles,
                ),
            )
            .add_systems(Startup, setup_speech_bubble_materials);
    }
}

/// Settings for speech bubble appearance and behavior
#[derive(Resource)]
pub struct SpeechBubbleSettings {
    pub enabled: bool,
    pub duration: f32,
    pub max_bubbles: usize,
}

impl Default for SpeechBubbleSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            duration: 2.5,  // 2.5 seconds visible
            max_bubbles: 5, // Max bubbles on screen
        }
    }
}

/// Resource containing SVG icon handles
#[derive(Resource)]
pub struct SpeechBubbleIcons {
    pub dice: Handle<Image>,
    pub combat: Handle<Image>,
    pub resource: Handle<Image>,
    pub trade: Handle<Image>,
    pub success: Handle<Image>,
    pub failure: Handle<Image>,
    pub experience: Handle<Image>,
}

/// Event to trigger a new speech bubble
#[derive(Event)]
pub struct SpeechBubbleEvent {
    pub bubble_type: SpeechBubbleType,
    pub position: Position3D,
    pub message: String,
    pub priority: BubblePriority,
}

/// Types of speech bubbles with different colors and styles
#[derive(Debug, Clone)]
pub enum SpeechBubbleType {
    DiceRoll {
        roll: u8,
        modifier: i8,
        final_result: u8,
    },
    Event {
        event_type: EventType,
    },
    Reward {
        tier: RewardTier,
    },
    ResourceGain {
        resources: Vec<(ResourceType, u32)>,
    },
    Experience {
        amount: u32,
    },
    Warning {
        message: String,
    },
    Success {
        message: String,
    },
    Failure {
        message: String,
    },
    Info {
        message: String,
    },
}

/// Priority levels for bubble display
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BubblePriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// Pending speech bubble waiting to be created
#[derive(Debug, Clone)]
pub struct PendingSpeechBubble {
    pub bubble_type: SpeechBubbleType,
    pub position: Position3D,
    pub message: String,
    pub priority: BubblePriority,
    pub created_at: f64,
}

/// Component for speech bubble entities
#[derive(Component)]
pub struct SpeechBubble {
    pub bubble_type: SpeechBubbleType,
    pub position: Position3D,
    pub created_at: f64,
    pub stack_index: usize,
    pub priority: BubblePriority,
}

/// Component for bubble animation state
#[derive(Component)]
pub struct BubbleAnimation {
    pub timer: Timer,
    pub phase: AnimationPhase,
    pub initial_scale: f32,
}

/// Animation phases for bubbles
#[derive(Debug, PartialEq)]
pub enum AnimationPhase {
    FadeIn,
    Display,
    FadeOut,
}

/// Setup speech bubble materials and icons
fn setup_speech_bubble_materials(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Setting up speech bubble system with SVG icons");

    // Load SVG icons (fallback to simple colored squares if SVGs not available)
    let icons = SpeechBubbleIcons {
        dice: asset_server.load("icons/dice.png"),
        combat: asset_server.load("icons/combat.png"),
        resource: asset_server.load("icons/resource.png"),
        trade: asset_server.load("icons/trade.png"),
        success: asset_server.load("icons/success.png"),
        failure: asset_server.load("icons/failure.png"),
        experience: asset_server.load("icons/experience.png"),
    };

    commands.insert_resource(icons);

    // Load font for bubble text
    let font_handle = asset_server.load("fonts/FiraSans-Bold.ttf");
    commands.insert_resource(BubbleFontHandle(font_handle));
}

#[derive(Resource)]
struct BubbleFontHandle(Handle<Font>);

/// Handle incoming speech bubble events - create bubbles directly
fn handle_speech_bubble_events(
    mut commands: Commands,
    mut bubble_events: EventReader<SpeechBubbleEvent>,
    settings: Res<SpeechBubbleSettings>,
    icons: Option<Res<SpeechBubbleIcons>>,
    font: Option<Res<BubbleFontHandle>>,
    time: Res<Time>,
    existing_bubbles: Query<&SpeechBubble>,
) {
    if !settings.enabled {
        bubble_events.clear();
        return;
    }

    let icons = match icons {
        Some(icons) => icons,
        None => return, // Icons not loaded yet
    };

    let font = match font {
        Some(font) => font.0.clone(),
        None => return, // Font not loaded yet
    };

    // Count existing bubbles to limit total
    if existing_bubbles.iter().count() >= settings.max_bubbles {
        bubble_events.clear();
        return;
    }

    for event in bubble_events.read() {
        create_simple_speech_bubble(
            &mut commands,
            &event,
            &icons,
            font.clone(),
            time.elapsed_seconds_f64(),
            &settings,
        );
    }
}

/// Create a simple speech bubble with icon and text
fn create_simple_speech_bubble(
    commands: &mut Commands,
    event: &SpeechBubbleEvent,
    icons: &SpeechBubbleIcons,
    font: Handle<Font>,
    current_time: f64,
    settings: &SpeechBubbleSettings,
) {
    // Select appropriate icon
    let icon_handle = match &event.bubble_type {
        SpeechBubbleType::DiceRoll { .. } => icons.dice.clone(),
        SpeechBubbleType::Event { event_type } => match event_type {
            EventType::Combat => icons.combat.clone(),
            EventType::ResourceDiscovery => icons.resource.clone(),
            EventType::Trade => icons.trade.clone(),
            _ => icons.resource.clone(),
        },
        SpeechBubbleType::Reward { tier } => match tier {
            RewardTier::CriticalSuccess | RewardTier::GreatSuccess | RewardTier::Success => {
                icons.success.clone()
            }
            _ => icons.failure.clone(),
        },
        SpeechBubbleType::ResourceGain { .. } => icons.resource.clone(),
        SpeechBubbleType::Experience { .. } => icons.experience.clone(),
        _ => icons.resource.clone(),
    };

    // Calculate world position
    let world_pos = Vec3::new(
        event.position.x as f32,
        event.position.y as f32 + 1.5, // Above the tile
        event.position.z as f32,
    );

    // Create bubble entity with icon and text
    commands
        .spawn(SpriteBundle {
            texture: icon_handle,
            transform: Transform::from_translation(world_pos).with_scale(Vec3::splat(0.5)),
            ..default()
        })
        .insert(SpeechBubble {
            bubble_type: event.bubble_type.clone(),
            position: event.position,
            created_at: current_time,
            stack_index: 0,
            priority: event.priority,
        })
        .insert(BubbleAnimation {
            timer: Timer::from_seconds(settings.duration, TimerMode::Once),
            phase: AnimationPhase::Display,
            initial_scale: 0.5,
        })
        .with_children(|parent| {
            // Add text below icon
            parent.spawn(Text2dBundle {
                text: Text::from_section(
                    event.message.clone(),
                    TextStyle {
                        font,
                        font_size: 14.0,
                        color: Color::WHITE,
                    },
                ),
                transform: Transform::from_xyz(0.0, -0.8, 0.1),
                ..default()
            });
        });

    info!("Created speech bubble: {}", event.message);
}

/// Update bubble animations - simpler fade out system
fn update_speech_bubbles(
    mut commands: Commands,
    mut bubbles: Query<(Entity, &mut BubbleAnimation, &mut Transform), With<SpeechBubble>>,
    time: Res<Time>,
) {
    for (entity, mut animation, mut transform) in bubbles.iter_mut() {
        animation.timer.tick(time.delta());

        // Simple fade out in the last 0.5 seconds
        let remaining = animation.timer.remaining_secs();
        if remaining < 0.5 {
            let alpha = remaining / 0.5;
            transform.scale = Vec3::splat(animation.initial_scale * alpha);
        }

        // Remove when timer finishes
        if animation.timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Cleanup expired bubbles
fn cleanup_expired_bubbles(
    mut commands: Commands,
    bubbles: Query<(Entity, &BubbleAnimation), With<SpeechBubble>>,
) {
    for (entity, animation) in bubbles.iter() {
        if animation.timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Cleanup expired bubbles
fn cleanup_expired_bubbles(
    mut commands: Commands,
    bubbles: Query<(Entity, &BubbleAnimation), With<SpeechBubble>>,
) {
    for (entity, animation) in bubbles.iter() {
        if animation.phase == AnimationPhase::FadeOut && animation.timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Get bubble background color based on type
fn get_bubble_color(bubble_type: &SpeechBubbleType) -> Color {
    match bubble_type {
        SpeechBubbleType::DiceRoll { .. } => Color::srgba(1.0, 0.8, 0.0, 0.9), // Yellow
        SpeechBubbleType::Event { event_type } => match event_type {
            EventType::Combat => Color::srgba(1.0, 0.2, 0.2, 0.9), // Red
            EventType::ResourceDiscovery => Color::srgba(0.2, 0.8, 0.2, 0.9), // Green
            EventType::Trade => Color::srgba(0.2, 0.6, 1.0, 0.9),  // Blue
            EventType::Boon => Color::srgba(0.8, 0.2, 1.0, 0.9),   // Purple
            EventType::Mystery => Color::srgba(0.6, 0.2, 0.8, 0.9), // Dark Purple
            _ => Color::srgba(0.5, 0.5, 0.5, 0.9),                 // Gray
        },
        SpeechBubbleType::Reward { tier } => match tier {
            RewardTier::CriticalSuccess => Color::srgba(1.0, 0.8, 0.0, 0.95), // Bright Gold
            RewardTier::GreatSuccess => Color::srgba(0.0, 0.8, 0.0, 0.9),     // Bright Green
            RewardTier::Success => Color::srgba(0.2, 0.8, 0.2, 0.9),          // Green
            RewardTier::Neutral => Color::srgba(0.6, 0.6, 0.6, 0.9),          // Gray
            RewardTier::Failure => Color::srgba(0.8, 0.4, 0.0, 0.9),          // Orange
            RewardTier::CriticalFailure => Color::srgba(1.0, 0.0, 0.0, 0.9),  // Red
        },
        SpeechBubbleType::ResourceGain { .. } => Color::srgba(0.0, 0.6, 0.8, 0.9), // Cyan
        SpeechBubbleType::Experience { .. } => Color::srgba(0.8, 0.0, 0.8, 0.9),   // Magenta
        SpeechBubbleType::Warning { .. } => Color::srgba(1.0, 0.6, 0.0, 0.9),      // Orange
        SpeechBubbleType::Success { .. } => Color::srgba(0.0, 0.8, 0.0, 0.9),      // Green
        SpeechBubbleType::Failure { .. } => Color::srgba(1.0, 0.0, 0.0, 0.9),      // Red
        SpeechBubbleType::Info { .. } => Color::srgba(0.0, 0.4, 0.8, 0.9),         // Blue
    }
}

/// Get text color based on bubble type
fn get_text_color(bubble_type: &SpeechBubbleType) -> Color {
    match bubble_type {
        SpeechBubbleType::DiceRoll { .. } => Color::BLACK,
        SpeechBubbleType::Reward { tier } => match tier {
            RewardTier::CriticalSuccess => Color::BLACK,
            _ => Color::WHITE,
        },
        _ => Color::WHITE,
    }
}

/// Create bubble mesh (rounded rectangle)
fn create_bubble_mesh() -> Handle<Mesh> {
    // This would create a simple quad mesh for the bubble background
    // In a real implementation, you'd create a rounded rectangle mesh
    Handle::default() // Placeholder - would need actual mesh creation
}

/// Create bubble material
fn create_bubble_material(color: Color) -> Handle<StandardMaterial> {
    // This would create a material with the specified color
    // In a real implementation, you'd create the actual material
    Handle::default() // Placeholder - would need actual material creation
}

/// Utility functions for creating speech bubble events

impl SpeechBubbleEvent {
    /// Create a dice roll bubble
    pub fn dice_roll(position: Position3D, roll: u8, modifier: i8, final_result: u8) -> Self {
        let message = if modifier != 0 {
            format!("Roll: {}", final_result)
        } else {
            format!("Roll: {}", roll)
        };

        Self {
            bubble_type: SpeechBubbleType::DiceRoll {
                roll,
                modifier,
                final_result,
            },
            position,
            message,
            priority: BubblePriority::High,
        }
    }

    /// Create an event bubble
    pub fn event(position: Position3D, event_type: EventType, title: String) -> Self {
        // Simplified title for better readability
        let short_title = match event_type {
            EventType::Combat => "Combat",
            EventType::ResourceDiscovery => "Resources Found",
            EventType::Trade => "Trade",
            EventType::Boon => "Lucky Find",
            EventType::Mystery => "Mystery",
            EventType::Hazard => "Hazard",
            EventType::Malfunction => "Malfunction",
            EventType::Narrative => "Event",
            EventType::BaseEvent => "Base Event",
        };

        Self {
            bubble_type: SpeechBubbleType::Event { event_type },
            position,
            message: short_title.to_string(),
            priority: BubblePriority::Normal,
        }
    }

    /// Create a reward tier bubble
    pub fn reward_tier(position: Position3D, tier: RewardTier) -> Self {
        let message = match tier {
            RewardTier::CriticalSuccess => "CRITICAL!",
            RewardTier::GreatSuccess => "Great!",
            RewardTier::Success => "Success",
            RewardTier::Neutral => "OK",
            RewardTier::Failure => "Failed",
            RewardTier::CriticalFailure => "DISASTER!",
        };

        Self {
            bubble_type: SpeechBubbleType::Reward { tier },
            position,
            message: message.to_string(),
            priority: BubblePriority::High,
        }
    }

    /// Create a resource gain bubble
    pub fn resource_gain(position: Position3D, resources: Vec<(ResourceType, u32)>) -> Self {
        let message = if resources.is_empty() {
            "Nothing".to_string()
        } else {
            let total: u32 = resources.iter().map(|(_, amount)| amount).sum();
            format!("Gained {} items", total)
        };

        Self {
            bubble_type: SpeechBubbleType::ResourceGain { resources },
            position,
            message,
            priority: BubblePriority::Normal,
        }
    }

    /// Create an experience gain bubble
    pub fn experience(position: Position3D, amount: u32) -> Self {
        Self {
            bubble_type: SpeechBubbleType::Experience { amount },
            position,
            message: format!("+{} XP", amount),
            priority: BubblePriority::Low,
        }
    }
}

/// System to toggle speech bubbles with 'T' key
pub fn toggle_speech_bubbles_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<SpeechBubbleSettings>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyT) {
        settings.enabled = !settings.enabled;
        info!(
            "Speech bubbles {}",
            if settings.enabled {
                "enabled"
            } else {
                "disabled"
            }
        );
    }
}
