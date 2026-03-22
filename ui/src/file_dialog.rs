use std::path::{Path, PathBuf};

/// 使用 rfd 打开单文件选择框
pub fn pick_single_file(
	initial_directory: Option<&Path>,
	title: &str,
	filters: &[(&str, &[&str])],
) -> Option<PathBuf> {
	let mut dialog = rfd::FileDialog::new().set_title(title);

	if let Some(directory) = initial_directory.filter(|path| path.exists()) {
		dialog = dialog.set_directory(directory);
	}

	for (label, extensions) in filters {
		dialog = dialog.add_filter(*label, extensions);
	}

	dialog.pick_file()
}

/// 使用 rfd 打开目录选择框
pub fn pick_single_directory(initial_directory: Option<&Path>, title: &str) -> Option<PathBuf> {
	let mut dialog = rfd::FileDialog::new().set_title(title);

	if let Some(directory) = initial_directory.filter(|path| path.exists()) {
		dialog = dialog.set_directory(directory);
	}

	dialog.pick_folder()
}
