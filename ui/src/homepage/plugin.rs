use crate::homepage::about::plugin::AboutPlugin;
use crate::homepage::common::{ChangeFunctionMessage, Functions};
use crate::homepage::realtime_plot::plugin::RealtimePlotPlugin;
use crate::homepage::setting::plugin::SettingPlugin;
use bevy::prelude::*;

/// Plugin for the homepage system
///
/// This plugin provides state management for the homepage content area
/// with About and Setting states that display different UI content below the title bar.
/// It serves as the main plugin that orchestrates all homepage functionality
/// by including both AboutPlugin and SettingPlugin as sub-plugins.
pub struct HomepagePlugin;

impl Plugin for HomepagePlugin {
	fn build(&self, app: &mut App) {
		// Initialize the main state enum for the homepage
		app.init_state::<Functions>();
		// 注册切换功能的消息
		app.add_message::<ChangeFunctionMessage>();
		// Add About and Setting state plugins
		// These plugins handle their own component registration and system setup
		app.add_plugins(AboutPlugin);
		app.add_plugins(RealtimePlotPlugin);
		app.add_plugins(SettingPlugin);
		// 注册systems
		app.add_systems(Update, change_function);
	}
}

/// 接收切换ContentArea内容的消息
fn change_function(
	mut messages: MessageReader<ChangeFunctionMessage>,
	current_state: Res<State<Functions>>,
	mut next_state: ResMut<NextState<Functions>>,
) {
	if messages.is_empty() {
		return;
	}
	for message in messages.read() {
		let current_function = current_state.get();
		let new_function = &message.0;
		if current_function != new_function {
			next_state.set(new_function.clone());
		} else {
			debug!("当前功能和新功能相同:{:?},无需切换", current_function);
		}
	}
}
