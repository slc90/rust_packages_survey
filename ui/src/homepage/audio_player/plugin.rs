use bevy::prelude::*;

use crate::homepage::audio_player::systems::{
	handle_close_audio, handle_open_audio_file, handle_play_pause, on_enter, on_exit,
	sync_audio_player_events, sync_audio_player_texts,
};
use crate::homepage::common::Functions;

/// 音频播放页面插件
pub struct AudioPlayerPlugin;

impl Plugin for AudioPlayerPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(Functions::AudioPlayer), on_enter)
			.add_systems(OnExit(Functions::AudioPlayer), on_exit)
			.add_systems(
				Update,
				(
					handle_open_audio_file,
					handle_play_pause,
					handle_close_audio,
					sync_audio_player_events,
					sync_audio_player_texts,
				)
					.run_if(in_state(Functions::AudioPlayer)),
			);
	}
}
