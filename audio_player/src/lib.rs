#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

pub mod error;
pub mod events;
pub mod player;
pub mod state;

pub use error::AudioPlayerError;
pub use events::{AudioPlaybackStatus, PlayerCommand, PlayerEvent};
pub use player::PlayerHandle;
pub use state::PlaybackSnapshot;
