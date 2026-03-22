use super::*;

/// 同步状态消息到页面。
pub fn sync_status_messages(
	mut messages: MessageReader<DlTaskStatusMessage>,
	mut state: ResMut<DeepLearningPageState>,
	mut text_queries: ParamSet<(
		Query<&mut Text, With<DeepLearningStatusTextMarker>>,
		Query<&mut Text, With<DeepLearningWhisperProgressTextMarker>>,
	)>,
	mut whisper_progress_query: Query<&mut Node, With<DeepLearningWhisperProgressFillMarker>>,
) {
	let Some(last_message) = messages.read().last().cloned() else {
		return;
	};

	state.status_text = format!(
		"状态：任务 {} / {:?} / {:.0}% / {}",
		last_message.id.0,
		last_message.state,
		last_message.progress * 100.0,
		last_message.message
	);
	update_single_text(&mut text_queries.p0(), &state.status_text);

	if last_message.kind != DlTaskKind::Whisper {
		return;
	}

	state.whisper_progress = last_message.progress.clamp(0.0, 1.0);
	state.whisper_status_text = format!(
		"Whisper 进度：{:?} / {:.0}% / {}",
		last_message.state,
		state.whisper_progress * 100.0,
		last_message.message
	);
	update_single_text(&mut text_queries.p1(), &state.whisper_status_text);

	for mut node in &mut whisper_progress_query {
		node.width = Val::Percent(state.whisper_progress * 100.0);
	}
}

/// 同步结果消息到页面。
pub fn sync_result_messages(
	mut messages: MessageReader<DlTaskResultMessage>,
	mut state: ResMut<DeepLearningPageState>,
	mut text_queries: ParamSet<(
		Query<&mut Text, With<DeepLearningResultTextMarker>>,
		Query<&mut Text, With<DeepLearningImageGenerationPreviewTextMarker>>,
	)>,
	mut preview_query: Query<&mut ImageNode, With<DeepLearningImageGenerationPreviewImageMarker>>,
	mut images: ResMut<Assets<Image>>,
) {
	let Some(last_message) = messages.read().last().cloned() else {
		return;
	};

	let output_path = last_message
		.output_path
		.as_ref()
		.map(std::path::PathBuf::from);
	let detail_text = output_path
		.as_deref()
		.and_then(|path| load_text_result_detail(last_message.kind, path));
	state.result_text = match (&last_message.output_path, detail_text) {
		(Some(path), Some(detail)) => format!(
			"结果：任务 {} / {:?} / {} / 输出：{}\n\n{}",
			last_message.id.0, last_message.kind, last_message.summary, path, detail
		),
		(Some(path), None) => format!(
			"结果：任务 {} / {:?} / {} / 输出：{}",
			last_message.id.0, last_message.kind, last_message.summary, path
		),
		(None, _) => format!(
			"结果：任务 {} / {:?} / {}",
			last_message.id.0, last_message.kind, last_message.summary
		),
	};
	update_single_text(&mut text_queries.p0(), &state.result_text);

	if last_message.kind != DlTaskKind::ImageGeneration {
		return;
	}

	let Some(path) = last_message.output_path.clone() else {
		update_single_text(&mut text_queries.p1(), "图片预览：本次任务没有输出图像文件");
		return;
	};

	let path_buf = std::path::PathBuf::from(&path);
	match load_preview_texture_from_path(
		&state.image_generation_preview_texture,
		&path_buf,
		&mut images,
	) {
		Ok(()) => {
			for mut image_node in &mut preview_query {
				image_node.image = state.image_generation_preview_texture.clone();
			}
			state.image_generation_preview_path = Some(path_buf.clone());
			update_single_text(
				&mut text_queries.p1(),
				&format!("图片预览：{}", path_buf.display()),
			);
		}
		Err(error) => {
			update_single_text(
				&mut text_queries.p1(),
				&format!("图片预览加载失败：{error}"),
			);
		}
	}
}

/// 读取文本类任务的结果详情，便于直接展示到页面。
fn load_text_result_detail(kind: DlTaskKind, path: &std::path::Path) -> Option<String> {
	match kind {
		DlTaskKind::Whisper | DlTaskKind::Translation => {}
		_ => return None,
	}

	let content = std::fs::read_to_string(path).ok()?;
	let trimmed = content.trim();
	if trimmed.is_empty() {
		return None;
	}

	Some(trimmed.to_string())
}
