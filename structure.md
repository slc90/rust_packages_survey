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
│   └── requirements.md                     # 功能需求列表
├── entry/                                  # 主程序crate
│   ├── Cargo.toml
│   └── src/
│       └── main.rs                         # 程序入口点
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
