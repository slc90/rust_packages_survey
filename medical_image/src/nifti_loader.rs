//! NIfTI 加载工具

use crate::volume::{MedicalImageError, VolumeData, VolumeModality};
use ndarray::ArrayViewD;
use nifti::{IntoNdArray, NiftiObject, ReaderOptions};
use std::path::Path;

/// 加载 NIfTI 文件并转换为统一体数据
pub fn load_nifti_file<P: AsRef<Path>>(path: P) -> Result<VolumeData, MedicalImageError> {
	let path = path.as_ref();
	let object = ReaderOptions::new()
		.read_file(path)
		.map_err(|error| MedicalImageError::Format(error.to_string()))?;

	let header = object.header().clone();
	let volume = object.into_volume();
	let array = volume
		.into_ndarray::<f32>()
		.map_err(|error| MedicalImageError::Format(error.to_string()))?;
	let shape = array.shape().to_vec();
	if shape.len() < 3 {
		return Err(MedicalImageError::Format(
			"当前仅支持至少三维的 NIfTI 体数据".to_string(),
		));
	}
	if shape[3..].iter().any(|size| *size != 1) {
		return Err(MedicalImageError::Format(
			"当前仅支持 3D NIfTI 体数据或尾部为 1 的单维扩展".to_string(),
		));
	}

	let dims = [shape[0], shape[1], shape[2]];
	let voxels = flatten_nifti_volume(array.view());
	let affine = build_nifti_affine(&header);
	let (spacing, origin, direction) = decompose_affine(affine);
	let modality = infer_nifti_modality(path, &header.descrip)?;

	VolumeData::new(dims, spacing, origin, direction, affine, voxels, modality)
}

/// 将 NIfTI 的 ndarray 数据整理为 x 最快变化的扁平数组
fn flatten_nifti_volume(array: ArrayViewD<'_, f32>) -> Vec<f32> {
	let shape = array.shape();
	let mut voxels = Vec::with_capacity(shape[0] * shape[1] * shape[2]);
	for z in 0..shape[2] {
		for y in 0..shape[1] {
			for x in 0..shape[0] {
				let mut index = vec![0; shape.len()];
				index[0] = x;
				index[1] = y;
				index[2] = z;
				voxels.push(array[index.as_slice()]);
			}
		}
	}
	voxels
}

/// 构造 NIfTI affine
fn build_nifti_affine(header: &nifti::NiftiHeader) -> [[f32; 4]; 4] {
	if header.sform_code != 0 {
		[
			header.srow_x,
			header.srow_y,
			header.srow_z,
			[0.0, 0.0, 0.0, 1.0],
		]
	} else {
		[
			[header.pixdim[1], 0.0, 0.0, 0.0],
			[0.0, header.pixdim[2], 0.0, 0.0],
			[0.0, 0.0, header.pixdim[3], 0.0],
			[0.0, 0.0, 0.0, 1.0],
		]
	}
}

/// 从 affine 中拆出 spacing、origin 和 direction
fn decompose_affine(affine: [[f32; 4]; 4]) -> ([f32; 3], [f32; 3], [[f32; 3]; 3]) {
	let column0 = [affine[0][0], affine[1][0], affine[2][0]];
	let column1 = [affine[0][1], affine[1][1], affine[2][1]];
	let column2 = [affine[0][2], affine[1][2], affine[2][2]];

	let spacing = [
		vector_norm(column0).max(f32::EPSILON),
		vector_norm(column1).max(f32::EPSILON),
		vector_norm(column2).max(f32::EPSILON),
	];
	let direction = [
		normalize_vector(column0, spacing[0]),
		normalize_vector(column1, spacing[1]),
		normalize_vector(column2, spacing[2]),
	];
	let origin = [affine[0][3], affine[1][3], affine[2][3]];

	(spacing, origin, direction)
}

/// 推断 NIfTI 模态
fn infer_nifti_modality(
	path: &Path,
	description: &[u8],
) -> Result<VolumeModality, MedicalImageError> {
	let mut hints = Vec::new();
	if let Some(file_name) = path.file_name().and_then(|value| value.to_str()) {
		hints.push(file_name.to_string());
	}

	let description = String::from_utf8_lossy(description)
		.trim_matches(char::from(0))
		.to_string();
	if !description.is_empty() {
		hints.push(description);
	}

	for hint in hints {
		let upper = hint.to_ascii_uppercase();
		let tokens: Vec<_> = upper
			.split(|character: char| !character.is_ascii_alphanumeric())
			.filter(|token| !token.is_empty())
			.collect();
		if tokens
			.iter()
			.any(|token| matches!(*token, "SEG" | "SEGMENTATION"))
		{
			return Ok(VolumeModality::Segmentation);
		}
		if tokens.contains(&"CT") {
			return Ok(VolumeModality::Ct);
		}
		if tokens.iter().any(|token| matches!(*token, "MR" | "MRI")) {
			return Ok(VolumeModality::Mr);
		}
	}

	Err(MedicalImageError::UnsupportedModality(
		path.to_string_lossy().to_string(),
	))
}

/// 计算向量模长
fn vector_norm(vector: [f32; 3]) -> f32 {
	(vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]).sqrt()
}

/// 对向量做归一化
fn normalize_vector(vector: [f32; 3], length: f32) -> [f32; 3] {
	if length <= f32::EPSILON {
		return [0.0, 0.0, 0.0];
	}
	[vector[0] / length, vector[1] / length, vector[2] / length]
}

#[cfg(test)]
mod tests {
	use super::load_nifti_file;
	use crate::volume::VolumeModality;
	use std::path::PathBuf;

	fn workspace_dir() -> PathBuf {
		PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.parent()
			.expect("workspace dir should exist")
			.to_path_buf()
	}

	#[test]
	fn should_load_ct_nifti_from_data_directory() {
		let path = workspace_dir()
			.join("data")
			.join("SubjectUCI29_CT_acpc_f.nii");
		let volume = load_nifti_file(path).expect("ct nifti should load successfully");
		assert_eq!(volume.modality, VolumeModality::Ct);
		assert!(volume.dims.iter().all(|value| *value > 0));
		assert_eq!(
			volume.voxel_count(),
			volume.dims[0] * volume.dims[1] * volume.dims[2]
		);
	}

	#[test]
	fn should_load_mr_nifti_from_data_directory() {
		let path = workspace_dir()
			.join("data")
			.join("SubjectUCI29_MR_acpc.nii");
		let volume = load_nifti_file(path).expect("mr nifti should load successfully");
		assert_eq!(volume.modality, VolumeModality::Mr);
		assert!(volume.dims.iter().all(|value| *value > 0));
		assert_eq!(
			volume.voxel_count(),
			volume.dims[0] * volume.dims[1] * volume.dims[2]
		);
	}
}
