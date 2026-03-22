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

> 2026-03-22 Medical Image DICOM Compatibility Update
>
> - `medical_image/src/dicom_loader.rs` 现在会跳过 `.WeDrive` 等隐藏同步附带文件，并记录 DICOM 元数据与像素解码日志
> - DICOM 加载现在兼容 `OT` 模态以及 `RGB` 单帧切片，加载时会转换为灰度体数据以复用现有渲染链路
> - 医学影像页新增 `加载DICOM目录`，并已有本地测试覆盖 `data/CT_DICOM` 目录加载
