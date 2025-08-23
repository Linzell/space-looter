//! Time Infrastructure - Cross-platform time utilities
//!
//! This module provides time utilities that work across both native and WASM targets.
//! It handles the differences in time APIs between platforms and provides a unified interface.

use crate::infrastructure::{InfrastructureError, InfrastructureResult};
use std::time::{Duration, SystemTime};

#[cfg(target_arch = "wasm32")]
use js_sys;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures;
#[cfg(target_arch = "wasm32")]
use web_sys;

/// Cross-platform time service
pub struct TimeService;

impl TimeService {
    /// Get current timestamp in milliseconds since Unix epoch
    pub fn now_millis() -> InfrastructureResult<u64> {
        #[cfg(target_arch = "wasm32")]
        {
            Self::web_now_millis()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::native_now_millis()
        }
    }

    /// Get current timestamp as Duration since Unix epoch
    pub fn now_duration() -> InfrastructureResult<Duration> {
        let millis = Self::now_millis()?;
        Ok(Duration::from_millis(millis))
    }

    /// Get high-resolution performance timestamp in milliseconds
    pub fn performance_now() -> InfrastructureResult<f64> {
        #[cfg(target_arch = "wasm32")]
        {
            Self::web_performance_now()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::native_performance_now()
        }
    }

    /// Create a Duration from milliseconds
    pub fn duration_from_millis(millis: u64) -> Duration {
        Duration::from_millis(millis)
    }

    /// Create a Duration from seconds
    pub fn duration_from_secs(secs: u64) -> Duration {
        Duration::from_secs(secs)
    }

    /// Convert Duration to milliseconds
    pub fn duration_to_millis(duration: Duration) -> u64 {
        duration.as_millis() as u64
    }

    /// Get elapsed time since a previous timestamp
    pub fn elapsed_since(start_millis: u64) -> InfrastructureResult<Duration> {
        let now = Self::now_millis()?;
        if now >= start_millis {
            Ok(Duration::from_millis(now - start_millis))
        } else {
            // Handle clock going backwards (rare but possible)
            Ok(Duration::from_millis(0))
        }
    }

    /// Sleep for a specified duration (blocking on native, non-blocking on web)
    pub fn sleep_sync(duration: Duration) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::thread::sleep(duration);
        }
        #[cfg(target_arch = "wasm32")]
        {
            // On web, we can't block, so this is a no-op
            // Use web_sleep_async for async operations
            web_sys::console::log_1(
                &format!("Sleep requested for {}ms", duration.as_millis()).into(),
            );
        }
    }

    /// Native implementation for getting current timestamp
    #[cfg(not(target_arch = "wasm32"))]
    fn native_now_millis() -> InfrastructureResult<u64> {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .map_err(|e| InfrastructureError::TimeError(format!("System time error: {}", e)))
    }

    /// Native implementation for performance timing
    #[cfg(not(target_arch = "wasm32"))]
    fn native_performance_now() -> InfrastructureResult<f64> {
        use std::time::Instant;
        // Use a static start time to measure relative performance
        static PERFORMANCE_START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
        let start = PERFORMANCE_START.get_or_init(|| Instant::now());
        Ok(start.elapsed().as_secs_f64() * 1000.0)
    }

    /// Web implementation for getting current timestamp
    #[cfg(target_arch = "wasm32")]
    fn web_now_millis() -> InfrastructureResult<u64> {
        let window = web_sys::window()
            .ok_or_else(|| InfrastructureError::WebError("No window object".to_string()))?;

        let date = js_sys::Date::new_0();
        Ok(date.get_time() as u64)
    }

    /// Web implementation for performance timing
    #[cfg(target_arch = "wasm32")]
    fn web_performance_now() -> InfrastructureResult<f64> {
        let window = web_sys::window()
            .ok_or_else(|| InfrastructureError::WebError("No window object".to_string()))?;

        let performance = window
            .performance()
            .ok_or_else(|| InfrastructureError::WebError("No performance object".to_string()))?;

        Ok(performance.now())
    }

    /// Web implementation for async sleep (for future use)
    #[cfg(target_arch = "wasm32")]
    pub async fn web_sleep_async(duration: Duration) -> Result<(), JsValue> {
        let millis = duration.as_millis() as i32;

        let promise = js_sys::Promise::new(&mut |resolve, _reject| {
            let window = web_sys::window().unwrap();
            let closure = Closure::once_into_js(move || {
                resolve.call0(&JsValue::UNDEFINED).unwrap();
            });

            window
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    millis,
                )
                .unwrap();
        });

        wasm_bindgen_futures::JsFuture::from(promise).await?;
        Ok(())
    }
}

/// Timer utility for measuring elapsed time
pub struct Timer {
    start_time: u64,
    name: String,
}

impl Timer {
    /// Create and start a new timer
    pub fn start(name: String) -> InfrastructureResult<Self> {
        let start_time = TimeService::now_millis()?;
        Ok(Self { start_time, name })
    }

    /// Get elapsed time since timer start
    pub fn elapsed(&self) -> InfrastructureResult<Duration> {
        TimeService::elapsed_since(self.start_time)
    }

    /// Stop the timer and return elapsed time with name
    pub fn stop(self) -> InfrastructureResult<(String, Duration)> {
        let elapsed = self.elapsed()?;
        let name = self.name.clone();
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::console::log_1(&format!("Timer '{}': {}ms", name, elapsed.as_millis()).into());
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            println!("Timer '{}': {}ms", name, elapsed.as_millis());
        }
        Ok((name, elapsed))
    }

    /// Get timer name
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Stopwatch for multiple measurements
pub struct Stopwatch {
    measurements: Vec<(String, Duration)>,
    current_timer: Option<Timer>,
}

impl Stopwatch {
    /// Create a new stopwatch
    pub fn new() -> Self {
        Self {
            measurements: Vec::new(),
            current_timer: None,
        }
    }

    /// Start measuring a new operation
    pub fn start(&mut self, name: String) -> InfrastructureResult<()> {
        // Stop current timer if running
        if let Some(timer) = self.current_timer.take() {
            let (name, elapsed) = timer.stop()?;
            self.measurements.push((name, elapsed));
        }

        // Start new timer
        self.current_timer = Some(Timer::start(name)?);
        Ok(())
    }

    /// Stop current timer
    pub fn stop(&mut self) -> InfrastructureResult<Option<Duration>> {
        if let Some(timer) = self.current_timer.take() {
            let (name, elapsed) = timer.stop()?;
            self.measurements.push((name, elapsed));
            Ok(Some(elapsed))
        } else {
            Ok(None)
        }
    }

    /// Get all measurements
    pub fn measurements(&self) -> &[(String, Duration)] {
        &self.measurements
    }

    /// Clear all measurements
    pub fn clear(&mut self) {
        self.measurements.clear();
        self.current_timer = None;
    }

    /// Get total time of all measurements
    pub fn total_time(&self) -> Duration {
        self.measurements
            .iter()
            .fold(Duration::from_secs(0), |acc, (_, duration)| acc + *duration)
    }
}

impl Default for Stopwatch {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for common time operations
pub mod utils {
    use super::*;

    /// Format duration as human-readable string
    pub fn format_duration(duration: Duration) -> String {
        let total_secs = duration.as_secs();
        let millis = duration.subsec_millis();

        if total_secs >= 3600 {
            let hours = total_secs / 3600;
            let mins = (total_secs % 3600) / 60;
            let secs = total_secs % 60;
            format!("{}h {}m {}s", hours, mins, secs)
        } else if total_secs >= 60 {
            let mins = total_secs / 60;
            let secs = total_secs % 60;
            format!("{}m {}s", mins, secs)
        } else if total_secs > 0 {
            format!("{}.{}s", total_secs, millis / 100)
        } else {
            format!("{}ms", millis)
        }
    }

    /// Get current time as ISO 8601 string (for logging)
    pub fn current_time_iso() -> InfrastructureResult<String> {
        #[cfg(target_arch = "wasm32")]
        {
            let date = js_sys::Date::new_0();
            Ok(date
                .to_iso_string()
                .as_string()
                .unwrap_or_else(|| "unknown".to_string()))
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            use chrono::{DateTime, Utc};
            let now = TimeService::now_millis()?;
            let datetime =
                DateTime::from_timestamp((now / 1000) as i64, ((now % 1000) * 1_000_000) as u32)
                    .unwrap_or_else(|| Utc::now());
            Ok(datetime.to_rfc3339())
        }
    }

    /// Check if enough time has passed since last timestamp
    pub fn has_elapsed_since(
        last_time: u64,
        required_duration: Duration,
    ) -> InfrastructureResult<bool> {
        let elapsed = TimeService::elapsed_since(last_time)?;
        Ok(elapsed >= required_duration)
    }

    /// Get timestamp for "now + duration"
    pub fn future_timestamp(duration: Duration) -> InfrastructureResult<u64> {
        let now = TimeService::now_millis()?;
        Ok(now + duration.as_millis() as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_service_now_works() {
        let now = TimeService::now_millis().unwrap();
        assert!(now > 0);
    }

    #[test]
    fn duration_conversion_works() {
        let duration = Duration::from_secs(5);
        let millis = TimeService::duration_to_millis(duration);
        assert_eq!(millis, 5000);

        let back = TimeService::duration_from_millis(millis);
        assert_eq!(back, duration);
    }

    #[test]
    fn timer_works() {
        let timer = Timer::start("test".to_string()).unwrap();
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::thread::sleep(std::time::Duration::from_millis(10));
            let elapsed = timer.elapsed().unwrap();
            assert!(elapsed.as_millis() >= 10);
        }
        #[cfg(target_arch = "wasm32")]
        {
            // On WASM, just check that the timer was created
            let elapsed = timer.elapsed().unwrap();
            assert!(elapsed.as_millis() >= 0);
        }
    }

    #[test]
    fn stopwatch_works() {
        let mut stopwatch = Stopwatch::new();
        stopwatch.start("test1".to_string()).unwrap();
        #[cfg(not(target_arch = "wasm32"))]
        std::thread::sleep(std::time::Duration::from_millis(10));
        stopwatch.start("test2".to_string()).unwrap();
        #[cfg(not(target_arch = "wasm32"))]
        std::thread::sleep(std::time::Duration::from_millis(10));
        stopwatch.stop().unwrap();

        let measurements = stopwatch.measurements();
        assert_eq!(measurements.len(), 2);
        assert_eq!(measurements[0].0, "test1");
        assert_eq!(measurements[1].0, "test2");
    }

    #[test]
    fn format_duration_works() {
        let duration = Duration::from_millis(1500);
        let formatted = utils::format_duration(duration);
        assert_eq!(formatted, "1.5s");

        let duration = Duration::from_secs(65);
        let formatted = utils::format_duration(duration);
        assert_eq!(formatted, "1m 5s");

        let duration = Duration::from_secs(3661);
        let formatted = utils::format_duration(duration);
        assert_eq!(formatted, "1h 1m 1s");
    }

    #[test]
    fn future_timestamp_works() {
        let duration = Duration::from_secs(10);
        let future = utils::future_timestamp(duration).unwrap();
        let now = TimeService::now_millis().unwrap();
        assert!(future > now);
        assert!(future - now >= 10000); // At least 10 seconds in the future
    }
}
