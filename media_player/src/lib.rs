#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

pub mod error;
pub mod events;
pub mod frame;
pub mod pipeline;
pub mod player;

pub use error::MediaPlayerError;
pub use events::{PlaybackStatus, PlayerCommand, PlayerEvent};
pub use frame::VideoFrame;
pub use player::PlayerHandle;
