use super::*;

/// 处理空任务测试按钮。
pub fn handle_smoke_test_click(
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<DeepLearningSmokeTestButtonMarker>,
		),
	>,
	mut state: ResMut<DeepLearningPageState>,
	mut writer: MessageWriter<DlTaskRequestMessage>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		let task_id = state.allocate_task_id();
		writer.write(DlTaskRequestMessage {
			id: task_id,
			kind: DlTaskKind::SmokeTest,
			payload: DlTaskPayload::SmokeTest,
		});
	}
}

/// 处理 Whisper 选择文件。
pub fn handle_whisper_open_file_click(
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<DeepLearningWhisperOpenFileButtonMarker>,
		),
	>,
	mut state: ResMut<DeepLearningPageState>,
	mut text_query: Query<&mut Text, With<DeepLearningWhisperFileTextMarker>>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		let Some(path) = pick_whisper_input_file() else {
			continue;
		};

		state.whisper_input_file = Some(path.clone());
		update_single_text(
			&mut text_query,
			&format!("Whisper 输入：{}", path.display()),
		);
	}
}

/// 处理 Whisper 语言切换。
pub fn handle_whisper_language_cycle_click(
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<DeepLearningWhisperLanguageCycleButtonMarker>,
		),
	>,
	mut state: ResMut<DeepLearningPageState>,
	mut text_query: Query<&mut Text, With<DeepLearningWhisperConfigTextMarker>>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		state.whisper_language_hint = next_whisper_language_hint(state.whisper_language_hint);
		update_single_text(&mut text_query, &whisper_config_text(&state));
	}
}

/// 处理 Whisper 模型切换。
pub fn handle_whisper_model_cycle_click(
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<DeepLearningWhisperModelCycleButtonMarker>,
		),
	>,
	mut state: ResMut<DeepLearningPageState>,
	mut text_query: Query<&mut Text, With<DeepLearningWhisperConfigTextMarker>>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		state.whisper_model = next_whisper_model(state.whisper_model);
		update_single_text(&mut text_query, &whisper_config_text(&state));
	}
}

/// 处理 Whisper 时间戳开关。
pub fn handle_whisper_timestamp_toggle_click(
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<DeepLearningWhisperTimestampToggleButtonMarker>,
		),
	>,
	mut state: ResMut<DeepLearningPageState>,
	mut text_query: Query<&mut Text, With<DeepLearningWhisperConfigTextMarker>>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		state.whisper_with_timestamps = !state.whisper_with_timestamps;
		update_single_text(&mut text_query, &whisper_config_text(&state));
	}
}

/// 处理 Whisper 开始按钮。
pub fn handle_whisper_start_click(
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<DeepLearningWhisperStartButtonMarker>,
		),
	>,
	mut state: ResMut<DeepLearningPageState>,
	mut writer: MessageWriter<DlTaskRequestMessage>,
	mut status_writer: MessageWriter<DlTaskStatusMessage>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		let Some(request) = state.build_whisper_request() else {
			status_writer.write(DlTaskStatusMessage {
				id: DlTaskId(0),
				kind: DlTaskKind::Whisper,
				state: DlTaskState::Failed,
				progress: 0.0,
				message: "请先选择 Whisper 输入文件".to_string(),
			});
			continue;
		};

		let task_id = state.allocate_task_id();
		writer.write(DlTaskRequestMessage {
			id: task_id,
			kind: DlTaskKind::Whisper,
			payload: DlTaskPayload::Whisper(request),
		});
	}
}

/// 处理翻译选择文件。
pub fn handle_translation_open_file_click(
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<DeepLearningTranslationOpenFileButtonMarker>,
		),
	>,
	mut state: ResMut<DeepLearningPageState>,
	mut text_query: Query<&mut Text, With<DeepLearningTranslationFileTextMarker>>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		let Some(path) = pick_text_input_file("选择翻译文本文件") else {
			continue;
		};

		state.translation_input_file = Some(path.clone());
		update_single_text(&mut text_query, &format!("翻译文件：{}", path.display()));
	}
}

/// 处理翻译语言切换。
pub fn handle_translation_language_cycle_click(
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<DeepLearningTranslationLanguageCycleButtonMarker>,
		),
	>,
	mut state: ResMut<DeepLearningPageState>,
	mut text_query: Query<&mut Text, With<DeepLearningTranslationConfigTextMarker>>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		state.translation_source_language =
			next_translation_source_language(state.translation_source_language);
		update_single_text(&mut text_query, &translation_config_text(&state));
	}
}

/// 处理翻译开始按钮。
pub fn handle_translation_start_click(
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<DeepLearningTranslationStartButtonMarker>,
		),
	>,
	mut state: ResMut<DeepLearningPageState>,
	mut writer: MessageWriter<DlTaskRequestMessage>,
	mut status_writer: MessageWriter<DlTaskStatusMessage>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		let Some(request) = state.build_translation_request() else {
			status_writer.write(DlTaskStatusMessage {
				id: DlTaskId(0),
				kind: DlTaskKind::Translation,
				state: DlTaskState::Failed,
				progress: 0.0,
				message: "请先选择翻译文本文件".to_string(),
			});
			continue;
		};

		let task_id = state.allocate_task_id();
		writer.write(DlTaskRequestMessage {
			id: task_id,
			kind: DlTaskKind::Translation,
			payload: DlTaskPayload::Translation(request),
		});
	}
}
