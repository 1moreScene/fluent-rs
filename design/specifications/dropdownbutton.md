# DropDownButton

2026-11-19 用 Python 核对本地微软 WinUI `main`（`07e0354d2e06572501ef48078a3cc91138749c9f`）及 WinUI Gallery `main`（`6de91feccadfbb0a796b71caedead5a2284f730e`）。参考常规样式。

| 规格 | 官方来源 |
| --- | --- |
| 单一按钮命中区域，文字与图标支持通用内容。使用 Button 的字体、内距 11/5/11/6、1 边框、系统焦点环和 83 ms 背景过渡 | `controls/dev/DropDownButton/DropDownButton.xaml:3–27,88–98`，`controls/dev/CommonStyles/Button_themeresources.xaml:148–173` |
| 12 × 12 箭头，距内容 8。常态为二级文字色，悬停和按下为三级，禁用为禁用文字色。高对比悬停和按下用 Highlight | `DropDownButton.xaml:28–98`，`DropDownButton_themeresources.xaml:4–17` |
| 使用与 SplitButton 相同的 AnimatedChevronDownSmallVisualSource。展开是独立的可访问状态，按钮模板使用标准指针及按压状态 | `DropDownButton.xaml:25–94`，`DropDownButton.cpp:27–75`，`DropDownButtonAutomationPeer.cpp:19–77` |
| Email 文本和仅图标两个示例，菜单左下对齐，提供 Send、Reply、Reply All | `WinUIGallery/Samples/DropDownButton/DropDownButtonPage.xaml:10–53` |

| 前置能力 | 路径与验证 |
| --- | --- |
| 主题、尺寸、按钮表面、焦点与键盘/点击 | `ui/tokens.slint`、`ui/button-surface.slint`、`ui/interaction.slint`。基础 Button 已有 Gallery 与冒烟验证 |
| 箭头动效和减少动画 | `ui/split-chevron.slint`、`ui/animated-icon.slint`。本轮已在 Mac 运行既有 disclosure 专项 |
| 浮层、外部关闭/恢复焦点、列表导航 | `ui/flyout.slint`、`ui/overlay-host.slint`、`ui/context-menu.slint`、`ui/selectable-list.slint`。提取共享菜单呈现层，原 ContextMenu 和下拉示例使用同一份菜单交互 |

组件通过默认插槽承载浮层内容。纯图标用法可用 `accessible-name` 提供名称。Gallery 验证整区触发、菜单导航/提交、Esc 和外部关闭、页面切换与禁用、亮暗及高对比、窄窗和背板缩放。Slint 1.19 提供展开操作和展开状态，折叠由 Esc、外部点击及 `dismiss-flyout()` 处理。

TODO(spec) 缺少同版本官方 Windows Gallery 截图。待取得后对照按钮内距、字体、焦点环、菜单材质和图标。

## 本机验证

2026-11-19 在 Apple Silicon、macOS 26.4.1 的 Skia Debug Gallery 运行 `target/debug/fluent-gallery --smoke-test button-flyouts`。Retina 2× 输出 1960×1400 和 640×1280 像素，加 `SLINT_SCALE_FACTOR=1.25` 输出 1226×876 和 400×800 像素。两种背板缩放均通过主题和像素、鼠标和键盘事件、禁用和页面退出、焦点恢复、窄窗触控目标尺寸及动效检查。旧专项中的动效检查覆盖箭头。默认 Mac 后端未提供帧通知，未测稳定打开后的渲染次数。

同一专项用 `ps` 约每 100 ms 采样进程 RSS，峰值为 175296 KiB（2×）和 156704 KiB（1.25×）。未测试基线内存差异、实体触控和 VoiceOver，分别需要同负载的旧版产物、对应输入设备和人工辅助功能验收。其他系统和 macOS 15 未测试。

工作区格式、编译、Clippy 和 21 项单元测试通过。1.25× Gallery 全量冒烟通过。2× 全量冒烟首次停在既有 RepeatButton 的初始延迟断言，当前工作区重新构建后通过全量复验。最终二进制在两种缩放下均通过本批专项。

## 浮层复验

2026-11-19 复查页内两例都是 `MenuFlyout Placement="BottomEdgeAlignedLeft"`。以窗口边距 4、左边对齐、内容区最小宽 96（触控资源 240）、上下内距 2、条目外距 4/2、普通描边 1 和高对比描边 2 复验。指针和键盘使用紧凑项内距 11/4/11/5，图标列占 28、图标 16。来源为 `FlyoutBase_Partial.cpp:64–68`、`MenuFlyoutPresenter_Partial.cpp:913–937` 和 `controls/dev/CommonStyles/MenuFlyout_themeresources.xaml:255–290,342–391`。条目高度以本项目缩放字号估算默认行高，应用放大时随字号增长。

`MenuFlyout_Partial.cpp:245–266`、`MenuPopupThemeTransition_Partial.h:23–24` 和 `LayoutTransition_partial.cpp:532–633` 定义贴着锚点边缘、从一半高度滑入、同步调整边框的 250ms 展开，收起为 83ms 线性淡出。使用共享 OverlayHost，把菜单内容平移与呈现区域裁切同步处理。WinUI 使用独立窗口 DesktopAcrylicBackdrop，本机弹层复用 Skia 实时 Acrylic 和已有纯色降级。独立窗口后台采色与系统阴影留待 Windows 对照。

Mac 的 Skia Debug 专项在 2× 与 1.25× 对照了打开前段和终态的像素、两种宽度、退出时机与焦点恢复，并取样验证实时 Acrylic 与减少透明度时的纯色有区别。2× Gallery 全量回归通过。Windows 实机画面、桌面 Acrylic 合成及实体触控未测试。
