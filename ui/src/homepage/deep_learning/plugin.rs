use bevy::prelude::*;
use deep_learning::task::{DlTaskRequestMessage, DlTaskResultMessage, DlTaskStatusMessage};

use crate::homepage::{
	common::Functions,
	deep_learning::systems::{
		handle_smoke_test_click, handle_task_requests, handle_translation_language_cycle_click,
		handle_translation_open_file_click, handle_translation_start_click,
		handle_tts_language_cycle_click, handle_tts_open_file_click, handle_tts_speed_cycle_click,
		handle_tts_start_click, handle_whisper_language_cycle_click,
		handle_whisper_open_file_click, handle_whisper_start_click,
		handle_whisper_timestamp_toggle_click, on_enter, on_exit, sync_result_messages,
		sync_status_messages, update_pending_tasks,
	},
};

/// 深度学习测试页插件。
pub struct DeepLearningPlugin;

impl Plugin for DeepLearningPlugin {
	fn build(&self, app: &mut App) {
		app.add_message::<DlTaskRequestMessage>()
			.add_message::<DlTaskStatusMessage>()
			.add_message::<DlTaskResultMessage>()
			.add_systems(OnEnter(Functions::DeepLearning), on_enter)
			.add_systems(OnExit(Functions::DeepLearning), on_exit)
			.add_systems(
				Update,
				(
					handle_smoke_test_click,
					handle_whisper_open_file_click,
					handle_whisper_language_cycle_click,
					handle_whisper_timestamp_toggle_click,
					handle_whisper_start_click,
					handle_translation_open_file_click,
					handle_translation_language_cycle_click,
					handle_translation_start_click,
					handle_tts_open_file_click,
					handle_tts_language_cycle_click,
					handle_tts_speed_cycle_click,
					handle_tts_start_click,
					handle_task_requests,
					update_pending_tasks,
					sync_status_messages,
					sync_result_messages,
				)
					.chain()
					.run_if(in_state(Functions::DeepLearning)),
			);
	}
}
