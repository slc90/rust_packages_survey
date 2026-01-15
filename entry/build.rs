fn main() {
	if let Ok(dir) = std::env::var("DEP_PRE_BUILD_CONFIG_DIR") {
		println!("cargo:rustc-env=CONFIG_DIR={dir}");
	} else {
		// fallback 或者 panic 更友好
		println!("cargo:warning=DEP_PRE_BUILD_CONFIG_DIR not present, using default path");
		println!("cargo:rustc-env=CONFIG_DIR=default_config_path");
	}
}
