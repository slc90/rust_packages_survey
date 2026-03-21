use std::{
	collections::VecDeque,
	ffi::c_void,
	path::PathBuf,
	sync::{
		Arc, Mutex, OnceLock,
		mpsc::{self, Sender, TryRecvError},
	},
	thread::{self, JoinHandle},
	time::Duration,
};

use gstreamer as gst;
use gstreamer::prelude::*;

use crate::{
	error::MediaPlayerError,
	events::{PlaybackStatus, PlayerCommand, PlayerEvent},
	pipeline::create_pipeline,
};

/// 播放器句柄
pub struct PlayerHandle {
	command_tx: Sender<PlayerCommand>,
	event_queue: Arc<Mutex<VecDeque<PlayerEvent>>>,
	worker: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl PlayerHandle {
	/// 创建新的播放器后台线程
	pub fn spawn() -> Result<Self, MediaPlayerError> {
		initialize_gstreamer()?;

		let (command_tx, command_rx) = mpsc::channel();
		let event_queue = Arc::new(Mutex::new(VecDeque::new()));
		let queue_for_thread = event_queue.clone();
		let worker = thread::Builder::new()
			.name("media-player-worker".to_string())
			.spawn(move || {
				let _com_guard = initialize_media_thread_com();
				let mut current_pipeline: Option<gst::Pipeline> = None;
				let mut current_status = PlaybackStatus::Idle;

				loop {
					loop {
						match command_rx.try_recv() {
							Ok(command) => {
								let keep_running = handle_command(
									command,
									&queue_for_thread,
									&mut current_pipeline,
									&mut current_status,
								);
								if !keep_running {
									return;
								}
							}
							Err(TryRecvError::Empty) => break,
							Err(TryRecvError::Disconnected) => return,
						}
					}

					if let Some(pipeline) = current_pipeline.as_ref() {
						drain_bus(pipeline, &queue_for_thread, &mut current_status);
						push_position_event(pipeline, &queue_for_thread);
					}

					thread::sleep(Duration::from_millis(15));
				}
			})
			.map_err(|error| MediaPlayerError::Initialization(error.to_string()))?;

		Ok(Self {
			command_tx,
			event_queue,
			worker: Arc::new(Mutex::new(Some(worker))),
		})
	}

	/// 加载文件并立即播放
	pub fn load(&self, path: PathBuf) -> Result<(), MediaPlayerError> {
		self.command_tx
			.send(PlayerCommand::Load(path))
			.map_err(|error| MediaPlayerError::Pipeline(error.to_string()))
	}

	/// 播放
	pub fn play(&self) -> Result<(), MediaPlayerError> {
		self.command_tx
			.send(PlayerCommand::Play)
			.map_err(|error| MediaPlayerError::Pipeline(error.to_string()))
	}

	/// 暂停
	pub fn pause(&self) -> Result<(), MediaPlayerError> {
		self.command_tx
			.send(PlayerCommand::Pause)
			.map_err(|error| MediaPlayerError::Pipeline(error.to_string()))
	}

	/// 关闭当前文件
	pub fn close(&self) -> Result<(), MediaPlayerError> {
		self.command_tx
			.send(PlayerCommand::Close)
			.map_err(|error| MediaPlayerError::Pipeline(error.to_string()))
	}

	/// 取出后台线程事件
	pub fn drain_events(&self) -> Vec<PlayerEvent> {
		let mut drained = Vec::new();
		if let Ok(mut events) = self.event_queue.lock() {
			while let Some(event) = events.pop_front() {
				drained.push(event);
			}
		}
		drained
	}
}

impl Drop for PlayerHandle {
	fn drop(&mut self) {
		let _ = self.command_tx.send(PlayerCommand::Shutdown);
		if let Ok(mut worker) = self.worker.lock()
			&& let Some(handle) = worker.take()
		{
			let _ = handle.join();
		}
	}
}

fn initialize_gstreamer() -> Result<(), MediaPlayerError> {
	static INIT_RESULT: OnceLock<Result<(), String>> = OnceLock::new();
	INIT_RESULT
		.get_or_init(|| gst::init().map_err(|error| error.to_string()))
		.clone()
		.map_err(MediaPlayerError::Initialization)
}

fn handle_command(
	command: PlayerCommand,
	event_queue: &Arc<Mutex<VecDeque<PlayerEvent>>>,
	current_pipeline: &mut Option<gst::Pipeline>,
	current_status: &mut PlaybackStatus,
) -> bool {
	match command {
		PlayerCommand::Load(path) => {
			if let Some(pipeline) = current_pipeline.take() {
				let _ = pipeline.set_state(gst::State::Null);
			}

			if !path.exists() {
				push_event(
					event_queue,
					PlayerEvent::Error(format!("文件不存在: {}", path.display())),
				);
				*current_status = PlaybackStatus::Error;
				push_event(event_queue, PlayerEvent::StatusChanged(*current_status));
				return true;
			}

			match create_pipeline(&path, event_queue.clone()) {
				Ok((pipeline, _appsink)) => {
					eprintln!("[media_player] begin startup for {}", path.display());
					match start_pipeline(&pipeline) {
						Ok(()) => {
							eprintln!("[media_player] startup success for {}", path.display());
							*current_status = PlaybackStatus::Playing;
							push_event(event_queue, PlayerEvent::Loaded(path));
							push_event(
								event_queue,
								PlayerEvent::StatusChanged(PlaybackStatus::Playing),
							);
							*current_pipeline = Some(pipeline);
						}
						Err(error) => {
							eprintln!(
								"[media_player] startup failed for {}: {}",
								path.display(),
								error
							);
							let detail = read_pipeline_startup_error(&pipeline)
								.unwrap_or_else(|| format!("状态切换返回错误: {error:?}"));
							let _ = pipeline.set_state(gst::State::Null);
							*current_status = PlaybackStatus::Error;
							push_event(
								event_queue,
								PlayerEvent::Error(format!("播放管线启动失败: {detail}")),
							);
							push_event(
								event_queue,
								PlayerEvent::StatusChanged(PlaybackStatus::Error),
							);
						}
					}
				}
				Err(error) => {
					*current_status = PlaybackStatus::Error;
					push_event(event_queue, PlayerEvent::Error(error.to_string()));
					push_event(event_queue, PlayerEvent::StatusChanged(*current_status));
				}
			}
		}
		PlayerCommand::Play => {
			if let Some(pipeline) = current_pipeline.as_ref() {
				let result = pipeline.set_state(gst::State::Playing);
				if result.is_ok() {
					*current_status = PlaybackStatus::Playing;
					push_event(event_queue, PlayerEvent::StatusChanged(*current_status));
				}
			}
		}
		PlayerCommand::Pause => {
			if let Some(pipeline) = current_pipeline.as_ref() {
				let result = pipeline.set_state(gst::State::Paused);
				if result.is_ok() {
					*current_status = PlaybackStatus::Paused;
					push_event(event_queue, PlayerEvent::StatusChanged(*current_status));
				}
			}
		}
		PlayerCommand::Close => {
			if let Some(pipeline) = current_pipeline.take() {
				let _ = pipeline.set_state(gst::State::Null);
			}
			*current_status = PlaybackStatus::Idle;
			push_event(event_queue, PlayerEvent::Closed);
			push_event(event_queue, PlayerEvent::StatusChanged(*current_status));
		}
		PlayerCommand::Shutdown => {
			if let Some(pipeline) = current_pipeline.take() {
				let _ = pipeline.set_state(gst::State::Null);
			}
			return false;
		}
	}

	true
}

fn drain_bus(
	pipeline: &gst::Pipeline,
	event_queue: &Arc<Mutex<VecDeque<PlayerEvent>>>,
	current_status: &mut PlaybackStatus,
) {
	let Some(bus) = pipeline.bus() else {
		return;
	};

	while let Some(message) = bus.pop() {
		use gst::MessageView;
		match message.view() {
			MessageView::Eos(..) => {
				*current_status = PlaybackStatus::Ended;
				push_event(event_queue, PlayerEvent::StatusChanged(*current_status));
			}
			MessageView::Error(error) => {
				*current_status = PlaybackStatus::Error;
				push_event(
					event_queue,
					PlayerEvent::Error(format!(
						"{} ({})",
						error.error(),
						error.debug().unwrap_or_default()
					)),
				);
				push_event(event_queue, PlayerEvent::StatusChanged(*current_status));
			}
			_ => {}
		}
	}
}

fn push_position_event(pipeline: &gst::Pipeline, event_queue: &Arc<Mutex<VecDeque<PlayerEvent>>>) {
	let position_ms = pipeline
		.query_position::<gst::ClockTime>()
		.map(|position| position.mseconds())
		.unwrap_or(0);
	let duration_ms = pipeline
		.query_duration::<gst::ClockTime>()
		.map(|duration| duration.mseconds());
	push_event(
		event_queue,
		PlayerEvent::PositionUpdated {
			position_ms,
			duration_ms,
		},
	);
}

fn push_event(event_queue: &Arc<Mutex<VecDeque<PlayerEvent>>>, event: PlayerEvent) {
	if let Ok(mut events) = event_queue.lock() {
		events.push_back(event);
	}
}

fn start_pipeline(pipeline: &gst::Pipeline) -> Result<(), MediaPlayerError> {
	let paused = pipeline
		.set_state(gst::State::Paused)
		.map_err(|error| MediaPlayerError::StateChange(format!("切到 Paused 失败: {error:?}")))?;
	eprintln!("[media_player] set_state(Paused) => {:?}", paused);

	let paused_state = pipeline.state(gst::ClockTime::from_seconds(5));
	eprintln!(
		"[media_player] get_state after Paused => result={:?}, current={:?}, pending={:?}",
		paused_state.0, paused_state.1, paused_state.2
	);
	if let Err(error) = paused_state.0 {
		return Err(MediaPlayerError::StateChange(format!(
			"等待 Paused 状态失败: {error:?}, current={:?}, pending={:?}",
			paused_state.1, paused_state.2
		)));
	}

	let playing = pipeline
		.set_state(gst::State::Playing)
		.map_err(|error| MediaPlayerError::StateChange(format!("切到 Playing 失败: {error:?}")))?;
	eprintln!("[media_player] set_state(Playing) => {:?}", playing);

	let playing_state = pipeline.state(gst::ClockTime::from_seconds(5));
	eprintln!(
		"[media_player] get_state after Playing => result={:?}, current={:?}, pending={:?}",
		playing_state.0, playing_state.1, playing_state.2
	);
	if let Err(error) = playing_state.0 {
		return Err(MediaPlayerError::StateChange(format!(
			"等待 Playing 状态失败: {error:?}, current={:?}, pending={:?}",
			playing_state.1, playing_state.2
		)));
	}

	Ok(())
}

fn read_pipeline_startup_error(pipeline: &gst::Pipeline) -> Option<String> {
	let bus = pipeline.bus()?;
	for _ in 0..20 {
		if let Some(message) = bus.timed_pop(gst::ClockTime::from_mseconds(50)) {
			use gst::MessageView;
			match message.view() {
				MessageView::Error(error) => {
					let detail =
						format!("{} ({})", error.error(), error.debug().unwrap_or_default());
					eprintln!("[media_player] startup bus error: {detail}");
					return Some(detail);
				}
				MessageView::Warning(warning) => {
					eprintln!(
						"[media_player] startup bus warning: {} ({})",
						warning.error(),
						warning.debug().unwrap_or_default()
					);
				}
				MessageView::StateChanged(state_changed) => {
					eprintln!(
						"[media_player] startup state changed: {:?} -> {:?} (pending {:?})",
						state_changed.old(),
						state_changed.current(),
						state_changed.pending()
					);
				}
				_ => {}
			}
		}
	}
	None
}

#[cfg(target_os = "windows")]
fn initialize_media_thread_com() -> Option<ComInitializationGuard> {
	let result = unsafe { CoInitializeEx(std::ptr::null(), COINIT_MULTITHREADED) };
	if result >= 0 {
		eprintln!("[media_player] CoInitializeEx succeeded: {}", result);
		Some(ComInitializationGuard)
	} else {
		eprintln!("[media_player] CoInitializeEx failed: {}", result);
		None
	}
}

#[cfg(not(target_os = "windows"))]
fn initialize_media_thread_com() -> Option<ComInitializationGuard> {
	None
}

struct ComInitializationGuard;

#[cfg(target_os = "windows")]
impl Drop for ComInitializationGuard {
	fn drop(&mut self) {
		unsafe {
			CoUninitialize();
		}
	}
}

#[cfg(target_os = "windows")]
const COINIT_MULTITHREADED: u32 = 0x0;

#[cfg(target_os = "windows")]
unsafe extern "system" {
	fn CoInitializeEx(pv_reserved: *const c_void, coinit: u32) -> i32;
	fn CoUninitialize();
}
