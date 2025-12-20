各种rust包的调研
# GUI
  - ~~egui~~ 即时模式,只适合用来写一些小工具
  - [x] gpui
  - [x] gpui-component
# 3D
  - [x] Bevy 游戏引擎,只能单独使用
  - [x] rend3 只渲染,可以尝试嵌入到gpui中
  - ~~利用gpui的backend手搓3D,理论可以,但不现实~~
# 数据处理
  - [x] ndarray
  - [x] ndarray-linalg
  - [x] ndarray-stats
  - [x] ndarray-rand
  - [x] rand
  - [x] rand_distr
  - [x] nalgebra 用于低维,物理和CG
  - [x] argmin
  - ~~rustfft~~
  - [x] plotters
  - [x] polars
# Image
  - [x] image
# Audio
  - [x] cpal
# Video
  - [x] gstreamer-rs 播放视频
# 异步和并行
  - ~~tokio~~ 功能全，重，侧重互联网企业级应用
  - [x] smol 轻量，异步消息方便
  - [x] rayon
# 日志
  - ~~tracing~~ 和tokio一起用,重,结构化,异步
  - [x] log4rs 可配置,像传统的python/C#的记日志方式
# 配置
  - [x] serde
  - [x] serde_json
  - [x] serde_yaml
  - [x] toml
# 深度学习
  - [x] candle
  - ~~burn~~ 全面,重,支持多backend当前意义不大
# GPU
  - [x] wgpu 学习图形学,手搓渲染管线,手搓GPU计算,事实上的标准库,全平台
  - [x] rust-cuda 手搓Nvidia的GPU计算
# 测试
  - [x] rstest 像pytest
  - [x] mockall 对trait进行Mock
  - [x] rust-pretty-assertions
  - [x] proptest 自动生成测试用例
  - [x] tarpaulin Code Coverage
  - [x] criterion benchmark
# 错误处理
  - [x] anyhow
  - [x] thiserror
# 数据库
  - ~~redb(KV型)~~
  - [x] sqlx(关系型)
  - ~~turso(关系型)~~ sqlite底层的重写,还不成熟
  - [x] qdrant(向量型)
# 文件相关
  - [x] walkdir
  - [x] glob
  - [x] csv
  - [x] printpdf
  - [x] docx-rs
  - [x] dicom-rs
  - [x] nifti
