//! Monotonic timing that works on native targets and in the browser.
//!
//! `std::time::Instant::now()` compiles for `wasm32` but panics at runtime.
//! Benchmarks therefore use `Instant` natively and `performance.now()` in the
//! browser. Both paths expose the same `elapsed()` shape so callers stay
//! platform-agnostic.

#[cfg(not(target_arch = "wasm32"))]
mod platform {
    use std::time::{Duration, Instant};

    #[derive(Debug, Clone, Copy)]
    pub(crate) struct Clock(Instant);

    impl Clock {
        pub(crate) fn now() -> Self {
            Self(Instant::now())
        }

        pub(crate) fn elapsed(&self) -> Duration {
            self.0.elapsed()
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod platform {
    use std::time::Duration;

    #[derive(Debug, Clone, Copy)]
    pub(crate) struct Clock(f64);

    impl Clock {
        pub(crate) fn now() -> Self {
            let milliseconds = web_sys::window()
                .and_then(|window| window.performance())
                .map(|performance| performance.now())
                .unwrap_or(0.0);
            Self(milliseconds)
        }

        pub(crate) fn elapsed(&self) -> Duration {
            let now = Self::now().0;
            let delta = (now - self.0).max(0.0);
            Duration::from_secs_f64(delta / 1_000.0)
        }
    }
}

pub(crate) use platform::Clock;

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::Clock;
    use std::time::Duration;

    #[test]
    fn elapsed_time_never_moves_backwards() {
        let start = Clock::now();
        std::thread::sleep(Duration::from_millis(2));
        assert!(start.elapsed() >= Duration::from_millis(2));
    }
}
