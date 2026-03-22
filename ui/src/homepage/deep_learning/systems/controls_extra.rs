use super::*;

/// 处理 TTS 选择文件。
pub fn handle_tts_open_file_click(
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<DeepLearningTtsOpenFileButtonMarker>,
		),
	>,
	mut state: ResMut<DeepLearningPageState>,
	mut text_query: Query<&mut Text, With<DeepLearningTtsFileTextMarker>>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		let Some(path) = pick_text_input_file("选择 TTS 文本文件") else {
			continue;
		};

		state.tts_input_file = Some(path.clone());
		update_single_text(&mut text_query, &format!("TTS 文件：{}", path.display()));
	}
}

/// 处理 TTS 语言切换。
pub fn handle_tts_language_cycle_click(
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<DeepLearningTtsLanguageCycleButtonMarker>,
		),
	>,
	mut state: ResMut<DeepLearningPageState>,
	mut text_query: Query<&mut Text, With<DeepLearningTtsConfigTextMarker>>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		state.tts_language = next_tts_language(state.tts_language);
		update_single_text(&mut text_query, &tts_config_text(&state));
	}
}

/// 处理 TTS 语速切换。
pub fn handle_tts_speed_cycle_click(
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<DeepLearningTtsSpeedCycleButtonMarker>,
		),
	>,
	mut state: ResMut<DeepLearningPageState>,
	mut text_query: Query<&mut Text, With<DeepLearningTtsConfigTextMarker>>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		state.tts_speed = next_tts_speed(state.tts_speed);
		update_single_text(&mut text_query, &tts_config_text(&state));
	}
}

/// 处理 TTS 开始按钮。
pub fn handle_tts_start_click(
	interaction_query: Query<
		&Interaction,
		(Changed<Interaction>, With<DeepLearningTtsStartButtonMarker>),
	>,
	mut state: ResMut<DeepLearningPageState>,
	mut writer: MessageWriter<DlTaskRequestMessage>,
	mut status_writer: MessageWriter<DlTaskStatusMessage>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		let Some(request) = state.build_tts_request() else {
			status_writer.write(DlTaskStatusMessage {
				id: DlTaskId(0),
				kind: DlTaskKind::Tts,
				state: DlTaskState::Failed,
				progress: 0.0,
				message: "请先选择 TTS 文本文件".to_string(),
			});
			continue;
		};

		let task_id = state.allocate_task_id();
		writer.write(DlTaskRequestMessage {
			id: task_id,
			kind: DlTaskKind::Tts,
			payload: DlTaskPayload::Tts(request),
		});
	}
}

/// 处理人声分离选择文件。
pub fn handle_separation_open_file_click(
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<DeepLearningSeparationOpenFileButtonMarker>,
		),
	>,
	mut state: ResMut<DeepLearningPageState>,
	mut text_query: Query<&mut Text, With<DeepLearningSeparationFileTextMarker>>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		let Some(path) = pick_separation_input_file() else {
			continue;
		};

		state.separation_input_file = Some(path.clone());
		update_single_text(&mut text_query, &format!("分离文件：{}", path.display()));
	}
}

/// 处理人声分离开始按钮。
pub fn handle_separation_start_click(
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<DeepLearningSeparationStartButtonMarker>,
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

		let Some(request) = state.build_separation_request() else {
			status_writer.write(DlTaskStatusMessage {
				id: DlTaskId(0),
				kind: DlTaskKind::Separation,
				state: DlTaskState::Failed,
				progress: 0.0,
				message: "请先选择人声分离输入文件".to_string(),
			});
			continue;
		};

		let task_id = state.allocate_task_id();
		writer.write(DlTaskRequestMessage {
			id: task_id,
			kind: DlTaskKind::Separation,
			payload: DlTaskPayload::Separation(request),
		});
	}
}

/// 处理图片生成选择 Prompt 文件。
pub fn handle_image_generation_open_file_click(
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<DeepLearningImageGenerationOpenFileButtonMarker>,
		),
	>,
	mut state: ResMut<DeepLearningPageState>,
	mut text_query: Query<&mut Text, With<DeepLearningImageGenerationFileTextMarker>>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		let Some(path) = pick_text_input_file("选择图像生成 Prompt 文本文件") else {
			continue;
		};

		state.image_generation_prompt_file = Some(path.clone());
		update_single_text(&mut text_query, &format!("Prompt 文件：{}", path.display()));
	}
}

/// 处理图片生成分辨率切换。
pub fn handle_image_generation_resolution_cycle_click(
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<DeepLearningImageGenerationResolutionCycleButtonMarker>,
		),
	>,
	mut state: ResMut<DeepLearningPageState>,
	mut text_query: Query<&mut Text, With<DeepLearningImageGenerationConfigTextMarker>>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		state.image_generation_resolution =
			next_image_generation_resolution(state.image_generation_resolution);
		update_single_text(&mut text_query, &image_generation_config_text(&state));
	}
}

/// 处理图片生成种子切换。
pub fn handle_image_generation_seed_cycle_click(
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<DeepLearningImageGenerationSeedCycleButtonMarker>,
		),
	>,
	mut state: ResMut<DeepLearningPageState>,
	mut text_query: Query<&mut Text, With<DeepLearningImageGenerationConfigTextMarker>>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		state.image_generation_seed = next_image_generation_seed(state.image_generation_seed);
		update_single_text(&mut text_query, &image_generation_config_text(&state));
	}
}

/// 处理图片生成步数切换。
pub fn handle_image_generation_steps_cycle_click(
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<DeepLearningImageGenerationStepsCycleButtonMarker>,
		),
	>,
	mut state: ResMut<DeepLearningPageState>,
	mut text_query: Query<&mut Text, With<DeepLearningImageGenerationConfigTextMarker>>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		state.image_generation_steps = next_image_generation_steps(state.image_generation_steps);
		update_single_text(&mut text_query, &image_generation_config_text(&state));
	}
}

/// 处理图片生成模型切换。
pub fn handle_image_generation_model_cycle_click(
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<DeepLearningImageGenerationModelCycleButtonMarker>,
		),
	>,
	mut state: ResMut<DeepLearningPageState>,
	mut text_query: Query<&mut Text, With<DeepLearningImageGenerationConfigTextMarker>>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		state.image_generation_model = next_image_generation_model(state.image_generation_model);
		update_single_text(&mut text_query, &image_generation_config_text(&state));
	}
}

/// 处理图片生成开始按钮。
pub fn handle_image_generation_start_click(
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<DeepLearningImageGenerationStartButtonMarker>,
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

		let Some(request) = state.build_image_generation_request() else {
			status_writer.write(DlTaskStatusMessage {
				id: DlTaskId(0),
				kind: DlTaskKind::ImageGeneration,
				state: DlTaskState::Failed,
				progress: 0.0,
				message: "请先选择图像生成 Prompt 文件".to_string(),
			});
			continue;
		};

		let task_id = state.allocate_task_id();
		writer.write(DlTaskRequestMessage {
			id: task_id,
			kind: DlTaskKind::ImageGeneration,
			payload: DlTaskPayload::ImageGeneration(request),
		});
	}
}
