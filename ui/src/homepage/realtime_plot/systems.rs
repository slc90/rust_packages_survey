use bevy::prelude::*;

use crate::homepage::realtime_plot::resources::WaveformData;

// ============================================================================
// REALTIME_PLOT STATE SYSTEMS - Lifecycle systems for RealtimePlot state
// ============================================================================

/// 进入RealtimePlot页面时触发，创建波形可视化资源
pub fn on_enter(mut commands: Commands) {
	info!("进入实时波形绘制页面");
	// 初始化波形数据资源，默认1通道，4096点
	let waveform_data = WaveformData::new(1, 4096);
	commands.insert_resource(waveform_data);
}

/// 离开RealtimePlot页面时触发，清理资源
pub fn on_exit(mut commands: Commands) {
	info!("离开实时波形绘制页面");
	// 移除波形数据资源
	commands.remove_resource::<WaveformData>();
}
