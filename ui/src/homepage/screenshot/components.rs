use bevy::prelude::*;
use screenshot::request::CaptureRegion;
use std::path::PathBuf;

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct ScreenshotContentMarker;

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct ScreenshotWindowButtonMarker;

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct ScreenshotRegionRenderButtonMarker;

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct ScreenshotCurrentDisplayButtonMarker;

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct ScreenshotScreenRegionButtonMarker;

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct ScreenshotStatusTextMarker;

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct ScreenshotRenderPreviewMarker;

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct ScreenshotRenderCameraMarker;

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct ScreenshotRenderSceneMarker;

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct ScreenshotCropAreaMarker;

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct ScreenRegionOverlayWindowMarker;

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct ScreenRegionOverlayCameraMarker;

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct ScreenRegionOverlayRootMarker;

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct ScreenRegionOverlaySelectionMarker;

#[derive(Component, Debug, Clone)]
pub struct PendingWindowScreenshotTask {
	pub path: PathBuf,
}

#[derive(Component, Debug, Clone)]
pub struct PendingRenderScreenshotTask {
	pub path: PathBuf,
}

#[derive(Component, Debug, Clone)]
pub struct PendingCropScreenshotTask {
	pub path: PathBuf,

	pub region: CaptureRegion,
}
