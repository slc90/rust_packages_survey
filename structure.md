# 目录结构
```
rust_packages_survey/
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
├── installer/                              # Windows 安装包工程目录
│   ├── manifests/                          # 安装资源整理清单
│   │   ├── cuda_runtime_dlls.txt           # CUDA 运行时 DLL 匹配清单
│   │   ├── gstreamer_runtime_roots.txt     # GStreamer 运行时目录清单
│   │   └── whisper_base_required_files.txt # Whisper Base 必需模型文件清单
│   ├── scripts/                            # 安装包构建脚本
│   │   ├── build_installer.ps1             # 安装包 staging 和 MSI 构建脚本
│   │   ├── prepare_cuda_runtime.ps1        # CUDA 运行时整理脚本
│   │   ├── prepare_models.ps1              # Whisper Base 模型整理脚本
│   │   └── prepare_runtime.ps1             # GStreamer 运行时整理脚本
│   └── wix/                                # WiX 模板和资源
│       ├── License.rtf                     # 安装器许可文本占位文件
│       └── main.wxs                        # MSI 主模板
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
```

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
>
> 2026-03-22 Deep Learning Plan Update
>
> - add `docs/深度学习设计方案.md` for the local AI module plan
> - `docs/requirements.md` deep learning section now includes the design document entry
>
> 2026-03-22 Deep Learning Phase 1 Update
>
> - add `deep_learning/` crate for runtime directory initialization, model metadata and task/result messages
> - add `ui/src/homepage/deep_learning/` for the Phase 1 test page shell and Bevy `Message`-based smoke task flow
> - homepage `Functions`, menu bar and `i18n` now include the deep learning entry
>
> 2026-03-22 Deep Learning Phase 2 Update
>
> - add `deep_learning/src/whisper.rs` and extend task payloads for Whisper request preflight
> - deep learning test page now supports Whisper file selection, language hint, timestamp toggle and request snapshot output
> - Phase 2 currently completes model directory preflight and request snapshot generation before the Candle inference core is wired in
>
> 2026-03-22 Deep Learning Phase 3 Update
>
> - add `deep_learning/src/translation.rs` and `deep_learning/src/tts.rs` for translation and TTS request preflight
> - deep learning test page now supports translation and TTS text-file selection, parameter toggles and request snapshot output
> - Phase 3 currently completes model directory preflight and request snapshot generation for translation and TTS before the Candle inference core is wired in
>
> 2026-03-22 Deep Learning Phase 4 Update
>
> - add `deep_learning/src/separation.rs` for vocal separation request preflight
> - deep learning test page now supports separation audio-file selection and request snapshot output
> - Phase 4 currently completes model directory preflight and request snapshot generation for separation before the Candle inference core is wired in
>
> 2026-03-22 Deep Learning Phase 5 Update
>
> - add `deep_learning/src/image_generation.rs` for image generation model preflight and PNG output
> - deep learning test page now supports prompt-file selection, size/model/seed/steps toggles and in-page image preview
> - Phase 5 currently completes model directory preflight, request snapshot generation and reusable PNG preview output for image generation
>
> 2026-03-22 Deep Learning Whisper CUDA Update
>
> - `deep_learning/src/runtime.rs` now requires CUDA-only initialization and fails fast when CUDA is unavailable
> - `deep_learning/src/whisper.rs` now completes real `openai/whisper-base` Candle inference against `deepl_models/whisper_base/`
> - Whisper tokenizer loading now merges `vocab.json` and `added_tokens.json`, fixing language/control token lookup such as `<|ja|>`
> - deep learning test page result area now expands Whisper text output directly from the generated result file
>
> 2026-03-22 Logger Workspace Root Update
>
> - `logger/src/lib.rs` now resolves `logs/` from the workspace root before creating rolling log files
> - launching `entry/src/main.rs` from editor working directories such as `entry/` no longer creates `entry/logs/`
>
> 2026-03-22 Paradigm Plan Update
>
> - add `docs/范式设计方案.md` for the paradigm module plan
> - `docs/requirements.md` paradigm section now includes the design document entry
>
> 2026-03-22 Paradigm Initial Implementation
>
> - add `embedded_assets/assets/paradigm/default.gif` as the default animated GIF preview asset for the paradigm page
> - add `ui/src/homepage/paradigm/` for the paradigm test page, animated GIF preview and monitor-targeted P300 playback
> - homepage `Functions`, menu bar and `i18n` now include the paradigm entry
>
> 2026-03-22 Embedded Asset Consolidation
>
> - move the paradigm default GIF into `embedded_assets/assets/paradigm/default.gif`
> - keep `medical_volume.wgsl` only under `embedded_assets/assets/shaders/` and remove the duplicate root-level asset copy
>
> 2026-03-22 Installer Plan Update
>
> - add `docs/安装包设计方案.md` for the Windows installer plan
> - `docs/requirements.md` installer section now includes the design document entry
>
> 2026-03-22 Installer MSI Strategy Update
>
> - `docs/安装包设计方案.md` now fixes the installer path to `cargo wix + WiX Toolset` with offline `MSI`
> - the installer plan now requires bundling a fixed `GStreamer Runtime` into the package instead of asking users to download it separately
>
> 2026-03-22 Installer Resource Scope Update
>
> - `docs/安装包设计方案.md` now treats `deepl_models/whisper_base/` as a required packaged model resource
> - installer Phase 4 no longer includes CI work and is limited to local build/release engineering steps
>
> 2026-03-22 Installer CUDA Runtime Update
>
> - `docs/安装包设计方案.md` now requires bundling the program's redistributable `CUDA` runtime DLLs into the installer
> - the installer plan now distinguishes packaged `CUDA` runtime files from machine-level `NVIDIA` driver prerequisites
>
> 2026-03-22 Installer Scaffold Update
>
> - add `installer/` with WiX template, PowerShell build scripts and runtime/model manifests for MSI packaging
> - `deep_learning/src/model.rs` now falls back to the executable directory in installed deployments so packaged model and output paths remain stable
>
> 2026-03-22 Installer WiX v5 Update
>
> - installer build flow now targets `WiX Toolset v5` and no longer depends on `heat.exe` or `cargo-wix`
> - `installer/wix/main.wxs` now uses WiX v5 authoring with `Files` bound from the staged application directory
>
> 2026-03-22 Installer Local Path Update
>
> - `installer/scripts/build_installer.ps1` now reads `GStreamerRoot` from `installer/local_paths.ps1` before falling back to command arguments or environment variables
> - `installer/local_paths.ps1` is treated as a local ignored file for machine-specific installer paths
> - `CudaBinRoot` now also reads from `installer/local_paths.ps1`, allowing both runtime paths to be kept in one local machine config
>
> 2026-03-22 Installer Runtime Fix Update
>
> - installer GStreamer staging now filters out `.pdb` and other non-runtime files, and also stages `libexec/gstreamer-1.0/` tools such as `gst-plugin-scanner.exe`
> - installed deployments now route logs and deep learning outputs to `LOCALAPPDATA/rust_packages_survey/`, while startup also prepends packaged `gstreamer` and `cuda` runtime directories to the process environment
> - screenshot output paths now follow the same rule, falling back to `LOCALAPPDATA/rust_packages_survey/screenshots/` in installed deployments
