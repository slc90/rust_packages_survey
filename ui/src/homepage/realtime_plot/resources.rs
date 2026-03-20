use bevy::prelude::*;

// ============================================================================
// WAVEFORM DATA RESOURCES - Core data structures for waveform storage
// ============================================================================

/// 波形数据资源，存储各通道的波形数据
#[derive(Resource, Debug, Clone)]
pub struct WaveformData {
	/// 各通道的波形数据，每个 Vec<f32> 是一个通道的数据
	pub channels: Vec<Vec<f32>>,
	/// 最大显示点数
	pub max_points: usize,
}

impl Default for WaveformData {
	fn default() -> Self {
		Self::new(1, 4096)
	}
}

impl WaveformData {
	/// 创建新的 WaveformData
	///
	/// # Arguments
	/// * `channel_count` - 通道数量
	/// * `max_points` - 每个通道的最大数据点数
	pub fn new(channel_count: usize, max_points: usize) -> Self {
		let channels = vec![Vec::with_capacity(max_points); channel_count];
		Self {
			channels,
			max_points,
		}
	}

	/// 添加一个数据点到指定通道
	pub fn push(&mut self, channel: usize, value: f32) {
		if channel >= self.channels.len() {
			return;
		}
		let ch = &mut self.channels[channel];
		if ch.len() >= self.max_points {
			// 环形缓冲区：移除最旧的数据
			ch.remove(0);
		}
		ch.push(value);
	}

	/// 添加多个数据点到指定通道
	pub fn push_batch(&mut self, channel: usize, values: &[f32]) {
		for &v in values {
			self.push(channel, v);
		}
	}

	/// 获取指定通道的所有数据
	pub fn get_channel(&self, channel: usize) -> &[f32] {
		if channel >= self.channels.len() {
			return &[];
		}
		&self.channels[channel]
	}

	/// 获取所有通道的数据
	pub fn get_all_channels(&self) -> &[Vec<f32>] {
		&self.channels
	}

	/// 获取通道数量
	pub fn channel_count(&self) -> usize {
		self.channels.len()
	}

	/// 重置所有通道数据
	pub fn clear(&mut self) {
		for ch in &mut self.channels {
			ch.clear();
		}
	}
}

/// 环形缓冲区，用于高效存储滚动数据
#[derive(Resource, Debug, Clone)]
pub struct RingBuffer<T: Clone + Default> {
	/// 存储数据的向量
	data: Vec<T>,
	/// 当前写入位置
	write_pos: usize,
	/// 缓冲区容量
	capacity: usize,
	/// 数据是否已满（覆盖过旧数据）
	is_full: bool,
}

impl<T: Clone + Default> RingBuffer<T> {
	/// 创建新的 RingBuffer
	///
	/// # Arguments
	/// * `capacity` - 缓冲区容量
	pub fn new(capacity: usize) -> Self {
		let data = vec![T::default(); capacity];
		Self {
			data,
			write_pos: 0,
			capacity,
			is_full: false,
		}
	}

	/// 添加元素到缓冲区
	///
	/// 如果缓冲区已满，最旧的数据会被新数据覆盖
	pub fn push(&mut self, item: T) {
		self.data[self.write_pos] = item;
		self.write_pos = (self.write_pos + 1) % self.capacity;
		if self.write_pos == 0 {
			self.is_full = true;
		}
	}

	/// 获取所有数据，按时间顺序排列
	///
	/// 如果缓冲区未满，只返回已写入的数据
	pub fn get_all(&self) -> Vec<T> {
		if !self.is_full && self.write_pos == 0 {
			return vec![];
		}

		let len = if self.is_full {
			self.capacity
		} else {
			self.write_pos
		};

		let mut result = Vec::with_capacity(len);

		if self.is_full {
			// 从 write_pos 开始（最旧的数据）到末尾
			result.extend_from_slice(&self.data[self.write_pos..]);
			// 从开头到 write_pos-1（最新的数据）
			result.extend_from_slice(&self.data[..self.write_pos]);
		} else {
			// 只返回已写入的数据
			result.extend_from_slice(&self.data[..self.write_pos]);
		}

		result
	}

	/// 获取缓冲区容量
	pub fn capacity(&self) -> usize {
		self.capacity
	}

	///获取当前数据长度
	pub fn len(&self) -> usize {
		if self.is_full {
			self.capacity
		} else {
			self.write_pos
		}
	}

	/// 检查缓冲区是否为空
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// 清空缓冲区
	pub fn clear(&mut self) {
		self.write_pos = 0;
		self.is_full = false;
		for item in &mut self.data {
			*item = T::default();
		}
	}
}

// ============================================================================
// MESSAGE TYPES - Communication between UI and data generation
// ============================================================================

/// 波形数据消息，从后台发送到前台
#[derive(Message, Clone, Debug)]
pub struct WaveformDataMessage {
	/// 通道索引
	pub channel: usize,
	/// 数据值
	pub values: Vec<f32>,
}

impl WaveformDataMessage {
	/// 创建新的波形数据消息
	pub fn new(channel: usize, values: Vec<f32>) -> Self {
		Self { channel, values }
	}
}

/// 波形设置消息，从前台发送到后台
#[derive(Message, Clone, Debug, Default)]
pub struct WaveformSettingsMessage {
	/// 通道数量
	pub channel_count: Option<usize>,
	/// 采样率 (Hz)
	pub sample_rate: Option<u32>,
	/// 最大显示点数
	pub max_points: Option<usize>,
}

impl WaveformSettingsMessage {
	/// 设置通道数量
	pub fn with_channel_count(mut self, count: usize) -> Self {
		self.channel_count = Some(count);
		self
	}

	/// 设置采样率
	pub fn with_sample_rate(mut self, rate: u32) -> Self {
		self.sample_rate = Some(rate);
		self
	}

	/// 设置最大显示点数
	pub fn with_max_points(mut self, points: usize) -> Self {
		self.max_points = Some(points);
		self
	}
}

// ============================================================================
// WAVEFORM GENERATOR - Generates random waveform data for testing
// ============================================================================

use rand::Rng;

/// 波形生成器，用于生成模拟波形数据（非 Resource，使用本地随机数生成）
#[derive(Debug, Clone)]
pub struct WaveformGenerator {
	/// 采样率 (Hz)
	pub sample_rate: u32,
	/// 信号幅度
	pub amplitude: f32,
	/// 噪声级别
	pub noise_level: f32,
	/// 基础频率 (Hz)
	pub base_frequency: f32,
	/// 内部时间偏移
	time: f32,
}

impl Default for WaveformGenerator {
	fn default() -> Self {
		Self::new(1000, 100.0, 5.0, 10.0)
	}
}

impl WaveformGenerator {
	/// 创建新的波形生成器
	///
	/// # Arguments
	/// * `sample_rate` - 采样率 (Hz)
	/// * `amplitude` - 信号幅度
	/// * `noise_level` - 噪声级别
	/// * `base_frequency` - 基础频率 (Hz)
	pub fn new(sample_rate: u32, amplitude: f32, noise_level: f32, base_frequency: f32) -> Self {
		Self {
			sample_rate,
			amplitude,
			noise_level,
			base_frequency,
			time: 0.0,
		}
	}

	/// 生成指定数量的波形数据点
	///
	/// # Arguments
	/// * `count` - 生成的数据点数量
	///
	/// # Returns
	/// 生成的波形数据向量
	pub fn generate(&mut self, count: usize) -> Vec<f32> {
		let dt = 1.0 / self.sample_rate as f32;
		let mut rng = rand::rng();

		let start_time = self.time;
		self.time += count as f32 * dt;

		(0..count)
			.map(|i| {
				let t = start_time + i as f32 * dt;
				// 叠加正弦波和均匀分布噪声
				let signal = (t * self.base_frequency * 2.0 * std::f32::consts::PI).sin();
				let noise: f32 = (rng.random::<f32>() - 0.5) * 2.0 * self.noise_level;
				signal * self.amplitude + noise
			})
			.collect()
	}

	/// 生成单点波形数据
	///
	/// # Returns
	/// 生成的波形数据点
	pub fn generate_single(&mut self) -> f32 {
		let dt = 1.0 / self.sample_rate as f32;
		let signal = (self.time * self.base_frequency * 2.0 * std::f32::consts::PI).sin();
		let noise: f32 = (rand::rng().random::<f32>() - 0.5) * 2.0 * self.noise_level;
		self.time += dt;
		signal * self.amplitude + noise
	}

	/// 重置随机数生成器
	pub fn reset(&mut self) {
		self.time = 0.0;
	}

	/// 获取当前时间
	pub fn current_time(&self) -> f32 {
		self.time
	}
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
	use super::*;

	// WaveformGenerator tests
	#[test]
	fn test_waveform_generator_new() {
		let generator = WaveformGenerator::new(1000, 100.0, 5.0, 10.0);
		assert_eq!(generator.sample_rate, 1000);
		assert_eq!(generator.amplitude, 100.0);
		assert_eq!(generator.noise_level, 5.0);
		assert_eq!(generator.base_frequency, 10.0);
	}

	#[test]
	fn test_waveform_generator_generate() {
		let mut generator = WaveformGenerator::default();
		let data = generator.generate(100);
		assert_eq!(data.len(), 100);
		// 检查数据是否在合理范围内（考虑噪声）
		for v in &data {
			assert!(v.abs() <= generator.amplitude + generator.noise_level * 3.0);
		}
	}

	#[test]
	fn test_waveform_generator_single() {
		let mut generator = WaveformGenerator::default();
		let value = generator.generate_single();
		assert!(value.abs() <= generator.amplitude + generator.noise_level * 3.0);
	}

	// WaveformData tests
	#[test]
	fn test_waveform_data_new() {
		let data = WaveformData::new(4, 1024);
		assert_eq!(data.channel_count(), 4);
		assert_eq!(data.max_points, 1024);
		for ch in &data.channels {
			assert!(ch.is_empty());
		}
	}

	#[test]
	fn test_waveform_data_push() {
		let mut data = WaveformData::new(2, 10);
		data.push(0, 1.0);
		data.push(0, 2.0);
		assert_eq!(data.get_channel(0), &[1.0, 2.0]);
	}

	#[test]
	fn test_waveform_data_push_overflow() {
		let mut data = WaveformData::new(1, 5);
		for i in 0..10 {
			data.push(0, i as f32);
		}
		// 应该只保留最后5个: 5,6,7,8,9
		let ch = data.get_channel(0);
		assert_eq!(ch.len(), 5);
		assert_eq!(ch, &[5.0, 6.0, 7.0, 8.0, 9.0]);
	}

	#[test]
	fn test_waveform_data_invalid_channel() {
		let mut data = WaveformData::new(2, 10);
		data.push(5, 1.0); // 无效通道
		assert_eq!(data.get_channel(5).len(), 0);
	}

	// RingBuffer tests
	#[test]
	fn test_ring_buffer_new() {
		let buffer: RingBuffer<i32> = RingBuffer::new(5);
		assert_eq!(buffer.capacity(), 5);
		assert!(buffer.is_empty());
		assert_eq!(buffer.len(), 0);
	}

	#[test]
	fn test_ring_buffer_push() {
		let mut buffer: RingBuffer<i32> = RingBuffer::new(3);
		buffer.push(1);
		buffer.push(2);
		buffer.push(3);

		let items = buffer.get_all();
		assert_eq!(items, &[1, 2, 3]);
		assert_eq!(buffer.len(), 3);
	}

	#[test]
	fn test_ring_buffer_overflow() {
		let mut buffer: RingBuffer<i32> = RingBuffer::new(3);
		buffer.push(1);
		buffer.push(2);
		buffer.push(3);
		buffer.push(4); // 覆盖

		let items = buffer.get_all();
		assert_eq!(items, &[2, 3, 4]); // 按时间顺序: 2是最旧的，4是最新的
		assert_eq!(buffer.len(), 3);
		assert!(buffer.is_full);
	}

	#[test]
	fn test_ring_buffer_clear() {
		let mut buffer: RingBuffer<i32> = RingBuffer::new(3);
		buffer.push(1);
		buffer.push(2);
		buffer.clear();

		assert!(buffer.is_empty());
		assert_eq!(buffer.get_all().len(), 0);
	}

	#[test]
	fn test_ring_buffer_wrap_around() {
		let mut buffer: RingBuffer<i32> = RingBuffer::new(4);
		for i in 1..=6 {
			buffer.push(i);
		}
		// 应该保留最后4个: 3,4,5,6
		let items = buffer.get_all();
		assert_eq!(items.len(), 4);
	}

	// WaveformSettingsMessage tests
	#[test]
	fn test_waveform_settings_message() {
		let msg = WaveformSettingsMessage::default()
			.with_channel_count(8)
			.with_sample_rate(1000)
			.with_max_points(4096);

		assert_eq!(msg.channel_count, Some(8));
		assert_eq!(msg.sample_rate, Some(1000));
		assert_eq!(msg.max_points, Some(4096));
	}
}
