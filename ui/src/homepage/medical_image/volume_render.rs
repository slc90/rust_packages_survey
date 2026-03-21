use bevy::image::ImageSampler;
use bevy::pbr::{Material, MaterialPlugin};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{AsBindGroup, Extent3d, TextureDimension, TextureFormat};
use bevy::shader::ShaderRef;
use medical_image::VolumeData;

const VOLUME_SHADER_ASSET_PATH: &str =
	"embedded://embedded_assets/../assets/shaders/medical_volume.wgsl";
/// 体渲染阶段允许上传到 GPU 的最大体纹理边长。
const MAX_VOLUME_TEXTURE_DIMENSION: usize = 256;
/// 体渲染阶段允许上传到 GPU 的最大体素数量。
const MAX_VOLUME_TEXTURE_VOXELS: usize =
	MAX_VOLUME_TEXTURE_DIMENSION * MAX_VOLUME_TEXTURE_DIMENSION * MAX_VOLUME_TEXTURE_DIMENSION;

/// 构建体纹理后的附加信息。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VolumeTextureBuildInfo {
	/// 上传到 GPU 的体纹理尺寸。
	pub texture_dims: [usize; 3],
	/// 各轴降采样步长；1 表示未降采样。
	pub downsample_factors: [usize; 3],
}

impl VolumeTextureBuildInfo {
	/// 当前体纹理是否发生了降采样。
	pub fn is_downsampled(self) -> bool {
		self.downsample_factors.iter().any(|factor| *factor > 1)
	}
}

/// 体纹理构建结果。
#[derive(Debug, Clone)]
pub struct VolumeTextureBuildResult {
	/// 最终上传到渲染世界的 3D 纹理。
	pub image: Image,
	/// 纹理尺寸和降采样信息。
	pub info: VolumeTextureBuildInfo,
}

/// 体渲染自定义材质
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct VolumeRenderMaterial {
	/// 窗口和步长参数：低阈值、高阈值、步长比例、保留位
	#[uniform(0)]
	pub render_params: Vec4,
	/// 包围盒最小点
	#[uniform(1)]
	pub bounds_min: Vec4,
	/// 包围盒最大点
	#[uniform(2)]
	pub bounds_max: Vec4,
	/// 归一化后的 3D 体纹理
	#[texture(3, dimension = "3d")]
	#[sampler(4)]
	pub volume_texture: Handle<Image>,
}

impl Material for VolumeRenderMaterial {
	fn fragment_shader() -> ShaderRef {
		VOLUME_SHADER_ASSET_PATH.into()
	}

	fn alpha_mode(&self) -> AlphaMode {
		AlphaMode::Blend
	}

	fn enable_prepass() -> bool {
		false
	}

	fn enable_shadows() -> bool {
		false
	}
}

/// 体渲染材质插件
pub type VolumeRenderMaterialPlugin = MaterialPlugin<VolumeRenderMaterial>;

/// 构建归一化后的 3D 纹理。
pub fn build_volume_texture(volume: &VolumeData) -> VolumeTextureBuildResult {
	let downsample_factors = choose_downsample_factors(volume.dims);
	let texture_dims = [
		volume.dims[0].div_ceil(downsample_factors[0]),
		volume.dims[1].div_ceil(downsample_factors[1]),
		volume.dims[2].div_ceil(downsample_factors[2]),
	];
	let [min_value, max_value] = volume.value_range;
	let range = (max_value - min_value).max(f32::EPSILON);
	let mut data = Vec::with_capacity(texture_dims[0] * texture_dims[1] * texture_dims[2]);
	for z in 0..texture_dims[2] {
		let source_z = (z * downsample_factors[2]).min(volume.dims[2] - 1);
		for y in 0..texture_dims[1] {
			let source_y = (y * downsample_factors[1]).min(volume.dims[1] - 1);
			for x in 0..texture_dims[0] {
				let source_x = (x * downsample_factors[0]).min(volume.dims[0] - 1);
				let source_index = source_z * volume.dims[0] * volume.dims[1]
					+ source_y * volume.dims[0]
					+ source_x;
				let normalized =
					((volume.voxels[source_index] - min_value) / range).clamp(0.0, 1.0);
				data.push((normalized * 255.0).round() as u8);
			}
		}
	}

	let mut image = Image::new_fill(
		Extent3d {
			width: texture_dims[0] as u32,
			height: texture_dims[1] as u32,
			depth_or_array_layers: texture_dims[2] as u32,
		},
		TextureDimension::D3,
		&data,
		TextureFormat::R8Unorm,
		bevy::asset::RenderAssetUsages::MAIN_WORLD | bevy::asset::RenderAssetUsages::RENDER_WORLD,
	);
	image.sampler = ImageSampler::linear();
	VolumeTextureBuildResult {
		image,
		info: VolumeTextureBuildInfo {
			texture_dims,
			downsample_factors,
		},
	}
}

/// 将体数据窗口参数转换为材质 uniform
pub fn build_render_params(
	volume: &VolumeData,
	window_center: f32,
	window_width: f32,
	step_size: f32,
) -> Vec4 {
	let [min_value, max_value] = volume.value_range;
	let range = (max_value - min_value).max(f32::EPSILON);
	let window_low = ((window_center - window_width * 0.5) - min_value) / range;
	let window_high = ((window_center + window_width * 0.5) - min_value) / range;
	Vec4::new(
		window_low,
		window_high.max(window_low + 0.0001),
		step_size,
		0.0,
	)
}

/// 为体纹理选择各轴降采样步长，避免直接上传过大的 3D 纹理。
fn choose_downsample_factors(dims: [usize; 3]) -> [usize; 3] {
	let mut factors = [1, 1, 1];
	let mut sampled_dims = dims;

	while sampled_dims.iter().copied().max().unwrap_or(1) > MAX_VOLUME_TEXTURE_DIMENSION
		|| sampled_dims.iter().product::<usize>() > MAX_VOLUME_TEXTURE_VOXELS
	{
		let axis = sampled_dims
			.iter()
			.enumerate()
			.max_by_key(|(_, value)| **value)
			.map(|(axis, _)| axis)
			.unwrap_or(0);
		factors[axis] *= 2;
		sampled_dims[axis] = dims[axis].div_ceil(factors[axis]);
	}

	factors
}

#[cfg(test)]
mod tests {
	use super::{build_volume_texture, choose_downsample_factors};
	use medical_image::{VolumeData, VolumeModality};

	fn sample_affine() -> [[f32; 4]; 4] {
		[
			[1.0, 0.0, 0.0, 0.0],
			[0.0, 1.0, 0.0, 0.0],
			[0.0, 0.0, 1.0, 0.0],
			[0.0, 0.0, 0.0, 1.0],
		]
	}

	#[test]
	fn should_choose_downsample_factors_for_large_volume() {
		assert_eq!(choose_downsample_factors([512, 199, 512]), [2, 1, 2]);
		assert_eq!(choose_downsample_factors([256, 256, 256]), [1, 1, 1]);
	}

	#[test]
	fn should_build_downsampled_volume_texture() {
		let dims = [512, 16, 8];
		let voxel_count = dims[0] * dims[1] * dims[2];
		let voxels = (0..voxel_count).map(|value| value as f32).collect();
		let volume = match VolumeData::new(
			dims,
			[1.0, 1.0, 1.0],
			[0.0, 0.0, 0.0],
			[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
			sample_affine(),
			voxels,
			VolumeModality::Mr,
		) {
			Ok(volume) => volume,
			Err(error) => panic!("sample volume should be valid: {error}"),
		};

		let result = build_volume_texture(&volume);
		assert_eq!(result.info.texture_dims, [256, 16, 8]);
		assert_eq!(result.info.downsample_factors, [2, 1, 1]);
		assert!(result.info.is_downsampled());
	}
}
