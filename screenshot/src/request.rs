#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureOutputKind {
	Window,
	RegionRender,
	RegionCrop,
	CurrentDisplay,
	ScreenRegion,
}

impl CaptureOutputKind {
	pub fn prefix(self) -> &'static str {
		match self {
			Self::Window => "window",
			Self::RegionRender => "region_render",
			Self::RegionCrop => "region_crop",
			Self::CurrentDisplay => "desktop",
			Self::ScreenRegion => "screen_region",
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaptureRegion {
	pub x: u32,
	pub y: u32,
	pub width: u32,
	pub height: u32,
}
