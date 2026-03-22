use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ScreenshotResult {
	pub width: u32,
	pub height: u32,
	pub path: PathBuf,
}
