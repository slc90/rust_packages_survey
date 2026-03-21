//! 表面重建工具

use crate::volume::{MedicalImageError, VolumeData};
use fast_surface_nets::ndshape::RuntimeShape;
use fast_surface_nets::{SurfaceNetsBuffer, surface_nets};

/// 表面提取参数
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceExtractOptions {
	/// 等值面阈值；体素值大于等于该值时视为内部
	pub threshold: f32,
}

impl Default for SurfaceExtractOptions {
	fn default() -> Self {
		Self { threshold: 300.0 }
	}
}

/// 表面网格统计信息
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceMeshStats {
	/// 顶点数量
	pub vertex_count: usize,
	/// 三角形数量
	pub triangle_count: usize,
}

/// 表面网格中间结构
#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceMeshData {
	/// 顶点位置，已经转换到物理空间
	pub positions: Vec<[f32; 3]>,
	/// 顶点法线，已经转换到物理空间并归一化
	pub normals: Vec<[f32; 3]>,
	/// 三角形索引
	pub indices: Vec<u32>,
	/// 包围盒最小点
	pub bounds_min: [f32; 3],
	/// 包围盒最大点
	pub bounds_max: [f32; 3],
}

impl SurfaceMeshData {
	/// 返回网格统计信息
	pub fn stats(&self) -> SurfaceMeshStats {
		SurfaceMeshStats {
			vertex_count: self.positions.len(),
			triangle_count: self.indices.len() / 3,
		}
	}

	/// 返回包围盒中心
	pub fn center(&self) -> [f32; 3] {
		[
			(self.bounds_min[0] + self.bounds_max[0]) * 0.5,
			(self.bounds_min[1] + self.bounds_max[1]) * 0.5,
			(self.bounds_min[2] + self.bounds_max[2]) * 0.5,
		]
	}

	/// 返回包围盒对角线长度
	pub fn diagonal_length(&self) -> f32 {
		let dx = self.bounds_max[0] - self.bounds_min[0];
		let dy = self.bounds_max[1] - self.bounds_min[1];
		let dz = self.bounds_max[2] - self.bounds_min[2];
		(dx * dx + dy * dy + dz * dz).sqrt()
	}
}

/// 从体数据中提取阈值等值面
pub fn extract_isosurface(
	volume: &VolumeData,
	options: SurfaceExtractOptions,
) -> Result<SurfaceMeshData, MedicalImageError> {
	let padded_dims = [volume.dims[0] + 2, volume.dims[1] + 2, volume.dims[2] + 2];
	let shape = RuntimeShape::<u32, 3>::new([
		padded_dims[0] as u32,
		padded_dims[1] as u32,
		padded_dims[2] as u32,
	]);

	let outside_value = volume.value_range[0].min(options.threshold - 1.0);
	let mut sdf = vec![1.0_f32; padded_dims[0] * padded_dims[1] * padded_dims[2]];

	for z in 0..padded_dims[2] {
		for y in 0..padded_dims[1] {
			for x in 0..padded_dims[0] {
				let sample = if x == 0
					|| y == 0 || z == 0
					|| x == padded_dims[0] - 1
					|| y == padded_dims[1] - 1
					|| z == padded_dims[2] - 1
				{
					outside_value
				} else {
					volume
						.value_at(x - 1, y - 1, z - 1)
						.unwrap_or(outside_value)
				};
				let linear_index = z * padded_dims[0] * padded_dims[1] + y * padded_dims[0] + x;
				sdf[linear_index] = options.threshold - sample;
			}
		}
	}

	let mut buffer = SurfaceNetsBuffer::default();
	surface_nets(
		&sdf,
		&shape,
		[0, 0, 0],
		[
			(padded_dims[0] - 1) as u32,
			(padded_dims[1] - 1) as u32,
			(padded_dims[2] - 1) as u32,
		],
		&mut buffer,
	);

	if buffer.positions.is_empty() || buffer.indices.is_empty() {
		return Err(MedicalImageError::Format(format!(
			"阈值 {} 未提取到可用表面",
			options.threshold
		)));
	}

	let mut positions = Vec::with_capacity(buffer.positions.len());
	let mut normals = Vec::with_capacity(buffer.normals.len());
	let mut bounds_min = [f32::INFINITY; 3];
	let mut bounds_max = [f32::NEG_INFINITY; 3];

	for position in &buffer.positions {
		let voxel_position = [position[0] - 1.0, position[1] - 1.0, position[2] - 1.0];
		let world_position = apply_affine(&volume.affine, voxel_position);
		for axis in 0..3 {
			bounds_min[axis] = bounds_min[axis].min(world_position[axis]);
			bounds_max[axis] = bounds_max[axis].max(world_position[axis]);
		}
		positions.push(world_position);
	}

	for normal in &buffer.normals {
		normals.push(transform_normal(volume, *normal));
	}

	Ok(SurfaceMeshData {
		positions,
		normals,
		indices: buffer.indices,
		bounds_min,
		bounds_max,
	})
}

/// 使用 affine 将体素坐标转换为物理空间坐标
fn apply_affine(affine: &[[f32; 4]; 4], voxel_position: [f32; 3]) -> [f32; 3] {
	[
		affine[0][0] * voxel_position[0]
			+ affine[0][1] * voxel_position[1]
			+ affine[0][2] * voxel_position[2]
			+ affine[0][3],
		affine[1][0] * voxel_position[0]
			+ affine[1][1] * voxel_position[1]
			+ affine[1][2] * voxel_position[2]
			+ affine[1][3],
		affine[2][0] * voxel_position[0]
			+ affine[2][1] * voxel_position[1]
			+ affine[2][2] * voxel_position[2]
			+ affine[2][3],
	]
}

/// 将体素空间法线转换到物理空间
fn transform_normal(volume: &VolumeData, normal: [f32; 3]) -> [f32; 3] {
	let scaled = [
		normal[0] / volume.spacing[0].max(f32::EPSILON),
		normal[1] / volume.spacing[1].max(f32::EPSILON),
		normal[2] / volume.spacing[2].max(f32::EPSILON),
	];
	let rotated = [
		volume.direction[0][0] * scaled[0]
			+ volume.direction[0][1] * scaled[1]
			+ volume.direction[0][2] * scaled[2],
		volume.direction[1][0] * scaled[0]
			+ volume.direction[1][1] * scaled[1]
			+ volume.direction[1][2] * scaled[2],
		volume.direction[2][0] * scaled[0]
			+ volume.direction[2][1] * scaled[1]
			+ volume.direction[2][2] * scaled[2],
	];
	normalize_vector(rotated)
}

/// 归一化向量；零向量回退到 Z 轴正方向
fn normalize_vector(vector: [f32; 3]) -> [f32; 3] {
	let length = (vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]).sqrt();
	if length <= f32::EPSILON {
		return [0.0, 0.0, 1.0];
	}

	[vector[0] / length, vector[1] / length, vector[2] / length]
}

#[cfg(test)]
mod tests {
	use super::{SurfaceExtractOptions, extract_isosurface};
	use crate::volume::{VolumeData, VolumeModality};

	fn sample_volume() -> VolumeData {
		let dims = [8, 8, 8];
		let mut voxels = vec![0.0; dims[0] * dims[1] * dims[2]];
		for z in 2..6 {
			for y in 2..6 {
				for x in 2..6 {
					let index = z * dims[0] * dims[1] + y * dims[0] + x;
					voxels[index] = 1000.0;
				}
			}
		}

		VolumeData::new(
			dims,
			[1.0, 1.0, 1.0],
			[0.0, 0.0, 0.0],
			[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
			[
				[1.0, 0.0, 0.0, 0.0],
				[0.0, 1.0, 0.0, 0.0],
				[0.0, 0.0, 1.0, 0.0],
				[0.0, 0.0, 0.0, 1.0],
			],
			voxels,
			VolumeModality::Ct,
		)
		.expect("sample volume should be valid")
	}

	#[test]
	fn should_extract_surface_mesh() {
		let mesh = extract_isosurface(&sample_volume(), SurfaceExtractOptions { threshold: 300.0 })
			.expect("surface should be extracted");

		assert!(!mesh.positions.is_empty());
		assert!(!mesh.indices.is_empty());
		assert_eq!(mesh.indices.len() % 3, 0);
		assert!(mesh.diagonal_length() > 0.0);
	}
}
