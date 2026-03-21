//! 三视图切片工具

use crate::volume::{MedicalImageError, VolumeData};

/// 切片方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliceAxis {
	/// 轴状切片，固定 z
	Axial,
	/// 冠状切片，固定 y
	Coronal,
	/// 矢状切片，固定 x
	Sagittal,
}

impl SliceAxis {
	/// 返回切片方向名称
	pub fn as_str(&self) -> &'static str {
		match self {
			Self::Axial => "轴状",
			Self::Coronal => "冠状",
			Self::Sagittal => "矢状",
		}
	}
}

/// 提取出的二维切片图像
#[derive(Debug, Clone, PartialEq)]
pub struct SliceImage {
	/// 切片宽度
	pub width: usize,
	/// 切片高度
	pub height: usize,
	/// 切片像素数据，按行优先顺序存储
	pub pixels: Vec<f32>,
}

impl SliceImage {
	/// 创建新的切片图像
	pub fn new(width: usize, height: usize, pixels: Vec<f32>) -> Self {
		Self {
			width,
			height,
			pixels,
		}
	}
}

/// 从体数据中提取二维切片
pub fn extract_slice(
	volume: &VolumeData,
	axis: SliceAxis,
	index: usize,
) -> Result<SliceImage, MedicalImageError> {
	match axis {
		SliceAxis::Axial => extract_axial_slice(volume, index),
		SliceAxis::Coronal => extract_coronal_slice(volume, index),
		SliceAxis::Sagittal => extract_sagittal_slice(volume, index),
	}
}

/// 提取轴状切片
fn extract_axial_slice(volume: &VolumeData, index: usize) -> Result<SliceImage, MedicalImageError> {
	let z_size = volume.dims[2];
	if index >= z_size {
		return Err(MedicalImageError::SliceIndexOutOfBounds {
			axis: SliceAxis::Axial.as_str(),
			index,
			size: z_size,
		});
	}

	let width = volume.dims[0];
	let height = volume.dims[1];
	let mut pixels = Vec::with_capacity(width * height);
	for y in 0..height {
		for x in 0..width {
			pixels.push(volume.value_at(x, y, index).unwrap_or_default());
		}
	}

	Ok(SliceImage::new(width, height, pixels))
}

/// 提取冠状切片
fn extract_coronal_slice(
	volume: &VolumeData,
	index: usize,
) -> Result<SliceImage, MedicalImageError> {
	let y_size = volume.dims[1];
	if index >= y_size {
		return Err(MedicalImageError::SliceIndexOutOfBounds {
			axis: SliceAxis::Coronal.as_str(),
			index,
			size: y_size,
		});
	}

	let width = volume.dims[0];
	let height = volume.dims[2];
	let mut pixels = Vec::with_capacity(width * height);
	for z in 0..height {
		for x in 0..width {
			pixels.push(volume.value_at(x, index, z).unwrap_or_default());
		}
	}

	Ok(SliceImage::new(width, height, pixels))
}

/// 提取矢状切片
fn extract_sagittal_slice(
	volume: &VolumeData,
	index: usize,
) -> Result<SliceImage, MedicalImageError> {
	let x_size = volume.dims[0];
	if index >= x_size {
		return Err(MedicalImageError::SliceIndexOutOfBounds {
			axis: SliceAxis::Sagittal.as_str(),
			index,
			size: x_size,
		});
	}

	let width = volume.dims[1];
	let height = volume.dims[2];
	let mut pixels = Vec::with_capacity(width * height);
	for z in 0..height {
		for y in 0..width {
			pixels.push(volume.value_at(index, y, z).unwrap_or_default());
		}
	}

	Ok(SliceImage::new(width, height, pixels))
}

#[cfg(test)]
mod tests {
	use crate::slice::{SliceAxis, extract_slice};
	use crate::volume::{MedicalImageError, VolumeData, VolumeModality};

	fn sample_volume() -> VolumeData {
		VolumeData::new(
			[2, 3, 2],
			[1.0, 1.0, 1.0],
			[0.0, 0.0, 0.0],
			[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
			[
				[1.0, 0.0, 0.0, 0.0],
				[0.0, 1.0, 0.0, 0.0],
				[0.0, 0.0, 1.0, 0.0],
				[0.0, 0.0, 0.0, 1.0],
			],
			vec![
				0.0, 1.0, 2.0, 3.0, 4.0, 5.0, // z=0
				6.0, 7.0, 8.0, 9.0, 10.0, 11.0, // z=1
			],
			VolumeModality::Ct,
		)
		.expect("sample volume should be valid")
	}

	#[test]
	fn should_extract_axial_slice() {
		let slice = extract_slice(&sample_volume(), SliceAxis::Axial, 1)
			.expect("axial slice should be valid");
		assert_eq!(slice.width, 2);
		assert_eq!(slice.height, 3);
		assert_eq!(slice.pixels, vec![6.0, 7.0, 8.0, 9.0, 10.0, 11.0]);
	}

	#[test]
	fn should_extract_coronal_slice() {
		let slice = extract_slice(&sample_volume(), SliceAxis::Coronal, 1)
			.expect("coronal slice should be valid");
		assert_eq!(slice.width, 2);
		assert_eq!(slice.height, 2);
		assert_eq!(slice.pixels, vec![2.0, 3.0, 8.0, 9.0]);
	}

	#[test]
	fn should_extract_sagittal_slice() {
		let slice = extract_slice(&sample_volume(), SliceAxis::Sagittal, 1)
			.expect("sagittal slice should be valid");
		assert_eq!(slice.width, 3);
		assert_eq!(slice.height, 2);
		assert_eq!(slice.pixels, vec![1.0, 3.0, 5.0, 7.0, 9.0, 11.0]);
	}

	#[test]
	fn should_reject_out_of_bounds_slice_index() {
		let error = extract_slice(&sample_volume(), SliceAxis::Axial, 5)
			.expect_err("slice index should be out of bounds");
		assert_eq!(
			error,
			MedicalImageError::SliceIndexOutOfBounds {
				axis: "轴状",
				index: 5,
				size: 2,
			}
		);
	}
}
