# 目录结构
```
rust_packages_survey/
├── assets/                                 # 运行时资源目录
│   └── shaders/
│       └── medical_volume.wgsl             # 医学影像体渲染 MIP shader
├── .cargo/                                 # Cargo构建配置
│   └── config.toml                         # 链接器配置
├── .zed/                                   # Zed编辑器配置
│   └── tasks.json                          # 任务定义文件
├── .github/                                # GitHub配置
│   └── workflows/                          # CI/CD工作流
│       ├── ci.yml                          # 持续集成配置
│       └── release.yml                     # 发布流程配置
├── docs/                                   # 项目文档
│   ├── helpers.md                          # 辅助文档记录
│   ├── homepage_content_area_switch.md     # 主页内容区域切换功能实现方案
│   ├── medical_imaging设计方案.md          # 医学影像功能设计方案
│   ├── playback_waveform设计方案.md        # 回放波形功能设计方案
│   ├── realtime_plot.md                    # 实时波形画图方案
│   ├── requirements.md                     # 功能需求列表
│   ├── setting_implementation.md           # 设置页面实现方案
│   ├── title_bar_implementation.md         # 标题栏实现文档
│   └── unresolved_issues.md                # 遗留问题记录
├── edf_io/                                 # EDF/BDF 文件读写库crate
│   ├── Cargo.toml
│   ├── src/                                # 源代码
│   │   ├── bdf_writer.rs                   # BDF 写入与测试数据生成
│   │   ├── generator.rs                    # EDF 测试数据生成与头部修正
│   │   ├── lib.rs                          # 库入口点
│   │   └── loader.rs                       # EDF 文件加载器
│   └── tools/                              # 数据生成工具
│       ├── generate_test_bdf.rs            # BDF 测试文件生成入口
│       └── main.rs                         # EDF 测试文件生成入口
├── embedded_assets/                        # 嵌入式资源crate
│   ├── Cargo.toml
│   ├── assets/                             # 静态资源文件
│   │   ├── SmileySans-Oblique.ttf          # 中文字体
│   │   ├── close.png                       # 关闭按钮图标
│   │   ├── logo.png                        # 应用程序logo
│   │   ├── maximize.png                    # 最大化按钮图标
│   │   ├── minimize.png                    # 最小化按钮图标
│   │   └── shaders/
│   │       └── medical_volume.wgsl         # 医学影像体渲染嵌入式 shader
│   └── src/                                # 源代码
│       ├── const_assets_path.rs            # 常量资源路径定义
│       ├── lib.rs                          # 库入口点
│       └── plugin.rs                       # Bevy插件定义
├── entry/                                  # 主程序crate
│   ├── Cargo.toml
│   ├── build.rs                            # 构建脚本
│   └── src/
│       └── main.rs                         # 程序入口点
├── i18n/                                   # 国际化库crate
│   ├── Cargo.toml
│   └── src/                                # 源代码
│       ├── data_structure.rs               # 数据结构定义
│       ├── lib.rs                          # 库入口点
│       ├── locale_en.rs                    # 英文本地化
│       └── locale_zh.rs                    # 中文本地化
├── logger/                                 # 日志库crate
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                          # 日志库代码
├── medical_image/                          # 医学影像基础库crate
│   ├── Cargo.toml
│   └── src/                                # 源代码
│       ├── dicom_loader.rs                 # DICOM Series 读取工具
│       ├── lib.rs                          # 库入口点
│       ├── nifti_loader.rs                 # NIfTI 读取工具
│       ├── slice.rs                        # 三视图切片工具
│       ├── surface.rs                      # 阈值表面重建工具
│       ├── volume.rs                       # 统一体数据结构与错误类型
│       └── windowing.rs                    # 窗宽窗位与灰度映射工具
├── config/                                 # 配置管理库crate
│   ├── Cargo.toml
│   ├── config_file/                        # 配置文件目录
│   │   └── config.json                     # 默认配置文件
│   └── src/                                # 源代码
│       ├── data_structure.rs               # 数据结构定义（Setting结构等）
│       └── lib.rs                          # 库入口点
├── logs/                                   # 日志文件目录
├── ui/                                     # 用户界面库crate
│   ├── Cargo.toml
│   ├── examples/                           # 示例代码
│   │   └── static_plot_line.rs            # 静态波形画图示例
│   └── src/                                # 源代码
│       ├── homepage/                       # 主页功能模块
│       │   ├── about/                     # 关于页面功能
│       │   │   ├── components.rs           # 组件定义
│       │   │   ├── plugin.rs              # 插件定义
│       │   │   └── systems.rs             # 系统定义
│       │   ├── medical_image/             # 医学影像功能
│       │   │   ├── components.rs          # 组件定义
│       │   │   ├── plugin.rs              # 插件定义
│       │   │   ├── resources.rs           # 医学影像状态、纹理和三维场景资源
│       │   │   ├── systems.rs             # 医学影像加载、切片显示、表面重建、体渲染和交互系统
│       │   │   └── volume_render.rs       # 体渲染材质、3D 纹理构建与降采样保护
│       │   ├── playback_plot/             # 回放波形功能
│       │   │   ├── components.rs          # 组件定义
│       │   │   ├── plugin.rs              # 插件定义
│       │   │   ├── resources.rs           # 回放数据和播放控制资源
│       │   │   └── systems.rs             # 回放加载、绘制和交互系统
│       │   ├── realtime_plot/             # 实时波形功能
│       │   │   ├── components.rs          # 组件定义
│       │   │   ├── plugin.rs              # 插件定义
│       │   │   └── systems.rs             # 系统定义
│       │   ├── setting/                   # 设置页面功能
│       │   │   ├── components.rs          # 组件定义
│       │   │   ├── plugin.rs              # 插件定义
│       │   │   └── systems.rs             # 系统定义
│       │   ├── about.rs                   # 关于页面模块导出
│       │   ├── common.rs                  # 公共组件和状态定义
│       │   ├── medical_image.rs           # 医学影像模块导出
│       │   ├── playback_plot.rs           # 回放波形模块导出
│       │   ├── plugin.rs                  # 主页主插件定义
│       │   ├── realtime_plot.rs           # 实时波形模块导出
│       │   └── setting.rs                 # 设置页面模块导出
│       ├── menu_bar/                      # 菜单栏模块
│       │   ├── components.rs              # 组件定义
│       │   ├── plugin.rs                  # 插件定义
│       │   └── systems.rs                 # 系统定义
│       ├── title_bar/                     # 标题栏模块
│       │   ├── components.rs              # ECS组件定义
│       │   ├── plugin.rs                  # Bevy插件定义
│       │   ├── resources.rs               # 资源定义
│       │   └── systems.rs                 # 系统定义
│       ├── homepage.rs                    # 主页模块导出
│       ├── lib.rs                         # 库入口点
│       ├── menu_bar.rs                    # 菜单栏模块导出
│       └── title_bar.rs                   # 标题栏模块导出
├── utils/                                 # 工具库crate
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                          # 工具库代码
├── target/                                 # 构建输出目录
├── .gitignore                              # Git忽略规则
├── .pre-commit-config.yaml                 # commit钩子配置
├── Cargo.lock                              # 依赖版本锁文件
├── Cargo.toml                              # 工作空间根配置
├── CLAUDE.md                               # Claude Code 指导文件
├── crates_survey.md                        # Rust包调研记录
├── rustfmt.toml                            # Rust代码格式化配置
└── structure.md                            # 项目目录结构文档
> 2026-03-21 å¢žé‡æ›´æ–°
>
> - æ–°å¢ž `media_player/` crateï¼Œç”¨äºŽåŸºäºŽ `gstreamer-rs` çš„è§†é¢‘æ’­æ”¾å†…æ ¸
> - æ–°å¢ž `ui/src/homepage/video_player/` æ¨¡å—ï¼Œç”¨äºŽä¸»çª—å£è§†é¢‘æ’­æ”¾é¡µé¢
> - ä¸»é¡µ `Functions` çŠ¶æ€æ–°å¢ž `VideoPlayer`
> - èœå•æ æ–°å¢žâ€œæ’­æ”¾è§†é¢‘â€å…¥å£
> - `i18n` æ–°å¢ž `VideoPlayer` å›½é™…åŒ– key
>
> 2026-03-21 Incremental Update
>
> - add `media_player/` crate for GStreamer-based video playback
> - add `ui/src/homepage/video_player/` with main-window player and popup player UI
> - video player state now supports one main player slot plus one popup player slot
> - add `ui/src/file_dialog.rs` to unify native file picking through `rfd`
> - `playback_plot` and `medical_image` now load input files from native file dialogs instead of fixed sample paths
>
> 2026-03-21 Audio Update
>
> - add `audio_player/` crate for local audio playback based on `rodio`
> - add `ui/src/homepage/audio_player/` for audio file selection, play/pause and status display
> - homepage `Functions`, menu bar and `i18n` now include the audio player entry
>
> 2026-03-21 Signal Processing Update
>
> - add `signal_processing/` crate for sine generation, FFT, spectrum analysis and FIR/IIR filtering
> - add `signal_processing/examples/` for sine waveform, FFT spectrum and filter comparison demos
> - data processing plan now targets crate-level examples instead of Bevy UI integration in the first phase
>
> 2026-03-21 Report Update
>
> - add `report_generator/` crate for Word and PDF export
> - add `report_generator/examples/` for minimal DOCX and PDF report generation
> - report export uses `docx-rs` for `.docx`, `printpdf` for `.pdf`, and reuses chart PNG files as embedded images
>
> 2026-03-22 Screenshot Update
>
> - add `screenshot/` crate for screenshot output directory management, PNG saving and region cropping
> - add `ui/src/homepage/screenshot/` for screenshot test page, four capture buttons and two validation regions
> - homepage `Functions`, menu bar and `i18n` now include the screenshot test entry
>
> 2026-03-22 Screenshot Phase 3 Update
>
> - `screenshot/` now includes Win32 current-display capture support for the monitor containing the main window
> - screenshot test page `程序所在桌面截图` button now captures and saves the current display to `screenshots/`
