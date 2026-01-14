use crate::menu_bar::components::*;
use bevy::{
	ecs::{
		query::With,
		system::{Commands, Query, ResMut, Single},
	},
	input_focus::InputFocus,
	log::debug,
	prelude::*,
	ui::{BackgroundColor, BorderColor, BoxShadow, OverrideClip},
	ui_widgets::{
		MenuAction, MenuEvent, observe,
		popover::{Popover, PopoverAlign, PopoverPlacement, PopoverSide},
	},
};

/// Handle function menu events
pub fn on_function_menu_event(
	menu_event: On<MenuEvent>,
	q_anchor: Single<(Entity, &Children), With<FunctionMenuMarker>>,
	q_popup: Query<Entity, With<MenuPopupMarker>>,
	mut focus: ResMut<InputFocus>,
	mut commands: Commands,
) {
	let (anchor, children) = q_anchor.into_inner();
	let popup = children.iter().find_map(|c| q_popup.get(c).ok());

	debug!("Function menu action: {:?}", menu_event.action);

	match menu_event.action {
		MenuAction::Open => {
			if popup.is_none() {
				spawn_function_menu(anchor, commands);
			}
		}
		MenuAction::Toggle => match popup {
			Some(popup) => commands.entity(popup).despawn(),
			None => spawn_function_menu(anchor, commands),
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

/// Handle language menu events
pub fn on_language_menu_event(
	menu_event: On<MenuEvent>,
	q_anchor: Single<(Entity, &Children), With<LanguageMenuMarker>>,
	q_popup: Query<Entity, With<MenuPopupMarker>>,
	mut focus: ResMut<InputFocus>,
	mut commands: Commands,
) {
	let (anchor, children) = q_anchor.into_inner();
	let popup = children.iter().find_map(|c| q_popup.get(c).ok());

	debug!("Language menu action: {:?}", menu_event.action);

	match menu_event.action {
		MenuAction::Open => {
			if popup.is_none() {
				spawn_language_menu(anchor, commands);
			}
		}
		MenuAction::Toggle => match popup {
			Some(popup) => commands.entity(popup).despawn(),
			None => spawn_language_menu(anchor, commands),
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

/// Spawn function menu popup with items for About and Test
fn spawn_function_menu(anchor: Entity, mut commands: Commands) {
	let menu = commands
		.spawn((
			Node {
				display: Display::Flex,
				flex_direction: FlexDirection::Column,
				min_height: Val::Px(40.0),
				min_width: Val::Px(150.0),
				border: UiRect::all(Val::Px(1.0)),
				position_type: PositionType::Absolute,
				..default()
			},
			MenuPopupBundle::default(),
			Visibility::Hidden, // Will be visible after positioning
			BorderColor::all(Color::srgb(0.7, 0.7, 0.7)),
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
					FunctionMenuItemBundle::new("About"),
					BackgroundColor(Color::WHITE),
					children![(Text::new("About"), TextColor(Color::BLACK),)],
				),
				(
					FunctionMenuItemBundle::new("Test"),
					BackgroundColor(Color::WHITE),
					children![(Text::new("Test"), TextColor(Color::BLACK),)],
				),
			],
		))
		.id();
	commands.entity(anchor).add_child(menu);
}

/// Spawn language menu popup with items for English and Chinese
fn spawn_language_menu(anchor: Entity, mut commands: Commands) {
	let menu = commands
		.spawn((
			Node {
				display: Display::Flex,
				flex_direction: FlexDirection::Column,
				min_height: Val::Px(40.0),
				min_width: Val::Px(150.0),
				border: UiRect::all(Val::Px(1.0)),
				position_type: PositionType::Absolute,
				..default()
			},
			MenuPopupBundle::default(),
			Visibility::Hidden, // Will be visible after positioning
			BorderColor::all(Color::srgb(0.7, 0.7, 0.7)),
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
					LanguageMenuItemBundle::new("English"),
					BackgroundColor(Color::WHITE),
					children![(Text::new("English"), TextColor(Color::BLACK),)],
				),
				(
					LanguageMenuItemBundle::new("Chinese"),
					BackgroundColor(Color::WHITE),
					children![(Text::new("Chinese"), TextColor(Color::BLACK),)],
				),
			],
		))
		.id();
	commands.entity(anchor).add_child(menu);
}

/// Returns a bundle that can be used as a component within a title bar
/// Creates a complete menu bar with both function and language dropdowns
pub fn build_menu_bar() -> impl Bundle {
	(
		MenuBarBundle::inline(),
		BackgroundColor(Color::srgb(0.95, 0.95, 0.95)),
		children![
			// Function menu button with text
			(
				FunctionMenuButtonBundle::default(),
				observe(on_function_menu_event),
				children![(Text::new("Function"), TextColor(Color::BLACK),)],
			),
			// Language menu button with text
			(
				LanguageMenuButtonBundle::default(),
				observe(on_language_menu_event),
				children![(Text::new("Language"), TextColor(Color::BLACK),)],
			),
		],
	)
}
