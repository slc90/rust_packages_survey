use bevy::{
	color::palettes::basic::*,
	input_focus::{InputDispatchPlugin, InputFocus, tab_navigation::TabNavigationPlugin},
	picking::hover::Hovered,
	prelude::*,
	ui::{InteractionDisabled, Pressed},
	ui_widgets::{
		MenuAction, MenuButton, MenuEvent, MenuItem, MenuPopup, UiWidgetsPlugins, observe,
		popover::{Popover, PopoverAlign, PopoverPlacement, PopoverSide},
	},
};

fn main() {
	App::new()
		.add_plugins((
			DefaultPlugins,
			UiWidgetsPlugins,
			InputDispatchPlugin,
			TabNavigationPlugin,
		))
		.add_systems(Startup, setup)
		.add_systems(Update, (update_menu_item_style, update_menu_item_style2))
		.run();
}

/// Menu anchor marker
#[derive(Component)]
struct DemoMenuAnchor;

/// Menu button styling marker
#[derive(Component)]
struct DemoMenuButton;

/// Menu item styling marker
#[derive(Component)]
struct DemoMenuItem;

fn setup(mut commands: Commands) {
	// ui camera
	commands.spawn(Camera2d);
	commands.spawn((
		Node {
			width: percent(100),
			height: percent(100),
			align_items: AlignItems::Center,
			justify_content: JustifyContent::Center,
			display: Display::Flex,
			flex_direction: FlexDirection::Column,
			row_gap: px(10),
			..default()
		},
		children![menu_button(),],
	));
}

fn menu_button() -> impl Bundle {
	(
		Node { ..default() },
		DemoMenuAnchor,
		observe(on_menu_event),
		children![(
			Node {
				width: px(200),
				height: px(65),
				border: UiRect::all(px(5)),
				box_sizing: BoxSizing::BorderBox,
				justify_content: JustifyContent::SpaceBetween,
				align_items: AlignItems::Center,
				padding: UiRect::axes(px(16), px(0)),
				border_radius: BorderRadius::all(px(5)),
				..default()
			},
			DemoMenuButton,
			MenuButton,
			Hovered::default(),
			BorderColor::all(Color::BLACK),
			BackgroundColor(NORMAL_BUTTON),
			children![
				(
					Text::new("Menu"),
					TextColor(Color::srgb(0.9, 0.9, 0.9)),
					TextShadow::default(),
				),
				(
					Node {
						width: px(12),
						height: px(12),
						..default()
					},
					BackgroundColor(GRAY.into()),
				)
			],
		)],
	)
}

fn on_menu_event(
	menu_event: On<MenuEvent>,
	q_anchor: Single<(Entity, &Children), With<DemoMenuAnchor>>,
	q_popup: Query<Entity, With<MenuPopup>>,
	mut focus: ResMut<InputFocus>,
	mut commands: Commands,
) {
	let (anchor, children) = q_anchor.into_inner();
	let popup = children.iter().find_map(|c| q_popup.get(c).ok());
	info!("Menu action: {:?}", menu_event.action);
	match menu_event.action {
		MenuAction::Open => {
			if popup.is_none() {
				spawn_menu(anchor, commands);
			}
		}
		MenuAction::Toggle => match popup {
			Some(popup) => commands.entity(popup).despawn(),
			None => spawn_menu(anchor, commands),
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

fn spawn_menu(anchor: Entity, mut commands: Commands) {
	let menu = commands
		.spawn((
			Node {
				display: Display::Flex,
				flex_direction: FlexDirection::Column,
				min_height: px(10.),
				min_width: Val::Percent(100.),
				border: UiRect::all(px(1)),
				position_type: PositionType::Absolute,
				..default()
			},
			MenuPopup::default(),
			Visibility::Hidden, // Will be visible after positioning
			BorderColor::all(GREEN),
			BackgroundColor(GRAY.into()),
			BoxShadow::new(
				Srgba::BLACK.with_alpha(0.9).into(),
				px(0),
				px(0),
				px(1),
				px(4),
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
			children![menu_item(), menu_item(), menu_item(), menu_item()],
		))
		.id();
	commands.entity(anchor).add_child(menu);
}

fn menu_item() -> impl Bundle {
	(
		Node {
			padding: UiRect::axes(px(8), px(2)),
			justify_content: JustifyContent::Center,
			align_items: AlignItems::Start,
			..default()
		},
		DemoMenuItem,
		MenuItem,
		Hovered::default(),
		BackgroundColor(NORMAL_BUTTON),
		children![(
			Text::new("Menu Item"),
			TextColor(Color::srgb(0.9, 0.9, 0.9)),
			TextShadow::default(),
		)],
	)
}

fn update_menu_item_style(
	mut buttons: Query<
		(
			Has<Pressed>,
			&Hovered,
			Has<InteractionDisabled>,
			&mut BackgroundColor,
		),
		(
			Or<(
				Changed<Pressed>,
				Changed<Hovered>,
				Added<InteractionDisabled>,
			)>,
			With<DemoMenuItem>,
		),
	>,
) {
	for (pressed, hovered, disabled, mut color) in &mut buttons {
		set_menu_item_style(disabled, hovered.get(), pressed, &mut color);
	}
}

/// Supplementary system to detect removed marker components
fn update_menu_item_style2(
	mut buttons: Query<
		(
			Has<Pressed>,
			&Hovered,
			Has<InteractionDisabled>,
			&mut BackgroundColor,
		),
		With<DemoMenuItem>,
	>,
	mut removed_depressed: RemovedComponents<Pressed>,
	mut removed_disabled: RemovedComponents<InteractionDisabled>,
) {
	removed_depressed
		.read()
		.chain(removed_disabled.read())
		.for_each(|entity| {
			if let Ok((pressed, hovered, disabled, mut color)) = buttons.get_mut(entity) {
				set_menu_item_style(disabled, hovered.get(), pressed, &mut color);
			}
		});
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

fn set_menu_item_style(disabled: bool, hovered: bool, pressed: bool, color: &mut BackgroundColor) {
	match (disabled, hovered, pressed) {
		// Pressed and hovered menu item
		(false, true, true) => {
			*color = PRESSED_BUTTON.into();
		}

		// Hovered, unpressed menu item
		(false, true, false) => {
			*color = HOVERED_BUTTON.into();
		}

		// Unhovered menu item (either pressed or not).
		_ => {
			*color = NORMAL_BUTTON.into();
		}
	}
}
