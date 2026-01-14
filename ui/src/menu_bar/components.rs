use bevy::prelude::*;
use bevy::ui_widgets::{MenuButton, MenuItem, MenuPopup};

// ============================================================================
// MARKER COMPONENTS - Used for querying and identifying UI elements
// ============================================================================

/// Marker component for the menu bar container
/// Used to identify the main menu bar entity in queries
#[derive(Component, Default, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct MenuBarMarker;

/// Marker component for function menu button
/// Used to identify the function selection menu button
#[derive(Component, Default, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct FunctionMenuMarker;

/// Marker component for language menu button
/// Used to identify the language selection menu button
#[derive(Component, Default, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct LanguageMenuMarker;

/// Marker component for menu popup
/// Used to identify menu popup entities
#[derive(Component, Default, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct MenuPopupMarker;

/// Marker component for menu items
/// Used to identify individual items within a menu
#[derive(Component, Default, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct MenuItemMarker;

/// Marker component for function menu items
/// Used to identify function-specific menu items
#[derive(Component, Default, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct FunctionMenuItemMarker;

/// Marker component for language menu items
/// Used to identify language-specific menu items
#[derive(Component, Default, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct LanguageMenuItemMarker;

// ============================================================================
// STYLE COMPONENTS - Visual appearance configuration
// ============================================================================

/// Style configuration for the menu bar
/// Controls the visual appearance of the menu bar
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct MenuBarStyle {
	/// Height of the menu bar in pixels
	pub height: f32,

	/// Background color of the menu bar
	pub background_color: Color,

	/// Spacing between menu items in pixels
	pub spacing: f32,

	/// Padding around the menu bar content
	pub padding: UiRect,
}

impl Default for MenuBarStyle {
	fn default() -> Self {
		Self {
			height: 40.0,
			background_color: Color::srgb(0.95, 0.95, 0.95),
			spacing: 8.0,
			padding: UiRect::all(Val::Px(8.0)),
		}
	}
}

/// Style configuration for menu buttons
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct MenuButtonStyle {
	/// Background color of the button
	pub background_color: Color,

	/// Text color
	pub text_color: Color,

	/// Padding inside the button
	pub padding: UiRect,

	/// Border radius for rounded corners
	pub border_radius: BorderRadius,
}

impl Default for MenuButtonStyle {
	fn default() -> Self {
		Self {
			background_color: Color::srgb(0.85, 0.85, 0.85),
			text_color: Color::BLACK,
			padding: UiRect::all(Val::Px(8.0)),
			border_radius: BorderRadius::all(Val::Px(4.0)),
		}
	}
}

/// Style configuration for menu popups
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct MenuPopupStyle {
	/// Background color of the menu popup
	pub background_color: Color,

	/// Border color
	pub border_color: Color,

	/// Border width for all sides
	pub border: UiRect,

	/// Padding inside the menu popup
	pub padding: UiRect,

	/// Border radius for rounded corners
	pub border_radius: BorderRadius,
}

impl Default for MenuPopupStyle {
	fn default() -> Self {
		Self {
			background_color: Color::WHITE,
			border_color: Color::srgb(0.7, 0.7, 0.7),
			border: UiRect::all(Val::Px(1.0)),
			padding: UiRect::all(Val::Px(4.0)),
			border_radius: BorderRadius::all(Val::Px(4.0)),
		}
	}
}

/// Style configuration for menu items
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct MenuItemStyle {
	/// Background color of the menu item
	pub background_color: Color,

	/// Background color when hovered
	pub hover_background_color: Color,

	/// Text color
	pub text_color: Color,

	/// Padding inside the menu item
	pub padding: UiRect,

	/// Border radius for rounded corners
	pub border_radius: BorderRadius,
}

impl Default for MenuItemStyle {
	fn default() -> Self {
		Self {
			background_color: Color::NONE,
			hover_background_color: Color::srgb(0.9, 0.9, 0.9),
			text_color: Color::BLACK,
			padding: UiRect::all(Val::Px(8.0)),
			border_radius: BorderRadius::all(Val::Px(2.0)),
		}
	}
}

// ============================================================================
// BUNDLES - Pre-configured component collections
// ============================================================================

/// Bundle for spawning a complete menu bar
/// Contains all components needed for a functional menu bar
#[derive(Bundle)]
pub struct MenuBarBundle {
	/// Base UI node with layout properties
	pub node: Node,

	/// Marker component for identification
	pub menu_bar_marker: MenuBarMarker,

	/// Visual style configuration
	pub style: MenuBarStyle,
}

impl Default for MenuBarBundle {
	fn default() -> Self {
		let style = MenuBarStyle::default();
		Self {
			node: Node {
				width: Val::Percent(100.0),
				height: Val::Px(style.height),
				flex_direction: FlexDirection::Row,
				align_items: AlignItems::Center,
				padding: style.padding,
				column_gap: Val::Px(style.spacing),
				position_type: PositionType::Absolute,
				top: Val::Px(40.0),
				..default()
			},
			menu_bar_marker: MenuBarMarker,
			style,
		}
	}
}

/// Bundle for spawning a menu button
/// Contains all components needed for a functional menu button using Bevy's MenuButton
#[derive(Bundle)]
pub struct FunctionMenuButtonBundle {
	/// Base UI node with layout properties
	pub node: Node,

	/// Bevy MenuButton component for menu functionality
	pub menu_button: MenuButton,

	/// Marker component for function menu
	pub function_marker: FunctionMenuMarker,

	/// Background color component
	pub background_color: BackgroundColor,

	/// Visual style configuration
	pub style: MenuButtonStyle,
}

impl Default for FunctionMenuButtonBundle {
	fn default() -> Self {
		let style = MenuButtonStyle::default();
		Self {
			node: Node {
				padding: style.padding,
				border_radius: style.border_radius,
				..default()
			},
			menu_button: MenuButton,
			function_marker: FunctionMenuMarker,
			background_color: BackgroundColor(style.background_color),
			style,
		}
	}
}

/// Bundle for spawning a language menu button
#[derive(Bundle)]
pub struct LanguageMenuButtonBundle {
	/// Base UI node with layout properties
	pub node: Node,

	/// Bevy MenuButton component for menu functionality
	pub menu_button: MenuButton,

	/// Marker component for language menu
	pub language_marker: LanguageMenuMarker,

	/// Background color component
	pub background_color: BackgroundColor,

	/// Visual style configuration
	pub style: MenuButtonStyle,
}

impl Default for LanguageMenuButtonBundle {
	fn default() -> Self {
		let style = MenuButtonStyle::default();
		Self {
			node: Node {
				padding: style.padding,
				border_radius: style.border_radius,
				..default()
			},
			menu_button: MenuButton,
			language_marker: LanguageMenuMarker,
			background_color: BackgroundColor(style.background_color),
			style,
		}
	}
}

/// Bundle for spawning a menu popup
#[derive(Bundle)]
pub struct MenuPopupBundle {
	/// Base UI node with layout properties
	pub node: Node,

	/// Bevy MenuPopup component for popup functionality
	pub menu_popup: MenuPopup,

	/// Marker component for menu popup
	pub popup_marker: MenuPopupMarker,

	/// Background color component
	pub background_color: BackgroundColor,

	/// Border color component
	pub border_color: BorderColor,

	/// Visual style configuration
	pub style: MenuPopupStyle,
}

impl Default for MenuPopupBundle {
	/// Creates a new MenuPopupBundle
	fn default() -> Self {
		let style = MenuPopupStyle::default();
		Self {
			node: Node {
				flex_direction: FlexDirection::Column,
				padding: style.padding,
				border: style.border,
				border_radius: style.border_radius,
				position_type: PositionType::Absolute,
				..default()
			},
			menu_popup: MenuPopup::default(),
			popup_marker: MenuPopupMarker,
			background_color: BackgroundColor(style.background_color),
			border_color: BorderColor::all(style.border_color),
			style,
		}
	}
}

/// Bundle for spawning a function menu item
#[derive(Bundle)]
pub struct FunctionMenuItemBundle {
	/// Base UI node with layout properties
	pub node: Node,

	/// Bevy MenuItem component for menu item functionality
	pub menu_item: MenuItem,

	/// Marker component for menu item
	pub item_marker: MenuItemMarker,

	/// Marker component for function menu item
	pub function_item_marker: FunctionMenuItemMarker,

	/// Background color component
	pub background_color: BackgroundColor,

	/// Visual style configuration
	pub style: MenuItemStyle,
}

impl FunctionMenuItemBundle {
	/// Creates a new FunctionMenuItemBundle with the given label
	pub fn new(_label: impl Into<String>) -> Self {
		let style = MenuItemStyle::default();
		Self {
			node: Node {
				padding: style.padding,
				border_radius: style.border_radius,
				..default()
			},
			menu_item: MenuItem,
			item_marker: MenuItemMarker,
			function_item_marker: FunctionMenuItemMarker,
			background_color: BackgroundColor(style.background_color),
			style,
		}
	}
}

/// Bundle for spawning a language menu item
#[derive(Bundle)]
pub struct LanguageMenuItemBundle {
	/// Base UI node with layout properties
	pub node: Node,

	/// Bevy MenuItem component for menu item functionality
	pub menu_item: MenuItem,

	/// Marker component for menu item
	pub item_marker: MenuItemMarker,

	/// Marker component for language menu item
	pub language_item_marker: LanguageMenuItemMarker,

	/// Background color component
	pub background_color: BackgroundColor,

	/// Visual style configuration
	pub style: MenuItemStyle,
}

impl LanguageMenuItemBundle {
	/// Creates a new LanguageMenuItemBundle with the given label
	pub fn new(_label: impl Into<String>) -> Self {
		let style = MenuItemStyle::default();
		Self {
			node: Node {
				padding: style.padding,
				border_radius: style.border_radius,
				..default()
			},
			menu_item: MenuItem,
			item_marker: MenuItemMarker,
			language_item_marker: LanguageMenuItemMarker,
			background_color: BackgroundColor(style.background_color),
			style,
		}
	}
}
