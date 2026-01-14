use crate::homepage::common::ContentAreaMarker;
use bevy::prelude::*;

use super::components::TestContentMarker;

// ============================================================================
// TEST STATE SYSTEMS - Lifecycle systems for Test state
// ============================================================================

/// Test状态进入时的系统
pub fn on_enter(mut commands: Commands, query: Query<Entity, With<ContentAreaMarker>>) {
	info!("进入测试页面");

	// 获取内容区域的实体
	if let Ok(content_area) = query.single() {
		// 在内容区域中创建Test界面
		commands.entity(content_area).with_children(|parent| {
			parent.spawn((
				TestContentMarker,
				Node {
					width: Val::Percent(100.0),
					height: Val::Percent(100.0),
					..default()
				},
				Text::new("Test Page\nThis is the Test section"),
				TextColor::BLACK,
			));
		});
	}
}

/// Test状态离开时的系统
pub fn on_exit(mut commands: Commands, query: Query<Entity, With<TestContentMarker>>) {
	info!("离开测试页面");

	// 清理所有Test内容实体
	for entity in query.iter() {
		commands.entity(entity).despawn();
	}
}
