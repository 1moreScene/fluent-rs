# ToggleSplitButton

2026-11-19 用 Python 核对本地微软 WinUI `main`（`07e0354d2e06572501ef48078a3cc91138749c9f`）及 WinUI Gallery `main`（`6de91feccadfbb0a796b71caedead5a2284f730e`）。参考常规样式。

| 规格 | 官方来源 |
| --- | --- |
| 与 SplitButton 共用模板。主区和箭头区最小宽 35，中间为 1 的分隔线，主区内距 11/6/11/7 | `controls/dev/SplitButton/SplitButton.xaml:209–246`，`SplitButton_themeresources.xaml:101–103` |
| 主区按下后翻转 IsChecked，再调用 Click。Space、Enter 调用主区，F4、Alt+↓ 与箭头区打开浮层 | `ToggleSplitButton.cpp:35–68`，`SplitButton.cpp:221–228,324–367` |
| 选中时两侧使用强调色，鼠标悬停或按压仅改变所在区。浮层打开及键盘/触控按下时两侧使用选中按压色和透明外边。禁用优先于选中 | `SplitButton.cpp:118–218`，`SplitButton.xaml:69–79,131–206`，`SplitButton_themeresources.xaml:5–33,68–97` |
| 选中分隔线为黑色、透明度字节 0x37，高对比使用 HighlightText；高对比分别映射选中、悬停、按下的系统颜色 | `controls/dev/CommonStyles/Common_themeresources_any.xaml:43,247`，`SplitButton_themeresources.xaml:68–97` |
| Gallery 主区切换列表格式，箭头区选项目符号或罗马数字。选择格式会启用选中状态 | `WinUIGallery/Samples/ToggleSplitButton/ToggleSplitButtonPage.xaml:14–47`，同目录 `ToggleSplitButtonPage.xaml.cs:19–55` |

复用 `ui/split-button.slint` 的双区输入、视觉几何、焦点、箭头动画和共享 Flyout。前置依赖已在 Mac 用 `--smoke-test disclosure` 验证。新增可双向绑定的 `checked`，沿用 `clicked` 作为主区操作，`changed checked` 可观察程序与用户修改。Gallery 用列表预览演示格式，官方示例的 RichEditBox 对文档选区应用格式。

专项验证覆盖选中/未选中、主副区、键盘与焦点、独立的用户点击与外部赋值、菜单选择、退出及禁用、亮暗与高对比、窄窗和背板缩放。

TODO(spec) 缺少同版本官方 Windows Gallery 截图。待取得后对照强调色分隔、模板边缘和焦点环，并在对应平台实测屏幕阅读器。

## 本机验证

同批 2× 和 1.25× Skia 专项均通过。测试验证外部 `checked` 赋值不触发主区命令、主区回调能读取切换后的值、选菜单格式自动选中、F4/Alt+↓ 只展开、Esc 恢复焦点，以及按区域的选中配色。环境、命令与未测试项目见 [DropDownButton 本机验证](dropdownbutton.md#本机验证)。

## 浮层复验

2026-11-19 复查 Gallery 此例使用 `Flyout Placement="Bottom"`。普通 Flyout 以整按钮为锚点居中，目标和窗口边缘留 4。内部使用 `FlyoutPresenter_themeresources.xaml:5–20,39–46` 的应用内 Acrylic、1/2 描边与 16/15/16/17 内距，两个图标按钮各有 4 内距与 6 外距。Gallery 采用相同的 20 逻辑像素图标盒，罗马数字列表使用自绘图形。来源为 Gallery `ToggleSplitButtonPage.xaml:18–37` 及 `FlyoutBase_Partial.cpp:64–68,3290–3300`。

普通 Flyout 依据 `FlyoutBase_Partial.cpp:2041–2080` 随放置方向从 50 逻辑像素处移入。`LayoutTransition_partial.cpp:1153–1163` 从系统主题取得 `TAS_SHOWPOPUP` 和 `TAS_HIDEPOPUP`。宿主可注入系统轨道。本机无该主题数据，采用项目的 250ms 位移与 83ms 淡入、淡出曲线。两种弹层均遵循减少动画、减少透明度与高对比设置。

Mac 专项在 2× 与 1.25× 验证了居中与尺寸、入场中间像素、淡出保持与焦点恢复，并取样对照实时 Acrylic 和纯色。Windows 系统主题时序与官方截图像素未测试。
