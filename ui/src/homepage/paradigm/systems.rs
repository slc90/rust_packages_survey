use std::{
	io::{BufReader, Cursor},
	path::PathBuf,
};

use bevy::{
	asset::RenderAssetUsages,
	camera::RenderTarget,
	prelude::*,
	render::render_resource::{Extent3d, TextureDimension, TextureFormat},
	time::Fixed,
	window::{Monitor, MonitorSelection, PrimaryMonitor, WindowMode, WindowPosition, WindowRef},
};
use embedded_assets::PARADIGM_DEFAULT_GIF_BYTES;
use image::{AnimationDecoder, codecs::gif::GifDecoder};
use rand::{rng, seq::SliceRandom};

use crate::homepage::{
	common::ContentAreaMarker,
	paradigm::{
		components::{
			ParadigmContentMarker, ParadigmCycleMonitorButtonMarker,
			ParadigmCycleTargetButtonMarker, ParadigmGifInfoTextMarker, ParadigmGifPreviewMarker,
			ParadigmMonitorTextMarker, ParadigmPauseResumeButtonMarker,
			ParadigmPauseResumeTextMarker, ParadigmPresentationCellMarker,
			ParadigmPresentationRootMarker, ParadigmStartButtonMarker, ParadigmStatusTextMarker,
			ParadigmStopButtonMarker, ParadigmTargetTextMarker,
		},
		resources::{
			ParadigmGifFrame, ParadigmMonitorOption, ParadigmPageState, ParadigmPlaybackState,
			ParadigmPresentationWindowState, ParadigmStimulusTarget,
		},
	},
};

const GIF_FRAME_FALLBACK_SECONDS: f32 = 0.1;
const P300_GRID_SIZE: usize = 6;
const P300_FLASH_DURATION_MS: u32 = 100;
const P300_INTERVAL_DURATION_MS: u32 = 75;
const P300_BLOCK_COUNT: u32 = 10;

/// 进入范式页面。
pub fn on_enter(
	mut commands: Commands,
	content_area_query: Query<Entity, With<ContentAreaMarker>>,
	monitor_query: Query<(Entity, &Monitor, Option<&PrimaryMonitor>)>,
	mut images: ResMut<Assets<Image>>,
) {
	let mut page_state = ParadigmPageState {
		gif_preview: load_default_gif_preview(&mut images),
		monitor_options: collect_monitor_options(&monitor_query),
		selected_monitor_index: 0,
		target_symbol: 'A',
		status_text: "请选择显示器并开始范式播放".to_string(),
	};
	page_state.selected_monitor_index = default_monitor_index(&page_state.monitor_options);

	commands.insert_resource(page_state);
	commands.insert_resource(ParadigmPlaybackState::default());
	commands.insert_resource(ParadigmPresentationWindowState::default());

	if let Ok(content_area) = content_area_query.single() {
		commands.entity(content_area).with_children(|parent| {
			parent
				.spawn((
					Node {
						width: Val::Percent(100.0),
						height: Val::Percent(100.0),
						flex_direction: FlexDirection::Column,
						padding: UiRect::all(Val::Px(16.0)),
						row_gap: Val::Px(12.0),
						..default()
					},
					BackgroundColor(Color::srgb(0.95, 0.95, 0.95)),
					ParadigmContentMarker,
				))
				.with_children(|column| {
					column.spawn((
						Node {
							width: Val::Percent(100.0),
							column_gap: Val::Px(8.0),
							row_gap: Val::Px(8.0),
							flex_wrap: FlexWrap::Wrap,
							align_items: AlignItems::Center,
							..default()
						},
						children![
							spawn_button(ParadigmCycleMonitorButtonMarker, "切换显示器"),
							spawn_button(ParadigmCycleTargetButtonMarker, "切换目标字符"),
							spawn_button(ParadigmStartButtonMarker, "开始播放"),
							spawn_pause_resume_button(),
							spawn_button(ParadigmStopButtonMarker, "停止播放"),
						],
					));

					column.spawn((
						Node {
							width: Val::Percent(100.0),
							height: Val::Px(320.0),
							align_items: AlignItems::Center,
							justify_content: JustifyContent::Center,
							border: UiRect::all(Val::Px(1.0)),
							..default()
						},
						BorderColor::all(Color::srgb(0.65, 0.65, 0.65)),
						BackgroundColor(Color::BLACK),
						ImageNode::new(Handle::default()),
						ParadigmGifPreviewMarker,
					));

					column.spawn((
						Text::new("GIF: 正在加载"),
						TextFont {
							font_size: 14.0,
							..default()
						},
						TextColor(Color::BLACK),
						ParadigmGifInfoTextMarker,
					));
					column.spawn((
						Text::new("显示器: 正在读取"),
						TextFont {
							font_size: 14.0,
							..default()
						},
						TextColor(Color::BLACK),
						ParadigmMonitorTextMarker,
					));
					column.spawn((
						Text::new("目标字符: A"),
						TextFont {
							font_size: 14.0,
							..default()
						},
						TextColor(Color::BLACK),
						ParadigmTargetTextMarker,
					));
					column.spawn((
						Text::new("状态: 空闲"),
						TextFont {
							font_size: 14.0,
							..default()
						},
						TextColor(Color::BLACK),
						ParadigmStatusTextMarker,
					));
				});
		});
	}
}

/// 离开范式页面。
pub fn on_exit(
	mut commands: Commands,
	query: Query<Entity, With<ParadigmContentMarker>>,
	mut window_state: ResMut<ParadigmPresentationWindowState>,
	mut playback_state: ResMut<ParadigmPlaybackState>,
) {
	for entity in &query {
		commands.entity(entity).despawn();
	}

	stop_p300_playback(
		&mut commands,
		playback_state.as_mut(),
		window_state.as_mut(),
	);
	commands.remove_resource::<ParadigmPageState>();
	commands.remove_resource::<ParadigmPlaybackState>();
	commands.remove_resource::<ParadigmPresentationWindowState>();
}

/// 刷新显示器列表。
pub fn update_monitor_inventory(
	monitor_query: Query<(Entity, &Monitor, Option<&PrimaryMonitor>)>,
	mut page_state: ResMut<ParadigmPageState>,
) {
	let current_entity = page_state
		.monitor_options
		.get(page_state.selected_monitor_index)
		.map(|option| option.entity);
	let monitor_options = collect_monitor_options(&monitor_query);
	if monitor_options.is_empty() {
		page_state.monitor_options.clear();
		page_state.selected_monitor_index = 0;
		return;
	}

	let selected_monitor_index = current_entity
		.and_then(|entity| {
			monitor_options
				.iter()
				.position(|option| option.entity == entity)
		})
		.unwrap_or_else(|| default_monitor_index(&monitor_options));

	page_state.monitor_options = monitor_options;
	page_state.selected_monitor_index = selected_monitor_index;
}

/// 处理切换显示器按钮。
pub fn handle_cycle_monitor_click(
	interaction_query: Query<
		&Interaction,
		(Changed<Interaction>, With<ParadigmCycleMonitorButtonMarker>),
	>,
	mut page_state: ResMut<ParadigmPageState>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		if page_state.monitor_options.is_empty() {
			page_state.status_text = "当前没有可用显示器".to_string();
			continue;
		}

		page_state.selected_monitor_index =
			(page_state.selected_monitor_index + 1) % page_state.monitor_options.len();
		page_state.status_text = "已切换目标显示器".to_string();
	}
}

/// 处理切换目标字符按钮。
pub fn handle_cycle_target_click(
	interaction_query: Query<
		&Interaction,
		(Changed<Interaction>, With<ParadigmCycleTargetButtonMarker>),
	>,
	mut page_state: ResMut<ParadigmPageState>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		page_state.target_symbol = next_target_symbol(page_state.target_symbol);
		page_state.status_text = format!("目标字符已切换为 {}", page_state.target_symbol);
	}
}

/// 处理开始播放按钮。
pub fn handle_start_click(
	mut commands: Commands,
	interaction_query: Query<&Interaction, (Changed<Interaction>, With<ParadigmStartButtonMarker>)>,
	monitor_query: Query<(Entity, &Monitor, Option<&PrimaryMonitor>)>,
	mut page_state: ResMut<ParadigmPageState>,
	mut playback_state: ResMut<ParadigmPlaybackState>,
	mut window_state: ResMut<ParadigmPresentationWindowState>,
	mut fixed_time: ResMut<Time<Fixed>>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		if page_state.monitor_options.is_empty() {
			page_state.status_text = "没有可用于播放的显示器".to_string();
			continue;
		}

		stop_p300_playback(
			&mut commands,
			playback_state.as_mut(),
			window_state.as_mut(),
		);

		let Some(selected_option) = page_state
			.monitor_options
			.get(page_state.selected_monitor_index)
			.cloned()
		else {
			page_state.status_text = "目标显示器索引无效".to_string();
			continue;
		};

		let refresh_rate_hz = selected_option.refresh_rate_hz.max(1.0);
		fixed_time.set_timestep_hz(refresh_rate_hz);

		let flash_frames = duration_to_frames(P300_FLASH_DURATION_MS, refresh_rate_hz);
		let interval_frames = duration_to_frames(P300_INTERVAL_DURATION_MS, refresh_rate_hz);

		playback_state.is_running = true;
		playback_state.is_paused = false;
		playback_state.sequence = build_p300_sequence(P300_BLOCK_COUNT);
		playback_state.sequence_index = 0;
		playback_state.remaining_frames = flash_frames;
		playback_state.flash_frames = flash_frames;
		playback_state.interval_frames = interval_frames;
		playback_state.is_flash_phase = true;
		playback_state.active_target = playback_state
			.sequence
			.first()
			.copied()
			.unwrap_or(ParadigmStimulusTarget::None);
		playback_state.block_count = P300_BLOCK_COUNT;

		open_presentation_window(
			&mut commands,
			monitor_query,
			&selected_option,
			window_state.as_mut(),
			page_state.target_symbol,
		);

		page_state.status_text = format!("已开始在 {} 上播放 P300", selected_option.name);
	}
}

/// 处理暂停恢复按钮。
pub fn handle_pause_resume_click(
	interaction_query: Query<
		&Interaction,
		(Changed<Interaction>, With<ParadigmPauseResumeButtonMarker>),
	>,
	mut page_state: ResMut<ParadigmPageState>,
	mut playback_state: ResMut<ParadigmPlaybackState>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		if !playback_state.is_running {
			page_state.status_text = "当前没有正在播放的范式".to_string();
			continue;
		}

		playback_state.is_paused = !playback_state.is_paused;
		page_state.status_text = if playback_state.is_paused {
			"范式播放已暂停".to_string()
		} else {
			"范式播放已恢复".to_string()
		};
	}
}

/// 处理停止播放按钮。
pub fn handle_stop_click(
	mut commands: Commands,
	interaction_query: Query<&Interaction, (Changed<Interaction>, With<ParadigmStopButtonMarker>)>,
	mut page_state: ResMut<ParadigmPageState>,
	mut playback_state: ResMut<ParadigmPlaybackState>,
	mut window_state: ResMut<ParadigmPresentationWindowState>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		stop_p300_playback(
			&mut commands,
			playback_state.as_mut(),
			window_state.as_mut(),
		);
		page_state.status_text = "范式播放已停止".to_string();
	}
}

/// 清理被手动关闭的播放窗口。
pub fn cleanup_closed_presentation_window(
	mut commands: Commands,
	window_query: Query<(), With<Window>>,
	mut page_state: ResMut<ParadigmPageState>,
	mut playback_state: ResMut<ParadigmPlaybackState>,
	mut window_state: ResMut<ParadigmPresentationWindowState>,
) {
	let Some(window_entity) = window_state.window_entity else {
		return;
	};
	if window_query.get(window_entity).is_ok() {
		return;
	}

	stop_p300_playback(
		&mut commands,
		playback_state.as_mut(),
		window_state.as_mut(),
	);
	page_state.status_text = "播放窗口已关闭".to_string();
}

/// 推进 GIF 预览帧。
pub fn tick_gif_preview(time: Res<Time>, mut page_state: ResMut<ParadigmPageState>) {
	let preview = &mut page_state.gif_preview;
	if preview.frames.len() <= 1 {
		return;
	}

	preview.accumulated_seconds += time.delta_secs();
	let current_duration = preview
		.frames
		.get(preview.current_frame_index)
		.map(|frame| frame.duration_seconds)
		.unwrap_or(GIF_FRAME_FALLBACK_SECONDS);

	if preview.accumulated_seconds < current_duration {
		return;
	}

	preview.accumulated_seconds = 0.0;
	preview.current_frame_index = (preview.current_frame_index + 1) % preview.frames.len();
}

/// 同步 GIF 预览图片。
pub fn sync_gif_preview_image(
	page_state: Res<ParadigmPageState>,
	mut preview_query: Query<&mut ImageNode, With<ParadigmGifPreviewMarker>>,
	mut info_query: Query<&mut Text, With<ParadigmGifInfoTextMarker>>,
) {
	let current_image = page_state
		.gif_preview
		.frames
		.get(page_state.gif_preview.current_frame_index)
		.map(|frame| frame.image.clone())
		.unwrap_or_default();

	for mut image_node in &mut preview_query {
		image_node.image = current_image.clone();
	}

	let total_duration_seconds = page_state
		.gif_preview
		.frames
		.iter()
		.map(|frame| frame.duration_seconds)
		.sum::<f32>();
	let info_text = if page_state.gif_preview.is_loaded {
		format!(
			"GIF: {} | 尺寸: {}x{} | 帧数: {} | 总时长: {:.2}s | {}",
			page_state.gif_preview.file_name,
			page_state.gif_preview.size.x,
			page_state.gif_preview.size.y,
			page_state.gif_preview.frames.len(),
			total_duration_seconds,
			page_state.gif_preview.status_text
		)
	} else {
		format!("GIF: {}", page_state.gif_preview.status_text)
	};

	for mut text in &mut info_query {
		text.0 = info_text.clone();
	}
}

/// 同步显示器文本。
pub fn sync_monitor_text(
	page_state: Res<ParadigmPageState>,
	mut text_query: Query<&mut Text, With<ParadigmMonitorTextMarker>>,
) {
	let monitor_text = page_state
		.monitor_options
		.get(page_state.selected_monitor_index)
		.map(|option| format!("显示器: {} | {:.2} Hz", option.name, option.refresh_rate_hz))
		.unwrap_or_else(|| "显示器: 当前没有可用显示器".to_string());

	for mut text in &mut text_query {
		text.0 = monitor_text.clone();
	}
}

/// 同步目标字符文本。
pub fn sync_target_text(
	page_state: Res<ParadigmPageState>,
	mut text_query: Query<&mut Text, With<ParadigmTargetTextMarker>>,
) {
	let target_text = format!("目标字符: {}", page_state.target_symbol);
	for mut text in &mut text_query {
		text.0 = target_text.clone();
	}
}

/// 同步暂停恢复按钮文本。
pub fn sync_window_button_text(
	playback_state: Res<ParadigmPlaybackState>,
	mut text_query: Query<&mut Text, With<ParadigmPauseResumeTextMarker>>,
) {
	let button_text = if playback_state.is_paused {
		"恢复"
	} else {
		"暂停"
	};

	for mut text in &mut text_query {
		text.0 = button_text.to_string();
	}
}

/// 同步状态文本。
pub fn sync_status_text(
	page_state: Res<ParadigmPageState>,
	playback_state: Res<ParadigmPlaybackState>,
	mut text_query: Query<&mut Text, With<ParadigmStatusTextMarker>>,
) {
	let playback_summary = if playback_state.is_running {
		if playback_state.is_paused {
			"播放状态: 已暂停"
		} else {
			"播放状态: 播放中"
		}
	} else {
		"播放状态: 空闲"
	};
	let status_text = format!("{playback_summary} | {}", page_state.status_text);
	for mut text in &mut text_query {
		text.0 = status_text.clone();
	}
}

/// 在 FixedUpdate 中推进 P300 逻辑。
pub fn tick_p300_playback(
	mut playback_state: ResMut<ParadigmPlaybackState>,
	mut page_state: ResMut<ParadigmPageState>,
) {
	if !playback_state.is_running || playback_state.is_paused {
		return;
	}

	if playback_state.sequence.is_empty() {
		playback_state.is_running = false;
		playback_state.active_target = ParadigmStimulusTarget::None;
		page_state.status_text = "P300 刺激序列为空".to_string();
		return;
	}

	if playback_state.remaining_frames > 0 {
		playback_state.remaining_frames -= 1;
		return;
	}

	if playback_state.is_flash_phase {
		playback_state.is_flash_phase = false;
		playback_state.active_target = ParadigmStimulusTarget::None;
		playback_state.remaining_frames = playback_state.interval_frames.saturating_sub(1);
		return;
	}

	playback_state.sequence_index += 1;
	if playback_state.sequence_index >= playback_state.sequence.len() {
		playback_state.is_running = false;
		playback_state.is_paused = false;
		playback_state.active_target = ParadigmStimulusTarget::None;
		page_state.status_text = "P300 播放已完成".to_string();
		return;
	}

	playback_state.is_flash_phase = true;
	playback_state.active_target = playback_state.sequence[playback_state.sequence_index];
	playback_state.remaining_frames = playback_state.flash_frames.saturating_sub(1);
}

/// 更新播放窗口中的 P300 视觉状态。
pub fn update_p300_presentation(
	playback_state: Res<ParadigmPlaybackState>,
	window_state: Res<ParadigmPresentationWindowState>,
	mut cell_query: Query<(&ParadigmPresentationCellMarker, &mut BackgroundColor)>,
) {
	if !window_state.is_open {
		return;
	}

	for (cell, mut background_color) in &mut cell_query {
		*background_color = BackgroundColor(presentation_cell_color(
			playback_state.active_target,
			cell.row,
			cell.col,
		));
	}
}

/// 加载默认 GIF 预览数据。
fn load_default_gif_preview(
	images: &mut Assets<Image>,
) -> crate::homepage::paradigm::resources::ParadigmGifPreviewState {
	let path = PathBuf::from("embedded://embedded_assets/../assets/paradigm/default.gif");
	let file_name = "default.gif".to_string();

	let decoder = match GifDecoder::new(BufReader::new(Cursor::new(
		PARADIGM_DEFAULT_GIF_BYTES.to_vec(),
	))) {
		Ok(decoder) => decoder,
		Err(error) => {
			return crate::homepage::paradigm::resources::ParadigmGifPreviewState {
				path,
				file_name,
				frames: Vec::new(),
				current_frame_index: 0,
				accumulated_seconds: 0.0,
				is_loaded: false,
				status_text: format!("解析 GIF 失败: {error}"),
				size: UVec2::ZERO,
			};
		}
	};

	let frames = match decoder.into_frames().collect_frames() {
		Ok(frames) => frames,
		Err(error) => {
			return crate::homepage::paradigm::resources::ParadigmGifPreviewState {
				path,
				file_name,
				frames: Vec::new(),
				current_frame_index: 0,
				accumulated_seconds: 0.0,
				is_loaded: false,
				status_text: format!("解码 GIF 帧失败: {error}"),
				size: UVec2::ZERO,
			};
		}
	};

	let mut preview_frames = Vec::with_capacity(frames.len());
	let mut size = UVec2::ZERO;

	for frame in frames {
		let (numerator, denominator) = frame.delay().numer_denom_ms();
		let duration_seconds = if denominator == 0 {
			GIF_FRAME_FALLBACK_SECONDS
		} else {
			(numerator as f32 / denominator as f32 / 1000.0).max(GIF_FRAME_FALLBACK_SECONDS)
		};
		let rgba = frame.into_buffer();
		let frame_width = rgba.width();
		let frame_height = rgba.height();
		size = UVec2::new(frame_width, frame_height);
		let image = images.add(create_bevy_image(
			frame_width,
			frame_height,
			rgba.into_raw(),
		));
		preview_frames.push(ParadigmGifFrame {
			image,
			duration_seconds,
		});
	}

	crate::homepage::paradigm::resources::ParadigmGifPreviewState {
		path,
		file_name,
		frames: preview_frames,
		current_frame_index: 0,
		accumulated_seconds: 0.0,
		is_loaded: true,
		status_text: "GIF 已加载".to_string(),
		size,
	}
}

/// 汇总当前显示器选项。
fn collect_monitor_options(
	monitor_query: &Query<(Entity, &Monitor, Option<&PrimaryMonitor>)>,
) -> Vec<ParadigmMonitorOption> {
	let mut monitor_options = monitor_query
		.iter()
		.map(|(entity, monitor, primary_monitor)| {
			let base_name = monitor
				.name
				.clone()
				.unwrap_or_else(|| "未知显示器".to_string());
			let name = if primary_monitor.is_some() {
				format!("{base_name} (主显示器)")
			} else {
				base_name
			};
			let refresh_rate_hz = monitor
				.refresh_rate_millihertz
				.map(|rate| rate as f64 / 1000.0)
				.unwrap_or(60.0);
			ParadigmMonitorOption {
				entity,
				name,
				refresh_rate_hz,
			}
		})
		.collect::<Vec<_>>();
	monitor_options.sort_by(|left, right| left.name.cmp(&right.name));
	monitor_options
}

/// 选择默认显示器索引。
fn default_monitor_index(monitor_options: &[ParadigmMonitorOption]) -> usize {
	monitor_options
		.iter()
		.position(|option| option.name.contains("主显示器"))
		.unwrap_or(0)
}

/// 打开范式播放窗口。
fn open_presentation_window(
	commands: &mut Commands,
	monitor_query: Query<(Entity, &Monitor, Option<&PrimaryMonitor>)>,
	selected_option: &ParadigmMonitorOption,
	window_state: &mut ParadigmPresentationWindowState,
	target_symbol: char,
) {
	if monitor_query.get(selected_option.entity).is_err() {
		return;
	}

	let window_entity = commands
		.spawn(Window {
			title: "范式播放窗口".to_string(),
			mode: WindowMode::BorderlessFullscreen(MonitorSelection::Entity(
				selected_option.entity,
			)),
			position: WindowPosition::Centered(MonitorSelection::Entity(selected_option.entity)),
			decorations: false,
			focused: true,
			..default()
		})
		.id();

	let camera_entity = commands
		.spawn((
			Camera2d,
			RenderTarget::Window(WindowRef::Entity(window_entity)),
		))
		.id();

	let root_entity = commands
		.spawn((
			Node {
				width: Val::Percent(100.0),
				height: Val::Percent(100.0),
				flex_direction: FlexDirection::Column,
				align_items: AlignItems::Center,
				justify_content: JustifyContent::Center,
				row_gap: Val::Px(18.0),
				..default()
			},
			BackgroundColor(Color::BLACK),
			UiTargetCamera(camera_entity),
			ParadigmPresentationRootMarker,
		))
		.with_children(|root| {
			root.spawn((
				Text::new(format!("目标字符: {target_symbol}")),
				TextFont {
					font_size: 40.0,
					..default()
				},
				TextColor(Color::WHITE),
			));

			root.spawn((
				Node {
					display: Display::Grid,
					grid_template_columns: RepeatedGridTrack::flex(P300_GRID_SIZE as u16, 1.0),
					grid_template_rows: RepeatedGridTrack::flex(P300_GRID_SIZE as u16, 1.0),
					column_gap: Val::Px(10.0),
					row_gap: Val::Px(10.0),
					width: Val::Px(900.0),
					height: Val::Px(900.0),
					padding: UiRect::all(Val::Px(16.0)),
					..default()
				},
				BackgroundColor(Color::srgb(0.05, 0.05, 0.05)),
			))
			.with_children(|grid| {
				for row in 0..P300_GRID_SIZE {
					for col in 0..P300_GRID_SIZE {
						grid.spawn((
							Node {
								align_items: AlignItems::Center,
								justify_content: JustifyContent::Center,
								border: UiRect::all(Val::Px(2.0)),
								..default()
							},
							BorderColor::all(Color::srgb(0.35, 0.35, 0.35)),
							BackgroundColor(presentation_cell_color(
								ParadigmStimulusTarget::None,
								row,
								col,
							)),
							ParadigmPresentationCellMarker { row, col },
							children![(
								Text::new(p300_symbol_at(row, col).to_string()),
								TextFont {
									font_size: 44.0,
									..default()
								},
								TextColor(Color::WHITE),
							)],
						));
					}
				}
			});
		})
		.id();

	window_state.window_entity = Some(window_entity);
	window_state.camera_entity = Some(camera_entity);
	window_state.root_entity = Some(root_entity);
	window_state.target_monitor_index = 0;
	window_state.is_open = true;
}

/// 停止 P300 播放并回收窗口。
fn stop_p300_playback(
	commands: &mut Commands,
	playback_state: &mut ParadigmPlaybackState,
	window_state: &mut ParadigmPresentationWindowState,
) {
	playback_state.is_running = false;
	playback_state.is_paused = false;
	playback_state.sequence.clear();
	playback_state.sequence_index = 0;
	playback_state.remaining_frames = 0;
	playback_state.is_flash_phase = false;
	playback_state.active_target = ParadigmStimulusTarget::None;

	if let Some(root_entity) = window_state.root_entity.take() {
		commands.entity(root_entity).despawn();
	}
	if let Some(camera_entity) = window_state.camera_entity.take() {
		commands.entity(camera_entity).despawn();
	}
	if let Some(window_entity) = window_state.window_entity.take() {
		commands.entity(window_entity).despawn();
	}
	window_state.is_open = false;
}

/// 创建通用按钮。
fn spawn_button<T: Component>(marker: T, label: &str) -> impl Bundle {
	(
		Button,
		marker,
		Node {
			height: Val::Px(36.0),
			padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
			align_items: AlignItems::Center,
			justify_content: JustifyContent::Center,
			..default()
		},
		BackgroundColor(Color::srgb(0.20, 0.38, 0.80)),
		children![(
			Text::new(label),
			TextFont {
				font_size: 14.0,
				..default()
			},
			TextColor(Color::WHITE),
		)],
	)
}

/// 创建暂停恢复按钮。
fn spawn_pause_resume_button() -> impl Bundle {
	(
		Button,
		ParadigmPauseResumeButtonMarker,
		Node {
			height: Val::Px(36.0),
			padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
			align_items: AlignItems::Center,
			justify_content: JustifyContent::Center,
			..default()
		},
		BackgroundColor(Color::srgb(0.20, 0.38, 0.80)),
		children![(
			Text::new("暂停"),
			TextFont {
				font_size: 14.0,
				..default()
			},
			TextColor(Color::WHITE),
			ParadigmPauseResumeTextMarker,
		)],
	)
}

/// 构建 Bevy 图片资源。
fn create_bevy_image(width: u32, height: u32, data: Vec<u8>) -> Image {
	Image::new(
		Extent3d {
			width,
			height,
			depth_or_array_layers: 1,
		},
		TextureDimension::D2,
		data,
		TextureFormat::Rgba8UnormSrgb,
		RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
	)
}

/// 计算指定毫秒时长对应的帧数。
fn duration_to_frames(duration_ms: u32, refresh_rate_hz: f64) -> u32 {
	((duration_ms as f64 / 1000.0) * refresh_rate_hz)
		.ceil()
		.max(1.0) as u32
}

/// 生成完整的 P300 行列刺激序列。
fn build_p300_sequence(block_count: u32) -> Vec<ParadigmStimulusTarget> {
	let mut sequence = Vec::with_capacity(block_count as usize * P300_GRID_SIZE * 2);
	let mut random = rng();

	for _ in 0..block_count {
		let mut block = (0..P300_GRID_SIZE)
			.map(ParadigmStimulusTarget::Row)
			.chain((0..P300_GRID_SIZE).map(ParadigmStimulusTarget::Column))
			.collect::<Vec<_>>();
		block.shuffle(&mut random);
		sequence.extend(block);
	}

	sequence
}

/// 返回单元格显示字符。
fn p300_symbol_at(row: usize, col: usize) -> char {
	const SYMBOLS: [char; 36] = [
		'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R',
		'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0',
	];
	SYMBOLS[row * P300_GRID_SIZE + col]
}

/// 根据当前高亮目标计算单元格颜色。
fn presentation_cell_color(target: ParadigmStimulusTarget, row: usize, col: usize) -> Color {
	let is_active = match target {
		ParadigmStimulusTarget::None => false,
		ParadigmStimulusTarget::Row(active_row) => active_row == row,
		ParadigmStimulusTarget::Column(active_col) => active_col == col,
	};

	if is_active {
		Color::srgb(0.95, 0.92, 0.35)
	} else {
		Color::srgb(0.16, 0.18, 0.22)
	}
}

/// 计算下一个目标字符。
fn next_target_symbol(current: char) -> char {
	const SYMBOLS: [char; 36] = [
		'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R',
		'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0',
	];
	let current_index = SYMBOLS
		.iter()
		.position(|symbol| *symbol == current)
		.unwrap_or(0);
	SYMBOLS[(current_index + 1) % SYMBOLS.len()]
}
