//! Font Service - Domain Service for Font Management
//!
//! This service defines the domain interface for font loading and management
//! following Domain-Driven Design principles.

use crate::domain::{DomainError, DomainResult};
use bevy::prelude::*;

/// Font types supported by the application
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FontType {
    /// Primary UI font with emoji support
    UiEmoji,
    /// Regular text font
    UiRegular,
    /// Monospace font for technical displays
    UiMonospace,
    /// Large display font for headers
    UiDisplay,
}

/// Font size categories for consistent UI scaling
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontSize {
    /// Small text (9-11px)
    Small,
    /// Regular text (12-14px)
    Regular,
    /// Medium text (15-18px)
    Medium,
    /// Large text (19-24px)
    Large,
    /// Extra large text (25px+)
    ExtraLarge,
}

impl FontSize {
    /// Get the pixel size for this font size
    pub fn to_pixels(self) -> f32 {
        match self {
            FontSize::Small => 10.0,
            FontSize::Regular => 13.0,
            FontSize::Medium => 16.0,
            FontSize::Large => 20.0,
            FontSize::ExtraLarge => 28.0,
        }
    }
}

/// Font weight options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontWeight {
    Normal,
    Bold,
}

/// Font configuration for rendering
#[derive(Debug, Clone)]
pub struct FontConfig {
    pub font_type: FontType,
    pub size: FontSize,
    pub weight: FontWeight,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            font_type: FontType::UiRegular,
            size: FontSize::Regular,
            weight: FontWeight::Normal,
        }
    }
}

/// Domain service interface for font management
pub trait FontService: Send + Sync {
    /// Load all required fonts into the system
    fn load_fonts(&self) -> DomainResult<()>;

    /// Get a font handle for the specified font type
    fn get_font_handle(&self, font_type: FontType) -> DomainResult<Handle<Font>>;

    /// Create a TextFont component with the specified configuration
    fn create_text_font(&self, config: FontConfig) -> DomainResult<TextFont>;

    /// Check if a font supports emoji rendering
    fn supports_emoji(&self, font_type: FontType) -> bool;

    /// Get the recommended font for displaying text with emoji
    fn get_emoji_font(&self) -> FontType;

    /// Validate that all required fonts are available
    fn validate_fonts(&self) -> DomainResult<()>;
}

/// Font loading errors
#[derive(Debug, thiserror::Error)]
pub enum FontError {
    #[error("Font file not found: {path}")]
    FontNotFound { path: String },

    #[error("Failed to load font: {reason}")]
    LoadFailed { reason: String },

    #[error("Unsupported font format: {format}")]
    UnsupportedFormat { format: String },

    #[error("Font validation failed: {details}")]
    ValidationFailed { details: String },

    #[error("Font type not configured: {font_type:?}")]
    FontTypeNotConfigured { font_type: FontType },
}

impl From<FontError> for DomainError {
    fn from(error: FontError) -> Self {
        DomainError::ServiceError {
            service: "FontService".to_string(),
            reason: error.to_string(),
        }
    }
}

/// Font path configuration - defines where fonts are located
pub struct FontPaths {
    pub ui_emoji: &'static str,
    pub ui_regular: &'static str,
    pub ui_monospace: Option<&'static str>,
    pub ui_display: Option<&'static str>,
}

impl Default for FontPaths {
    fn default() -> Self {
        Self {
            ui_emoji: "fonts/FiraSans-Regular.ttf",
            ui_regular: "fonts/FiraSans-Regular.ttf",
            ui_monospace: None, // Use system default
            ui_display: None,   // Use regular font
        }
    }
}

/// Helper functions for font configuration
impl FontConfig {
    /// Create a new font configuration
    pub fn new(font_type: FontType, size: FontSize, weight: FontWeight) -> Self {
        Self {
            font_type,
            size,
            weight,
        }
    }

    /// Create configuration for emoji-enabled text
    pub fn emoji(size: FontSize) -> Self {
        Self {
            font_type: FontType::UiEmoji,
            size,
            weight: FontWeight::Normal,
        }
    }

    /// Create configuration for regular text
    pub fn regular(size: FontSize) -> Self {
        Self {
            font_type: FontType::UiRegular,
            size,
            weight: FontWeight::Normal,
        }
    }

    /// Create configuration for display text (headers)
    pub fn display(size: FontSize) -> Self {
        Self {
            font_type: FontType::UiDisplay,
            size,
            weight: FontWeight::Bold,
        }
    }

    /// Create configuration for monospace text (technical displays)
    pub fn monospace(size: FontSize) -> Self {
        Self {
            font_type: FontType::UiMonospace,
            size,
            weight: FontWeight::Normal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_size_to_pixels() {
        assert_eq!(FontSize::Small.to_pixels(), 10.0);
        assert_eq!(FontSize::Regular.to_pixels(), 13.0);
        assert_eq!(FontSize::Large.to_pixels(), 20.0);
    }

    #[test]
    fn font_config_creation() {
        let config = FontConfig::emoji(FontSize::Medium);
        assert_eq!(config.font_type, FontType::UiEmoji);
        assert_eq!(config.size, FontSize::Medium);
        assert_eq!(config.weight, FontWeight::Normal);
    }

    #[test]
    fn font_config_default() {
        let config = FontConfig::default();
        assert_eq!(config.font_type, FontType::UiRegular);
        assert_eq!(config.size, FontSize::Regular);
        assert_eq!(config.weight, FontWeight::Normal);
    }

    #[test]
    fn font_paths_default() {
        let paths = FontPaths::default();
        assert_eq!(paths.ui_emoji, "fonts/FiraSans-Regular.ttf");
        assert_eq!(paths.ui_regular, "fonts/FiraSans-Regular.ttf");
        assert!(paths.ui_monospace.is_none());
        assert!(paths.ui_display.is_none());
    }
}
