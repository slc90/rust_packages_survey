use bevy::prelude::*;

use crate::homepage::common::ContentAreaMarker;
use crate::homepage::realtime_plot::components::{
	ChannelSliderMarker, ControlPanelMarker, RealtimePlotContentMarker, SampleRateDropdownMarker,
	WaveformMeshMarker,
};
use crate::homepage::realtime_plot::resources::{WaveformData, WaveformGenerator};
use config::data_structure::Setting;

// ============================================================================
// REALTIME_PLOT CONSTANTS
// ============================================================================

/// 波形显示区域宽度
const WAVEFORM_WIDTH: f32 = 800.0;
/// 波形显示区域高度（预留）
#[allow(dead_code)]
const WAVEFORM_HEIGHT: f32 = 900.0;
/// 每个通道的高度
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
pub fn on_enter(
	mut commands: Commands,
	settings: Res<Setting>,
	content_area_query: Query<Entity, With<ContentAreaMarker>>,
) {
	info!("进入实时波形绘制页面");
	// 从全局配置读取波形设置
	if let Ok(content_area) = content_area_query.single() {
		commands
			.entity(content_area)
			.insert(BackgroundColor(Color::NONE));
	}
	let waveform_config = &settings.waveform;
	// 初始化波形数据资源
	let waveform_data =
		WaveformData::new(waveform_config.channel_count, waveform_config.buffer_size);
	commands.insert_resource(waveform_data);
	// 插入波形设置资源
	let waveform_settings = WaveformSettings {
		channel_count: waveform_config.channel_count,
		sample_rate: waveform_config.sample_rate,
		max_points: waveform_config.buffer_size,
	};
	commands.insert_resource(waveform_settings);
	// 初始化波形生成器和计时器
	commands.insert_resource(WaveformGeneratorState::default());
	commands.insert_resource(WaveformTimer::default());
}

/// 离开RealtimePlot页面时触发，清理资源
pub fn on_exit(
	mut commands: Commands,
	query: Query<Entity, With<RealtimePlotContentMarker>>,
	content_area_query: Query<Entity, With<ContentAreaMarker>>,
) {
	info!("离开实时波形绘制页面");
	// 移除波形数据资源
	for entity in &query {
		commands.entity(entity).despawn();
	}
	if let Ok(content_area) = content_area_query.single() {
		commands
			.entity(content_area)
			.insert(BackgroundColor(Color::WHITE));
	}
	commands.remove_resource::<WaveformData>();
	commands.remove_resource::<WaveformSettings>();
	commands.remove_resource::<WaveformGeneratorState>();
	commands.remove_resource::<WaveformTimer>();
	commands.remove_resource::<WaveformMeshes>();
	commands.remove_resource::<WaveformMaterials>();
}

// ============================================================================
// WAVEFORM RENDERING SYSTEMS
// ============================================================================

/// 通道颜色映射
pub fn get_channel_color(channel_index: usize) -> Color {
	let color = CHANNEL_COLORS[channel_index % CHANNEL_COLORS.len()];
	Color::Srgba(Srgba::new(color[0], color[1], color[2], color[3]))
}

fn channel_height(channel_count: usize) -> f32 {
	WAVEFORM_HEIGHT / channel_count.max(1) as f32
}

fn channel_center_y(channel_index: usize, channel_count: usize) -> f32 {
	let height = channel_height(channel_count);
	WAVEFORM_HEIGHT / 2.0 - ((channel_index as f32) + 0.5) * height
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
	channel_count: usize,
	point_count: usize,
) -> Vec<Vec2> {
	// 如果没有数据或点数为0，返回一个默认点
	if channel_data.is_empty() || point_count == 0 {
		// 将波形显示在通道中心偏上位置，避免与x轴重叠
		let y_offset = channel_center_y(channel_index, channel_count);
		return vec![Vec2::new(0.0, y_offset)];
	}

	// 将波形显示在通道中心偏上位置，避免与x轴重叠
	let y_offset = channel_center_y(channel_index, channel_count);
	let current_channel_height = channel_height(channel_count);
	let step = if point_count <= 1 {
		0.0
	} else {
		WAVEFORM_WIDTH / (point_count - 1) as f32
	};

	channel_data
		.iter()
		.enumerate()
		.map(|(i, &value)| {
			let x = (i as f32 * step) - WAVEFORM_WIDTH / 2.0;
			// 将值归一化到 [-1, 1] 范围，然后映射到通道高度
			// 使用更大的振幅(通道高度的45%)使波形更明显
			let y = (value / 100.0).clamp(-1.0, 1.0) * (current_channel_height * 0.4) + y_offset;
			Vec2::new(x, y)
		})
		.collect()
}

/// 波形颜色材质资源标签
#[derive(Resource, Default)]
#[allow(dead_code)]
pub struct WaveformMaterials {
	materials: Vec<Handle<ColorMaterial>>,
}

/// 波形网格资源标签
#[derive(Resource, Default)]
#[allow(dead_code)]
pub struct WaveformMeshes {
	handles: Vec<Handle<Mesh>>,
}

/// 初始化波形渲染资源
pub fn init_waveform_rendering(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<ColorMaterial>>,
	settings: Res<Setting>,
) {
	info!("初始化波形渲染资源");

	// 创建材质
	let channel_count = settings.waveform.channel_count.max(1);
	let mat_handles: Vec<_> = (0..channel_count)
		.map(|i| materials.add(get_channel_color(i)))
		.collect();

	// 创建初始网格（使用位于通道中心的水平线，与x轴有一定偏移确保可见）
	// 注意：x轴在 y=50，而波形显示在 y=100 附近，避免重叠
	let default_y = channel_center_y(0, channel_count);
	let default_points = vec![Vec2::new(-400.0, default_y), Vec2::new(400.0, default_y)];
	let mesh_handles: Vec<_> = (0..channel_count)
		.map(|_| meshes.add(Mesh::from(Polyline2d::new(default_points.clone()))))
		.collect();

	// 创建渲染波形用的实体
	for (i, (mesh_handle, mat_handle)) in mesh_handles.iter().zip(mat_handles.iter()).enumerate() {
		commands.spawn((
			Mesh2d(mesh_handle.clone()),
			MeshMaterial2d(mat_handle.clone()),
			Transform::from_xyz(0.0, 0.0, 0.0),
			RealtimePlotContentMarker,
			WaveformMeshMarker,
		));
		info!("Created waveform entity {}", i);
	}

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
	_materials: ResMut<Assets<ColorMaterial>>,
	waveform_data: Option<Res<WaveformData>>,
	mut query: Query<(Entity, &mut Mesh2d), With<WaveformMeshMarker>>,
) {
	// 如果资源不可用，跳过
	let Some(waveform_data) = waveform_data else {
		return;
	};

	// 暂时移除is_changed()检查，强制每帧更新以便调试
	// if !waveform_data.is_changed() {
	//     return;
	// }

	let channels = waveform_data.get_all_channels();
	let channel_count = channels.len().max(1);
	let point_count = channels.first().map(|c| c.len()).unwrap_or(0);

	// 获取所有波形实体并更新其网格
	let mut query_iter = query.iter_mut();

	for (i, channel_data) in channels.iter().enumerate() {
		// 获取或跳过实体
		let (_entity, mut mesh2d) = match query_iter.next() {
			Some((entity, mesh2d)) => (entity, mesh2d),
			None => continue,
		};

		// 生成波形点
		let points = generate_waveform_points(channel_data, i, channel_count, point_count.max(1));
		if i == 0 && !points.is_empty() {
			debug!(
				"Channel 0: {} points, first={:?}, last={:?}",
				points.len(),
				points.first(),
				points.last()
			);
		}

		let new_mesh = Mesh::from(Polyline2d::new(points));
		let new_mesh_handle = meshes.add(new_mesh);

		// 更新实体的网格
		*mesh2d = Mesh2d(new_mesh_handle);
	}
}

/// 清理波形渲染资源
pub fn cleanup_waveform_rendering(
	mut commands: Commands,
	query: Query<Entity, With<WaveformMeshMarker>>,
) {
	info!("清理波形渲染资源");

	// 删除所有波形实体
	for entity in &query {
		commands.entity(entity).despawn();
	}

	commands.remove_resource::<WaveformMeshes>();
	commands.remove_resource::<WaveformMaterials>();
}

// ============================================================================
// WAVEFORM AXIS AND GRID
// ============================================================================

/// 坐标轴颜色
#[allow(dead_code)]
const AXIS_COLOR: [f32; 4] = [0.5, 0.5, 0.5, 1.0];
/// 网格颜色
#[allow(dead_code)]
const GRID_COLOR: [f32; 4] = [0.3, 0.3, 0.3, 1.0];

/// 初始化坐标轴和网格
pub fn spawn_axis_grid(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<ColorMaterial>>,
	settings: Res<Setting>,
) {
	info!("Spawning axis and grid");

	// 获取设置值（使用默认值兜底）
	let channel_count = settings.waveform.channel_count.max(1);
	let channel_height = channel_height(channel_count);
	let total_height = WAVEFORM_HEIGHT;
	let width = WAVEFORM_WIDTH;

	// 创建坐标轴材质
	let axis_mat = materials.add(ColorMaterial::from(Color::srgb(0.5, 0.5, 0.5)));
	let grid_mat = materials.add(ColorMaterial::from(Color::srgb(0.3, 0.3, 0.3)));

	// X轴（水平中心线）- 为每个通道创建一条中心线
	for ch in 0..channel_count {
		let y_offset = channel_center_y(ch, channel_count);
		// 确保至少有2个点来绘制线条
		let x_axis_points = if width > 0.0 {
			vec![
				Vec2::new(-width / 2.0, y_offset),
				Vec2::new(width / 2.0, y_offset),
			]
		} else {
			vec![Vec2::new(-100.0, y_offset), Vec2::new(100.0, y_offset)]
		};
		let x_axis_mesh = meshes.add(Polyline2d::new(x_axis_points));

		commands.spawn((
			Mesh2d(x_axis_mesh),
			MeshMaterial2d(axis_mat.clone()),
			Transform::from_xyz(0.0, 0.0, -0.1),
			RealtimePlotContentMarker,
		));
	}

	// Y轴（垂直中心线）
	let y_axis_points = vec![Vec2::new(-width / 2.0, 0.0), Vec2::new(width / 2.0, 0.0)];
	let y_axis_mesh = meshes.add(Polyline2d::new(y_axis_points));

	commands.spawn((
		Mesh2d(y_axis_mesh),
		MeshMaterial2d(axis_mat.clone()),
		Transform::from_xyz(0.0, 0.0, -0.1),
		RealtimePlotContentMarker,
	));

	// 垂直网格线
	let grid_spacing = width / 10.0;
	if grid_spacing > 0.0 {
		for i in 0..=10 {
			let x = -width / 2.0 + i as f32 * grid_spacing;
			let grid_points = vec![
				Vec2::new(x, -total_height / 2.0),
				Vec2::new(x, total_height / 2.0),
			];
			let grid_mesh = meshes.add(Polyline2d::new(grid_points));

			commands.spawn((
				Mesh2d(grid_mesh),
				MeshMaterial2d(grid_mat.clone()),
				Transform::from_xyz(0.0, 0.0, -0.2),
				RealtimePlotContentMarker,
			));
		}
	}

	// 水平网格线
	for ch in 0..=channel_count {
		let y = (total_height / 2.0) - (ch as f32 * channel_height);
		let grid_points = vec![Vec2::new(-width / 2.0, y), Vec2::new(width / 2.0, y)];
		let grid_mesh = meshes.add(Polyline2d::new(grid_points));

		commands.spawn((
			Mesh2d(grid_mesh),
			MeshMaterial2d(grid_mat.clone()),
			Transform::from_xyz(0.0, 0.0, -0.2),
			RealtimePlotContentMarker,
		));
	}

	info!("Axis and grid spawned successfully");
}

// ============================================================================
// UI INTERACTION SYSTEMS
// ============================================================================

/// 处理通道数滑动条点击
pub fn handle_channel_slider_click(
	settings: Option<ResMut<WaveformSettings>>,
	interaction_query: Query<&Interaction, (Changed<Interaction>, With<ChannelSliderMarker>)>,
) {
	// 如果 WaveformSettings 不可用，跳过
	let Some(mut settings) = settings else {
		return;
	};

	for interaction in &interaction_query {
		if matches!(interaction, Interaction::Pressed) {
			// 点击时增加通道数（循环）
			settings.channel_count = if settings.channel_count >= CHANNEL_COUNT_MAX {
				CHANNEL_COUNT_MIN
			} else {
				settings.channel_count + 1
			};
			info!("Channel count changed to {}", settings.channel_count);
		}
	}
}

/// 处理采样率下拉框点击
pub fn handle_sample_rate_click(
	settings: Option<ResMut<WaveformSettings>>,
	interaction_query: Query<&Interaction, (Changed<Interaction>, With<SampleRateDropdownMarker>)>,
) {
	// 如果 WaveformSettings 不可用，跳过
	let Some(mut settings) = settings else {
		return;
	};

	for interaction in &interaction_query {
		if matches!(interaction, Interaction::Pressed) {
			// 点击时切换到下一个采样率选项（循环）
			let current_idx = SAMPLE_RATE_OPTIONS
				.iter()
				.position(|&r| r == settings.sample_rate)
				.unwrap_or(0);
			let next_idx = (current_idx + 1) % SAMPLE_RATE_OPTIONS.len();
			settings.sample_rate = SAMPLE_RATE_OPTIONS[next_idx];
			info!("Sample rate changed to {} Hz", settings.sample_rate);
		}
	}
}

// ============================================================================
// WAVEFORM SETTINGS - UI settings for waveform display
// ============================================================================

/// 波形设置资源
#[derive(Resource, Debug, Clone)]
pub struct WaveformSettings {
	/// 通道数量
	pub channel_count: usize,
	/// 采样率 (Hz)
	pub sample_rate: u32,
	/// 最大显示点数
	pub max_points: usize,
}

impl Default for WaveformSettings {
	fn default() -> Self {
		Self {
			channel_count: 1,
			sample_rate: 1000,
			max_points: 4096,
		}
	}
}

/// 可用的采样率选项
pub const SAMPLE_RATE_OPTIONS: [u32; 4] = [500, 1000, 2000, 4000];

/// 通道数量范围
pub const CHANNEL_COUNT_MIN: usize = 1;
pub const CHANNEL_COUNT_MAX: usize = 32;

/// 初始化波形设置UI
pub fn spawn_waveform_settings_ui(mut commands: Commands) {
	info!("Spawning waveform settings UI");

	// 创建控制面板
	commands
		.spawn((
			Node {
				width: Val::Px(200.0),
				height: Val::Percent(100.0),
				flex_direction: FlexDirection::Column,
				padding: UiRect::all(Val::Px(10.0)),
				..Default::default()
			},
			BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
			RealtimePlotContentMarker,
			ControlPanelMarker,
		))
		.with_children(|parent| {
			// 通道数标签
			parent.spawn((
				Text::new("Channel Count"),
				TextFont {
					font_size: 14.0,
					..Default::default()
				},
				TextColor(Color::WHITE),
			));

			// 通道数滑动条（使用按钮模拟滑块行为）
			parent.spawn((
				Button,
				ChannelSliderMarker,
				Node {
					width: Val::Px(180.0),
					height: Val::Px(30.0),
					margin: UiRect::all(Val::Px(5.0)),
					..Default::default()
				},
			));

			// 采样率标签
			parent.spawn((
				Text::new("Sample Rate"),
				TextFont {
					font_size: 14.0,
					..Default::default()
				},
				TextColor(Color::WHITE),
			));

			// 采样率下拉框（使用按钮模拟）
			parent.spawn((
				Button,
				SampleRateDropdownMarker,
				Node {
					width: Val::Px(180.0),
					height: Val::Px(30.0),
					margin: UiRect::all(Val::Px(5.0)),
					..Default::default()
				},
			));
		});
}

/// 更新波形设置
pub fn update_waveform_settings(
	settings: Option<ResMut<WaveformSettings>>,
	waveform_data: Option<ResMut<WaveformData>>,
) {
	// 如果资源不可用，跳过
	let (Some(settings), Some(mut waveform_data)) = (settings, waveform_data) else {
		return;
	};

	// 如果通道数变化，更新波形数据
	if settings.channel_count != waveform_data.channel_count() {
		waveform_data.clear();
		// 重新创建通道数据
		waveform_data
			.channels
			.resize(settings.channel_count, Vec::new());
	}
}

// ============================================================================
// WAVEFORM DATA GENERATION SYSTEMS
// ============================================================================

/// 波形生成器资源，用于存储生成器状态
#[derive(Resource, Debug, Default)]
pub struct WaveformGeneratorState {
	/// 波形生成器实例
	pub generator: WaveformGenerator,
}

/// 波形生成计时器
#[derive(Resource, Debug)]
pub struct WaveformTimer {
	/// 距离下次生成的剩余时间（秒）
	pub remaining: f32,
	/// 生成间隔（秒）
	pub interval: f32,
}

impl Default for WaveformTimer {
	fn default() -> Self {
		Self::new(1000) // 默认 1000 Hz
	}
}

impl WaveformTimer {
	/// 根据采样率创建新的计时器
	///
	/// # Arguments
	/// * `sample_rate` - 采样率 (Hz)
	pub fn new(sample_rate: u32) -> Self {
		// 每秒生成 sample_rate 个点
		let interval = 1.0 / sample_rate as f32;
		Self {
			remaining: 0.0,
			interval,
		}
	}

	/// 重置计时器
	pub fn reset(&mut self) {
		self.remaining = 0.0;
	}

	/// 更新计时器
	///
	/// # Arguments
	/// * `dt` - 距上次更新经过的时间（秒）
	///
	/// # Returns
	/// 如果应该生成数据，返回 true
	pub fn update(&mut self, dt: f32) -> bool {
		self.remaining += dt;
		if self.remaining >= self.interval {
			self.remaining -= self.interval;
			true
		} else {
			false
		}
	}

	/// 更新采样率
	///
	/// # Arguments
	/// * `sample_rate` - 新的采样率 (Hz)
	pub fn set_sample_rate(&mut self, sample_rate: u32) {
		self.interval = 1.0 / sample_rate as f32;
	}
}

/// 初始化波形生成器（已废弃，资源初始化移至 on_enter）
/// @deprecated 资源现在在 on_enter 中初始化
pub fn init_waveform_generator() {
	debug!("初始化波形生成器 (废弃)");
}

/// 生成波形数据
///
/// 该系统根据采样率定时生成波形数据并添加到 WaveformData 中
#[allow(clippy::too_many_arguments)]
pub fn generate_waveform_data(
	waveform_data: Option<ResMut<WaveformData>>,
	generator_state: Option<ResMut<WaveformGeneratorState>>,
	timer: Option<ResMut<WaveformTimer>>,
	settings: Option<Res<WaveformSettings>>,
	time: Option<Res<Time>>,
) {
	let (
		Some(mut waveform_data),
		Some(mut generator_state),
		Some(mut timer),
		Some(settings),
		Some(time),
	) = (waveform_data, generator_state, timer, settings, time)
	else {
		return;
	};

	// 更新计时器的采样率
	timer.set_sample_rate(settings.sample_rate);

	// 更新生成器的采样率
	generator_state.generator.sample_rate = settings.sample_rate;

	// 检查是否应该生成数据
	let dt = time.delta_secs();
	let should_generate = timer.update(dt);
	if should_generate {
		// 为每个通道生成一个数据点
		for ch in 0..settings.channel_count {
			let value = generator_state.generator.generate_single();
			waveform_data.push(ch, value);
		}
		// 每秒打印一次数据状态
		if timer.remaining < 0.001 {
			let channels = waveform_data.get_all_channels();
			let point_count = channels.first().map(|c| c.len()).unwrap_or(0);
			if let Some(ch) = channels.first()
				&& let Some(&last_val) = ch.last()
			{
				info!(
					"Waveform: {} points, last={:.2}, generator_time={:.4}",
					point_count, last_val, generator_state.generator.time
				);
			}
		}
	}
}
