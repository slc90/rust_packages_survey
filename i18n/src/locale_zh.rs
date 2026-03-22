use crate::data_structure::LanguageKey;

/// 放所有的中文翻译
pub fn lookup(language_key: LanguageKey) -> &'static str {
	match language_key {
		LanguageKey::Title => "Rust包调研",
		LanguageKey::Function => "功能",
		LanguageKey::Language => "语言",
		LanguageKey::About => "关于",
		LanguageKey::Setting => "设置",
		LanguageKey::RealtimePlot => "实时波形",
		LanguageKey::PlaybackPlot => "回放波形",
		LanguageKey::MedicalImage => "医学影像",
		LanguageKey::VideoPlayer => "播放视频",
		LanguageKey::AudioPlayer => "播放音频",
		LanguageKey::Screenshot => "截图测试",
		LanguageKey::DeepLearning => "深度学习",
	}
}
