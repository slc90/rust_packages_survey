use bevy::prelude::*;

// ============================================================================
// REALTIME_PLOT STATE SYSTEMS - Lifecycle systems for RealtimePlot state
// ============================================================================

/// 进入RealtimePlot页面时触发，创建波形可视化资源
pub fn on_enter() {
	info!("进入实时波形绘制页面");
}

/// 离开RealtimePlot页面时触发，清理资源
pub fn on_exit() {
	info!("离开实时波形绘制页面");
}
