use std::{env, fs, path::Path};

fn main() {
	let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
	let workspace_root = Path::new(&manifest_dir).parent().unwrap();
	let source_dir = workspace_root.join("config").join("config_file");
	println!("cargo:rerun-if-changed={}", source_dir.display());
	let out_dir = env::var("OUT_DIR").unwrap();
	let target = Path::new(&out_dir).join("config_file");
	copy_dir_all(&source_dir, &target).unwrap();
	// 向依赖它的 crate 导出这个路径
	println!("cargo:config_dir={}", target.display());
}

/// 递归复制目录
fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
	fs::create_dir_all(dst)?;
	for e in fs::read_dir(src)? {
		let e = e?;
		let src = e.path();
		let dst = dst.join(e.file_name());
		if src.is_dir() {
			copy_dir_all(&src, &dst)?;
		} else {
			fs::copy(&src, &dst)?;
		}
	}
	Ok(())
}
