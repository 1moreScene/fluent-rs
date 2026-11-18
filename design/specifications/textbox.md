# TextBox

分析日期 2026-11-18。两份参考仓库均使用本地只读副本，许可证为 MIT。

| 参考仓库 | 本地位置 | 分支与提交 |
| --- | --- | --- |
| Microsoft WinUI 源码 | `references/microsoft-ui-xaml` | `main` · `354bf6017e6443023214d34309dfdf0fe2d526b1` |
| Microsoft WinUI Gallery | 同级目录 `../WinUI-Gallery` | detached HEAD · `7360ac322414e118eadfb0c7c2f4ecc31a218f59` |

```powershell
python -B tools/reference_analysis/analyze_textbox.py --gallery-source ../WinUI-Gallery
```

报告提取常规模板 `DefaultTextBoxStyle`、主题资源、原生状态切换和 Gallery 示例。样式分析使用 `TextBox_themeresources.xaml`，该提交中的 `_perf2026` 变体发布范围待核实。脚本也接受 `--winui-source`、`--gallery-source` 或对应的 `WINUI_SOURCE_DIR`、`WINUI_GALLERY_DIR`。

## 结构与状态

| 规格 | 结果 | 源码 |
| --- | --- | --- |
| 基础尺寸与排版 | 最小宽 64、高 32，圆角 4，内容默认字号 14 逻辑像素 | `dxaml/xcp/dxaml/themes/generic.xaml` 14、36、96–97 行及 `controls/dev/CommonStyles/CornerRadius_themeresources.xaml` 5、9、13 行 |
| 内边距与描边 | 左、上、右、下为 10、5、6、6。普通描边 1，亮暗主题的焦点描边为 1、1、1、2。 | `controls/dev/CommonStyles/Common_themeresources.xaml` 10–12、24–26 行 |
| 标签与占位符 | 顶部标签下方留 8，标签和占位符可分开提供。占位符根据文本是否为空展示。 | `controls/dev/CommonStyles/TextBox_themeresources.xaml` 175、335–338 行及 `dxaml/xcp/core/native/text/Controls/TextBox.cpp` 870–876 行 |
| 视觉优先级 | Disabled、Focused、PointerOver、Normal。普通与悬停共享底边加深的描边；焦点使用强调色底边。前景、填充和描边切换使用离散关键帧。 | `TextBox_themeresources.xaml` 48–65、155–172、253–309 行及 `dxaml/xcp/core/native/text/Controls/TextBoxBase.cpp` 3557–3569 行 |
| 只读 | 原生文本服务设置 `TXTBIT_READONLY`。只读可以保留焦点和选择，禁用同时拦截焦点和输入。 | `TextBoxBase.cpp` 590、3223–3226 行，Gallery `ReadOnlyTextboxVarious.txt` |
| 单行与多行 | Gallery 同时提供基本输入、标签和占位符、只读，以及开启 `AcceptsReturn` 与换行的多行示例。 | `WinUIGallery/Samples/TextBox/TextBoxPage.xaml` 21–47 行 |
| 内联清除入口 | 文本非空且已聚焦、可编辑、不换行、单行且控件宽度大于 `5em` 时显示。位于右侧，初始宽度 30，布局后按实际高度形成方形点击区。`IsTabStop=False`。 | `TextBox.cpp` 276–297、829–861 行，`TextBox_themeresources.xaml` 311–323、339 行，`dxaml/xcp/dxaml/lib/TextBox_Partial.cpp` 1096–1121 行 |
| 清除按钮外观 | 模板图标为 12 逻辑像素，内部绘制区上、右、下各留 4。悬停和按下复用 `SubtleFillColorSecondary/Tertiary`，图标从 `TextFillColorSecondary` 切换到按下时的 `TextFillColorTertiary`。 | `TextBox_themeresources.xaml` 40–47、147–154、176–177、205–246 行 |

[WinUI TextBox 模板](https://github.com/microsoft/microsoft-ui-xaml/blob/354bf6017e6443023214d34309dfdf0fe2d526b1/controls/dev/CommonStyles/TextBox_themeresources.xaml) · [WinUI Gallery TextBox 页面](https://github.com/microsoft/WinUI-Gallery/blob/7360ac322414e118eadfb0c7c2f4ecc31a218f59/WinUIGallery/Samples/TextBox/TextBoxPage.xaml)

## 主题资源

下表的半透明值使用微软 XAML 的 `#AARRGGBB` 顺序。Slint 实现转换为 `#RRGGBBAA`。

| 资源 | Light | Default（暗色） |
| --- | --- | --- |
| `ControlFillColorDefault` | `#B3FFFFFF` | `#0FFFFFFF` |
| `ControlFillColorSecondary` | `#80F9F9F9` | `#15FFFFFF` |
| `ControlFillColorInputActive` | `#FFFFFF` | `#B31E1E1E` |
| `ControlFillColorDisabled` | `#4DF9F9F9` | `#0BFFFFFF` |
| `ControlStrokeColorDefault` | `#0F000000` | `#12FFFFFF` |
| `ControlStrongStrokeColorDefault` | `#72000000` | `#8BFFFFFF` |
| `TemporaryTextFillColorDisabled` | `#5C010101` | `#5DFEFEFE` |
| 焦点底边 | `SystemAccentColorDark1` | `SystemAccentColorLight2` |
| `AccentFillColorSelectedTextBackgroundBrush` | `SystemAccentColor` | `SystemAccentColor` |

普通文字、占位符与禁用占位符分别复用 `TextFillColorPrimary/Secondary/Disabled`。禁用文字使用模板的单独资源。几何、排版、常规填充、强调色和高对比来自已有共享令牌。 [WinUI 公共主题资源](https://github.com/microsoft/microsoft-ui-xaml/blob/354bf6017e6443023214d34309dfdf0fe2d526b1/controls/dev/CommonStyles/Common_themeresources_any.xaml)

## 实现范围

内联清除依赖 `ui/tokens.slint` 的尺寸、色彩和状态令牌，以及 `ui/text-box.slint` 内的 Slint `TextInput`、`TouchArea` 和焦点绑定。Gallery 的交互页和状态样本消费同一个控件外观。弹层依赖属于文本菜单。触控、真实 IME 和辅助技术仍需平台验证。

`ui/text-box.slint` 使用自研 Slint 外观和原生 `TextInput` 编辑原语，提供标签、占位符、双向文本绑定、禁用、只读、单行与可换行的多行模式、用户编辑和单行提交回调。单行超长时按光标位置水平滚动，多行随光标垂直滚动，并支持滚轮。移动端 Gallery 将输入区的最小触控高度设置为 44 逻辑像素。Gallery 的交互示例和状态样本共用一个外观组件。 [Slint TextInput](https://docs.slint.dev/latest/docs/slint/reference/keyboard-input/textinput/)

右侧清除钮与文本上下文菜单各自独立。点击清空文本和选区，恢复占位符，焦点留在输入框，并触发一次 `edited`。微软的 `OnDeleteButtonClick` 经 `SetTextServicesBuffer` 调用 `TxSetText`，该方法会丢弃撤销历史。项目通过替换 `TextInput.text` 实现相同的撤销行为。清除图标采用自绘 Slint Path，宿主把单行输入框设得很高时，会额外检查文本区仍有空间。 [Microsoft TxSetText](https://learn.microsoft.com/en-us/windows/win32/api/textserv/nf-textserv-itextservices-txsettext)

`TODO(spec)`：当前选区使用项目主题的强调色与配对前景，Windows 系统强调色和系统高对比主题的同步仍待核验。文本上下文和选区菜单对应模板的 `ContextFlyout` 与 `SelectionFlyout`，可按 `TextBox_themeresources.xaml` 196–197 行另行实现。官方对照截图待提供，跨平台字体和边缘合成差异待用同主题同缩放的截图测量。

## 验证记录

P1 将边界命中、描边、可见标签和清除图标接入共享原语。编辑区仍使用 Slint TextInput，Focused 优先于 PointerOver。 [P1 共享基础](foundations.md) · [P1 验证记录](../../docs/p1-validation.md)

- Windows 11 Pro Build 26200、Skia OpenGL、125% 缩放。发布版连续三次通过 `--smoke-test`。事件循环内验证了悬停、Tab 焦点、中英文输入、光标移动、Backspace 和 Delete、选择替换、撤销重做、长单行首尾编辑、只读选择与禁止修改、禁用保护、单行 Enter 提交、多行 Enter 换行，以及切换页面保留文本。两页分别渲染至 900×775 和 400×800 物理像素，并检查了 TextBox 亮暗主题下的焦点底边颜色。
- 同机启动发布版并停在默认 Button 页。4 秒后每隔 500 毫秒读取三次 Private Bytes，本轮均为 42,893,312 字节，前一轮为 42,676,224 字节。这是启动后空闲采样。TextBox 活动页的长期内存趋势与帧时间未测试。
- `cargo fmt --all -- --check`、`cargo check --locked --workspace`、`cargo clippy --locked --workspace --all-targets -- -D warnings` 和 `cargo test --locked --workspace` 通过。控件行为由 Gallery 的冒烟模式验证。
- Android ARM64 编译通过。API 35 调试 APK 打包、16 KiB 对齐与签名通过。Android 实机、触控、真实输入法组合、系统剪贴板交互、屏幕阅读器、Windows 10 1809、macOS、iOS 和官方截图像素对照均未测试，需要对应设备或参考截图。
- 清理了 Android 未对齐中间 APK 和 Python 字节码缓存。保留发布版 EXE、已签名 APK、分析脚本和 Cargo 增量缓存。

内联清除的回归检查覆盖聚焦非空时显示、空内容和失焦时隐藏、Tab 跳过、拖出取消、清除后立即继续输入、只发送一次编辑通知，以及清空后使用撤销时不恢复原值。只读、禁用和多行也分别检查了隐藏行为。Windows 11 发布版连续三次通过冒烟检查。相同启动空闲场景的 Private Bytes 三次采样均为 43,208,704 字节。Android ARM64 调试 APK 已重新构建，签名与 16 KiB 对齐检查通过。
