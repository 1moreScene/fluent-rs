# Flyout

2026-11-18 分析本地 WinUI `main` 的 `07e0354d2e06572501ef48078a3cc91138749c9f` 与 WinUI Gallery `main` 的 `6de91feccadfbb0a796b71caedead5a2284f730e`。复现命令为 `python3 -B tools/reference_analysis/analyze_overlays.py`。

- `dxaml/xcp/dxaml/lib/FlyoutBase_partial.cpp` 62、1449–1451、1712、2254–2305 行定义默认上方、外部点按关闭，以及按打开设备把焦点交给内容。Gallery `WinUIGallery/Samples/Flyout/FlyoutPage.xaml` 20–43 行展示共享内容和按钮锚定的交互内容。
- `dxaml/xcp/core/dll/focusmgr.cpp` 3161–3178、3201–3243 行将鼠标、触控和笔映射为指针焦点，将键盘等导航设备映射为键盘焦点。程序设置焦点时沿用最近的输入设备。`dxaml/xcp/core/core/elements/Popup.cpp` 599–607、875–891、5345–5351 行记录打开前的焦点，外部点按关闭时恢复指针焦点。项目的 `FluentFocusScope` 在共享输入层应用此规则，Flyout、ComboBox 和 ContextMenu 使用同一策略。
- `controls/dev/CommonStyles/FlyoutPresenter_themeresources.xaml` 5–20、29–46 行定义应用内 Acrylic、普通边框 1、高对比边框 2、内距 16/15/16/17 与滚动呈现。`dxaml/xcp/dxaml/themes/generic.xaml` 50–53 行给出 96–456 的宽度和 40–758 的高度边界。
- 上游 `OverlayHost` 已验证 Slint 弹层、外部点按、Esc、焦点恢复和视口变化。共享定位支持顶部、底部、左侧、右侧和贴边回退。Flyout 通过内容插槽复用宿主，Gallery 用共享滚动视口容纳受限高度的内容。ContextMenu 复用同一定位及宿主。
- 验证交互触发、焦点、确认、退出、窗口变化、窄窗与亮暗绘制。Skia 实现应用内模糊和着色，高对比或减少透明度时使用纯色。Windows 系统 Acrylic 合成、原始动效和 Windows 画面对照为 TODO(spec)，由对应平台和对照场景验证。
