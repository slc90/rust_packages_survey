use bevy::prelude::*;

use crate::homepage::realtime_plot::components::{
	ChannelSliderMarker, ControlPanelMarker, SampleRateDropdownMarker,
};
use crate::homepage::realtime_plot::resources::{WaveformData, WaveformGenerator};

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

// ============================================================================
// WAVEFORM AXIS AND GRID
// ============================================================================

/// 坐标轴颜色
const AXIS_COLOR: [f32; 4] = [0.5, 0.5, 0.5, 1.0];
/// 网格颜色
const GRID_COLOR: [f32; 4] = [0.3, 0.3, 0.3, 1.0];

/// 初始化坐标轴和网格
pub fn spawn_axis_grid(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<ColorMaterial>>,
	settings: Res<WaveformSettings>,
) {
	info!("Spawning axis and grid");

	let channel_height = CHANNEL_HEIGHT;
	let total_height = channel_height * settings.channel_count as f32;
	let width = WAVEFORM_WIDTH;

	// X轴（水平中心线）
	for ch in 0..settings.channel_count {
		let y_offset = (total_height / 2.0) - (ch as f32 * channel_height);
		let x_axis_points = vec![
			Vec2::new(-width / 2.0, y_offset),
			Vec2::new(width / 2.0, y_offset),
		];
		let x_axis_mesh = meshes.add(Polyline2d::new(x_axis_points));
		let axis_mat = materials.add(ColorMaterial::from(Color::Srgba(Srgba::new(
			AXIS_COLOR[0],
			AXIS_COLOR[1],
			AXIS_COLOR[2],
			AXIS_COLOR[3],
		))));

		commands.spawn((
			Mesh2d(x_axis_mesh),
			MeshMaterial2d(axis_mat),
			Transform::from_xyz(0.0, 0.0, -0.1),
		));
	}

	// Y轴（垂直中心线）
	let y_axis_points = vec![Vec2::new(-width / 2.0, 0.0), Vec2::new(width / 2.0, 0.0)];
	let y_axis_mesh = meshes.add(Polyline2d::new(y_axis_points));
	let y_axis_mat = materials.add(ColorMaterial::from(Color::Srgba(Srgba::new(
		AXIS_COLOR[0],
		AXIS_COLOR[1],
		AXIS_COLOR[2],
		AXIS_COLOR[3],
	))));

	commands.spawn((
		Mesh2d(y_axis_mesh),
		MeshMaterial2d(y_axis_mat),
		Transform::from_xyz(0.0, 0.0, -0.1),
	));

	// 垂直网格线
	let grid_spacing = width / 10.0;
	for i in 0..=10 {
		let x = -width / 2.0 + i as f32 * grid_spacing;
		let grid_points = vec![
			Vec2::new(x, -total_height / 2.0),
			Vec2::new(x, total_height / 2.0),
		];
		let grid_mesh = meshes.add(Polyline2d::new(grid_points));
		let grid_mat = materials.add(ColorMaterial::from(Color::Srgba(Srgba::new(
			GRID_COLOR[0],
			GRID_COLOR[1],
			GRID_COLOR[2],
			GRID_COLOR[3],
		))));

		commands.spawn((
			Mesh2d(grid_mesh),
			MeshMaterial2d(grid_mat),
			Transform::from_xyz(0.0, 0.0, -0.2),
		));
	}

	// 水平网格线
	for ch in 0..=settings.channel_count {
		let y = (total_height / 2.0) - (ch as f32 * channel_height);
		let grid_points = vec![Vec2::new(-width / 2.0, y), Vec2::new(width / 2.0, y)];
		let grid_mesh = meshes.add(Polyline2d::new(grid_points));
		let grid_mat = materials.add(ColorMaterial::from(Color::Srgba(Srgba::new(
			GRID_COLOR[0],
			GRID_COLOR[1],
			GRID_COLOR[2],
			GRID_COLOR[3],
		))));

		commands.spawn((
			Mesh2d(grid_mesh),
			MeshMaterial2d(grid_mat),
			Transform::from_xyz(0.0, 0.0, -0.2),
		));
	}
}

// ============================================================================
// UI INTERACTION SYSTEMS
// ============================================================================

/// 处理通道数滑动条点击
pub fn handle_channel_slider_click(
	mut settings: ResMut<WaveformSettings>,
	interaction_query: Query<&Interaction, (Changed<Interaction>, With<ChannelSliderMarker>)>,
) {
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
	mut settings: ResMut<WaveformSettings>,
	interaction_query: Query<&Interaction, (Changed<Interaction>, With<SampleRateDropdownMarker>)>,
) {
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
	settings: ResMut<WaveformSettings>,
	mut waveform_data: ResMut<WaveformData>,
) {
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

/// 初始化波形生成器
pub fn init_waveform_generator(mut commands: Commands) {
	info!("初始化波形生成器");
	commands.insert_resource(WaveformGeneratorState::default());
	commands.insert_resource(WaveformTimer::default());
}

/// 生成波形数据
///
/// 该系统根据采样率定时生成波形数据并添加到 WaveformData 中
#[allow(clippy::too_many_arguments)]
pub fn generate_waveform_data(
	mut waveform_data: ResMut<WaveformData>,
	mut generator_state: ResMut<WaveformGeneratorState>,
	mut timer: ResMut<WaveformTimer>,
	settings: Res<WaveformSettings>,
	time: Res<Time>,
) {
	// 更新计时器的采样率
	timer.set_sample_rate(settings.sample_rate);

	// 更新生成器的采样率
	generator_state.generator.sample_rate = settings.sample_rate;

	// 检查是否应该生成数据
	if timer.update(time.delta_secs()) {
		// 为每个通道生成一个数据点
		for ch in 0..settings.channel_count {
			let value = generator_state.generator.generate_single();
			waveform_data.push(ch, value);
		}
	}
}
