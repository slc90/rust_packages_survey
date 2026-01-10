#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

use bevy::{
	log::{Level, LogPlugin},
	prelude::*,
};
use logger::{custom_layer, fmt_layer};

fn main() {
	App::new()
		.add_plugins(DefaultPlugins.set(LogPlugin {
			custom_layer,
			fmt_layer,
			level: Level::DEBUG,
			..default()
		}))
		.run();
}
