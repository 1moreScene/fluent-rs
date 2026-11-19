# P4 ListBox 与 ComboBox 验证

2026-11-18，MacBook Air Apple M3，macOS 26.4.1（25E253），Rust 1.100.0、Slint 1.19.0、Skia 调试版。

先独立实现 ListBox，并运行工作区检查和 200% 缩放下的原生窗口冒烟。ListBox 通过后，ComboBox 接入同一套选择模型、虚拟列表和弹层。两份微软参考仓库的工作树干净。 [ListBox 溯源](../design/specifications/listbox.md) · [ComboBox 溯源](../design/specifications/combobox.md)

## 复现

```sh
python3 -B tools/reference_analysis/analyze_selectors.py
cargo fmt --all -- --check
cargo check --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo run --locked -p fluent-gallery --bin fluent-gallery -- --smoke-test
SLINT_SCALE_FACTOR=1.25 cargo run --locked -p fluent-gallery --bin fluent-gallery -- --smoke-test
```

## 结果

格式检查、工作区编译、Clippy 和 4 项单元测试通过。原生 Gallery 在 125% 和 200% 缩放下渲染了 720×620 和 320×640 逻辑尺寸的窗口。用 Slint 分发键鼠事件，并在内存中采样 Skia 像素缓冲。

| 项目 | 结果 |
| --- | --- |
| ListBox | 单选、多选、Shift 扩展与回缩、选择修饰键切换、全选与取消全选、PageDown、首尾跳转、禁用项、空模型、宿主同步以及滚轮滚动。亮暗和高对比选中色通过像素检查。 |
| ComboBox | 点击、Enter、Space、F4 和 Option 方向键打开或关闭。候选导航跳过禁用项，Enter 或点按确认，Esc 和外部点按保留原选择，Tab 关闭后继续移焦。选项文本和索引同步，宿主写入保持用户回调计数。禁用、空模型和切页时的关闭行为通过。 |
| ComboBox 视觉 | 采样覆盖触发器悬停、按下、禁用、键盘焦点环，以及窄窗亮暗弹层底色。窄窗验证触发器至少 44 逻辑像素及弹层位于可用区域。 |
| 长模型 | 每个控件独立使用 100,000 行惰性模型。ListBox 验证 Home 和 End 的首尾定位，两种缩放下分别读取 51、52 次。ComboBox 验证 End 后能点按最后一项提交，再次打开定位到已提交项，每轮读取 44 次。计数包含重复读取。 |
| 回归 | P3 的外部点按、Esc、焦点恢复、模态确认和响应式场景与旧控件一起运行。 |

## 初版验收边界

ComboBox 提供非编辑的提交模式。可编辑输入、文本搜索和 Always 触发模式列在规格中。

官方画面、物理触控、读屏、内存与帧时间未测试。需要 Windows 的同主题同缩放画面、目标输入设备、辅助技术和同场景性能采样。其他系统和机型未测试，本轮构建和运行均在这台 Mac 完成。

参考结果和像素缓冲留在内存中。检查 `.artifacts/`、`tests/visual/tmp/` 和 Python 缓存后，确认本轮没有过时临时文件。保留当前构建与有效的增量缓存。

## 焦点与下拉定位复验

2026-11-18，同一台 Apple M3 Mac。增加两项布局单元测试，覆盖所选行和中间行对齐、窗口边缘裁剪和十万行尾部定位。 [WinUI 溯源](../design/specifications/combobox.md#焦点定位与下拉动效)

- 鼠标打开时候选没有键盘焦点环。Esc 关闭并恢复触发器焦点后，触发器外缘像素与初始状态一致。从键盘打开时才显示列表的焦点提示。
- 开始无选中项时验证中间行中心与显示区中心重合。预选和再次打开以所选行对齐。候选变动不会移动下拉位置。十万行案例通过实际点按显示区覆盖的行验证虚拟滚动偏移与位置一致。
- 像素采样覆盖展开早期显示区中心已绘制且上下边缘尚未展开，到稳定帧时两侧均绘制。外部点按后验证面板进入收起状态并在过渡后关闭。减少动效时两种操作即时完成。

同一轮通过格式检查、工作区编译、Clippy、6 项 Rust 测试及 125% 与 200% 的原生宽窄窗口冒烟。像素缓冲在内存中检查，临时输出目录和 Python 缓存没有本轮产物。Windows 原版同场景实录和移动端 Carousel 未测试。

## 可编辑 ComboBox 与 macOS 启动

2026-11-18，Apple M3，macOS 26.4.1（25E253），arm64 Debug，Skia。两份参考仓库的版本和提取方式见 [可编辑规格](../design/specifications/combobox.md#可编辑模式)。

- 在 Gallery 打开 ComboBox 页的 Font size 示例。原生窗口冒烟验证初始选项、后缀补全与 Backspace、编辑时保持原选择、Enter 与 Tab 提交、Esc 恢复、校验拒绝、鼠标点选和外部点按提交。展开时验证编辑器保留焦点、输入与方向键更新候选、下拉贴合输入框。Rust 单元测试覆盖底边展开、上翻、长列表裁切和 UTF-8 匹配。
- 本机构建和启动路径为 `target/debug/fluent-gallery`。Mac 默认字号与控件尺寸为 125%，正文 17.5 逻辑像素，ComboBox 最小高度 40 逻辑像素。在内置 Retina 显示器上，启动内容窗口为 980×700 逻辑像素，Skia 帧为 1960×1400。使用 Slint 的 1.25 倍背板缩放复验时，宽窗帧为 1226×876，窗口逻辑尺寸在请求值的 1 像素内。
- 320×640 逻辑像素窄窗启用 125% 界面与触控尺寸，用滚轮将示例移入视口，验证两种主题下的弹层边界与 Skia 底色。Retina 和覆盖缩放下的帧分别为 640×1280、400×800。
- 格式、工作区编译、Clippy、8 项 Rust 测试和两种背板缩放的窗口冒烟均通过。像素缓冲留在内存中，未生成临时截图或调试脚本。保留当前构建及已有增量缓存。
- Windows 官方画面对照、实际输入法候选、触控设备、读屏、内存与帧时间未测试。需要同场景 Windows Gallery 截图、目标输入设备、辅助技术与性能采样。

## 接缝圆角与滚动条

2026-11-18，同一台 Mac，Skia。用户提供的 564×562 截图用于定位接缝。使用官方源码的上开和下开圆角规则，并以 Gallery 的 Skia 帧验证。 [ComboBox 规格](../design/specifications/combobox.md#可编辑模式) · [滚动条规格](../design/specifications/scrollbar.md)

- 下开验证两处浮层顶角、接缝处的文本框焦点下划线，以及收起后恢复圆角。窄窗口验证上翻的浮层底角和亮暗主题。
- 验证短列表能容纳全部选项时隐藏滑块、长列表显示滑块、拖动跨越十万项后可点击第一项，以及多行 TextBox 通过滚动条更新视口。ComboBox 的完整长列表场景读取 124 次行数据，包含三次大跨度滑块移动。Gallery 页面、Flyout 示例、ContextMenu 与 ListBox 使用同一套纵向滚动条。
- 在 2× 原生 Retina 和 1.25× 渲染缩放两种配置下通过完整冒烟。像素缓冲在内存中使用，临时输出目录无新增文件。Windows 实机画面对照和真实触控未测试。
