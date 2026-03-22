#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![allow(clippy::type_complexity)]

pub mod const_assets_path;
pub mod plugin;

/// 范式默认 GIF 的嵌入字节。
pub const PARADIGM_DEFAULT_GIF_BYTES: &[u8] = include_bytes!("../assets/paradigm/default.gif");
