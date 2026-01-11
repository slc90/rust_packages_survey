use crate::title_bar::{
	components::{PreviousInteraction, TitleBarButtonEnum, TitleBarMarker},
	resources::{LeftClickAction, TitleBarButtonCooldown, WindowState},
};
use bevy::{
	app::AppExit,
	ecs::{
		query::With,
		system::{Query, Res, ResMut, Single},
	},
	input::{ButtonInput, mouse::MouseButton},
	log::{debug, info},
	prelude::{MessageReader, MessageWriter},
	time::Time,
	ui::{Interaction, RelativeCursorPosition, widget::Button},
	window::{PrimaryWindow, Window, WindowFocused},
};

/// Handles dragging and moving windows when clicking on the title bar
pub fn drag_and_move_window(
	mut windows: Query<&mut Window>,
	action: Res<LeftClickAction>,
	input: Res<ButtonInput<MouseButton>>,
	cursor_relative_to_title_bar: Query<&RelativeCursorPosition, With<TitleBarMarker>>,
	cursor_relative_to_buttons: Query<&RelativeCursorPosition, With<Button>>,
	mut buttons: Query<&mut TitleBarButtonEnum>,
) {
	// Both `start_drag_move()` and `start_drag_resize()` must be called after a
	// left mouse button press as done here.
	//
	// winit 0.30.5 may panic when initiated without a left mouse button press.
	if !input.just_pressed(MouseButton::Left) {
		return;
	}
	for mut window in windows.iter_mut() {
		match *action {
			LeftClickAction::Move => {
				for relative_cursor_position in cursor_relative_to_buttons {
					// 如果鼠标是点在按钮上的，就不要拖动
					if relative_cursor_position.cursor_over() {
						return;
					}
				}
				for relative_cursor_position in &cursor_relative_to_title_bar {
					if relative_cursor_position.cursor_over() {
						for mut button_type in buttons.iter_mut() {
							// 在已经最大化时拖动，要先改变按钮状态
							if *button_type == TitleBarButtonEnum::Restore {
								*button_type = TitleBarButtonEnum::Maximize
							}
						}
						debug!("开始拖动窗口");
						window.start_drag_move();
					}
				}
			}
		}
	}
}

/// Handles clicks on title bar control buttons
pub fn handle_button_clicks(
	mut app_exit_writer: MessageWriter<AppExit>,
	mut buttons: Query<(
		&Interaction,
		&mut TitleBarButtonEnum,
		&mut PreviousInteraction,
	)>,
	mut window: Single<&mut Window, With<PrimaryWindow>>,
	mut window_state: ResMut<WindowState>,
	mut cooldown: ResMut<TitleBarButtonCooldown>,
	time: Res<Time>,
) {
	// Update the cooldown timer
	cooldown.timer.tick(time.delta());

	// If timer is still running, skip button clicks
	if !cooldown.timer.is_finished() {
		return;
	}

	for (interaction, mut button_type, mut prev_interaction) in buttons.iter_mut() {
		// Check if this is a new press (previously not pressed)
		let is_new_press = *interaction == Interaction::Pressed
			&& prev_interaction.interaction != Some(Interaction::Pressed);

		// Update previous interaction state
		prev_interaction.interaction = Some(*interaction);

		if !is_new_press {
			continue;
		}
		// Reset the cooldown timer when a button is pressed
		cooldown.timer.reset();

		match *button_type {
			TitleBarButtonEnum::Close => {
				// Send application exit event
				info!("点击关闭按钮");
				app_exit_writer.write(AppExit::Success);
			}
			TitleBarButtonEnum::Minimize => {
				info!("点击最小化按钮");
				window.set_minimized(true);
				window_state.was_minimized = true;
			}
			TitleBarButtonEnum::Maximize => {
				info!("点击最大化按钮");
				window.set_maximized(true);
				*button_type = TitleBarButtonEnum::Restore;
			}
			TitleBarButtonEnum::Restore => {
				info!("点击恢复按钮");
				window.set_maximized(false);
				*button_type = TitleBarButtonEnum::Maximize;
			}
		}
	}
}

/// Handles window visibility and focus changes to ensure window gets focus when restored
pub fn handle_window_visibility(
	mut window_state: ResMut<WindowState>,
	mut focus_events: MessageReader<WindowFocused>,
	mut mouse_input: ResMut<ButtonInput<MouseButton>>,
) {
	// Handle window focus events
	for event in focus_events.read() {
		if event.focused {
			debug!("窗口获得焦点:{}", window_state.was_minimized);
			// If window was minimized and now has focus, ensure it's visible
			if window_state.was_minimized {
				// Restore window from minimized state
				window_state.was_minimized = false;
				mouse_input.reset(MouseButton::Left);
				debug!("重置鼠标状态");
			}
		} else {
			debug!("窗口失去焦点");
			// We can't directly check if window is minimized in Bevy 0.17.3
			// We'll track this through the minimize button click instead
		}
	}
}
