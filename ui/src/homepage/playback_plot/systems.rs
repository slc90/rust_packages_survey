//! 回放波形系统
//!
//! 实现回放波形的生命周期、数据加载、渲染和播放控制

use bevy::prelude::*;
use edf_io::EdfLoader;

use crate::homepage::common::ContentAreaMarker;
use crate::homepage::playback_plot::components::{
	FilePathDisplayMarker, NextPageButtonMarker, PageDisplayMarker, PlayButtonMarker,
	PlayButtonTextMarker, PlaybackControlPanelMarker, PlaybackPlotContentMarker,
	PlaybackWaveformMeshMarker, PositionDisplayMarker, PrevPageButtonMarker, SpeedButtonMarker,
	SpeedButtonTextMarker,
};
use crate::homepage::playback_plot::resources::{
	PlaybackControl, PlaybackData, PlaybackSpeed, PlaybackStatus,
};

const WAVEFORM_WIDTH: f32 = 800.0;
const WAVEFORM_HEIGHT: f32 = 900.0;
const AXIS_COLOR: [f32; 4] = [0.45, 0.45, 0.45, 1.0];
const GRID_COLOR: [f32; 4] = [0.25, 0.25, 0.25, 1.0];

const CHANNEL_COLORS: [[f32; 4]; 8] = [
	[0.2, 0.6, 0.9, 1.0],
	[0.9, 0.3, 0.3, 1.0],
	[0.2, 0.8, 0.4, 1.0],
	[0.9, 0.8, 0.2, 1.0],
	[0.6, 0.3, 0.9, 1.0],
	[0.3, 0.9, 0.9, 1.0],
	[0.9, 0.5, 0.7, 1.0],
	[0.5, 0.5, 0.5, 1.0],
];

#[derive(Resource, Debug)]
pub struct PlaybackTimer {
	remaining: f32,
	interval: f32,
}

impl Default for PlaybackTimer {
	fn default() -> Self {
		Self::new(1000)
	}
}

impl PlaybackTimer {
	pub fn new(sample_rate: u32) -> Self {
		Self {
			remaining: 0.0,
			interval: 1.0 / sample_rate.max(1) as f32,
		}
	}

	pub fn update(&mut self, dt: f32) -> bool {
		self.remaining += dt;
		if self.remaining >= self.interval {
			self.remaining -= self.interval;
			true
		} else {
			false
		}
	}

	pub fn set_sample_rate(&mut self, sample_rate: u32) {
		self.interval = 1.0 / sample_rate.max(1) as f32;
	}
}

pub fn on_enter(
	mut commands: Commands,
	content_area_query: Query<Entity, With<ContentAreaMarker>>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<ColorMaterial>>,
) {
	info!("进入回放波形页面");

	if let Ok(content_area) = content_area_query.single() {
		commands
			.entity(content_area)
			.insert(BackgroundColor(Color::NONE));
	}

	let file_path = default_playback_file_path();
	let playback_data = match EdfLoader::from_file(&file_path) {
		Ok(loader) => PlaybackData {
			file_path: loader.path().to_string(),
			channels: loader.channels().to_vec(),
			channel_count: loader.channel_count(),
			sample_rate: loader.sample_rate(),
			total_points: loader.total_points(),
		},
		Err(err) => {
			error!("加载回放文件失败: {}", err);
			PlaybackData::default()
		}
	};

	let mut control = PlaybackControl::new(4096);
	if playback_data.total_points > 0 {
		control.total_pages = playback_data.total_points.div_ceil(control.page_size);
	}

	let timer = PlaybackTimer::new(playback_data.sample_rate.max(1));
	let channel_count = playback_data.channel_count.max(1);

	commands.insert_resource(playback_data);
	commands.insert_resource(control);
	commands.insert_resource(PlaybackSpeed::default());
	commands.insert_resource(timer);

	init_waveform_rendering(&mut commands, &mut meshes, &mut materials, channel_count);
	spawn_axis_grid(&mut commands, &mut meshes, &mut materials, channel_count);
}

pub fn on_exit(
	mut commands: Commands,
	query: Query<Entity, With<PlaybackPlotContentMarker>>,
	content_area_query: Query<Entity, With<ContentAreaMarker>>,
) {
	info!("离开回放波形页面");

	for entity in &query {
		commands.entity(entity).despawn();
	}

	if let Ok(content_area) = content_area_query.single() {
		commands
			.entity(content_area)
			.insert(BackgroundColor(Color::WHITE));
	}

	commands.remove_resource::<PlaybackData>();
	commands.remove_resource::<PlaybackControl>();
	commands.remove_resource::<PlaybackSpeed>();
	commands.remove_resource::<PlaybackTimer>();
}

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

fn default_playback_file_path() -> String {
	let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	let base_dir = manifest_dir
		.parent()
		.map(std::path::Path::to_path_buf)
		.unwrap_or(manifest_dir);
	base_dir
		.join("data")
		.join("test_64ch_4000hz_10min.edf")
		.to_string_lossy()
		.to_string()
}

fn init_waveform_rendering(
	commands: &mut Commands,
	meshes: &mut ResMut<Assets<Mesh>>,
	materials: &mut ResMut<Assets<ColorMaterial>>,
	channel_count: usize,
) {
	let mat_handles: Vec<_> = (0..channel_count)
		.map(|i| materials.add(get_channel_color(i)))
		.collect();

	let mesh_handles: Vec<_> = (0..channel_count)
		.map(|i| {
			let y = channel_center_y(i, channel_count);
			let points = vec![Vec2::new(-400.0, y), Vec2::new(400.0, y)];
			meshes.add(Mesh::from(Polyline2d::new(points)))
		})
		.collect();

	for (mesh_handle, mat_handle) in mesh_handles.iter().zip(mat_handles.iter()) {
		commands.spawn((
			Mesh2d(mesh_handle.clone()),
			MeshMaterial2d(mat_handle.clone()),
			Transform::from_xyz(0.0, 0.0, 0.0),
			PlaybackPlotContentMarker,
			PlaybackWaveformMeshMarker,
		));
	}
}

fn spawn_axis_grid(
	commands: &mut Commands,
	meshes: &mut ResMut<Assets<Mesh>>,
	materials: &mut ResMut<Assets<ColorMaterial>>,
	channel_count: usize,
) {
	let axis_mat = materials.add(ColorMaterial::from(Color::srgba(
		AXIS_COLOR[0],
		AXIS_COLOR[1],
		AXIS_COLOR[2],
		AXIS_COLOR[3],
	)));
	let grid_mat = materials.add(ColorMaterial::from(Color::srgba(
		GRID_COLOR[0],
		GRID_COLOR[1],
		GRID_COLOR[2],
		GRID_COLOR[3],
	)));

	for ch in 0..channel_count {
		let y = channel_center_y(ch, channel_count);
		let points = vec![
			Vec2::new(-WAVEFORM_WIDTH / 2.0, y),
			Vec2::new(WAVEFORM_WIDTH / 2.0, y),
		];
		commands.spawn((
			Mesh2d(meshes.add(Polyline2d::new(points))),
			MeshMaterial2d(axis_mat.clone()),
			Transform::from_xyz(0.0, 0.0, -0.1),
			PlaybackPlotContentMarker,
		));
	}

	let grid_spacing = WAVEFORM_WIDTH / 10.0;
	for i in 0..=10 {
		let x = -WAVEFORM_WIDTH / 2.0 + i as f32 * grid_spacing;
		let points = vec![
			Vec2::new(x, -WAVEFORM_HEIGHT / 2.0),
			Vec2::new(x, WAVEFORM_HEIGHT / 2.0),
		];
		commands.spawn((
			Mesh2d(meshes.add(Polyline2d::new(points))),
			MeshMaterial2d(grid_mat.clone()),
			Transform::from_xyz(0.0, 0.0, -0.2),
			PlaybackPlotContentMarker,
		));
	}

	for ch in 0..=channel_count {
		let y = WAVEFORM_HEIGHT / 2.0 - ch as f32 * channel_height(channel_count);
		let points = vec![
			Vec2::new(-WAVEFORM_WIDTH / 2.0, y),
			Vec2::new(WAVEFORM_WIDTH / 2.0, y),
		];
		commands.spawn((
			Mesh2d(meshes.add(Polyline2d::new(points))),
			MeshMaterial2d(grid_mat.clone()),
			Transform::from_xyz(0.0, 0.0, -0.2),
			PlaybackPlotContentMarker,
		));
	}
}

fn generate_waveform_points(
	channel_data: &[f32],
	channel_index: usize,
	channel_count: usize,
	window_start: usize,
	window_end: usize,
) -> Vec<Vec2> {
	let y_offset = channel_center_y(channel_index, channel_count);
	let current_channel_height = channel_height(channel_count);
	let page =
		&channel_data[window_start.min(channel_data.len())..window_end.min(channel_data.len())];

	if page.is_empty() {
		return vec![Vec2::new(0.0, y_offset)];
	}

	let step = if page.len() <= 1 {
		0.0
	} else {
		WAVEFORM_WIDTH / (page.len() - 1) as f32
	};

	page.iter()
		.enumerate()
		.map(|(i, &value)| {
			let x = (i as f32 * step) - WAVEFORM_WIDTH / 2.0;
			let y = (value / 100.0).clamp(-1.0, 1.0) * (current_channel_height * 0.4) + y_offset;
			Vec2::new(x, y)
		})
		.collect()
}

fn visible_window(control: &PlaybackControl, total_points: usize) -> (usize, usize) {
	if total_points == 0 {
		return (0, 0);
	}

	let window_size = control.page_size.min(total_points).max(1);
	let half_window = window_size / 2;

	let mut start = control.position.saturating_sub(half_window);
	if start + window_size > total_points {
		start = total_points.saturating_sub(window_size);
	}

	(start, start + window_size)
}

pub fn update_playback(
	mut control: ResMut<PlaybackControl>,
	mut speed: ResMut<PlaybackSpeed>,
	data: Res<PlaybackData>,
	time: Res<Time>,
	mut timer: ResMut<PlaybackTimer>,
) {
	control.speed = speed.multiplier;

	if data.total_points == 0 || !control.is_playing() {
		return;
	}

	timer.set_sample_rate(data.sample_rate.max(1));

	if timer.update(time.delta_secs()) {
		control.position =
			(control.position + control.speed as usize).min(data.total_points.saturating_sub(1));
		control.current_page = control.position / control.page_size;

		if control.position + 1 >= data.total_points {
			control.position = 0;
			control.current_page = 0;
			speed.multiplier = 1.0;
			control.speed = 1.0;
			control.status = PlaybackStatus::Paused;
		}
	}
}

pub fn update_playback_waveform(
	mut meshes: ResMut<Assets<Mesh>>,
	data: Res<PlaybackData>,
	control: Res<PlaybackControl>,
	mut query: Query<&mut Mesh2d, With<PlaybackWaveformMeshMarker>>,
) {
	if data.total_points == 0 || (!data.is_changed() && !control.is_changed()) {
		return;
	}

	let (window_start, window_end) = visible_window(&control, data.total_points);
	let channel_count = data.channel_count.max(1);

	for (idx, mut mesh2d) in query.iter_mut().enumerate() {
		let Some(channel_data) = data.channels.get(idx) else {
			continue;
		};
		let points =
			generate_waveform_points(channel_data, idx, channel_count, window_start, window_end);
		*mesh2d = Mesh2d(meshes.add(Mesh::from(Polyline2d::new(points))));
	}
}

pub fn handle_play_pause(
	mut control: ResMut<PlaybackControl>,
	interaction_query: Query<&Interaction, (Changed<Interaction>, With<PlayButtonMarker>)>,
) {
	for interaction in &interaction_query {
		if matches!(interaction, Interaction::Pressed) {
			control.toggle();
		}
	}
}

pub fn handle_speed_change(
	mut speed: ResMut<PlaybackSpeed>,
	interaction_query: Query<&Interaction, (Changed<Interaction>, With<SpeedButtonMarker>)>,
) {
	for interaction in &interaction_query {
		if matches!(interaction, Interaction::Pressed) {
			speed.next_speed();
		}
	}
}

pub fn handle_prev_page(
	mut control: ResMut<PlaybackControl>,
	interaction_query: Query<&Interaction, (Changed<Interaction>, With<PrevPageButtonMarker>)>,
) {
	for interaction in &interaction_query {
		if matches!(interaction, Interaction::Pressed) && control.current_page > 0 {
			control.current_page -= 1;
			control.position = control.current_page * control.page_size;
		}
	}
}

pub fn handle_next_page(
	mut control: ResMut<PlaybackControl>,
	interaction_query: Query<&Interaction, (Changed<Interaction>, With<NextPageButtonMarker>)>,
	data: Res<PlaybackData>,
) {
	for interaction in &interaction_query {
		if matches!(interaction, Interaction::Pressed)
			&& control.current_page + 1 < control.total_pages
		{
			control.current_page += 1;
			control.position =
				(control.current_page * control.page_size).min(data.total_points.saturating_sub(1));
		}
	}
}

pub fn spawn_playback_control_ui(mut commands: Commands) {
	commands
		.spawn((
			Node {
				width: Val::Px(220.0),
				height: Val::Percent(100.0),
				flex_direction: FlexDirection::Column,
				padding: UiRect::all(Val::Px(10.0)),
				position_type: PositionType::Absolute,
				right: Val::Px(0.0),
				top: Val::Px(0.0),
				..Default::default()
			},
			BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.85)),
			PlaybackPlotContentMarker,
			PlaybackControlPanelMarker,
		))
		.with_children(|parent| {
			parent.spawn((
				Text::new("文件: -"),
				TextFont {
					font_size: 12.0,
					..Default::default()
				},
				TextColor(Color::WHITE),
				FilePathDisplayMarker,
			));

			parent
				.spawn((
					Button,
					PlayButtonMarker,
					Node {
						width: Val::Px(180.0),
						height: Val::Px(40.0),
						margin: UiRect::all(Val::Px(5.0)),
						..Default::default()
					},
				))
				.with_children(|button| {
					button.spawn((
						Text::new("播放"),
						TextFont {
							font_size: 16.0,
							..Default::default()
						},
						TextColor(Color::WHITE),
						PlayButtonTextMarker,
					));
				});

			parent
				.spawn((
					Button,
					SpeedButtonMarker,
					Node {
						width: Val::Px(180.0),
						height: Val::Px(30.0),
						margin: UiRect::all(Val::Px(5.0)),
						..Default::default()
					},
				))
				.with_children(|button| {
					button.spawn((
						Text::new("速度: 1x"),
						TextFont {
							font_size: 14.0,
							..Default::default()
						},
						TextColor(Color::WHITE),
						SpeedButtonTextMarker,
					));
				});

			parent.spawn((
				Text::new("位置: 0 / 0"),
				TextFont {
					font_size: 12.0,
					..Default::default()
				},
				TextColor(Color::WHITE),
				PositionDisplayMarker,
			));

			parent
				.spawn((
					Button,
					PrevPageButtonMarker,
					Node {
						width: Val::Px(180.0),
						height: Val::Px(30.0),
						margin: UiRect::all(Val::Px(5.0)),
						..Default::default()
					},
				))
				.with_children(|button| {
					button.spawn((
						Text::new("上一页"),
						TextFont {
							font_size: 14.0,
							..Default::default()
						},
						TextColor(Color::WHITE),
					));
				});

			parent
				.spawn((
					Button,
					NextPageButtonMarker,
					Node {
						width: Val::Px(180.0),
						height: Val::Px(30.0),
						margin: UiRect::all(Val::Px(5.0)),
						..Default::default()
					},
				))
				.with_children(|button| {
					button.spawn((
						Text::new("下一页"),
						TextFont {
							font_size: 14.0,
							..Default::default()
						},
						TextColor(Color::WHITE),
					));
				});

			parent.spawn((
				Text::new("页码: 0 / 0"),
				TextFont {
					font_size: 12.0,
					..Default::default()
				},
				TextColor(Color::WHITE),
				PageDisplayMarker,
			));
		});
}

pub fn update_playback_position_display(
	control: Res<PlaybackControl>,
	data: Res<PlaybackData>,
	speed: Res<PlaybackSpeed>,
	mut text_queries: ParamSet<(
		Query<&mut Text, With<PlayButtonTextMarker>>,
		Query<&mut Text, With<SpeedButtonTextMarker>>,
		Query<
			&mut Text,
			(
				With<FilePathDisplayMarker>,
				Without<PositionDisplayMarker>,
				Without<PageDisplayMarker>,
			),
		>,
		Query<
			&mut Text,
			(
				With<PositionDisplayMarker>,
				Without<FilePathDisplayMarker>,
				Without<PageDisplayMarker>,
			),
		>,
		Query<
			&mut Text,
			(
				With<PageDisplayMarker>,
				Without<FilePathDisplayMarker>,
				Without<PositionDisplayMarker>,
			),
		>,
	)>,
) {
	for mut text in &mut text_queries.p0() {
		text.0 = match control.status {
			PlaybackStatus::Playing => "暂停".to_string(),
			PlaybackStatus::Paused => "播放".to_string(),
		};
	}

	for mut text in &mut text_queries.p1() {
		text.0 = format!("速度: {}x", speed.multiplier);
	}

	for mut text in &mut text_queries.p2() {
		text.0 = format!("文件: {}", data.file_path);
	}

	for mut text in &mut text_queries.p3() {
		text.0 = format!("位置: {} / {}", control.position, data.total_points);
	}

	for mut text in &mut text_queries.p4() {
		text.0 = format!(
			"页码: {} / {} | 速度: {}x",
			control.current_page + 1,
			control.total_pages.max(1),
			speed.multiplier
		);
	}
}
