use bevy::prelude::*;

use crate::homepage::realtime_plot::resources::WaveformData;

// ============================================================================
// REALTIME_PLOT CONSTANTS
// ============================================================================

/// 波形显示区域宽度
const WAVEFORM_WIDTH: f32 = 800.0;
/// 波形显示区域高度（预留）
#[allow(dead_code)]
const WAVEFORM_HEIGHT: f32 = 400.0;
/// 每个通道的高度
const CHANNEL_HEIGHT: f32 = 100.0;
/// 通道颜色数组
const CHANNEL_COLORS: [[f32; 4]; 8] = [
	[0.2, 0.6, 0.9, 1.0], // 蓝色
	[0.9, 0.3, 0.3, 1.0], // 红色
	[0.2, 0.8, 0.4, 1.0], // 绿色
	[0.9, 0.8, 0.2, 1.0], // 黄色
	[0.6, 0.3, 0.9, 1.0], // 紫色
	[0.3, 0.9, 0.9, 1.0], // 青色
	[0.9, 0.5, 0.7, 1.0], // 粉色
	[0.5, 0.5, 0.5, 1.0], // 灰色
];

// ============================================================================
// REALTIME_PLOT STATE SYSTEMS - Lifecycle systems for RealtimePlot state
// ============================================================================

/// 进入RealtimePlot页面时触发，创建波形可视化资源
pub fn on_enter(mut commands: Commands) {
	info!("进入实时波形绘制页面");
	// 初始化波形数据资源，默认1通道，4096点
	let waveform_data = WaveformData::new(1, 4096);
	commands.insert_resource(waveform_data);
}

/// 离开RealtimePlot页面时触发，清理资源
pub fn on_exit(mut commands: Commands) {
	info!("离开实时波形绘制页面");
	// 移除波形数据资源
	commands.remove_resource::<WaveformData>();
}

// ============================================================================
// WAVEFORM RENDERING SYSTEMS
// ============================================================================

/// 通道颜色映射
pub fn get_channel_color(channel_index: usize) -> Color {
	let color = CHANNEL_COLORS[channel_index % CHANNEL_COLORS.len()];
	Color::Srgba(Srgba::new(color[0], color[1], color[2], color[3]))
}

/// 生成波形点数据
///
/// # Arguments
/// * `channel_data` - 通道数据
/// * `channel_index` - 通道索引
/// * `point_count` - 采样点数
///
/// # Returns
/// 波形点向量
fn generate_waveform_points(
	channel_data: &[f32],
	channel_index: usize,
	point_count: usize,
) -> Vec<Vec2> {
	let y_offset = (CHANNEL_HEIGHT / 2.0) - (channel_index as f32 * CHANNEL_HEIGHT);
	let step = (WAVEFORM_WIDTH / point_count as f32).max(1.0);

	channel_data
		.iter()
		.enumerate()
		.map(|(i, &value)| {
			let x = (i as f32 * step) - WAVEFORM_WIDTH / 2.0;
			// 将值归一化到 [-1, 1] 范围，然后映射到通道高度
			let y = (value / 100.0).clamp(-1.0, 1.0) * (CHANNEL_HEIGHT / 2.0 - 10.0) + y_offset;
			Vec2::new(x, y)
		})
		.collect()
}

/// 波形颜色材质资源标签
#[derive(Resource, Default)]
pub struct WaveformMaterials {
	materials: Vec<Handle<ColorMaterial>>,
}

/// 波形网格资源标签
#[derive(Resource, Default)]
pub struct WaveformMeshes {
	handles: Vec<Handle<Mesh>>,
}

/// 初始化波形渲染资源
pub fn init_waveform_rendering(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<ColorMaterial>>,
) {
	info!("初始化波形渲染资源");

	// 创建材质
	let mat_handles: Vec<_> = CHANNEL_COLORS
		.iter()
		.map(|c| {
			let color = Color::Srgba(Srgba::new(c[0], c[1], c[2], c[3]));
			materials.add(color)
		})
		.collect();

	// 创建初始网格（空数据）
	let mesh_handles: Vec<_> = (0..4)
		.map(|_| meshes.add(Polyline2d::new(vec![])))
		.collect();

	commands.insert_resource(WaveformMaterials {
		materials: mat_handles,
	});
	commands.insert_resource(WaveformMeshes {
		handles: mesh_handles,
	});
}

/// 更新波形显示
#[allow(clippy::too_many_arguments)]
pub fn update_waveform_display(
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<ColorMaterial>>,
	waveform_data: Res<WaveformData>,
	mut waveform_meshes: ResMut<WaveformMeshes>,
	mut waveform_materials: ResMut<WaveformMaterials>,
) {
	if !waveform_data.is_changed() {
		return;
	}

	let channels = waveform_data.get_all_channels();
	let point_count = channels.first().map(|c| c.len()).unwrap_or(0);

	// 确保有足够的网格和材质
	let current_material_count = waveform_materials.materials.len();
	for i in current_material_count..channels.len() {
		waveform_meshes
			.handles
			.push(meshes.add(Polyline2d::new(vec![])));
		let idx = i % CHANNEL_COLORS.len();
		let c = CHANNEL_COLORS[idx];
		waveform_materials
			.materials
			.push(materials.add(ColorMaterial::from(Color::Srgba(Srgba::new(
				c[0], c[1], c[2], c[3],
			)))));
	}

	// 更新每个通道的网格
	for (i, channel_data) in channels.iter().enumerate() {
		if i >= waveform_meshes.handles.len() {
			break;
		}

		let points = generate_waveform_points(channel_data, i, point_count.max(1));
		let new_mesh = meshes.add(Polyline2d::new(points));

		// 替换旧网格
		waveform_meshes.handles[i] = new_mesh;
	}
}

/// 清理波形渲染资源
pub fn cleanup_waveform_rendering(mut commands: Commands) {
	info!("清理波形渲染资源");
	commands.remove_resource::<WaveformMeshes>();
	commands.remove_resource::<WaveformMaterials>();
}
