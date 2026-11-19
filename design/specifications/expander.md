# Expander

2026-11-19 核对本地 WinUI `main`（`07e0354d2e06572501ef48078a3cc91138749c9f`）和 Gallery `main`（`6de91feccadfbb0a796b71caedead5a2284f730e`）。规格来自普通 `Expander.xaml`，`perf2026` 变体单独演进。

| 规格 | 来源 |
| --- | --- |
| Header 最小高 48，控件最小宽使用 FlyoutThemeMinWidth，Header 左距 16。箭头背景为 32，左留距 20，右留距 8，字形 12 | `controls/dev/Expander/Expander.xaml:16–26,111`、`Expander_themeresources.xaml:77–88,279–285` |
| Header 使用 CardBackgroundFillColorDefault，内容使用 CardBackgroundFillColorSecondary，内容内距 16，接缝只画一条线。高对比 Header 和箭头描边 2 | `Expander_themeresources.xaml:5–74,86–88`、`controls/dev/CommonStyles/Common_themeresources_any.xaml:57,261,475` |
| 上下均可展开。裁剪内的位移展开 333 ms，曲线 (0,0,0,1)。收起位移 167 ms，曲线 (1,1,0,1)。向上内容收起的可见性在 200 ms 结束 | `Expander.xaml:32–103`、`Expander.cpp:148–173` |
| 上下使用同一 AnimatedChevronUpDownSmallVisualSource，状态按方向和展开状态映射 | `Expander_themeresources.xaml:140–163,203–248,281–285,487–491` |
| Gallery 提供文字标题、可改方向及自定义 Header/Content | `WinUIGallery/Samples/Expander/ExpanderPage.xaml:22–64` |

复用 `FluentPressable`、`FluentSurface`、`FluentFocusRing`、`AnimatedIcon` 和已有 ChevronAnimation 时间线。正常及禁用控件复用颜色令牌和 Slint 布局。Slint 布局随裁剪高度变化，保证相邻内容可响应；Windows 布局重排逐帧时机未测试。准入验证包括双向布局、独立内容、Space/Enter、禁用、减弱动画、明暗高对比切换以及展开中间帧。真实触控和屏幕阅读器留待对应设备验证。

macOS Apple Silicon 的 Skia Debug 在 2× 和 1.25× 背板缩放下通过专项验证，在 2× 下通过 Gallery 全量窗口回归。`target/debug/fluent-gallery --smoke-test disclosure` 可复现专项验证。记录保存在 `.artifacts/expander-split-button/`，过时的编译诊断已清理。

## 布局复验

2026-11-19 重新核对上述版本。用户截图第一行箭头墨迹的纵向中心约为 178，标题条中心约为 196，两者相差约 18 个背板像素。标题的固定尺寸箭头独立以标题条的中心计算纵坐标，标题插槽使用完整内容列。标题高度和展开裁剪仍由共享组件计算。

`ExpanderPage.xaml:22–64` 和对应 code-behind `:15–29` 的两个示例各自使用按内容测量的宽度、Up/Down ComboBox 选项、500 宽的居中标题和零内容区内距。第二例的文字自身内距为 4。参考 `Controls/ControlExample.xaml:133–185,278–295`，示例内距 12、选项内距 16，窄窗把选项放到下方。500 的示例宽度受可用宽度约束，避免内容裁切。

本轮在 macOS Skia 2× 和 1.25× 下采样箭头和自定义标题的真实墨迹，并验证标称尺寸、收放和窄窗选项位置。两个缩放的专项通过，2× 全量窗口回归停在 RepeatButton 首次重复的延时检查。结果保存在 `.artifacts/disclosure-layout/`。
