use bevy::prelude::*;

/// Determine what do on left click.
#[derive(Resource, Debug)]
pub enum LeftClickAction {
	/// Move the window on left click.
	Move,
}

/// Cooldown resource for title bar button clicks to prevent rapid consecutive clicks.
#[derive(Resource, Debug)]
pub struct TitleBarButtonCooldown {
	/// Timer for click cooldown
	pub timer: Timer,
}

impl Default for TitleBarButtonCooldown {
	fn default() -> Self {
		Self {
			// Set cooldown duration to 0.3 seconds (300 milliseconds)
			timer: Timer::from_seconds(0.3, TimerMode::Once),
		}
	}
}

/// Resource to track window state for focus management
#[derive(Resource, Debug, Default)]
pub struct WindowState {
	/// Track if window was minimized (to handle focus when restored)
	pub was_minimized: bool,
}
