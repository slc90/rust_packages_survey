use super::*;

/// 进入深度学习测试页。
pub fn on_enter(
	mut commands: Commands,
	content_area_query: Query<Entity, With<ContentAreaMarker>>,
	mut images: ResMut<Assets<Image>>,
) {
	info!("进入深度学习测试页面");

	let directories = match initialize_runtime_directories() {
		Ok(directories) => directories,
		Err(error) => {
			error!("初始化深度学习目录失败: {error}");
			return;
		}
	};

	let preview_texture = create_preview_placeholder_texture(&mut images);
	let page_state = DeepLearningPageState::new(&directories, preview_texture.clone());

	commands.insert_resource(page_state.clone());
	commands.insert_resource(DeepLearningPendingTasks::default());

	if let Ok(content_area) = content_area_query.single() {
		commands.entity(content_area).with_children(|parent| {
			parent
				.spawn((
					DeepLearningContentMarker,
					Node {
						width: Val::Percent(100.0),
						height: Val::Percent(100.0),
						flex_direction: FlexDirection::Column,
						padding: UiRect::all(Val::Px(16.0)),
						row_gap: Val::Px(16.0),
						overflow: Overflow::scroll_y(),
						..default()
					},
					BackgroundColor(Color::srgb(0.95, 0.96, 0.97)),
				))
				.with_children(|column| {
					column.spawn((
						Text::new("深度学习测试页（Phase 5）"),
						TextFont {
							font_size: 24.0,
							..default()
						},
						TextColor(Color::BLACK),
					));

					column.spawn((
						Text::new(format!(
							"模型目录：{}\n输出目录：{}",
							directories.model_root.display(),
							directories.output_root.display()
						)),
						TextFont {
							font_size: 16.0,
							..default()
						},
						TextColor(Color::srgb(0.18, 0.18, 0.18)),
					));

					spawn_whisper_section(column, &page_state);
					spawn_translation_section(column, &page_state);
					spawn_tts_section(column, &page_state);
					spawn_separation_section(column);
					spawn_image_generation_section(column, &page_state, preview_texture);

					column.spawn((
						Node {
							width: Val::Percent(100.0),
							column_gap: Val::Px(8.0),
							..default()
						},
						children![spawn_action_button(
							DeepLearningSmokeTestButtonMarker,
							"空任务测试",
						)],
					));

					column.spawn((
						Text::new("状态：等待任务"),
						TextFont {
							font_size: 16.0,
							..default()
						},
						TextColor(Color::srgb(0.14, 0.14, 0.14)),
						DeepLearningStatusTextMarker,
					));

					column.spawn((
						Text::new("结果：暂无结果"),
						TextFont {
							font_size: 16.0,
							..default()
						},
						TextColor(Color::srgb(0.14, 0.14, 0.14)),
						DeepLearningResultTextMarker,
					));
				});
		});
	}
}

/// 离开深度学习测试页。
pub fn on_exit(
	mut commands: Commands,
	content_query: Query<Entity, With<DeepLearningContentMarker>>,
) {
	info!("离开深度学习测试页面");

	for entity in &content_query {
		commands.entity(entity).despawn();
	}

	commands.remove_resource::<DeepLearningPageState>();
	commands.remove_resource::<DeepLearningPendingTasks>();
}

fn spawn_whisper_section(builder: &mut ChildSpawnerCommands, state: &DeepLearningPageState) {
	builder
		.spawn(spawn_section_card(
			Color::srgb(0.62, 0.72, 0.82),
			Color::srgb(0.91, 0.96, 0.99),
		))
		.with_children(|section| {
			section.spawn((
				Text::new("Whisper"),
				TextFont {
					font_size: 18.0,
					..default()
				},
				TextColor(Color::BLACK),
			));
			section.spawn((
				Text::new(
					"支持 whisper-base 与 whisper-large-v3。开启原生时间轴后，将输出基于 Whisper timestamp token 的片段时间。"
				),
				TextFont {
					font_size: 13.0,
					..default()
				},
				TextColor(Color::srgb(0.24, 0.30, 0.36)),
			));
			section.spawn((
				Node {
					width: Val::Percent(100.0),
					column_gap: Val::Px(8.0),
					flex_wrap: FlexWrap::Wrap,
					..default()
				},
				children![
					spawn_action_button(
						DeepLearningWhisperOpenFileButtonMarker,
						"选择音频"
					),
					spawn_action_button(
						DeepLearningWhisperLanguageCycleButtonMarker,
						"切换语言"
					),
					spawn_action_button(
						DeepLearningWhisperModelCycleButtonMarker,
						"切换模型"
					),
					spawn_action_button(
						DeepLearningWhisperTimestampToggleButtonMarker,
						"切换时间轴"
					),
					spawn_action_button(DeepLearningWhisperStartButtonMarker, "开始识别"),
				],
			));
			section.spawn((
				Text::new("Whisper 输入：未选择音频或视频文件"),
				TextFont {
					font_size: 16.0,
					..default()
				},
				TextColor(Color::srgb(0.12, 0.12, 0.12)),
				DeepLearningWhisperFileTextMarker,
			));
			section.spawn((
				Text::new(whisper_config_text(state)),
				TextFont {
					font_size: 16.0,
					..default()
				},
				TextColor(Color::srgb(0.12, 0.12, 0.12)),
				DeepLearningWhisperConfigTextMarker,
			));
			section.spawn((
				Node {
					width: Val::Percent(100.0),
					height: Val::Px(18.0),
					border: UiRect::all(Val::Px(1.0)),
					..default()
				},
				BorderColor::all(Color::srgb(0.58, 0.66, 0.72)),
				BackgroundColor(Color::srgb(0.85, 0.90, 0.94)),
				children![(
					Node {
						width: Val::Percent(0.0),
						height: Val::Percent(100.0),
						..default()
					},
					BackgroundColor(Color::srgb(0.18, 0.55, 0.82)),
					DeepLearningWhisperProgressFillMarker,
				)],
			));
			section.spawn((
				Text::new(&state.whisper_status_text),
				TextFont {
					font_size: 14.0,
					..default()
				},
				TextColor(Color::srgb(0.12, 0.12, 0.12)),
				DeepLearningWhisperProgressTextMarker,
			));
		});
}

fn spawn_translation_section(builder: &mut ChildSpawnerCommands, state: &DeepLearningPageState) {
	builder
		.spawn(spawn_section_card(
			Color::srgb(0.73, 0.68, 0.56),
			Color::srgb(0.98, 0.96, 0.90),
		))
		.with_children(|section| {
			section.spawn((
				Text::new("本地翻译"),
				TextFont {
					font_size: 18.0,
					..default()
				},
				TextColor(Color::BLACK),
			));
			section.spawn((
				Node {
					width: Val::Percent(100.0),
					column_gap: Val::Px(8.0),
					flex_wrap: FlexWrap::Wrap,
					..default()
				},
				children![
					spawn_action_button(
						DeepLearningTranslationOpenFileButtonMarker,
						"翻译 选择文本"
					),
					spawn_action_button(
						DeepLearningTranslationLanguageCycleButtonMarker,
						"翻译 切换源语言"
					),
					spawn_action_button(DeepLearningTranslationStartButtonMarker, "翻译 开始任务"),
				],
			));
			section.spawn((
				Text::new("翻译文件：未选择"),
				TextFont {
					font_size: 16.0,
					..default()
				},
				TextColor(Color::srgb(0.12, 0.12, 0.12)),
				DeepLearningTranslationFileTextMarker,
			));
			section.spawn((
				Text::new(translation_config_text(state)),
				TextFont {
					font_size: 16.0,
					..default()
				},
				TextColor(Color::srgb(0.12, 0.12, 0.12)),
				DeepLearningTranslationConfigTextMarker,
			));
		});
}

fn spawn_tts_section(builder: &mut ChildSpawnerCommands, state: &DeepLearningPageState) {
	builder
		.spawn(spawn_section_card(
			Color::srgb(0.59, 0.68, 0.60),
			Color::srgb(0.93, 0.98, 0.94),
		))
		.with_children(|section| {
			section.spawn((
				Text::new("语音生成"),
				TextFont {
					font_size: 18.0,
					..default()
				},
				TextColor(Color::BLACK),
			));
			section.spawn((
				Node {
					width: Val::Percent(100.0),
					column_gap: Val::Px(8.0),
					flex_wrap: FlexWrap::Wrap,
					..default()
				},
				children![
					spawn_action_button(DeepLearningTtsOpenFileButtonMarker, "TTS 选择文本"),
					spawn_action_button(DeepLearningTtsLanguageCycleButtonMarker, "TTS 切换语言"),
					spawn_action_button(DeepLearningTtsSpeedCycleButtonMarker, "TTS 切换语速"),
					spawn_action_button(DeepLearningTtsStartButtonMarker, "TTS 开始任务"),
				],
			));
			section.spawn((
				Text::new("TTS 文件：未选择"),
				TextFont {
					font_size: 16.0,
					..default()
				},
				TextColor(Color::srgb(0.12, 0.12, 0.12)),
				DeepLearningTtsFileTextMarker,
			));
			section.spawn((
				Text::new(tts_config_text(state)),
				TextFont {
					font_size: 16.0,
					..default()
				},
				TextColor(Color::srgb(0.12, 0.12, 0.12)),
				DeepLearningTtsConfigTextMarker,
			));
		});
}

fn spawn_separation_section(builder: &mut ChildSpawnerCommands) {
	builder
		.spawn(spawn_section_card(
			Color::srgb(0.72, 0.60, 0.68),
			Color::srgb(0.98, 0.93, 0.96),
		))
		.with_children(|section| {
			section.spawn((
				Text::new("人声分离"),
				TextFont {
					font_size: 18.0,
					..default()
				},
				TextColor(Color::BLACK),
			));
			section.spawn((
				Node {
					width: Val::Percent(100.0),
					column_gap: Val::Px(8.0),
					flex_wrap: FlexWrap::Wrap,
					..default()
				},
				children![
					spawn_action_button(
						DeepLearningSeparationOpenFileButtonMarker,
						"分离 选择音频"
					),
					spawn_action_button(DeepLearningSeparationStartButtonMarker, "分离 开始任务"),
				],
			));
			section.spawn((
				Text::new("分离文件：未选择"),
				TextFont {
					font_size: 16.0,
					..default()
				},
				TextColor(Color::srgb(0.12, 0.12, 0.12)),
				DeepLearningSeparationFileTextMarker,
			));
		});
}

fn spawn_image_generation_section(
	builder: &mut ChildSpawnerCommands,
	state: &DeepLearningPageState,
	preview_texture: Handle<Image>,
) {
	builder
		.spawn(spawn_section_card(
			Color::srgb(0.65, 0.60, 0.38),
			Color::srgb(0.98, 0.97, 0.88),
		))
		.with_children(|section| {
			section.spawn((
				Text::new("图像生成"),
				TextFont {
					font_size: 18.0,
					..default()
				},
				TextColor(Color::BLACK),
			));
			section.spawn((
				Node {
					width: Val::Percent(100.0),
					column_gap: Val::Px(8.0),
					flex_wrap: FlexWrap::Wrap,
					..default()
				},
				children![
					spawn_action_button(
						DeepLearningImageGenerationOpenFileButtonMarker,
						"图像 Prompt 文件"
					),
					spawn_action_button(
						DeepLearningImageGenerationResolutionCycleButtonMarker,
						"图像 切换尺寸"
					),
					spawn_action_button(
						DeepLearningImageGenerationSeedCycleButtonMarker,
						"图像 切换种子"
					),
					spawn_action_button(
						DeepLearningImageGenerationStepsCycleButtonMarker,
						"图像 切换步数"
					),
					spawn_action_button(
						DeepLearningImageGenerationModelCycleButtonMarker,
						"图像 切换模型"
					),
					spawn_action_button(
						DeepLearningImageGenerationStartButtonMarker,
						"图像 开始生成"
					),
				],
			));
			section.spawn((
				Text::new("Prompt 文件：未选择"),
				TextFont {
					font_size: 16.0,
					..default()
				},
				TextColor(Color::srgb(0.12, 0.12, 0.12)),
				DeepLearningImageGenerationFileTextMarker,
			));
			section.spawn((
				Text::new(image_generation_config_text(state)),
				TextFont {
					font_size: 16.0,
					..default()
				},
				TextColor(Color::srgb(0.12, 0.12, 0.12)),
				DeepLearningImageGenerationConfigTextMarker,
			));
			section.spawn((
				Node {
					width: Val::Px(320.0),
					height: Val::Px(180.0),
					border: UiRect::all(Val::Px(1.0)),
					..default()
				},
				ImageNode::new(preview_texture),
				BorderColor::all(Color::srgb(0.70, 0.70, 0.70)),
				BackgroundColor(Color::srgb(0.88, 0.90, 0.92)),
				DeepLearningImageGenerationPreviewImageMarker,
			));
			section.spawn((
				Text::new("图片预览：等待生成结果"),
				TextFont {
					font_size: 14.0,
					..default()
				},
				TextColor(Color::srgb(0.20, 0.20, 0.20)),
				DeepLearningImageGenerationPreviewTextMarker,
			));
		});
}
