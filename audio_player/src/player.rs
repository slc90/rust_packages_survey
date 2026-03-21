use std::{
	collections::VecDeque,
	fs::File,
	path::{Path, PathBuf},
	sync::{
		Arc, Mutex,
		mpsc::{self, Sender, TryRecvError},
	},
	thread::{self, JoinHandle},
	time::Duration,
};

use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source};

use crate::{
	error::AudioPlayerError,
	events::{AudioPlaybackStatus, PlayerCommand, PlayerEvent},
};

/// 音频播放器句柄
pub struct PlayerHandle {
	command_tx: Sender<PlayerCommand>,
	event_queue: Arc<Mutex<VecDeque<PlayerEvent>>>,
	worker: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl PlayerHandle {
	/// 创建新的音频播放线程
	pub fn spawn() -> Result<Self, AudioPlayerError> {
		let mut stream = DeviceSinkBuilder::open_default_sink()
			.map_err(|error| AudioPlayerError::Initialization(error.to_string()))?;
		stream.log_on_drop(false);
		drop(stream);

		let (command_tx, command_rx) = mpsc::channel();
		let event_queue = Arc::new(Mutex::new(VecDeque::new()));
		let queue_for_thread = event_queue.clone();
		let worker = thread::Builder::new()
			.name("audio-player-worker".to_string())
			.spawn(move || {
				let mut active_playback: Option<ActivePlayback> = None;
				let mut current_status = AudioPlaybackStatus::Idle;

				loop {
					loop {
						match command_rx.try_recv() {
							Ok(command) => {
								let keep_running = handle_command(
									command,
									&queue_for_thread,
									&mut active_playback,
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

					update_playback_state(
						&queue_for_thread,
						&mut active_playback,
						&mut current_status,
					);
					thread::sleep(Duration::from_millis(50));
				}
			})
			.map_err(|error| AudioPlayerError::Initialization(error.to_string()))?;

		Ok(Self {
			command_tx,
			event_queue,
			worker: Arc::new(Mutex::new(Some(worker))),
		})
	}

	/// 加载本地音频文件并立即播放
	pub fn load(&self, path: PathBuf) -> Result<(), AudioPlayerError> {
		self.command_tx
			.send(PlayerCommand::Load(path))
			.map_err(|error| AudioPlayerError::Command(error.to_string()))
	}

	/// 播放
	pub fn play(&self) -> Result<(), AudioPlayerError> {
		self.command_tx
			.send(PlayerCommand::Play)
			.map_err(|error| AudioPlayerError::Command(error.to_string()))
	}

	/// 暂停
	pub fn pause(&self) -> Result<(), AudioPlayerError> {
		self.command_tx
			.send(PlayerCommand::Pause)
			.map_err(|error| AudioPlayerError::Command(error.to_string()))
	}

	/// 关闭当前音频
	pub fn close(&self) -> Result<(), AudioPlayerError> {
		self.command_tx
			.send(PlayerCommand::Close)
			.map_err(|error| AudioPlayerError::Command(error.to_string()))
	}

	/// 提取后台线程事件
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

struct ActivePlayback {
	/// 保持底层设备句柄存活，否则音频输出会被系统立即释放。
	_stream: MixerDeviceSink,
	player: Player,
	duration_ms: Option<u64>,
}

fn handle_command(
	command: PlayerCommand,
	event_queue: &Arc<Mutex<VecDeque<PlayerEvent>>>,
	active_playback: &mut Option<ActivePlayback>,
	current_status: &mut AudioPlaybackStatus,
) -> bool {
	match command {
		PlayerCommand::Load(path) => {
			stop_active_playback(active_playback);

			if !path.exists() {
				*current_status = AudioPlaybackStatus::Error;
				push_event(
					event_queue,
					PlayerEvent::Error(AudioPlayerError::FileNotFound(path).to_string()),
				);
				push_event(event_queue, PlayerEvent::StatusChanged(*current_status));
				return true;
			}

			push_event(
				event_queue,
				PlayerEvent::StatusChanged(AudioPlaybackStatus::Loading),
			);

			match create_playback(&path) {
				Ok(playback) => {
					let duration_ms = playback.duration_ms;
					*active_playback = Some(playback);
					*current_status = AudioPlaybackStatus::Playing;
					push_event(event_queue, PlayerEvent::Loaded { path, duration_ms });
					push_event(event_queue, PlayerEvent::StatusChanged(*current_status));
				}
				Err(error) => {
					*current_status = AudioPlaybackStatus::Error;
					push_event(event_queue, PlayerEvent::Error(error.to_string()));
					push_event(event_queue, PlayerEvent::StatusChanged(*current_status));
				}
			}
		}
		PlayerCommand::Play => {
			if let Some(playback) = active_playback.as_ref() {
				playback.player.play();
				*current_status = AudioPlaybackStatus::Playing;
				push_event(event_queue, PlayerEvent::StatusChanged(*current_status));
			}
		}
		PlayerCommand::Pause => {
			if let Some(playback) = active_playback.as_ref() {
				playback.player.pause();
				*current_status = AudioPlaybackStatus::Paused;
				push_event(event_queue, PlayerEvent::StatusChanged(*current_status));
			}
		}
		PlayerCommand::Close => {
			stop_active_playback(active_playback);
			*current_status = AudioPlaybackStatus::Idle;
			push_event(event_queue, PlayerEvent::Closed);
			push_event(event_queue, PlayerEvent::StatusChanged(*current_status));
		}
		PlayerCommand::Shutdown => {
			stop_active_playback(active_playback);
			return false;
		}
	}

	true
}

fn create_playback(path: &Path) -> Result<ActivePlayback, AudioPlayerError> {
	let mut stream = DeviceSinkBuilder::open_default_sink()
		.map_err(|error| AudioPlayerError::Initialization(error.to_string()))?;
	stream.log_on_drop(false);

	let player = Player::connect_new(stream.mixer());
	let file = File::open(path).map_err(|error| AudioPlayerError::FileOpen(error.to_string()))?;
	let decoder =
		Decoder::try_from(file).map_err(|error| AudioPlayerError::Decode(error.to_string()))?;
	let duration_ms = decoder
		.total_duration()
		.map(|duration| duration.as_millis() as u64);
	player.append(decoder);
	player.play();

	Ok(ActivePlayback {
		_stream: stream,
		player,
		duration_ms,
	})
}

fn update_playback_state(
	event_queue: &Arc<Mutex<VecDeque<PlayerEvent>>>,
	active_playback: &mut Option<ActivePlayback>,
	current_status: &mut AudioPlaybackStatus,
) {
	let Some(playback) = active_playback.as_ref() else {
		return;
	};

	let position_ms = playback.player.get_pos().as_millis() as u64;
	push_event(
		event_queue,
		PlayerEvent::PositionUpdated {
			position_ms,
			duration_ms: playback.duration_ms,
		},
	);

	if *current_status != AudioPlaybackStatus::Ended
		&& !playback.player.is_paused()
		&& playback.player.empty()
	{
		*current_status = AudioPlaybackStatus::Ended;
		push_event(event_queue, PlayerEvent::StatusChanged(*current_status));
		*active_playback = None;
	}
}

fn stop_active_playback(active_playback: &mut Option<ActivePlayback>) {
	if let Some(playback) = active_playback.take() {
		playback.player.stop();
	}
}

fn push_event(event_queue: &Arc<Mutex<VecDeque<PlayerEvent>>>, event: PlayerEvent) {
	if let Ok(mut events) = event_queue.lock() {
		events.push_back(event);
	}
}
