//! DICOM Series 加载工具

use crate::volume::{MedicalImageError, VolumeData, VolumeModality};
use dicom_object::open_file;
use dicom_pixeldata::PixelDecoder;
use ndarray::Array4;
use std::fs;
use std::path::{Path, PathBuf};

/// DICOM 序列基本信息
#[derive(Debug, Clone, PartialEq)]
pub struct DicomSeriesInfo {
	/// 序列 UID
	pub series_instance_uid: String,
	/// 模态
	pub modality: VolumeModality,
	/// 切片文件列表
	pub files: Vec<PathBuf>,
}

#[derive(Debug)]
struct DicomSlice {
	modality: VolumeModality,
	array: Array4<f32>,
	instance_number: i32,
	position: Vec<f32>,
	spacing_xy: [f32; 2],
	slice_thickness: f32,
	orientation: Vec<f32>,
}

/// 从目录中加载 DICOM 序列
pub fn load_dicom_series<P: AsRef<Path>>(directory: P) -> Result<VolumeData, MedicalImageError> {
	let directory = directory.as_ref();
	let files = collect_dicom_files(directory)?;
	if files.is_empty() {
		return Err(MedicalImageError::Format(format!(
			"DICOM 目录中没有可用文件: {}",
			directory.display()
		)));
	}

	let mut slices = Vec::new();
	for file in files {
		let object =
			open_file(&file).map_err(|error| MedicalImageError::Format(error.to_string()))?;
		let modality_text = object
			.element_by_name("Modality")
			.map_err(|error| MedicalImageError::Format(error.to_string()))?
			.to_str()
			.map_err(|error| MedicalImageError::Format(error.to_string()))?;
		let modality = VolumeModality::parse_supported(modality_text.as_ref())?;
		let position = parse_multi_f32(
			object
				.element_by_name("ImagePositionPatient")
				.ok()
				.and_then(|element| element.to_str().ok())
				.as_deref(),
		);
		let instance_number = object
			.element_by_name("InstanceNumber")
			.ok()
			.and_then(|element| element.to_str().ok())
			.and_then(|value| value.parse::<i32>().ok())
			.unwrap_or_default();
		let spacing_values = parse_multi_f32(
			object
				.element_by_name("PixelSpacing")
				.ok()
				.and_then(|element| element.to_str().ok())
				.as_deref(),
		);
		let slice_thickness = object
			.element_by_name("SliceThickness")
			.ok()
			.and_then(|element| element.to_str().ok())
			.and_then(|value| value.parse::<f32>().ok())
			.unwrap_or(1.0);
		let orientation = parse_multi_f32(
			object
				.element_by_name("ImageOrientationPatient")
				.ok()
				.and_then(|element| element.to_str().ok())
				.as_deref(),
		);

		let pixel_data = object
			.decode_pixel_data()
			.map_err(|error| MedicalImageError::Format(error.to_string()))?;
		let array = pixel_data
			.to_ndarray::<f32>()
			.map_err(|error| MedicalImageError::Format(error.to_string()))?;
		let dims = array.dim();
		if dims.0 != 1 || dims.3 != 1 {
			return Err(MedicalImageError::Format(
				"当前仅支持单帧单通道 DICOM 切片".to_string(),
			));
		}

		slices.push(DicomSlice {
			modality,
			array,
			instance_number,
			position,
			spacing_xy: [
				spacing_values.first().copied().unwrap_or(1.0),
				spacing_values.get(1).copied().unwrap_or(1.0),
			],
			slice_thickness,
			orientation,
		});
	}

	slices.sort_by(|left, right| {
		let left_key = left
			.position
			.get(2)
			.copied()
			.unwrap_or(left.instance_number as f32);
		let right_key = right
			.position
			.get(2)
			.copied()
			.unwrap_or(right.instance_number as f32);
		left_key
			.partial_cmp(&right_key)
			.unwrap_or(std::cmp::Ordering::Equal)
	});

	let first = slices
		.first()
		.ok_or_else(|| MedicalImageError::Format("没有可用的 DICOM 切片".to_string()))?;
	let rows = first.array.dim().1;
	let cols = first.array.dim().2;
	let dims = [cols, rows, slices.len()];
	let mut voxels = Vec::with_capacity(cols * rows * slices.len());
	for slice in &slices {
		for y in 0..rows {
			for x in 0..cols {
				voxels.push(slice.array[[0, y, x, 0]]);
			}
		}
	}

	let spacing = [
		first.spacing_xy[1],
		first.spacing_xy[0],
		first.slice_thickness.max(1.0),
	];
	let origin = [
		first.position.first().copied().unwrap_or(0.0),
		first.position.get(1).copied().unwrap_or(0.0),
		first.position.get(2).copied().unwrap_or(0.0),
	];
	let direction = build_direction(&first.orientation);
	let affine = build_affine(spacing, origin, direction);

	VolumeData::new(
		dims,
		spacing,
		origin,
		direction,
		affine,
		voxels,
		first.modality,
	)
}

/// 递归收集 DICOM 文件
fn collect_dicom_files(directory: &Path) -> Result<Vec<PathBuf>, MedicalImageError> {
	let mut files = Vec::new();
	for entry in fs::read_dir(directory)? {
		let entry = entry?;
		let path = entry.path();
		if path.is_dir() {
			files.extend(collect_dicom_files(&path)?);
		} else {
			files.push(path);
		}
	}
	Ok(files)
}

/// 解析 DICOM 多值字段
fn parse_multi_f32(value: Option<&str>) -> Vec<f32> {
	value
		.unwrap_or_default()
		.split('\\')
		.filter_map(|item| item.trim().parse::<f32>().ok())
		.collect()
}

/// 根据方向余弦构造方向矩阵
fn build_direction(orientation: &[f32]) -> [[f32; 3]; 3] {
	if orientation.len() >= 6 {
		let row = [orientation[0], orientation[1], orientation[2]];
		let col = [orientation[3], orientation[4], orientation[5]];
		let normal = cross(row, col);
		[row, col, normal]
	} else {
		[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
	}
}

/// 构造 affine 矩阵
fn build_affine(spacing: [f32; 3], origin: [f32; 3], direction: [[f32; 3]; 3]) -> [[f32; 4]; 4] {
	[
		[
			direction[0][0] * spacing[0],
			direction[1][0] * spacing[1],
			direction[2][0] * spacing[2],
			origin[0],
		],
		[
			direction[0][1] * spacing[0],
			direction[1][1] * spacing[1],
			direction[2][1] * spacing[2],
			origin[1],
		],
		[
			direction[0][2] * spacing[0],
			direction[1][2] * spacing[1],
			direction[2][2] * spacing[2],
			origin[2],
		],
		[0.0, 0.0, 0.0, 1.0],
	]
}

/// 三维向量叉积
fn cross(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
	[
		left[1] * right[2] - left[2] * right[1],
		left[2] * right[0] - left[0] * right[2],
		left[0] * right[1] - left[1] * right[0],
	]
}

#[cfg(test)]
mod tests {
	use super::parse_multi_f32;

	#[test]
	fn should_parse_multi_value_f32() {
		assert_eq!(parse_multi_f32(Some("1.0\\2.5\\3.5")), vec![1.0, 2.5, 3.5]);
		assert!(parse_multi_f32(None).is_empty());
	}
}
