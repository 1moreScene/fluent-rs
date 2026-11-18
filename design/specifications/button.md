# Button

分析日期 2026-11-18。两份参考仓库均使用本地只读副本，许可证为 MIT。

| 参考仓库 | 本地位置 | 分支与提交 |
| --- | --- | --- |
| Microsoft WinUI 源码 | `references/microsoft-ui-xaml` | `main` · `354bf6017e6443023214d34309dfdf0fe2d526b1` |
| Microsoft WinUI Gallery | 同级目录 `../WinUI-Gallery` | detached HEAD · `7360ac322414e118eadfb0c7c2f4ecc31a218f59` |

使用 Python 解析模板、主题字典及 Gallery 的 XAML，并定位原生输入处理代码。选用常规模板 `Button_themeresources.xaml`。该提交同时存在 `_perf2026` 模板，后者的发布范围待核实。

```powershell
python tools/reference_analysis/analyze_button.py --gallery-source ../WinUI-Gallery
```

脚本输出 JSON，包含两份仓库的版本、模板状态、主题资源键、渐变参数及 Gallery 示例。仓库放在其他位置时可传入 `--winui-source` 和 `--gallery-source`，也可设置 `WINUI_SOURCE_DIR` 和 `WINUI_GALLERY_DIR`。

## 模板与输入证据

| 规格 | 取值 | 来源 |
| --- | --- | --- |
| 圆角 | 四角均为 4 逻辑像素 | `controls/dev/CommonStyles/CornerRadius_themeresources.xaml` 5、9、13 行，`ControlCornerRadius` |
| 内边距、边框 | `ButtonPadding` 按左、上、右、下为 11、5、11、6，边框为 1 | `controls/dev/CommonStyles/Button_themeresources.xaml` 29、90、127、152–168 行 |
| 排版 | 正常字重，内容字号 14，字体资源为 `XamlAutoFontFamily` | 同上 163–165、306–308 行及 `dxaml/xcp/dxaml/themes/generic.xaml` 14、36 行 |
| 填充范围与边框 | Standard、Subtle 在边框内侧填充，Accent 填充到外缘。Standard 明色在底边加深、暗色在顶边提亮。Accent 在底边加强描边。 | `Button_themeresources.xaml` 的 `BackgroundSizing`，`Common_themeresources_any.xaml` 186–205、382–404 行 |
| 常见状态 | Normal、PointerOver、Pressed、Disabled。优先级从高到低为 Disabled、Pressed、PointerOver、Normal。 | Button 三份模板及 `dxaml/xcp/dxaml/lib/Button_Partial.cpp` 18–60 行 |
| 焦点 | 键盘焦点与常见状态独立。默认模板指定向外扩 3，通用高可见焦点描边为外 2、内 1。 | `Button_themeresources.xaml` 166–168、309–311 行与 [微软焦点视觉规范](https://learn.microsoft.com/en-us/windows/apps/develop/input/guidelines-for-visualfeedback) |
| 背景过渡 | 三种样式均为 83 毫秒。前景和边框使用离散关键帧。 | `Button_themeresources.xaml` 173–193、245–259、316–336 行 |
| 激活与取消 | 默认 `ClickMode.Release`。Enter 和 Space 按下进入按压，松开激活。指针按下可取得指针焦点，拖动时根据边界更新按压状态，释放时依据按压和捕获状态决定是否激活。 | `dxaml/xcp/dxaml/lib/ButtonBase_Partial.cpp` 449–530、603–799 行 |

Gallery 的标准示例 23–42 行绑定启禁状态，图像示例 49–55 行有独立的自动化名称，77–84 行展示 Accent 和 Subtle，90–110 行覆盖窄布局与文本换行。点击处理在同目录 `ButtonPage.xaml.cs` 16–29 行更新反馈。 [WinUI Gallery Button 页面](https://github.com/microsoft/WinUI-Gallery/blob/7360ac322414e118eadfb0c7c2f4ecc31a218f59/WinUIGallery/Samples/Button/ButtonPage.xaml) · [WinUI Button 模板](https://github.com/microsoft/microsoft-ui-xaml/blob/354bf6017e6443023214d34309dfdf0fe2d526b1/controls/dev/CommonStyles/Button_themeresources.xaml)

## 颜色与实现

`Button_themeresources.xaml` 4–150 行将三种样式的各状态映射到 `Common_themeresources_any.xaml` 中的主题资源。下表的颜色沿用 XAML 的 `#AARRGGBB` 表示法，Slint 令牌换算为 `#RRGGBBAA`。

| 资源 | Light | Default（暗色） |
| --- | --- | --- |
| `SolidBackgroundFillColorBase` | `#F3F3F3` | `#202020` |
| `ControlFillColorDefault/Secondary/Tertiary/Disabled` | `#B3FFFFFF` / `#80F9F9F9` / `#4DF9F9F9` / `#4DF9F9F9` | `#0FFFFFFF` / `#15FFFFFF` / `#08FFFFFF` / `#0BFFFFFF` |
| `SubtleFillColorSecondary/Tertiary` | `#09000000` / `#06000000` | `#0FFFFFFF` / `#0AFFFFFF` |
| `TextFillColorPrimary/Secondary/Disabled` | `#E4000000` / `#9E000000` / `#5C000000` | `#FFFFFF` / `#C5FFFFFF` / `#5DFFFFFF` |
| `AccentFillColorDefaultBrush` | `SystemAccentColorDark1` | `SystemAccentColorLight2` |
| `AccentFillColorSecondary/TertiaryBrush` | 默认强调色的 0.9 / 0.8 不透明度 | 默认强调色的 0.9 / 0.8 不透明度 |
| `AccentFillColorDisabled` | `#37000000` | `#28FFFFFF` |
| `FocusStrokeColorOuter/Inner` | `#E4000000` / `#B3FFFFFF` | `#FFFFFF` / `#B3000000` |

当前 Button 提供文本标签、可选换行、启禁状态、三种样式、键盘焦点和 `clicked`。默认标签在空间受限时省略，辅助技术仍可读完整标签。Gallery 的移动入口设置 44 逻辑像素最小触控目标，桌面按钮按内容和边距计算尺寸。Accent 在项目中沿用相同排版和内边距，官方 Accent 样式在此提交中只绑定这些属性，实际默认值待参照截图核验。 [WinUI 公共主题资源](https://github.com/microsoft/microsoft-ui-xaml/blob/354bf6017e6443023214d34309dfdf0fe2d526b1/controls/dev/CommonStyles/Common_themeresources_any.xaml)

`TODO(spec)`：默认强调色 `#005FB8` 和 `#60CDFF` 是项目跨平台回退值，可由宿主配置。高对比演示使用可替换的黑、白、黄色调色板，Windows 用户对比度主题的系统色映射待验证。背景过渡沿用 83 毫秒，WinUI 插值曲线和各平台字形、焦点边缘合成有待对照。当前使用 Slint 的系统字体选择。

## 验证记录

P1 将按键和指针激活迁到 `FluentPressable`，外框和焦点环复用共享原语。Button 继续按 Disabled、Pressed、PointerOver、Normal 的优先级选用样式，键盘焦点环独立显示。 [P1 共享基础](foundations.md) · [P1 验证记录](../../docs/p1-validation.md)

- Windows 11 Pro Build 26200、Skia OpenGL、125% 缩放。发布版连续三次通过事件循环内的输入与像素缓冲检查，覆盖鼠标悬停、松开激活、拖出取消、拖回恢复按压、Tab 焦点、Space/Enter 松开激活、自动重复和禁用。用 Light、Dark、High contrast 渲染 900×775 和 400×800 物理像素场景，检查页面主题色、非空像素、窄窗示例布局和 44 逻辑像素触控目标。Gallery 固定样本展示三种样式的五种状态。
- 同机空闲内存快速采样。启动发布版后等待 4 秒，再每 500 毫秒读取三次进程 Private Bytes。P0 空窗口为 39,489,536 字节，P1 Button 页为 42,676,224 字节。启动耗时、帧时间和长期内存趋势未测试。
- `cargo fmt --all -- --check`、`cargo check --locked --workspace`、`cargo clippy --locked --workspace --all-targets -- -D warnings` 和 `cargo test --locked --workspace` 通过。输入和渲染检查由 `--smoke-test` 执行。
- Android ARM64 编译、API 35 调试 APK 打包和签名检查通过。Android 15 设备、实际触控、屏幕阅读器、Windows 10 1809、macOS 与 iOS 运行未测试，分别需要对应设备或测试环境。
- 官方参考截图来源待提供。像素级差异未测试，复核需用同一主题和缩放的微软原版 Button 截图。
- 像素检查在内存中完成。清理了 Android 未对齐中间包，保留签名后的 APK、Windows 发布版、Cargo 增量缓存和正式分析脚本。
