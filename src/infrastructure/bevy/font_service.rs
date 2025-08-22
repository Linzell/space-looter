//! Font Service Infrastructure Implementation
//!
//! This module provides the Bevy-specific implementation of the FontService domain interface.
//! It handles font loading, asset management, and rendering configuration using Bevy's
//! asset system following DDD principles.

use crate::domain::services::font_service::{FontConfig, FontError, FontService, FontType};
use crate::domain::{DomainError, DomainResult};
use bevy::prelude::*;
use std::collections::HashMap;

/// Bevy-specific implementation of the FontService
#[derive(Resource, Default)]
pub struct BevyFontService {
    /// Loaded font handles mapped by font type
    font_handles: HashMap<FontType, Handle<Font>>,
    /// Font paths configuration
    font_paths: FontPaths,
    /// Whether fonts have been loaded
    fonts_loaded: bool,
}

/// Font path configuration
#[derive(Debug, Clone)]
pub struct FontPaths {
    pub ui_emoji: String,
    pub ui_regular: String,
    pub ui_monospace: Option<String>,
    pub ui_display: Option<String>,
}

impl Default for FontPaths {
    fn default() -> Self {
        Self {
            ui_emoji: "fonts/FiraSans-Regular.ttf".to_string(),
            ui_regular: "fonts/FiraSans-Regular.ttf".to_string(),
            ui_monospace: None,
            ui_display: None,
        }
    }
}

impl BevyFontService {
    /// Create a new font service with default paths
    pub fn new() -> Self {
        Self {
            font_handles: HashMap::new(),
            font_paths: FontPaths::default(),
            fonts_loaded: false,
        }
    }

    /// Create a new font service with custom paths
    pub fn with_paths(font_paths: FontPaths) -> Self {
        Self {
            font_handles: HashMap::new(),
            font_paths,
            fonts_loaded: false,
        }
    }

    /// Initialize fonts using Bevy's asset server
    pub fn initialize_fonts(&mut self, asset_server: &AssetServer) -> DomainResult<()> {
        info!("🔤 Loading font assets...");

        // Load emoji font
        let emoji_handle = asset_server.load(&self.font_paths.ui_emoji);
        self.font_handles.insert(FontType::UiEmoji, emoji_handle);

        // Load regular font
        let regular_handle = asset_server.load(&self.font_paths.ui_regular);
        self.font_handles
            .insert(FontType::UiRegular, regular_handle.clone());

        // Use regular font as fallback for display if no custom display font
        let display_handle = if let Some(ref display_path) = self.font_paths.ui_display {
            asset_server.load(display_path)
        } else {
            regular_handle.clone()
        };
        self.font_handles
            .insert(FontType::UiDisplay, display_handle);

        // Use regular font as fallback for monospace if no custom monospace font
        let monospace_handle = if let Some(ref mono_path) = self.font_paths.ui_monospace {
            asset_server.load(mono_path)
        } else {
            regular_handle
        };
        self.font_handles
            .insert(FontType::UiMonospace, monospace_handle);

        self.fonts_loaded = true;
        info!("✅ Font assets loaded successfully");

        Ok(())
    }

    /// Check if fonts are ready for use
    pub fn fonts_ready(&self, asset_server: &AssetServer) -> bool {
        if !self.fonts_loaded {
            return false;
        }

        // Check if all font handles are loaded
        self.font_handles.values().all(|handle| {
            matches!(
                asset_server.load_state(handle),
                bevy::asset::LoadState::Loaded
            )
        })
    }
}

impl FontService for BevyFontService {
    fn load_fonts(&self) -> DomainResult<()> {
        if !self.fonts_loaded {
            return Err(DomainError::ServiceError {
                service: "FontService".to_string(),
                reason: "Fonts not initialized. Call initialize_fonts first.".to_string(),
            });
        }
        Ok(())
    }

    fn get_font_handle(&self, font_type: FontType) -> DomainResult<Handle<Font>> {
        self.font_handles
            .get(&font_type)
            .cloned()
            .ok_or_else(|| FontError::FontTypeNotConfigured { font_type }.into())
    }

    fn create_text_font(&self, config: FontConfig) -> DomainResult<TextFont> {
        let font_handle = self.get_font_handle(config.font_type)?;

        Ok(TextFont {
            font: font_handle,
            font_size: config.size.to_pixels(),
            ..default()
        })
    }

    fn supports_emoji(&self, font_type: FontType) -> bool {
        matches!(font_type, FontType::UiEmoji)
    }

    fn get_emoji_font(&self) -> FontType {
        FontType::UiEmoji
    }

    fn validate_fonts(&self) -> DomainResult<()> {
        if !self.fonts_loaded {
            return Err(FontError::ValidationFailed {
                details: "Fonts not loaded".to_string(),
            }
            .into());
        }

        // Check that all essential font types are available
        let essential_fonts = [FontType::UiEmoji, FontType::UiRegular];
        for font_type in &essential_fonts {
            if !self.font_handles.contains_key(font_type) {
                return Err(FontError::FontTypeNotConfigured {
                    font_type: font_type.clone(),
                }
                .into());
            }
        }

        Ok(())
    }
}

/// Bevy plugin for font system integration
pub struct FontPlugin;

impl Plugin for FontPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BevyFontService::new())
            .add_systems(Startup, initialize_font_system)
            .add_systems(Update, monitor_font_loading);
    }
}

/// System to initialize the font service
fn initialize_font_system(
    mut font_service: ResMut<BevyFontService>,
    asset_server: Res<AssetServer>,
) {
    if let Err(e) = font_service.initialize_fonts(&asset_server) {
        error!("Failed to initialize font service: {}", e);
    }
}

/// System to monitor font loading status
fn monitor_font_loading(font_service: Res<BevyFontService>, asset_server: Res<AssetServer>) {
    static mut FONTS_READY_LOGGED: bool = false;

    unsafe {
        if !FONTS_READY_LOGGED && font_service.fonts_ready(&asset_server) {
            info!("🎨 All fonts are ready for rendering");
            FONTS_READY_LOGGED = true;
        }
    }
}

/// Helper function to create text with emoji support
pub fn create_emoji_text(
    text: &str,
    size: crate::domain::services::font_service::FontSize,
) -> (Text, TextFont) {
    let config = FontConfig::emoji(size);

    // For now, create with default font - the actual font will be applied by systems
    (
        Text::new(text),
        TextFont {
            font_size: size.to_pixels(),
            ..default()
        },
    )
}

/// Helper function to create regular text
pub fn create_regular_text(
    text: &str,
    size: crate::domain::services::font_service::FontSize,
) -> (Text, TextFont) {
    let config = FontConfig::regular(size);

    (
        Text::new(text),
        TextFont {
            font_size: size.to_pixels(),
            ..default()
        },
    )
}

/// Component to mark text that should use emoji fonts
#[derive(Component, Debug)]
pub struct EmojiText;

/// Component to mark text that should use regular fonts
#[derive(Component, Debug)]
pub struct RegularText;

/// System to apply proper fonts to text components
pub fn apply_font_to_text(
    mut emoji_query: Query<&mut TextFont, (With<EmojiText>, Without<RegularText>)>,
    mut regular_query: Query<&mut TextFont, (With<RegularText>, Without<EmojiText>)>,
    font_service: Res<BevyFontService>,
) {
    // Apply emoji font to emoji text components
    if let Ok(emoji_font) = font_service.get_font_handle(FontType::UiEmoji) {
        for mut text_font in emoji_query.iter_mut() {
            if text_font.font == Handle::default() {
                text_font.font = emoji_font.clone();
            }
        }
    }

    // Apply regular font to regular text components
    if let Ok(regular_font) = font_service.get_font_handle(FontType::UiRegular) {
        for mut text_font in regular_query.iter_mut() {
            if text_font.font == Handle::default() {
                text_font.font = regular_font.clone();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bevy_font_service_creation() {
        let service = BevyFontService::new();
        assert!(!service.fonts_loaded);
        assert!(service.font_handles.is_empty());
    }

    #[test]
    fn font_paths_default() {
        let paths = FontPaths::default();
        assert_eq!(paths.ui_emoji, "fonts/FiraSans-Regular.ttf");
        assert_eq!(paths.ui_regular, "fonts/FiraSans-Regular.ttf");
    }

    #[test]
    fn emoji_support_detection() {
        let service = BevyFontService::new();
        assert!(service.supports_emoji(FontType::UiEmoji));
        assert!(!service.supports_emoji(FontType::UiRegular));
    }

    #[test]
    fn font_validation_fails_when_not_loaded() {
        let service = BevyFontService::new();
        assert!(service.validate_fonts().is_err());
    }
}
