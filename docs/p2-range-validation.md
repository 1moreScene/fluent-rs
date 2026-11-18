# P2 Slider、ProgressBar 与 ProgressRing 验证

日期 2026-11-18。Windows 11 Pro Build 26200 x64，Core i5-1235U、Intel Iris Xe（驱动 32.0.101.7084），1920×1080，Rust 1.100.0、Slint 1.19.0、Skia OpenGL。

## 复现

```powershell
python -B tools/reference_analysis/analyze_ranges.py --gallery-source ../WinUI-Gallery
cargo fmt --all -- --check
cargo check --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo run --locked -p fluent-gallery --bin fluent-gallery -- --smoke-test
```

参考分析读取本地 WinUI 源码和 Gallery 提交，提取 Slider 11 个模板状态及 4 个示例、ProgressBar 9 个状态及 2 个示例、ProgressRing 3 个状态及 2 个示例。 [Slider 规格](../design/specifications/slider.md) · [ProgressBar 规格](../design/specifications/progressbar.md) · [ProgressRing 规格](../design/specifications/progressring.md)

## 验收证据

四项 Cargo 检查均通过。发布版冒烟检查在 100%、125%、150% 和 200% 缩放通过，覆盖 720×620 和 320×640 逻辑像素。设置 `SLINT_SCALE_FACTOR` 后分别启动进程。现有 Gallery 进程占用了 `target/release/fluent-gallery.exe`，Cargo 无法覆盖该路径。本轮运行的是 Cargo 当次链接到构建目录的发布版。

| 项目 | Windows Gallery 冒烟检查 |
| --- | --- |
| Slider | Tab 焦点、Home/End、方向键及重复键、反向、非整步最大值、垂直刻度步进、点击吸附、越界拖动、禁用取消、宿主写入不发用户回调。Skia 检查 Normal、PointerOver、Pressed、Disabled、键盘焦点环及窄窗 44 逻辑像素触控目标。 |
| ProgressBar | 零跨度、0%、50%、超出上限的进度、错误／暂停色和全宽状态、亮暗和高对比、两段不确定循环及宿主停用动效后的静态帧。 |
| ProgressRing | 零跨度、半环和完整圆环、停用、亮暗和高对比、不确定循环、宿主停用动效后的静态帧，以及确定模式的中间帧和终点。 |
| 回归 | 现有 Button、TextBox、ToggleButton、CheckBox、RadioButton／RadioGroup、主题和材质继续参加同一冒烟流程。 |

Gallery 在页面不可见或窗口最小化时关闭进度控件的循环动效。控件宿主可通过 `animations-enabled` 绑定自身的页面或应用生命周期。

同机发布版在 125% 缩放下启动默认 Button 页。4 秒后每 500 毫秒采样一次 Private Bytes，三次均为 45,137,920 字节。这组数据记录的是 Gallery 默认页短时空闲开销。

## 未测试

- 官方画面对照未测试。需要同主题与缩放的微软 Gallery 截图或录屏。真实按键和触控手势、操作系统后台生命周期、屏幕阅读器未测试，分别需要输入设备、后台切换场景及读屏软件。Windows 冒烟用 Slint 事件分发验证鼠标和键盘。
- 帧时间、后台功耗、持续重绘计数和长期内存趋势未测试。需要采集帧通知与性能数据。
- Windows 10 1809 运行未测试，需 Build 17763 环境。macOS、iOS 和 Android 未测试，本轮仅在 Windows 开发环境验证，需相应设备和工具链。

## 产物

像素缓冲和参考分析输出均在内存中检查。盘点 `.artifacts/`、`tests/visual/tmp/` 和 Python 字节码，未产生待清理的本轮截图、缓存或一次性脚本。保留有效构建产物和增量缓存。
