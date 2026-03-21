use std::{
	path::Path,
	sync::{Arc, Mutex},
};

use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use gstreamer_app::AppSink;

use crate::{
	error::MediaPlayerError,
	events::{PlaybackStatus, PlayerEvent},
	frame::VideoFrame,
};

/// 构建播放管线并返回 pipeline 和 appsink
pub fn create_pipeline(
	path: &Path,
	event_queue: Arc<Mutex<std::collections::VecDeque<PlayerEvent>>>,
) -> Result<(gst::Pipeline, AppSink), MediaPlayerError> {
	let absolute_path = path.canonicalize()?;
	let uri = file_path_to_uri(&absolute_path);
	let launch = format!(
		concat!(
			"uridecodebin uri=\"{uri}\" force-sw-decoders=true name=src ",
			"src. ! queue ! videoconvert ! video/x-raw,format=RGBA ! ",
			"appsink name=video_sink sync=true max-buffers=1 leaky-type=downstream ",
			"src. ! queue ! audioconvert ! audioresample ! autoaudiosink sync=true"
		),
		uri = uri
	);
	eprintln!("[media_player] pipeline launch: {launch}");

	let element = gst::parse::launch(&launch)
		.map_err(|error| MediaPlayerError::Pipeline(error.to_string()))?;
	let pipeline = element
		.downcast::<gst::Pipeline>()
		.map_err(|_| MediaPlayerError::Pipeline("无法转换为 Pipeline".to_string()))?;
	let appsink = pipeline
		.by_name("video_sink")
		.ok_or_else(|| MediaPlayerError::Pipeline("未找到 video_sink".to_string()))?
		.downcast::<gst_app::AppSink>()
		.map_err(|_| MediaPlayerError::Pipeline("video_sink 不是 AppSink".to_string()))?;

	appsink.set_max_buffers(1);
	appsink.set_wait_on_eos(false);

	let queue = event_queue.clone();
	appsink.set_callbacks(
		gst_app::AppSinkCallbacks::builder()
			.new_sample(move |sink| match sink.pull_sample() {
				Ok(sample) => {
					if let Some(frame) = sample_to_frame(&sample) {
						push_event(&queue, PlayerEvent::FrameReady(frame));
					}
					Ok(gst::FlowSuccess::Ok)
				}
				Err(_) => Err(gst::FlowError::Eos),
			})
			.build(),
	);

	push_event(
		&event_queue,
		PlayerEvent::StatusChanged(PlaybackStatus::Loading),
	);
	Ok((pipeline, appsink))
}

fn sample_to_frame(sample: &gst::Sample) -> Option<VideoFrame> {
	let caps = sample.caps()?;
	let structure = caps.structure(0)?;
	let width = structure.get::<i32>("width").ok()? as u32;
	let height = structure.get::<i32>("height").ok()? as u32;
	let buffer = sample.buffer()?;
	let map = buffer.map_readable().ok()?;
	let pixels = map.as_slice().to_vec();
	let pts_ns = buffer.pts().map(|pts| pts.nseconds()).unwrap_or(0);

	Some(VideoFrame {
		width,
		height,
		pixels_rgba: pixels,
		pts_ns,
	})
}

fn file_path_to_uri(path: &Path) -> String {
	let raw = path.to_string_lossy();
	let trimmed = raw
		.strip_prefix(r"\\?\")
		.or_else(|| raw.strip_prefix(r"//?/"))
		.unwrap_or(&raw);
	let normalized = trimmed.replace('\\', "/");
	format!("file:///{}", normalized)
}

fn push_event(queue: &Arc<Mutex<std::collections::VecDeque<PlayerEvent>>>, event: PlayerEvent) {
	if let Ok(mut events) = queue.lock() {
		events.push_back(event);
	}
}
