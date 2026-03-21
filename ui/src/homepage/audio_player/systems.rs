use std::path::{Path, PathBuf};

use audio_player::{AudioPlaybackStatus, PlayerEvent};
use bevy::prelude::*;

use crate::file_dialog::pick_single_file;
use crate::homepage::audio_player::components::{
	AudioCloseButtonMarker, AudioFileTextMarker, AudioOpenFileButtonMarker,
	AudioPlayPauseButtonMarker, AudioPlayPauseTextMarker, AudioPlayerContentMarker,
	AudioStatusTextMarker,
};
use crate::homepage::audio_player::resources::AudioPlaybackState;
use crate::homepage::common::ContentAreaMarker;

/// 进入音频播放页面
pub fn on_enter(
	mut commands: Commands,
	content_area_query: Query<Entity, With<ContentAreaMarker>>,
) {
	info!("进入音频播放页面");

	let state = AudioPlaybackState {
		initial_directory: default_audio_directory(),
		..Default::default()
	};
	commands.insert_resource(state);

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
					BackgroundColor(Color::srgb(0.96, 0.96, 0.96)),
					AudioPlayerContentMarker,
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
							spawn_button(AudioOpenFileButtonMarker, "打开文件"),
							spawn_play_pause_button(),
							spawn_button(AudioCloseButtonMarker, "关闭音频"),
						],
					));

					column
						.spawn((
							Node {
								width: Val::Percent(100.0),
								height: Val::Px(240.0),
								flex_direction: FlexDirection::Column,
								justify_content: JustifyContent::Center,
								padding: UiRect::all(Val::Px(16.0)),
								row_gap: Val::Px(12.0),
								border: UiRect::all(Val::Px(1.0)),
								..default()
							},
							BorderColor::all(Color::srgb(0.72, 0.72, 0.72)),
							BackgroundColor(Color::srgb(0.10, 0.12, 0.14)),
						))
						.with_children(|panel| {
							panel.spawn((
								Text::new("文件: 未加载"),
								TextFont {
									font_size: 16.0,
									..default()
								},
								TextColor(Color::WHITE),
								AudioFileTextMarker,
							));
							panel.spawn((
								Text::new("状态: 空闲"),
								TextFont {
									font_size: 16.0,
									..default()
								},
								TextColor(Color::WHITE),
								AudioStatusTextMarker,
							));
							panel.spawn((
								Text::new("预留区域：后续可加入进度条、音量和频谱"),
								TextFont {
									font_size: 14.0,
									..default()
								},
								TextColor(Color::srgb(0.85, 0.85, 0.85)),
							));
						});
				});
		});
	}
}

/// 离开音频播放页面
pub fn on_exit(
	mut commands: Commands,
	query: Query<Entity, With<AudioPlayerContentMarker>>,
	state: ResMut<AudioPlaybackState>,
) {
	info!("离开音频播放页面");

	if let Some(player) = state.player.as_ref() {
		let _ = player.close();
	}

	for entity in &query {
		commands.entity(entity).despawn();
	}

	commands.remove_resource::<AudioPlaybackState>();
}

/// 处理打开音频文件
pub fn handle_open_audio_file(
	interaction_query: Query<&Interaction, (Changed<Interaction>, With<AudioOpenFileButtonMarker>)>,
	mut state: ResMut<AudioPlaybackState>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		let Some(path) = pick_audio_file(&state.initial_directory) else {
			continue;
		};
		load_audio_file(&mut state, path);
	}
}

/// 处理播放暂停
pub fn handle_play_pause(
	interaction_query: Query<
		&Interaction,
		(Changed<Interaction>, With<AudioPlayPauseButtonMarker>),
	>,
	mut state: ResMut<AudioPlaybackState>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		let Some(player) = state.player.as_ref() else {
			state.status_text = "请先打开音频文件".to_string();
			continue;
		};

		let result = match state.status {
			AudioPlaybackStatus::Playing => player.pause(),
			AudioPlaybackStatus::Paused
			| AudioPlaybackStatus::Idle
			| AudioPlaybackStatus::Ended => player.play(),
			AudioPlaybackStatus::Loading | AudioPlaybackStatus::Error => Ok(()),
		};

		if let Err(error) = result {
			state.status = AudioPlaybackStatus::Error;
			state.status_text = error.to_string();
		}
	}
}

/// 处理关闭音频
pub fn handle_close_audio(
	interaction_query: Query<&Interaction, (Changed<Interaction>, With<AudioCloseButtonMarker>)>,
	mut state: ResMut<AudioPlaybackState>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		if let Some(player) = state.player.as_ref() {
			let _ = player.close();
		}
		state.player = None;
		state.current_file = None;
		state.status = AudioPlaybackStatus::Idle;
		state.position_ms = 0;
		state.duration_ms = None;
		state.status_text = "已关闭当前音频".to_string();
	}
}

/// 同步后台线程事件
pub fn sync_audio_player_events(mut state: ResMut<AudioPlaybackState>) {
	let events = if let Some(player) = state.player.as_ref() {
		player.drain_events()
	} else {
		return;
	};

	for event in events {
		match event {
			PlayerEvent::StatusChanged(status) => {
				state.status = status;
			}
			PlayerEvent::Loaded { path, duration_ms } => {
				state.current_file = Some(path);
				state.duration_ms = duration_ms;
				state.position_ms = 0;
				state.status_text = "音频已加载并开始播放".to_string();
			}
			PlayerEvent::PositionUpdated {
				position_ms,
				duration_ms,
			} => {
				state.position_ms = position_ms;
				state.duration_ms = duration_ms;
			}
			PlayerEvent::Error(message) => {
				state.status = AudioPlaybackStatus::Error;
				state.status_text = message;
			}
			PlayerEvent::Closed => {
				state.status = AudioPlaybackStatus::Idle;
				state.current_file = None;
				state.position_ms = 0;
				state.duration_ms = None;
				state.status_text = "播放器已关闭".to_string();
			}
		}
	}
}

/// 同步按钮和文本显示
pub fn sync_audio_player_texts(
	state: Res<AudioPlaybackState>,
	mut text_queries: ParamSet<(
		Query<&mut Text, With<AudioPlayPauseTextMarker>>,
		Query<&mut Text, With<AudioFileTextMarker>>,
		Query<
			&mut Text,
			(
				With<AudioStatusTextMarker>,
				Without<AudioFileTextMarker>,
				Without<AudioPlayPauseTextMarker>,
			),
		>,
	)>,
) {
	if !state.is_changed() {
		return;
	}

	for mut text in &mut text_queries.p0() {
		text.0 = match state.status {
			AudioPlaybackStatus::Playing => "暂停".to_string(),
			_ => "播放".to_string(),
		};
	}

	for mut text in &mut text_queries.p1() {
		text.0 = match state.current_file.as_ref() {
			Some(path) => format!("文件: {}", path.display()),
			None => "文件: 未加载".to_string(),
		};
	}

	for mut text in &mut text_queries.p2() {
		text.0 = format!(
			"状态: {:?} | 位置: {} / {} ms | {}",
			state.status,
			state.position_ms,
			state.duration_ms.unwrap_or(0),
			state.status_text
		);
	}
}

/// 创建统一按钮
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

/// 创建播放暂停按钮
fn spawn_play_pause_button() -> impl Bundle {
	(
		Button,
		AudioPlayPauseButtonMarker,
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
			AudioPlayPauseTextMarker
		)],
	)
}

/// 获取默认音频目录
fn default_audio_directory() -> PathBuf {
	let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	let base_dir = manifest_dir
		.parent()
		.map(Path::to_path_buf)
		.unwrap_or(manifest_dir);
	base_dir.join("data")
}

/// 打开音频文件选择框
fn pick_audio_file(initial_directory: &Path) -> Option<PathBuf> {
	pick_single_file(
		Some(initial_directory),
		"选择音频文件",
		&[("音频文件", &["mp3", "wav", "flac", "ogg"])],
	)
}

/// 加载音频文件
fn load_audio_file(state: &mut AudioPlaybackState, path: PathBuf) {
	if state.player.is_none() {
		match audio_player::PlayerHandle::spawn() {
			Ok(player) => state.player = Some(player),
			Err(error) => {
				state.status = AudioPlaybackStatus::Error;
				state.status_text = error.to_string();
				return;
			}
		}
	}

	if let Some(player) = state.player.as_ref() {
		match player.load(path.clone()) {
			Ok(()) => {
				state.current_file = Some(path);
				state.status = AudioPlaybackStatus::Loading;
				state.position_ms = 0;
				state.duration_ms = None;
				state.status_text = "正在加载音频".to_string();
			}
			Err(error) => {
				state.status = AudioPlaybackStatus::Error;
				state.status_text = error.to_string();
			}
		}
	}
}
