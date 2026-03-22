use super::*;

/// 同步状态消息到页面。
pub fn sync_status_messages(
	mut messages: MessageReader<DlTaskStatusMessage>,
	mut state: ResMut<DeepLearningPageState>,
	mut text_query: Query<&mut Text, With<DeepLearningStatusTextMarker>>,
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
	update_single_text(&mut text_query, &state.status_text);
}

/// 同步结果消息到页面。
pub fn sync_result_messages(
	mut messages: MessageReader<DlTaskResultMessage>,
	mut state: ResMut<DeepLearningPageState>,
	mut result_text_query: Query<&mut Text, With<DeepLearningResultTextMarker>>,
	mut preview_text_query: Query<&mut Text, With<DeepLearningImageGenerationPreviewTextMarker>>,
	mut preview_query: Query<&mut ImageNode, With<DeepLearningImageGenerationPreviewImageMarker>>,
	mut images: ResMut<Assets<Image>>,
) {
	let Some(last_message) = messages.read().last().cloned() else {
		return;
	};

	state.result_text = match &last_message.output_path {
		Some(path) => format!(
			"结果：任务 {} / {:?} / {} / 输出：{}",
			last_message.id.0, last_message.kind, last_message.summary, path
		),
		None => format!(
			"结果：任务 {} / {:?} / {}",
			last_message.id.0, last_message.kind, last_message.summary
		),
	};
	update_single_text(&mut result_text_query, &state.result_text);

	if last_message.kind != DlTaskKind::ImageGeneration {
		return;
	}

	let Some(path) = last_message.output_path.clone() else {
		update_single_text(
			&mut preview_text_query,
			"图片预览：本次任务没有输出图像文件",
		);
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
				&mut preview_text_query,
				&format!("图片预览：{}", path_buf.display()),
			);
		}
		Err(error) => {
			update_single_text(
				&mut preview_text_query,
				&format!("图片预览加载失败：{error}"),
			);
		}
	}
}
