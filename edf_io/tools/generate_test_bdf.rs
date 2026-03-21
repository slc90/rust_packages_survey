//! 测试 BDF 数据生成工具
//!
//! 生成 64 通道、4000 Hz、10 分钟的 BDF 测试文件

use edf_io::generate_test_bdf;
use std::path::PathBuf;

fn main() {
	// 获取项目根目录的 data 文件夹路径
	// CARGO_MANIFEST_DIR = edf_io/ 目录，parent() = 项目根目录
	let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
		.parent()
		.unwrap()
		.join("data");

	// 确保 data 目录存在
	std::fs::create_dir_all(&path).expect("创建 data 目录失败");

	// 设置输出文件路径
	let output_file = path.join("test_64ch_4000hz_10min.bdf");

	println!("生成测试 BDF 文件...");
	println!("输出路径: {:?}", output_file);

	match generate_test_bdf(&output_file, 64, 4000, 600) {
		Ok(()) => println!("文件生成成功！"),
		Err(e) => eprintln!("文件生成失败: {}", e),
	}
}
