use serde::{Deserialize, Serialize};

/// 所有可用的语言
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Language {
	English,
	Chinese,
}

/// 所有需要翻译的Key
/// 每次添加新的Key时，可以让编译器检查是不是有遗漏翻译
pub enum LanguageKey {
	Title,
	Function,
	Language,
	About,
	Setting,
	RealtimePlot,
}
