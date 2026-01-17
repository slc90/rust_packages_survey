use std::path::Path;

// 在entry这个package编译前把配置文件复制到目标目录
fn main() {
	// println!("cargo::warning=entry build.rs");
	let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
	let config_path = workspace_root
		.join("config")
		.join("config_file")
		.join("config.json");
	// println!("cargo::warning=config_path:{:?}", config_path);
	// 获取编译 profile
	let profile = std::env::var("PROFILE");
	match profile {
		Ok(profile) => {
			// println!("cargo::warning=profile:{:?}", profile);
			let target_dir = workspace_root
				.join("target")
				.join(&profile)
				.join("config_file")
				.join("config.json");
			// println!("cargo::warning=target_dir:{:?}", target_dir);
			// 如果config_file文件夹不存在就先创建
			if !target_dir.exists() {
				std::fs::create_dir_all(target_dir.parent().unwrap()).unwrap();
			}
			let result = std::fs::copy(config_path, target_dir);
			match result {
				Ok(_) => (),
				Err(e) => panic!("cargo::warning=复制配置文件夹失败:{}", e),
			}
		}
		Err(e) => panic!("cargo::warning=获取编译profile失败:{}", e),
	}
}
