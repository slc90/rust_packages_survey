//! 窗宽窗位工具

use crate::slice::SliceImage;
use crate::volume::MedicalImageError;

/// 对单个体素值执行窗宽窗位映射
pub fn window_value(
	value: f32,
	window_center: f32,
	window_width: f32,
) -> Result<u8, MedicalImageError> {
	if window_width <= 0.0 {
		return Err(MedicalImageError::InvalidWindowWidth(window_width));
	}

	let lower = window_center - window_width / 2.0;
	let upper = window_center + window_width / 2.0;
	if value <= lower {
		return Ok(0);
	}
	if value >= upper {
		return Ok(255);
	}

	let normalized = ((value - lower) / window_width * 255.0).round();
	Ok(normalized.clamp(0.0, 255.0) as u8)
}

/// 将切片数据映射为 8 位灰度图
pub fn normalize_slice_to_u8(
	slice: &SliceImage,
	window_center: f32,
	window_width: f32,
) -> Result<Vec<u8>, MedicalImageError> {
	let mut mapped = Vec::with_capacity(slice.pixels.len());
	for value in &slice.pixels {
		mapped.push(window_value(*value, window_center, window_width)?);
	}
	Ok(mapped)
}

#[cfg(test)]
mod tests {
	use crate::slice::SliceImage;
	use crate::volume::MedicalImageError;
	use crate::windowing::{normalize_slice_to_u8, window_value};

	#[test]
	fn should_map_window_value_into_u8() {
		assert_eq!(window_value(-100.0, 50.0, 200.0), Ok(0));
		assert_eq!(window_value(50.0, 50.0, 200.0), Ok(128));
		assert_eq!(window_value(200.0, 50.0, 200.0), Ok(255));
	}

	#[test]
	fn should_reject_invalid_window_width() {
		assert_eq!(
			window_value(10.0, 0.0, 0.0),
			Err(MedicalImageError::InvalidWindowWidth(0.0))
		);
	}

	#[test]
	fn should_normalize_slice() {
		let slice = SliceImage::new(2, 2, vec![0.0, 50.0, 100.0, 150.0]);
		let normalized =
			normalize_slice_to_u8(&slice, 75.0, 150.0).expect("windowing should succeed");
		assert_eq!(normalized, vec![0, 85, 170, 255]);
	}
}
