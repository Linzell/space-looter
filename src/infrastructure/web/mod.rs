//! Web Infrastructure - WebAssembly Integration
//!
//! This module provides web-specific infrastructure for running the game
//! in web browsers using WebAssembly. It handles browser APIs, web-specific
//! optimizations, and WASM bindings.

use crate::infrastructure::{InfrastructureError, InfrastructureResult};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures;

/// Web-specific configuration and utilities
pub struct WebInfrastructure {
    /// Canvas element ID
    pub canvas_id: String,
    /// Performance monitoring enabled
    pub performance_monitoring: bool,
}

impl WebInfrastructure {
    /// Create new web infrastructure
    pub fn new(canvas_id: String) -> Self {
        Self {
            canvas_id,
            performance_monitoring: true,
        }
    }

    /// Initialize web-specific features
    pub fn initialize(&self) -> InfrastructureResult<()> {
        #[cfg(target_arch = "wasm32")]
        {
            // Set up panic hook for better error messages in browser console
            console_error_panic_hook::set_once();

            // Initialize logging for web
            wasm_logger::init(wasm_logger::Config::default());

            web_sys::console::log_1(&"Web infrastructure initialized".into());
        }

        Ok(())
    }

    /// Get canvas element from DOM
    #[cfg(target_arch = "wasm32")]
    pub fn get_canvas(&self) -> InfrastructureResult<web_sys::HtmlCanvasElement> {
        let window = web_sys::window()
            .ok_or_else(|| InfrastructureError::WebError("No window object".to_string()))?;

        let document = window
            .document()
            .ok_or_else(|| InfrastructureError::WebError("No document object".to_string()))?;

        let canvas = document
            .get_element_by_id(&self.canvas_id)
            .ok_or_else(|| {
                InfrastructureError::WebError(format!(
                    "Canvas element '{}' not found",
                    self.canvas_id
                ))
            })?
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| InfrastructureError::WebError("Element is not a canvas".to_string()))?;

        Ok(canvas)
    }

    /// Get window dimensions
    #[cfg(target_arch = "wasm32")]
    pub fn get_window_size(&self) -> InfrastructureResult<(f32, f32)> {
        let window = web_sys::window()
            .ok_or_else(|| InfrastructureError::WebError("No window object".to_string()))?;

        let width = window
            .inner_width()
            .map_err(|_| InfrastructureError::WebError("Failed to get window width".to_string()))?
            .as_f64()
            .unwrap_or(800.0) as f32;

        let height = window
            .inner_height()
            .map_err(|_| InfrastructureError::WebError("Failed to get window height".to_string()))?
            .as_f64()
            .unwrap_or(600.0) as f32;

        Ok((width, height))
    }

    /// Request animation frame (for smooth rendering)
    #[cfg(target_arch = "wasm32")]
    pub fn request_animation_frame(
        &self,
        callback: &Closure<dyn FnMut()>,
    ) -> InfrastructureResult<i32> {
        let window = web_sys::window()
            .ok_or_else(|| InfrastructureError::WebError("No window object".to_string()))?;

        window
            .request_animation_frame(callback.as_ref().unchecked_ref())
            .map_err(|_| {
                InfrastructureError::WebError("Failed to request animation frame".to_string())
            })
    }

    /// Log performance metrics
    #[cfg(target_arch = "wasm32")]
    pub fn log_performance(&self, name: &str, value: f64) {
        if self.performance_monitoring {
            web_sys::console::log_2(
                &format!("Performance - {}: {}ms", name, value).into(),
                &JsValue::from_f64(value),
            );
        }
    }
}

impl Default for WebInfrastructure {
    fn default() -> Self {
        Self::new("bevy".to_string())
    }
}

/// Web-specific utilities
pub mod utils {
    use super::*;
    use crate::infrastructure::{InfrastructureError, InfrastructureResult};
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen::prelude::*;

    /// Check if running in web browser
    pub fn is_web() -> bool {
        cfg!(target_arch = "wasm32")
    }

    /// Get user agent string
    #[cfg(target_arch = "wasm32")]
    pub fn get_user_agent() -> Option<String> {
        web_sys::window()?.navigator().user_agent().ok()
    }

    /// Check if WebGL2 is supported
    #[cfg(target_arch = "wasm32")]
    pub fn is_webgl2_supported() -> InfrastructureResult<bool> {
        let window = web_sys::window()
            .ok_or_else(|| InfrastructureError::WebError("No window object".to_string()))?;

        let document = window
            .document()
            .ok_or_else(|| InfrastructureError::WebError("No document object".to_string()))?;

        let canvas = document
            .create_element("canvas")
            .map_err(|_| InfrastructureError::WebError("Failed to create test canvas".to_string()))?
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| InfrastructureError::WebError("Failed to cast to canvas".to_string()))?;

        let context = canvas.get_context("webgl2").map_err(|_| {
            InfrastructureError::WebError("Failed to get WebGL2 context".to_string())
        })?;

        Ok(context.is_some())
    }

    /// Get available memory (if supported)
    #[cfg(target_arch = "wasm32")]
    pub fn get_memory_info() -> Option<f64> {
        let window = web_sys::window()?;
        let performance = window.performance()?;
        let memory = js_sys::Reflect::get(&performance, &JsValue::from_str("memory")).ok()?;

        if !memory.is_undefined() {
            let used_heap =
                js_sys::Reflect::get(&memory, &JsValue::from_str("usedJSHeapSize")).ok()?;
            used_heap.as_f64()
        } else {
            None
        }
    }

    /// Set document title
    #[cfg(target_arch = "wasm32")]
    pub fn set_title(title: &str) -> InfrastructureResult<()> {
        let window = web_sys::window()
            .ok_or_else(|| InfrastructureError::WebError("No window object".to_string()))?;

        let document = window
            .document()
            .ok_or_else(|| InfrastructureError::WebError("No document object".to_string()))?;

        document.set_title(title);
        Ok(())
    }

    /// Show/hide loading indicator
    #[cfg(target_arch = "wasm32")]
    pub fn set_loading_state(loading: bool) -> InfrastructureResult<()> {
        let window = web_sys::window()
            .ok_or_else(|| InfrastructureError::WebError("No window object".to_string()))?;

        let document = window
            .document()
            .ok_or_else(|| InfrastructureError::WebError("No document object".to_string()))?;

        if let Some(loading_element) = document.get_element_by_id("loading") {
            let style = loading_element
                .dyn_ref::<web_sys::HtmlElement>()
                .ok_or_else(|| {
                    InfrastructureError::WebError("Loading element is not HTML".to_string())
                })?
                .style();

            if loading {
                style.set_property("display", "block")
            } else {
                style.set_property("display", "none")
            }
            .map_err(|_| {
                InfrastructureError::WebError("Failed to set loading state".to_string())
            })?;
        }

        Ok(())
    }
}

/// Web-specific event handling
pub mod events {
    use super::*;

    /// Web event listener wrapper
    pub struct WebEventListener {
        #[cfg(target_arch = "wasm32")]
        _closure: Option<Closure<dyn FnMut(web_sys::Event)>>,
        #[cfg(not(target_arch = "wasm32"))]
        _phantom: std::marker::PhantomData<()>,
    }

    impl WebEventListener {
        /// Create new event listener for resize events
        #[cfg(target_arch = "wasm32")]
        pub fn on_resize<F>(mut callback: F) -> InfrastructureResult<Self>
        where
            F: FnMut(f32, f32) + 'static,
        {
            let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                if let Ok((width, height)) = WebInfrastructure::default().get_window_size() {
                    callback(width, height);
                }
            }) as Box<dyn FnMut(_)>);

            let window = web_sys::window()
                .ok_or_else(|| InfrastructureError::WebError("No window object".to_string()))?;

            window
                .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())
                .map_err(|_| {
                    InfrastructureError::WebError("Failed to add resize listener".to_string())
                })?;

            Ok(Self {
                _closure: Some(closure),
            })
        }

        /// Create new event listener for visibility change
        #[cfg(target_arch = "wasm32")]
        pub fn on_visibility_change<F>(mut callback: F) -> InfrastructureResult<Self>
        where
            F: FnMut(bool) + 'static,
        {
            let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                let window = web_sys::window().unwrap();
                let document = window.document().unwrap();
                let hidden = document.hidden();
                callback(!hidden);
            }) as Box<dyn FnMut(_)>);

            let window = web_sys::window()
                .ok_or_else(|| InfrastructureError::WebError("No window object".to_string()))?;

            let document = window
                .document()
                .ok_or_else(|| InfrastructureError::WebError("No document object".to_string()))?;

            document
                .add_event_listener_with_callback(
                    "visibilitychange",
                    closure.as_ref().unchecked_ref(),
                )
                .map_err(|_| {
                    InfrastructureError::WebError("Failed to add visibility listener".to_string())
                })?;

            Ok(Self {
                _closure: Some(closure),
            })
        }

        #[cfg(not(target_arch = "wasm32"))]
        pub fn on_resize<F>(_callback: F) -> InfrastructureResult<Self>
        where
            F: FnMut(f32, f32) + 'static,
        {
            Ok(Self {
                _phantom: std::marker::PhantomData,
            })
        }

        #[cfg(not(target_arch = "wasm32"))]
        pub fn on_visibility_change<F>(_callback: F) -> InfrastructureResult<Self>
        where
            F: FnMut(bool) + 'static,
        {
            Ok(Self {
                _phantom: std::marker::PhantomData,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn web_infrastructure_creation() {
        let web_infra = WebInfrastructure::new("test-canvas".to_string());
        assert_eq!(web_infra.canvas_id, "test-canvas");
        assert!(web_infra.performance_monitoring);
    }

    #[test]
    fn web_infrastructure_default() {
        let web_infra = WebInfrastructure::default();
        assert_eq!(web_infra.canvas_id, "bevy");
    }

    #[test]
    fn web_utils_is_web() {
        let is_web = utils::is_web();
        assert_eq!(is_web, cfg!(target_arch = "wasm32"));
    }

    #[test]
    fn web_infrastructure_initialize() {
        let web_infra = WebInfrastructure::default();
        // Should not panic on any platform
        let result = web_infra.initialize();
        assert!(result.is_ok());
    }

    #[test]
    fn event_listener_creation() {
        // Test that event listeners can be created without panicking
        let _resize_listener = events::WebEventListener::on_resize(|_w, _h| {});
        let _visibility_listener = events::WebEventListener::on_visibility_change(|_visible| {});
    }
}
