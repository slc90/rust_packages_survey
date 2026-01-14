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
│   ├── requirements.md                     # 功能需求列表
│   ├── title_bar_implementation.md         # 标题栏实现文档
│   └── unresolved_issues.md                # 遗留问题记录
├── embedded_assets/                        # 嵌入式资源crate
│   ├── Cargo.toml
│   ├── assets/                             # 静态资源文件
│   │   ├── SmileySans-Oblique.ttf          # 中文字体
│   │   ├── close.png                       # 关闭按钮图标
│   │   ├── logo.png                        # 应用程序logo
│   │   ├── maximize.png                    # 最大化按钮图标
│   │   └── minimize.png                    # 最小化按钮图标
│   └── src/                                # 源代码
│       ├── lib.rs                          # 库入口点
│       ├── const_assets_path.rs            # 常量资源路径定义
│       └── plugin.rs                       # Bevy插件定义
├── entry/                                  # 主程序crate
│   ├── Cargo.toml
│   └── src/
│       └── main.rs                         # 程序入口点
├── i18n/                                   # 国际化库crate
│   ├── Cargo.toml
│   └── src/                                # 源代码
│       ├── lib.rs                          # 库入口点
│       ├── data_structure.rs               # 数据结构定义
│       ├── locale_en.rs                    # 英文本地化
│       └── locale_zh.rs                    # 中文本地化
├── logger/                                 # 日志库crate
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                          # 日志库代码
├── ui/                                     # 用户界面库crate
│   ├── Cargo.toml
│   └── src/                                # 源代码
│       ├── lib.rs                          # 库入口点
│       ├── title_bar.rs                    # 标题栏模块定义
│       └── title_bar/                      # 标题栏具体实现
│           ├── components.rs               # ECS组件定义
│           ├── plugin.rs                   # Bevy插件定义
│           ├── resources.rs                # 资源定义
│           └── systems.rs                  # 系统定义
├── utils/                                  # 工具库crate
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                          # 工具库代码
├── target/                                 # 构建输出目录
├── .gitignore                              # Git忽略规则
├── Cargo.lock                              # 依赖版本锁文件
├── Cargo.toml                              # 工作空间根配置
├── crates_survey.md                        # Rust包调研记录
└── rustfmt.toml                            # Rust代码格式化配置
```
