//! edf_io - EDF+/BDF 文件读写库
//!
//! 提供 EDF+ 和 BDF 文件的读取和测试数据生成功能

mod bdf_writer;
mod generator;
mod loader;

pub use bdf_writer::{BdfSignalParam, BdfWriter, BdfWriterError, generate_test_bdf};
pub use generator::TestEdfGenerator;
pub use loader::{EdfLoader, EdfLoaderError};
