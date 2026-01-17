use bevy::{prelude::*, ui::RelativeCursorPosition};

// ============================================================================
// MARKER COMPONENTS - Used for querying and identifying UI elements
// ============================================================================

/// Marker component for the title bar container
/// Used to identify the main title bar entity in queries
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct TitleBarMarker;

/// Marker component for placeholder areas in the title bar
/// Used to identify placeholder entities where custom UI can be inserted
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct TitleBarPlaceholderMarker;

// ============================================================================
// BUSINESS LOGIC COMPONENTS - Core functionality components
// ============================================================================

/// Button type enumeration for title bar control buttons
/// Each variant represents a different window control function
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitleBarButtonEnum {
	/// Minimize window button - reduces window to taskbar/dock
	Minimize,

	/// Maximize window button - expands window to fill screen
	Maximize,

	/// 从最大化中恢复
	Restore,

	/// Close window button - closes the window or application
	Close,
}

/// Component to track previous interaction state for debouncing button clicks
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct PreviousInteraction {
	/// Previous interaction state
	pub interaction: Option<Interaction>,
}

// ============================================================================
// STYLE COMPONENTS - Visual appearance configuration
// ============================================================================

/// Style configuration for the title bar
/// Controls the visual appearance of the title bar
#[derive(Component, Clone, Debug)]
pub struct TitleBarStyle {
	/// Height of the title bar in pixels
	pub height: f32,
}

impl Default for TitleBarStyle {
	fn default() -> Self {
		Self { height: 40.0 }
	}
}

// ============================================================================
// BUNDLES - Pre-configured component collections
// ============================================================================

/// Bundle for spawning a complete title bar
/// Contains all components needed for a functional title bar
#[derive(Bundle)]
pub struct TitleBarBundle {
	/// Base UI node with layout properties
	pub node: Node,

	/// Marker component for identification
	pub title_bar_marker: TitleBarMarker,

	/// 鼠标相对这个的位置
	pub relative_cursor_position: RelativeCursorPosition,

	/// Visual style configuration
	pub style: TitleBarStyle,
}

impl Default for TitleBarBundle {
	fn default() -> Self {
		let style = TitleBarStyle::default();
		Self {
			node: Node {
				width: Val::Percent(100.0),
				height: Val::Px(style.height),
				flex_direction: FlexDirection::Row,
				align_items: AlignItems::Center,
				padding: UiRect::horizontal(Val::Px(10.0)),
				..default()
			},
			title_bar_marker: TitleBarMarker,
			style,
			relative_cursor_position: RelativeCursorPosition::default(),
		}
	}
}

/// Bundle for spawning a title bar control button
/// Contains all components needed for a functional control button
#[derive(Bundle)]
pub struct TitleBarButtonBundle {
	/// Button marker component
	pub button: Button,

	/// Icon
	pub icon: ImageNode,

	/// Node with layout properties
	pub node: Node,

	/// Type of button (minimize/maximize/close)
	pub button_type: TitleBarButtonEnum,

	/// 鼠标相对这个的位置
	pub relative_cursor_position: RelativeCursorPosition,

	/// Previous interaction state for debouncing clicks
	pub previous_interaction: PreviousInteraction,
}

impl TitleBarButtonBundle {
	/// Creates a new TitleBarButtonBundle for a specific button type
	pub fn new(button_type: TitleBarButtonEnum, size: f32, image_handle: Handle<Image>) -> Self {
		Self {
			button: Button,
			node: Node {
				width: Val::Px(size),
				height: Val::Px(size),
				margin: UiRect::right(Val::Px(8.0)),
				..default()
			},
			button_type,
			icon: ImageNode::new(image_handle),
			relative_cursor_position: RelativeCursorPosition::default(),
			previous_interaction: PreviousInteraction::default(),
		}
	}
}

/// Bundle for spawning title text
#[derive(Bundle)]
pub struct TitleBarTextBundle {
	/// Text component for rendering
	pub text: Text,
}

impl TitleBarTextBundle {
	/// Creates a new TitleBarTextBundle with the given text
	pub fn new(text_content: impl Into<String>) -> Self {
		Self {
			text: Text::new(text_content),
		}
	}
}

/// Bundle for spawning application logo
#[derive(Bundle)]
pub struct TitleBarLogoBundle {
	/// SVG component for displaying the logo
	pub icon: ImageNode,

	/// Node with layout properties
	pub node: Node,
}

impl TitleBarLogoBundle {
	/// Creates a new TitleBarLogoBundle with the given SVG
	pub fn new(image_handle: Handle<Image>, size: f32) -> Self {
		Self {
			icon: ImageNode::new(image_handle),
			node: Node {
				width: Val::Px(size),
				height: Val::Px(size),
				margin: UiRect::right(Val::Px(8.0)),
				..default()
			},
		}
	}
}

/// Bundle for spawning a placeholder area in the title bar
/// This provides a flexible container where custom UI components can be inserted
#[derive(Bundle)]
pub struct TitleBarPlaceholderBundle {
	/// Node with layout properties
	pub node: Node,

	/// Marker component for identification
	pub placeholder_marker: TitleBarPlaceholderMarker,
}

impl Default for TitleBarPlaceholderBundle {
	fn default() -> Self {
		Self {
			node: Node {
				flex_grow: 0.0,
				..default()
			},
			placeholder_marker: TitleBarPlaceholderMarker,
		}
	}
}

impl TitleBarPlaceholderBundle {
	/// Creates a new TitleBarPlaceholderBundle with custom layout
	pub fn with_layout(node: Node) -> Self {
		Self {
			node,
			placeholder_marker: TitleBarPlaceholderMarker,
		}
	}

	/// Creates a new TitleBarPlaceholderBundle that grows to fill available space
	pub fn flexible() -> Self {
		Self {
			node: Node {
				flex_grow: 1.0,
				..default()
			},
			placeholder_marker: TitleBarPlaceholderMarker,
		}
	}

	/// Creates a new TitleBarPlaceholderBundle with fixed width
	pub fn with_width(width: Val) -> Self {
		Self {
			node: Node { width, ..default() },
			placeholder_marker: TitleBarPlaceholderMarker,
		}
	}

	/// Creates a new TitleBarPlaceholderBundle with fixed size
	pub fn with_size(width: Val, height: Val) -> Self {
		Self {
			node: Node {
				width,
				height,
				..default()
			},
			placeholder_marker: TitleBarPlaceholderMarker,
		}
	}
}
