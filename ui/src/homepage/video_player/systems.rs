use std::{
	path::{Path, PathBuf},
	process::Command,
};

use bevy::{
	asset::RenderAssetUsages,
	camera::RenderTarget,
	prelude::*,
	render::render_resource::{Extent3d, TextureDimension, TextureFormat},
	window::{WindowRef, WindowResolution},
};
use media_player::{PlaybackStatus, PlayerEvent};

use crate::homepage::common::ContentAreaMarker;
use crate::homepage::video_player::components::{
	PopupVideoCloseButtonMarker, PopupVideoDisplayMarker, PopupVideoFileTextMarker,
	PopupVideoOpenFileButtonMarker, PopupVideoPlayPauseButtonMarker, PopupVideoPlayPauseTextMarker,
	PopupVideoPlayerRootMarker, PopupVideoStatusTextMarker, VideoCloseButtonMarker,
	VideoDisplayMarker, VideoFileTextMarker, VideoOpenFileButtonMarker, VideoPlayPauseButtonMarker,
	VideoPlayPauseTextMarker, VideoPlayerContentMarker, VideoPopupButtonMarker,
	VideoStatusTextMarker,
};
use crate::homepage::video_player::resources::{
	MainVideoPlayerState, PopupVideoPlayerState, VideoPlayerSlotState,
};

const PLACEHOLDER_TEXTURE_SIZE: u32 = 4;
const PLACEHOLDER_PIXEL: [u8; 4] = [24, 24, 24, 255];

/// 进入视频播放页面
pub fn on_enter(
	mut commands: Commands,
	content_area_query: Query<Entity, With<ContentAreaMarker>>,
	mut images: ResMut<Assets<Image>>,
) {
	info!("进入视频播放页面");

	let initial_directory = default_video_directory();
	let mut main_state = MainVideoPlayerState {
		slot: VideoPlayerSlotState::default(),
		initial_directory,
	};
	main_state.slot.texture = Some(create_placeholder_texture(&mut images));
	main_state.slot.status_text = "请点击打开文件选择主窗口视频".to_string();

	let mut popup_state = PopupVideoPlayerState::default();
	popup_state.slot.texture = Some(create_placeholder_texture(&mut images));
	popup_state.slot.status_text = "弹窗未开启".to_string();

	commands.insert_resource(main_state);
	commands.insert_resource(popup_state);

	if let Ok(content_area) = content_area_query.single() {
		let placeholder = create_placeholder_handle();
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
					BackgroundColor(Color::srgb(0.96, 0.96, 0.96)),
					VideoPlayerContentMarker,
				))
				.with_children(|column| {
					column.spawn((
						Node {
							width: Val::Percent(100.0),
							height: Val::Px(56.0),
							column_gap: Val::Px(8.0),
							flex_wrap: FlexWrap::Wrap,
							align_items: AlignItems::Center,
							..default()
						},
						children![
							spawn_button(VideoOpenFileButtonMarker, "打开文件"),
							spawn_play_pause_button(),
							spawn_button(VideoCloseButtonMarker, "关闭视频"),
							spawn_button(VideoPopupButtonMarker, "独立弹窗"),
						],
					));

					column.spawn((
						Node {
							width: Val::Percent(100.0),
							height: Val::Px(720.0),
							align_items: AlignItems::Center,
							justify_content: JustifyContent::Center,
							border: UiRect::all(Val::Px(1.0)),
							..default()
						},
						BorderColor::all(Color::srgb(0.7, 0.7, 0.7)),
						BackgroundColor(Color::BLACK),
						ImageNode::new(placeholder),
						VideoDisplayMarker,
					));

					column.spawn((
						Text::new("文件: 未加载"),
						TextFont {
							font_size: 14.0,
							..default()
						},
						TextColor(Color::BLACK),
						VideoFileTextMarker,
					));
					column.spawn((
						Text::new("状态: 空闲"),
						TextFont {
							font_size: 14.0,
							..default()
						},
						TextColor(Color::BLACK),
						VideoStatusTextMarker,
					));
				});
		});
	}
}

/// 离开视频页面
pub fn on_exit(
	mut commands: Commands,
	query: Query<Entity, With<VideoPlayerContentMarker>>,
	mut popup_state: ResMut<PopupVideoPlayerState>,
) {
	info!("离开视频播放页面");
	for entity in &query {
		commands.entity(entity).despawn();
	}
	close_popup_entities(&mut commands, popup_state.as_mut(), true);
	commands.remove_resource::<MainVideoPlayerState>();
	commands.remove_resource::<PopupVideoPlayerState>();
}

/// 主窗口打开文件
pub fn handle_main_open_file(
	interaction_query: Query<&Interaction, (Changed<Interaction>, With<VideoOpenFileButtonMarker>)>,
	mut state: ResMut<MainVideoPlayerState>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		let selected = pick_video_file(&state.initial_directory);
		load_file_into_slot(&mut state.slot, selected);
	}
}

/// 主窗口播放暂停
pub fn handle_play_pause(
	interaction_query: Query<
		&Interaction,
		(Changed<Interaction>, With<VideoPlayPauseButtonMarker>),
	>,
	mut state: ResMut<MainVideoPlayerState>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		toggle_play_pause(&mut state.slot, "请先为主窗口打开视频文件");
	}
}

/// 主窗口关闭当前视频
pub fn handle_close(
	interaction_query: Query<&Interaction, (Changed<Interaction>, With<VideoCloseButtonMarker>)>,
	mut state: ResMut<MainVideoPlayerState>,
	mut images: ResMut<Assets<Image>>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		close_slot(&mut state.slot, &mut images, "已关闭主窗口当前视频");
	}
}

/// 打开独立弹窗
pub fn handle_show_popup(
	mut commands: Commands,
	interaction_query: Query<&Interaction, (Changed<Interaction>, With<VideoPopupButtonMarker>)>,
	mut popup_state: ResMut<PopupVideoPlayerState>,
	mut images: ResMut<Assets<Image>>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		if popup_state.window_entity.is_some() {
			popup_state.slot.status_text = "独立弹窗已存在".to_string();
			continue;
		}

		if popup_state.slot.texture.is_none() {
			popup_state.slot.texture = Some(create_placeholder_texture(&mut images));
		}
		popup_state.slot.status_text = "请在弹窗中点击打开文件".to_string();

		let window_entity = commands
			.spawn(Window {
				title: "独立视频弹窗".to_string(),
				resolution: WindowResolution::new(960, 720),
				resizable: true,
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
					padding: UiRect::all(Val::Px(16.0)),
					row_gap: Val::Px(12.0),
					..default()
				},
				BackgroundColor(Color::srgb(0.10, 0.10, 0.12)),
				UiTargetCamera(camera_entity),
				PopupVideoPlayerRootMarker,
			))
			.with_children(|column| {
				column.spawn((
					Node {
						width: Val::Percent(100.0),
						height: Val::Px(56.0),
						column_gap: Val::Px(8.0),
						flex_wrap: FlexWrap::Wrap,
						align_items: AlignItems::Center,
						..default()
					},
					children![
						spawn_button(PopupVideoOpenFileButtonMarker, "打开文件"),
						spawn_popup_play_pause_button(),
						spawn_button(PopupVideoCloseButtonMarker, "关闭视频"),
					],
				));

				column.spawn((
					Node {
						width: Val::Percent(100.0),
						height: Val::Percent(100.0),
						align_items: AlignItems::Center,
						justify_content: JustifyContent::Center,
						border: UiRect::all(Val::Px(1.0)),
						..default()
					},
					BorderColor::all(Color::srgb(0.35, 0.35, 0.35)),
					BackgroundColor(Color::BLACK),
					ImageNode::new(create_placeholder_handle()),
					PopupVideoDisplayMarker,
				));

				column.spawn((
					Text::new("文件: 未加载"),
					TextFont {
						font_size: 14.0,
						..default()
					},
					TextColor(Color::WHITE),
					PopupVideoFileTextMarker,
				));
				column.spawn((
					Text::new("状态: 空闲"),
					TextFont {
						font_size: 14.0,
						..default()
					},
					TextColor(Color::WHITE),
					PopupVideoStatusTextMarker,
				));
			})
			.id();

		popup_state.window_entity = Some(window_entity);
		popup_state.camera_entity = Some(camera_entity);
		popup_state.root_entity = Some(root_entity);
	}
}

/// 弹窗打开文件
pub fn handle_popup_open_file(
	interaction_query: Query<
		&Interaction,
		(Changed<Interaction>, With<PopupVideoOpenFileButtonMarker>),
	>,
	main_state: Res<MainVideoPlayerState>,
	mut popup_state: ResMut<PopupVideoPlayerState>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		let selected = pick_video_file(&main_state.initial_directory);
		load_file_into_slot(&mut popup_state.slot, selected);
	}
}

/// 弹窗播放暂停
pub fn handle_popup_play_pause(
	interaction_query: Query<
		&Interaction,
		(Changed<Interaction>, With<PopupVideoPlayPauseButtonMarker>),
	>,
	mut popup_state: ResMut<PopupVideoPlayerState>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		toggle_play_pause(&mut popup_state.slot, "请先在弹窗中打开视频文件");
	}
}

/// 弹窗关闭当前视频
pub fn handle_popup_close(
	interaction_query: Query<
		&Interaction,
		(Changed<Interaction>, With<PopupVideoCloseButtonMarker>),
	>,
	mut popup_state: ResMut<PopupVideoPlayerState>,
	mut images: ResMut<Assets<Image>>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		close_slot(&mut popup_state.slot, &mut images, "已关闭弹窗当前视频");
	}
}

/// 同步两个播放器事件
pub fn sync_player_events(
	mut main_state: ResMut<MainVideoPlayerState>,
	mut popup_state: ResMut<PopupVideoPlayerState>,
	mut images: ResMut<Assets<Image>>,
) {
	sync_slot_events(&mut main_state.slot, &mut images);
	sync_slot_events(&mut popup_state.slot, &mut images);
}

/// 同步主窗口文本与纹理
pub fn sync_main_video_player_texts(
	main_state: Res<MainVideoPlayerState>,
	mut main_display_query: Query<&mut ImageNode, With<VideoDisplayMarker>>,
	mut main_text_queries: ParamSet<(
		Query<&mut Text, With<VideoPlayPauseTextMarker>>,
		Query<&mut Text, With<VideoFileTextMarker>>,
		Query<&mut Text, With<VideoStatusTextMarker>>,
	)>,
) {
	if !main_state.is_changed() {
		return;
	}

	update_display_texture(&main_state.slot, &mut main_display_query);
	update_play_text(&main_state.slot, &mut main_text_queries.p0());
	update_file_text(&main_state.slot, &mut main_text_queries.p1());
	update_status_text(&main_state.slot, &mut main_text_queries.p2());
}

/// 同步弹窗文本与纹理
pub fn sync_popup_video_player_texts(
	popup_state: Res<PopupVideoPlayerState>,
	mut popup_display_query: Query<&mut ImageNode, With<PopupVideoDisplayMarker>>,
	mut popup_text_queries: ParamSet<(
		Query<&mut Text, With<PopupVideoPlayPauseTextMarker>>,
		Query<&mut Text, With<PopupVideoFileTextMarker>>,
		Query<&mut Text, With<PopupVideoStatusTextMarker>>,
	)>,
) {
	if !popup_state.is_changed() {
		return;
	}

	update_display_texture(&popup_state.slot, &mut popup_display_query);
	update_play_text(&popup_state.slot, &mut popup_text_queries.p0());
	update_file_text(&popup_state.slot, &mut popup_text_queries.p1());
	update_status_text(&popup_state.slot, &mut popup_text_queries.p2());
}

/// 清理被手动关闭的弹窗
pub fn cleanup_closed_popup_window(
	mut commands: Commands,
	window_query: Query<(), With<Window>>,
	mut popup_state: ResMut<PopupVideoPlayerState>,
) {
	let Some(window_entity) = popup_state.window_entity else {
		return;
	};
	if window_query.get(window_entity).is_ok() {
		return;
	}

	if let Some(player) = popup_state.slot.player.as_ref() {
		let _ = player.close();
	}
	popup_state.slot.player = None;
	popup_state.slot.current_file = None;
	popup_state.slot.status = PlaybackStatus::Idle;
	popup_state.slot.position_ms = 0;
	popup_state.slot.duration_ms = None;
	popup_state.slot.status_text = "弹窗已关闭".to_string();

	close_popup_entities(&mut commands, popup_state.as_mut(), false);
}

/// 通用按钮
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
		BackgroundColor(Color::srgb(0.18, 0.38, 0.82)),
		children![(
			Text::new(label),
			TextFont {
				font_size: 14.0,
				..default()
			},
			TextColor(Color::WHITE)
		)],
	)
}

/// 主窗口播放按钮
fn spawn_play_pause_button() -> impl Bundle {
	(
		Button,
		VideoPlayPauseButtonMarker,
		Node {
			height: Val::Px(36.0),
			padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
			align_items: AlignItems::Center,
			justify_content: JustifyContent::Center,
			..default()
		},
		BackgroundColor(Color::srgb(0.18, 0.38, 0.82)),
		children![(
			Text::new("播放"),
			TextFont {
				font_size: 14.0,
				..default()
			},
			TextColor(Color::WHITE),
			VideoPlayPauseTextMarker
		)],
	)
}

/// 弹窗播放按钮
fn spawn_popup_play_pause_button() -> impl Bundle {
	(
		Button,
		PopupVideoPlayPauseButtonMarker,
		Node {
			height: Val::Px(36.0),
			padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
			align_items: AlignItems::Center,
			justify_content: JustifyContent::Center,
			..default()
		},
		BackgroundColor(Color::srgb(0.18, 0.38, 0.82)),
		children![(
			Text::new("播放"),
			TextFont {
				font_size: 14.0,
				..default()
			},
			TextColor(Color::WHITE),
			PopupVideoPlayPauseTextMarker
		)],
	)
}

/// 创建占位纹理
fn create_placeholder_texture(images: &mut Assets<Image>) -> Handle<Image> {
	images.add(Image::new_fill(
		Extent3d {
			width: PLACEHOLDER_TEXTURE_SIZE,
			height: PLACEHOLDER_TEXTURE_SIZE,
			depth_or_array_layers: 1,
		},
		TextureDimension::D2,
		&PLACEHOLDER_PIXEL,
		TextureFormat::Rgba8UnormSrgb,
		RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
	))
}

/// 为 UI 初始化空纹理句柄
fn create_placeholder_handle() -> Handle<Image> {
	Handle::default()
}

/// 获取默认视频目录
fn default_video_directory() -> PathBuf {
	let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	let base_dir = manifest_dir
		.parent()
		.map(Path::to_path_buf)
		.unwrap_or(manifest_dir);
	base_dir.join("data").join("video_samples")
}

/// 打开文件选择框
fn pick_video_file(initial_directory: &Path) -> Option<PathBuf> {
	pick_video_file_with_powershell(initial_directory)
}

/// 把文件加载到播放器槽位
fn load_file_into_slot(slot: &mut VideoPlayerSlotState, path: Option<PathBuf>) {
	let Some(path) = path else {
		slot.status_text = "未选择视频文件".to_string();
		return;
	};

	if slot.player.is_none() {
		match media_player::PlayerHandle::spawn() {
			Ok(player) => slot.player = Some(player),
			Err(error) => {
				slot.status = PlaybackStatus::Error;
				slot.status_text = error.to_string();
				return;
			}
		}
	}

	if let Some(player) = slot.player.as_ref() {
		match player.load(path.clone()) {
			Ok(()) => {
				slot.current_file = Some(path);
				slot.status = PlaybackStatus::Loading;
				slot.position_ms = 0;
				slot.duration_ms = None;
				slot.status_text = "正在加载视频".to_string();
			}
			Err(error) => {
				slot.status = PlaybackStatus::Error;
				slot.status_text = error.to_string();
			}
		}
	}
}

/// 切换播放与暂停
fn toggle_play_pause(slot: &mut VideoPlayerSlotState, idle_message: &str) {
	let Some(player) = slot.player.as_ref() else {
		slot.status_text = idle_message.to_string();
		return;
	};

	let result = match slot.status {
		PlaybackStatus::Playing => player.pause(),
		PlaybackStatus::Paused | PlaybackStatus::Idle | PlaybackStatus::Ended => player.play(),
		PlaybackStatus::Loading | PlaybackStatus::Error => Ok(()),
	};

	if let Err(error) = result {
		slot.status = PlaybackStatus::Error;
		slot.status_text = error.to_string();
	}
}

/// 关闭当前槽位视频
fn close_slot(slot: &mut VideoPlayerSlotState, images: &mut Assets<Image>, message: &str) {
	if let Some(player) = slot.player.as_ref() {
		let _ = player.close();
	}
	slot.player = None;
	slot.current_file = None;
	slot.status = PlaybackStatus::Idle;
	slot.position_ms = 0;
	slot.duration_ms = None;
	slot.status_text = message.to_string();
	reset_texture(slot, images);
}

/// 同步单个槽位事件
fn sync_slot_events(slot: &mut VideoPlayerSlotState, images: &mut Assets<Image>) {
	let events = if let Some(player) = slot.player.as_ref() {
		player.drain_events()
	} else {
		return;
	};

	for event in events {
		apply_player_event(slot, event, images);
	}
}

/// 应用播放器事件
fn apply_player_event(
	slot: &mut VideoPlayerSlotState,
	event: PlayerEvent,
	images: &mut Assets<Image>,
) {
	match event {
		PlayerEvent::StatusChanged(status) => {
			slot.status = status;
		}
		PlayerEvent::Loaded(path) => {
			slot.current_file = Some(path);
			slot.status_text = "视频已加载并开始播放".to_string();
		}
		PlayerEvent::FrameReady(frame) => {
			if slot.texture.is_none() {
				slot.texture = Some(create_placeholder_texture(images));
			}
			if let Some(texture_handle) = slot.texture.as_ref()
				&& let Some(image) = images.get_mut(texture_handle)
			{
				if image.texture_descriptor.size.width != frame.width
					|| image.texture_descriptor.size.height != frame.height
				{
					*image = create_video_image(frame.width, frame.height, frame.pixels_rgba);
				} else {
					image.data = Some(frame.pixels_rgba);
				}
			}
		}
		PlayerEvent::PositionUpdated {
			position_ms,
			duration_ms,
		} => {
			slot.position_ms = position_ms;
			slot.duration_ms = duration_ms;
		}
		PlayerEvent::Error(message) => {
			slot.status = PlaybackStatus::Error;
			slot.status_text = message;
		}
		PlayerEvent::Closed => {
			slot.status = PlaybackStatus::Idle;
			slot.current_file = None;
			slot.position_ms = 0;
			slot.duration_ms = None;
			slot.status_text = "播放器已关闭".to_string();
			reset_texture(slot, images);
		}
	}
}

/// 同步显示纹理
fn update_display_texture<T: Component>(
	slot: &VideoPlayerSlotState,
	display_query: &mut Query<&mut ImageNode, With<T>>,
) {
	let Some(texture) = slot.texture.clone() else {
		return;
	};
	for mut image_node in display_query.iter_mut() {
		image_node.image = texture.clone();
	}
}

/// 同步播放按钮文本
fn update_play_text<T: Component>(
	slot: &VideoPlayerSlotState,
	text_query: &mut Query<&mut Text, With<T>>,
) {
	for mut text in text_query.iter_mut() {
		text.0 = match slot.status {
			PlaybackStatus::Playing => "暂停".to_string(),
			_ => "播放".to_string(),
		};
	}
}

/// 同步文件文本
fn update_file_text<T: Component>(
	slot: &VideoPlayerSlotState,
	text_query: &mut Query<&mut Text, With<T>>,
) {
	for mut text in text_query.iter_mut() {
		text.0 = match slot.current_file.as_ref() {
			Some(path) => format!("文件: {}", path.display()),
			None => "文件: 未加载".to_string(),
		};
	}
}

/// 同步状态文本
fn update_status_text<T: Component>(
	slot: &VideoPlayerSlotState,
	text_query: &mut Query<&mut Text, With<T>>,
) {
	for mut text in text_query.iter_mut() {
		text.0 = format!(
			"状态: {:?} | 位置: {} / {} ms | {}",
			slot.status,
			slot.position_ms,
			slot.duration_ms.unwrap_or(0),
			slot.status_text
		);
	}
}

/// 创建视频纹理
fn create_video_image(width: u32, height: u32, data: Vec<u8>) -> Image {
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

/// 重置到占位纹理
fn reset_texture(slot: &mut VideoPlayerSlotState, images: &mut Assets<Image>) {
	let Some(handle) = slot.texture.as_ref() else {
		return;
	};
	if let Some(image) = images.get_mut(handle) {
		*image = Image::new_fill(
			Extent3d {
				width: PLACEHOLDER_TEXTURE_SIZE,
				height: PLACEHOLDER_TEXTURE_SIZE,
				depth_or_array_layers: 1,
			},
			TextureDimension::D2,
			&PLACEHOLDER_PIXEL,
			TextureFormat::Rgba8UnormSrgb,
			RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
		);
	}
}

/// 关闭弹窗相关实体
fn close_popup_entities(
	commands: &mut Commands,
	popup_state: &mut PopupVideoPlayerState,
	despawn_window: bool,
) {
	if let Some(root_entity) = popup_state.root_entity.take() {
		commands.entity(root_entity).despawn();
	}
	if let Some(camera_entity) = popup_state.camera_entity.take() {
		commands.entity(camera_entity).despawn();
	}
	if despawn_window {
		if let Some(window_entity) = popup_state.window_entity.take() {
			commands.entity(window_entity).despawn();
		}
	} else {
		popup_state.window_entity = None;
	}
}

/// 使用 PowerShell 原生文件对话框选择 mp4
fn pick_video_file_with_powershell(initial_directory: &Path) -> Option<PathBuf> {
	if !cfg!(target_os = "windows") {
		return None;
	}

	let initial_directory = initial_directory.to_string_lossy().replace('\'', "''");
	let script = format!(
		"[void][System.Reflection.Assembly]::LoadWithPartialName('System.Windows.Forms'); \
		$dialog = New-Object System.Windows.Forms.OpenFileDialog; \
		$dialog.Filter = 'MP4 files (*.mp4)|*.mp4'; \
		$dialog.InitialDirectory = '{initial_directory}'; \
		$dialog.Multiselect = $false; \
		if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {{ \
			Write-Output $dialog.FileName \
		}}"
	);

	let output = Command::new("powershell")
		.args(["-NoProfile", "-Command", &script])
		.output()
		.ok()?;
	if !output.status.success() {
		return None;
	}

	let selected_path = String::from_utf8(output.stdout).ok()?;
	let trimmed = selected_path.trim();
	if trimmed.is_empty() {
		None
	} else {
		Some(PathBuf::from(trimmed))
	}
}
