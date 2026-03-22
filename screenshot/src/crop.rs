use bevy::prelude::Image;
use image::{DynamicImage, GenericImageView};

use crate::{error::ScreenshotError, request::CaptureRegion};

pub fn crop_image(
	image: &Image,
	region: CaptureRegion,
) -> Result<(DynamicImage, u32, u32), ScreenshotError> {
	let dynamic = image
		.clone()
		.try_into_dynamic()
		.map_err(|error| ScreenshotError::DynamicImage(error.to_string()))?;
	let (width, height) = dynamic.dimensions();
	let x2 = region.x.saturating_add(region.width);
	let y2 = region.y.saturating_add(region.height);
	if x2 > width || y2 > height {
		return Err(ScreenshotError::InvalidCropRegion {
			width,
			height,
			x: region.x,
			y: region.y,
			crop_width: region.width,
			crop_height: region.height,
		});
	}
	let cropped = dynamic.crop_imm(region.x, region.y, region.width, region.height);
	Ok((cropped, region.width, region.height))
}

pub fn image_dimensions(image: &Image) -> Result<(u32, u32), ScreenshotError> {
	let dynamic = image
		.clone()
		.try_into_dynamic()
		.map_err(|error| ScreenshotError::DynamicImage(error.to_string()))?;
	Ok(dynamic.dimensions())
}
