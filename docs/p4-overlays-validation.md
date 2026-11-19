# P4 Flyout、ContextMenu、Tooltip 与选择器复验

2026-11-18，Apple M3，macOS 26.4.1（25E253），Rust 1.100.0、Slint 1.19.0、Skia 调试版。先在原生 Gallery 验证 Flyout，再接入 ContextMenu 与 Tooltip。两份微软参考仓库的工作树干净。 [Flyout](../design/specifications/flyout.md) · [ContextMenu](../design/specifications/context-menu.md) · [Tooltip](../design/specifications/tooltip.md)

## 复现

```sh
python3 -B tools/reference_analysis/analyze_overlays.py
python3 -B tools/reference_analysis/analyze_selectors.py
cargo fmt --all -- --check
cargo check --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo run --locked -p fluent-gallery --bin fluent-gallery -- --smoke-test
SLINT_SCALE_FACTOR=1.25 cargo run --locked -p fluent-gallery --bin fluent-gallery -- --smoke-test
```

## 验证范围

| 场景 | 证据 |
| --- | --- |
| Flyout | 默认在锚点上方。左侧空间不足时翻到右侧。首个可操作内容获取焦点，确认、Esc 与外部点按关闭并恢复焦点。窗口尺寸变化、切页关闭，禁用目标不可打开。窄窗检查最小触控尺寸与亮暗回退色。 |
| ContextMenu | 右键释放时按指针位置打开，应用菜单键与 Shift+F10 按控件定位。首项焦点、循环导航、跳过禁用项、点按与 Enter 执行、Esc 和外部关闭通过。窄窗检查可用区域与亮暗回退色。 |
| Tooltip | 原生事件循环验证悬停等待、鼠标离开、键盘获焦、Esc、鼠标激活及超时。使用短计时验证状态机，Gallery 的默认值为 800ms 和 5s。Tooltip 保留 owner 焦点，亮暗像素检查通过。 |
| ListBox 视觉 | 亮、暗和高对比的 Skia 采样确认选中色叠在 ListBox 自身的容器色上。选中、悬停和按压采用 ListBoxItem 的离散状态。 |
| ComboBox 视觉 | 检查触发器按下、悬停、焦点、禁用与选中行指示条。启用 Acrylic 后用黑白底图测到真实模糊，减少透明度时同一区域出现均匀的纯色回退。验证箭头按压位移与回位、下拉展开前后帧以及减少动效时即时显示。 [ListBox 视觉溯源](../design/specifications/listbox.md) · [ComboBox 视觉溯源](../design/specifications/combobox.md) |

在 125% 和 200% 下分别执行 720×620 与 320×640 逻辑尺寸场景。格式检查、编译、Clippy、4 项 Rust 单元测试和原有控件回归与本轮冒烟一起执行。运行时经 Slint 分发键鼠事件，像素缓冲保存在内存中。

## 构建路径

该轮使用 `host-tuple`，产物位于 `target/aarch64-apple-darwin/debug/fluent-gallery`。可编辑 ComboBox 一轮改用 `target/debug/fluent-gallery` 作为统一的本机构建入口，见 [当前构建说明](../README.md#平台构建)。

## 未测试

- WinUI 对照画面、字体和滚动条像素校准、Windows Acrylic 的噪声与亮度混合、原版动画轨迹、真实触控、读屏、内存及帧时间未测试。需要对应 Windows 参考画面、辅助技术和同场景采样。
- macOS 15、Intel、Windows、Android 和 iOS 未执行本轮项目构建与运行。
- 检查 `.artifacts/`、`tests/visual/tmp/` 与 Python 缓存，无本轮过时临时文件。保留参考仓库和构建缓存。

## 共享焦点、按钮圆角与滚动条对齐

2026-11-18，同一台 Mac。使用官方 Flyout、FocusManager 和 Popup 的输入设备规则核对焦点。用户提供的 1860×612 截图用于定位按钮描边。 [Flyout 规格](../design/specifications/flyout.md) · [滚动条规格](../design/specifications/scrollbar.md)

- 鼠标打开 Flyout 后，首个交互按钮接收键盘输入。程序移焦延续指针来源，键盘打开时则绘制焦点框。验证 Esc 恢复键盘焦点框、外部点按恢复指针焦点，以及 ContextMenu 右键与 Shift+F10 下的初始焦点差异。既有 ComboBox 交互检查继续通过。
- `controls/dev/CommonStyles/Button_themeresources.xaml` 168、172 行将 `ControlCornerRadius` 传给表面。共享表面的四个描边角沿用该令牌，弹层接缝单独设定角半径。Skia 像素检查覆盖普通按钮的四角、可编辑下拉接缝和虚拟下拉滚动条的静止端点。Retina 和 1.25× 渲染缩放的窗口测试通过。
