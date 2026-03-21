use bevy::image::ImageSampler;
use bevy::pbr::{Material, MaterialPlugin};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{AsBindGroup, Extent3d, TextureDimension, TextureFormat};
use bevy::shader::ShaderRef;
use medical_image::VolumeData;

const VOLUME_SHADER_ASSET_PATH: &str = "shaders/medical_volume.wgsl";

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

/// 构建归一化后的 3D 纹理
pub fn build_volume_texture(volume: &VolumeData) -> Image {
	let [min_value, max_value] = volume.value_range;
	let range = (max_value - min_value).max(f32::EPSILON);
	let mut data = Vec::with_capacity(volume.voxels.len());
	for value in &volume.voxels {
		let normalized = ((*value - min_value) / range).clamp(0.0, 1.0);
		data.push((normalized * 255.0).round() as u8);
	}

	let mut image = Image::new_fill(
		Extent3d {
			width: volume.dims[0] as u32,
			height: volume.dims[1] as u32,
			depth_or_array_layers: volume.dims[2] as u32,
		},
		TextureDimension::D3,
		&data,
		TextureFormat::R8Unorm,
		bevy::asset::RenderAssetUsages::MAIN_WORLD | bevy::asset::RenderAssetUsages::RENDER_WORLD,
	);
	image.sampler = ImageSampler::linear();
	image
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
