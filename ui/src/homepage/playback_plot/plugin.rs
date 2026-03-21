//! 回放波形插件
//!
//! 提供回放波形功能的 Bevy 插件

use crate::homepage::common::Functions;
use crate::homepage::playback_plot::systems::{
	handle_next_page, handle_open_file, handle_play_pause, handle_prev_page, handle_speed_change,
	on_enter, on_exit, spawn_playback_control_ui, update_playback,
	update_playback_position_display, update_playback_waveform,
};
use bevy::prelude::*;

/// 回放波形插件
pub struct PlaybackPlotPlugin;

impl Plugin for PlaybackPlotPlugin {
	fn build(&self, app: &mut App) {
		// 添加回放页面生命周期系统
		app.add_systems(
			OnEnter(Functions::PlaybackPlot),
			(on_enter, spawn_playback_control_ui),
		)
		.add_systems(OnExit(Functions::PlaybackPlot), on_exit)
		// 添加播放控制交互系统
		.add_systems(
			Update,
			(
				update_playback,
				update_playback_waveform,
				update_playback_position_display,
				handle_open_file,
				handle_play_pause,
				handle_speed_change,
				handle_prev_page,
				handle_next_page,
			)
				.run_if(in_state(Functions::PlaybackPlot)),
		);
	}
}
