//! 统一体数据结构定义

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// 医学影像统一错误类型
#[derive(Debug, Clone, PartialEq)]
pub enum MedicalImageError {
	/// 不支持的医学影像模态
	UnsupportedModality(String),
	/// 非法维度
	InvalidDimensions([usize; 3]),
	/// 体素数量与尺寸不匹配
	VoxelCountMismatch { expected: usize, actual: usize },
	/// 切片索引越界
	SliceIndexOutOfBounds {
		axis: &'static str,
		index: usize,
		size: usize,
	},
	/// 空体数据
	EmptyVolume,
	/// 非法窗宽参数
	InvalidWindowWidth(f32),
	/// I/O 错误
	Io(String),
	/// 数据格式错误
	Format(String),
}

impl Display for MedicalImageError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::UnsupportedModality(modality) => {
				write!(f, "不支持的医学影像模态: {modality}")
			}
			Self::InvalidDimensions(dims) => write!(f, "非法体数据尺寸: {:?}", dims),
			Self::VoxelCountMismatch { expected, actual } => {
				write!(f, "体素数量与尺寸不匹配，期望 {expected}，实际 {actual}")
			}
			Self::SliceIndexOutOfBounds { axis, index, size } => {
				write!(f, "{axis} 切片索引越界: index={index}, size={size}")
			}
			Self::EmptyVolume => write!(f, "体数据为空"),
			Self::InvalidWindowWidth(width) => write!(f, "非法窗宽: {width}"),
			Self::Io(message) => write!(f, "I/O 错误: {message}"),
			Self::Format(message) => write!(f, "数据格式错误: {message}"),
		}
	}
}

impl Error for MedicalImageError {}

impl From<std::io::Error> for MedicalImageError {
	fn from(value: std::io::Error) -> Self {
		Self::Io(value.to_string())
	}
}

/// 医学影像模态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolumeModality {
	/// CT 体数据
	Ct,
	/// MR 体数据
	Mr,
	/// 分割结果体数据
	Segmentation,
}

impl VolumeModality {
	/// 从模态字符串解析支持的枚举值
	pub fn parse_supported(modality: &str) -> Result<Self, MedicalImageError> {
		Self::from_str(modality)
	}
}

impl FromStr for VolumeModality {
	type Err = MedicalImageError;

	fn from_str(modality: &str) -> Result<Self, Self::Err> {
		match modality.trim().to_ascii_uppercase().as_str() {
			"CT" => Ok(Self::Ct),
			"MR" => Ok(Self::Mr),
			"SEG" | "SEGMENTATION" => Ok(Self::Segmentation),
			other => Err(MedicalImageError::UnsupportedModality(other.to_string())),
		}
	}
}

/// 统一的三维体数据结构
#[derive(Debug, Clone, PartialEq)]
pub struct VolumeData {
	/// 体数据尺寸，顺序为 x、y、z
	pub dims: [usize; 3],
	/// 每个体素在三个方向上的物理间距，单位 mm
	pub spacing: [f32; 3],
	/// 体数据原点，对应世界坐标系中的起点
	pub origin: [f32; 3],
	/// 方向余弦矩阵，用于表示体数据朝向
	pub direction: [[f32; 3]; 3],
	/// 从 voxel 坐标到病人/世界坐标的 4x4 仿射矩阵
	pub affine: [[f32; 4]; 4],
	/// 连续存储的体素数据，统一转换为 f32
	pub voxels: Vec<f32>,
	/// 当前体数据的最小值和最大值
	pub value_range: [f32; 2],
	/// 医学影像模态，只保留当前明确支持的类型
	pub modality: VolumeModality,
}

impl VolumeData {
	/// 创建并校验新的体数据
	pub fn new(
		dims: [usize; 3],
		spacing: [f32; 3],
		origin: [f32; 3],
		direction: [[f32; 3]; 3],
		affine: [[f32; 4]; 4],
		voxels: Vec<f32>,
		modality: VolumeModality,
	) -> Result<Self, MedicalImageError> {
		if dims.contains(&0) {
			return Err(MedicalImageError::InvalidDimensions(dims));
		}

		let expected = dims[0] * dims[1] * dims[2];
		let actual = voxels.len();
		if expected != actual {
			return Err(MedicalImageError::VoxelCountMismatch { expected, actual });
		}
		if voxels.is_empty() {
			return Err(MedicalImageError::EmptyVolume);
		}

		let value_range = Self::compute_value_range(&voxels)?;
		Ok(Self {
			dims,
			spacing,
			origin,
			direction,
			affine,
			voxels,
			value_range,
			modality,
		})
	}

	/// 计算体素数量
	pub fn voxel_count(&self) -> usize {
		self.voxels.len()
	}

	/// 计算体素线性索引
	pub fn voxel_index(&self, x: usize, y: usize, z: usize) -> Option<usize> {
		if x >= self.dims[0] || y >= self.dims[1] || z >= self.dims[2] {
			return None;
		}

		Some(z * self.dims[0] * self.dims[1] + y * self.dims[0] + x)
	}

	/// 读取指定位置的体素值
	pub fn value_at(&self, x: usize, y: usize, z: usize) -> Option<f32> {
		let index = self.voxel_index(x, y, z)?;
		self.voxels.get(index).copied()
	}

	/// 计算数值范围
	pub fn compute_value_range(voxels: &[f32]) -> Result<[f32; 2], MedicalImageError> {
		let mut iter = voxels.iter().copied();
		let Some(first) = iter.next() else {
			return Err(MedicalImageError::EmptyVolume);
		};

		let (mut min_value, mut max_value) = (first, first);
		for value in iter {
			if value < min_value {
				min_value = value;
			}
			if value > max_value {
				max_value = value;
			}
		}

		Ok([min_value, max_value])
	}
}

#[cfg(test)]
mod tests {
	use super::{MedicalImageError, VolumeData, VolumeModality};

	fn sample_affine() -> [[f32; 4]; 4] {
		[
			[1.0, 0.0, 0.0, 0.0],
			[0.0, 1.0, 0.0, 0.0],
			[0.0, 0.0, 1.0, 0.0],
			[0.0, 0.0, 0.0, 1.0],
		]
	}

	#[test]
	fn should_create_valid_volume() {
		let volume = VolumeData::new(
			[2, 2, 2],
			[1.0, 1.0, 1.0],
			[0.0, 0.0, 0.0],
			[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
			sample_affine(),
			vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0],
			VolumeModality::Ct,
		)
		.expect("volume should be valid");

		assert_eq!(volume.value_range, [0.0, 7.0]);
		assert_eq!(volume.voxel_count(), 8);
		assert_eq!(volume.value_at(1, 1, 1), Some(7.0));
	}

	#[test]
	fn should_reject_unsupported_modality() {
		let error = VolumeModality::parse_supported("PET").expect_err("pet should be unsupported");
		assert_eq!(
			error,
			MedicalImageError::UnsupportedModality("PET".to_string())
		);
	}

	#[test]
	fn should_reject_mismatched_voxel_count() {
		let error = VolumeData::new(
			[2, 2, 2],
			[1.0, 1.0, 1.0],
			[0.0, 0.0, 0.0],
			[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
			sample_affine(),
			vec![0.0; 7],
			VolumeModality::Mr,
		)
		.expect_err("volume should reject mismatched voxel count");

		assert_eq!(
			error,
			MedicalImageError::VoxelCountMismatch {
				expected: 8,
				actual: 7,
			}
		);
	}
}
