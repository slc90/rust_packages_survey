use crate::homepage::common::ContentAreaMarker;
use bevy::prelude::*;

use super::components::AboutContentMarker;

// ============================================================================
// ABOUT STATE SYSTEMS - Lifecycle systems for About state
// ============================================================================

/// 进入About页面时触发，需要创建资源
pub fn on_enter(mut commands: Commands, query: Query<Entity, With<ContentAreaMarker>>) {
	info!("进入关于页面");

	// 获取内容区域的实体
	if let Ok(content_area) = query.single() {
		// 在内容区域中创建About界面
		commands.entity(content_area).with_children(|parent| {
			parent.spawn((
				AboutContentMarker,
				Node {
					width: Val::Percent(100.0),
					height: Val::Percent(100.0),
					..default()
				},
				Text::new("About Page\nThis is the About section"),
				TextColor::BLACK,
			));
		});
	}
}

/// 离开About页面时触发，需要清理资源
pub fn on_exit(mut commands: Commands, query: Query<Entity, With<AboutContentMarker>>) {
	info!("离开关于页面");

	// 清理所有About内容实体
	for entity in query.iter() {
		commands.entity(entity).despawn();
	}
}
