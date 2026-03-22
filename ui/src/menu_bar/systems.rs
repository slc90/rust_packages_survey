use crate::{
	homepage::common::{ChangeFunctionMessage, Functions},
	menu_bar::components::*,
};
use bevy::{
	ecs::{
		query::With,
		system::{Commands, Query, ResMut, Single},
	},
	input_focus::InputFocus,
	prelude::*,
	ui::{BoxShadow, OverrideClip},
	ui_widgets::{
		Activate, MenuAction, MenuEvent, observe,
		popover::{Popover, PopoverAlign, PopoverPlacement, PopoverSide},
	},
};
use i18n::{LanguageManager, data_structure::LanguageKey};

/// Handle function menu events
pub fn on_function_menu_event(
	menu_event: On<MenuEvent>,
	q_anchor: Single<(Entity, &Children), With<FunctionMenuMarker>>,
	q_popup: Query<Entity, With<MenuPopupMarker>>,
	mut focus: ResMut<InputFocus>,
	mut commands: Commands,
	language_manager: Res<LanguageManager>,
) {
	let (anchor, children) = q_anchor.into_inner();
	let popup = children.iter().find_map(|c| q_popup.get(c).ok());
	match menu_event.action {
		MenuAction::Open => {
			if popup.is_none() {
				spawn_function_menu(anchor, commands, language_manager);
			}
		}
		MenuAction::Toggle => match popup {
			Some(popup) => commands.entity(popup).despawn(),
			None => spawn_function_menu(anchor, commands, language_manager),
		},
		MenuAction::Close | MenuAction::CloseAll => {
			if let Some(popup) = popup {
				commands.entity(popup).despawn();
			}
		}
		MenuAction::FocusRoot => {
			focus.0 = Some(anchor);
		}
	}
}

/// Spawn function menu popup with items for About and Setting
fn spawn_function_menu(
	anchor: Entity,
	mut commands: Commands,
	language_manager: Res<LanguageManager>,
) {
	let menu = commands
		.spawn((
			MenuPopupBundle::default(),
			Visibility::Hidden, // Will be visible after positioning
			BoxShadow::new(
				Color::srgb(0.0, 0.0, 0.0).with_alpha(0.9),
				Val::Px(0.0),
				Val::Px(0.0),
				Val::Px(1.0),
				Val::Px(4.0),
			),
			GlobalZIndex(100),
			Popover {
				positions: vec![
					PopoverPlacement {
						side: PopoverSide::Bottom,
						align: PopoverAlign::Start,
						gap: 2.0,
					},
					PopoverPlacement {
						side: PopoverSide::Top,
						align: PopoverAlign::Start,
						gap: 2.0,
					},
				],
				window_margin: 10.0,
			},
			OverrideClip,
			children![
				(
					FunctionMenuItemBundle::default(),
					observe(
						|_activated: On<Activate>,
						 mut writer: MessageWriter<ChangeFunctionMessage>| {
							debug!("点击关于按钮");
							writer.write(ChangeFunctionMessage(Functions::About));
						}
					),
					children![(
						Text::new(language_manager.lookup(LanguageKey::About)),
						TextColor(Color::BLACK),
					)],
				),
				(
					FunctionMenuItemBundle::default(),
					observe(
						|_activated: On<Activate>,
						 mut writer: MessageWriter<ChangeFunctionMessage>| {
							debug!("点击设置按钮");
							writer.write(ChangeFunctionMessage(Functions::Setting));
						}
					),
					children![(
						Text::new(language_manager.lookup(LanguageKey::Setting)),
						TextColor(Color::BLACK),
					)],
				),
				(
					FunctionMenuItemBundle::default(),
					observe(
						|_activated: On<Activate>,
						 mut writer: MessageWriter<ChangeFunctionMessage>| {
							debug!("点击实时波形按钮");
							writer.write(ChangeFunctionMessage(Functions::RealtimePlot));
						}
					),
					children![(
						Text::new(language_manager.lookup(LanguageKey::RealtimePlot)),
						TextColor(Color::BLACK),
					)],
				),
				(
					FunctionMenuItemBundle::default(),
					observe(
						|_activated: On<Activate>,
						 mut writer: MessageWriter<ChangeFunctionMessage>| {
							debug!("点击播放音频按钮");
							writer.write(ChangeFunctionMessage(Functions::AudioPlayer));
						}
					),
					children![(
						Text::new(language_manager.lookup(LanguageKey::AudioPlayer)),
						TextColor(Color::BLACK),
					)],
				),
				(
					FunctionMenuItemBundle::default(),
					observe(
						|_activated: On<Activate>,
						 mut writer: MessageWriter<ChangeFunctionMessage>| {
							debug!("点击回放波形按钮");
							writer.write(ChangeFunctionMessage(Functions::PlaybackPlot));
						}
					),
					children![(
						Text::new(language_manager.lookup(LanguageKey::PlaybackPlot)),
						TextColor(Color::BLACK),
					)],
				),
				(
					FunctionMenuItemBundle::default(),
					observe(
						|_activated: On<Activate>,
						 mut writer: MessageWriter<ChangeFunctionMessage>| {
							debug!("点击医学影像按钮");
							writer.write(ChangeFunctionMessage(Functions::MedicalImage));
						}
					),
					children![(
						Text::new(language_manager.lookup(LanguageKey::MedicalImage)),
						TextColor(Color::BLACK),
					)],
				),
				(
					FunctionMenuItemBundle::default(),
					observe(
						|_activated: On<Activate>,
						 mut writer: MessageWriter<ChangeFunctionMessage>| {
							debug!("点击播放视频按钮");
							writer.write(ChangeFunctionMessage(Functions::VideoPlayer));
						}
					),
					children![(
						Text::new(language_manager.lookup(LanguageKey::VideoPlayer)),
						TextColor(Color::BLACK),
					)],
				),
				(
					FunctionMenuItemBundle::default(),
					observe(
						|_activated: On<Activate>,
						 mut writer: MessageWriter<ChangeFunctionMessage>| {
							debug!("点击截图测试按钮");
							writer.write(ChangeFunctionMessage(Functions::Screenshot));
						}
					),
					children![(
						Text::new(language_manager.lookup(LanguageKey::Screenshot)),
						TextColor(Color::BLACK),
					)],
				),
			],
		))
		.id();
	commands.entity(anchor).add_child(menu);
}

/// Returns a bundle that can be used as a component within a title bar
/// Creates a complete menu bar with both function and language dropdowns
pub fn build_menu_bar(language_manager: Res<LanguageManager>) -> impl Bundle {
	(
		MenuBarBundle::default(),
		children![
			// Function menu button with text
			(
				FunctionMenuButtonBundle::default(),
				observe(on_function_menu_event),
				children![(
					Text::new(language_manager.lookup(LanguageKey::Function)),
					TextColor(Color::BLACK),
				)],
			)
		],
	)
}
