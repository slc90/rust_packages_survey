///自动生成常量
macro_rules! embed_with_const {
	($const_name:ident, $path:literal) => {
		pub const $const_name: &str = concat!("embedded://embedded_assets/", $path);
	};
}

embed_with_const!(LOGO, "assets/logo.png");
embed_with_const!(MINIMIZE_ICON, "assets/minimize.png");
embed_with_const!(MAXIMIZE_ICON, "assets/maximize.png");
embed_with_const!(CLOSE_ICON, "assets/close.png");
