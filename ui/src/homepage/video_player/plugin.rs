use crate::homepage::common::Functions;
use crate::homepage::video_player::systems::{
	cleanup_closed_popup_window, handle_close, handle_main_open_file, handle_play_pause,
	handle_popup_close, handle_popup_open_file, handle_popup_play_pause, handle_show_popup,
	on_enter, on_exit, sync_main_video_player_texts, sync_player_events,
	sync_popup_video_player_texts,
};
use bevy::prelude::*;

/// 视频播放页面插件
pub struct VideoPlayerPlugin;

impl Plugin for VideoPlayerPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(Functions::VideoPlayer), on_enter)
			.add_systems(OnExit(Functions::VideoPlayer), on_exit)
			.add_systems(
				Update,
				(
					handle_main_open_file,
					handle_play_pause,
					handle_close,
					handle_show_popup,
					handle_popup_open_file,
					handle_popup_play_pause,
					handle_popup_close,
					sync_player_events,
					sync_main_video_player_texts,
					sync_popup_video_player_texts,
					cleanup_closed_popup_window,
				)
					.run_if(in_state(Functions::VideoPlayer)),
			);
	}
}
