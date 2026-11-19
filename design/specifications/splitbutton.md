# SplitButton

2026-11-19 核对本地 WinUI `main`（`07e0354d2e06572501ef48078a3cc91138749c9f`）和 Gallery `main`（`6de91feccadfbb0a796b71caedead5a2284f730e`）。采用普通 SplitButton 模板。

| 规格 | 来源 |
| --- | --- |
| 主区最小 35、分隔 1、箭头区 35、主内容内距 11/6/11/7，箭头 12，距右 12 | `controls/dev/SplitButton/SplitButton.xaml:219–239`、`SplitButton_themeresources.xaml:101–103` |
| 主/副区独立 Hover、Pressed，浮层打开、键盘按下或触控按下时两侧为 Pressed；WinUI 按下键盘时将模板副区扩展为全宽 | `SplitButton.xaml:69–129,209–217`、`SplitButton.cpp:118–218` |
| 鼠标主区调用，副区打开浮层；Space/Enter 调用，Alt+Down/F4 打开。Gallery 两例都显式使用 `Bottom`，浮层以整控件为目标居中，目标和窗口边缘各留 4 | `SplitButton.cpp:221–228,270–287,324–367`、`dxaml/xcp/dxaml/lib/FlyoutBase_Partial.cpp:63–97,3290–3300`、`SplitButtonPage.xaml:37,89` |
| 箭头使用 AnimatedChevronDownSmallVisualSource，Press 约 151 ms、Release 300 ms | `SplitButton.xaml:43–61,228–235`、`controls/dev/AnimatedIcon/AnimatedVisuals/AnimatedChevronDownSmallVisualSource.cpp:104,324–347,470–487` |
| Gallery 演示选择颜色并回写主命令，主区应用当前选择 | `WinUIGallery/Samples/SplitButton/SplitButtonPage.xaml:17–68,78–154` |

复用 `FluentPressable`、`FluentPointerArea`、`FluentFocusRing`、`AnimatedIcon`，以及 `Flyout → OverlayHost → OverlayLayout` 的材质、翻转定位、轻触关闭和焦点恢复。浮层动效通过共享接口接收系统轨道。macOS 使用与 ToggleSplitButton 共用的回退时序，见 [浮层复验](togglesplitbutton.md#浮层复验)。准入验证包括主副独立命中、禁用、单 Tab 衔接、键盘打开、位置、选项提交、Esc 和外部点击恢复、箭头中间帧和三个主题。实体触控、Gamepad 和屏幕阅读器留待对应设备验证。

macOS Apple Silicon 的 Skia Debug 在 2× 和 1.25× 背板缩放下通过专项验证，覆盖鼠标、Space/Enter、Alt+↓、F4、变更选项、Esc、点击外部、还原焦点、窄窗边界以及箭头按下和释放的中间帧。`target/debug/fluent-gallery --smoke-test disclosure` 可复现。

## 布局复验

2026-11-19 继续核对上述版本。用户截图的按钮表面宽约 1224 个背板像素，主区标签居于整行中部。默认宽度现由主内容、35 的副区和 1 的分隔共同决定，避免父布局把示例拉伸。主内容支持自定义插槽和独立内距。

`WinUIGallery/Samples/SplitButton/SplitButtonPage.xaml:10–74,78–153` 的第一例是零内距的 32 色块按钮，色板为三列、八个颜色，样例旁的文本编辑区宽 240、最小高 96。第二例使用 5 的内距和三列、九个颜色按钮，色块仍是 32，每格外距为 6。第一种色板的单元最小尺寸 44 和右下间距 4 来自 `GridViewItem_themeresources.xaml:141–149` 及 `dxaml/xcp/dxaml/themes/generic.xaml:109–110`。两个色板共用 `FlyoutPresenter_themeresources.xaml:20,39–46` 的 16/15/16/17 内距及共享弹层。Gallery 文本演示复用现有 TextBox，把颜色应用到整段文字，官方 RichEditBox 按选区应用颜色。

macOS Skia 2× 和 1.25× 均验证了 71×32 的色块按钮、左侧主按钮的命中中心、三列色板的颜色和步距，以及键盘选色和关闭后的焦点恢复。记录保存在 `.artifacts/disclosure-layout/`。临时验证副本及可执行文件已清理。

## 两个色板

第一个色板的数据和色块外观由页面提供，SDK `GridView` 负责条目的创建、布局和选中状态。三列布局和八种颜色来自 Gallery `SplitButtonPage.xaml:38–62`。GridView 另有 10 的底部留白，见 `dxaml/xcp/dxaml/themes/generic.xaml:9347–9360`。沿用 WinUI `ListViewBase` 的默认单选和键盘焦点跟随选择，见 `dxaml/xcp/components/DependencyObject/DependencyProperty.cpp:634–636` 及 `dxaml/xcp/core/core/elements/DefaultValues.cpp:59–62`。方向键更新网格中的选择，项调用将颜色同步到主按钮和演示文字。Gallery 的 `SplitButtonPage.xaml.cs:27–38` 在提交后把焦点转给文本编辑区，本项目按交付要求将焦点恢复到打开色板的 SplitButton。

第二个色板来自 Gallery `SplitButtonPage.xaml:78–153`，包含三列、九个标准 Button。每个按钮装入 32×32 的色块，内距为 0，外距为 6。WinUI `ContentPresenter.cpp:839–858` 在期望尺寸中计入边框，`Button_themeresources.xaml:29,89,159–172` 的标准按钮边框为 1，所以桌面单元按钮外尺寸为 34，步距为 46。本项目用可换行布局和 SDK Button，点击单项关闭该浮层。键盘沿按钮的 Tab 顺序移动，提交后恢复到文字 SplitButton。

`target/debug/fluent-gallery --smoke-test disclosure` 从打开的 GridView 读取实际列数、实例化条目数和色块位置，检查颜色像素。键盘用例检查网格本身的焦点、选中索引和选中边框，再验证颜色提交、关闭与焦点恢复。第二个色板检查浮层定位、色块行列、标准按钮的 Tab 顺序和关闭后的焦点。

macOS arm64 Skia Debug 在背板缩放 2× 和 1.25× 下通过两例专项。亮色逻辑尺寸为 178×188 和 172×172，分别验证了 48 和 46 的色块步距。共享 GridView、DropDownButton 和 ToggleSplitButton 的专项同时通过。日志见 `.artifacts/split-style/`。
