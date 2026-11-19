# ContextMenu

2026-11-18 核对本地 WinUI `main` 的 `07e0354d2e06572501ef48078a3cc91138749c9f` 与 WinUI Gallery `main` 的 `6de91feccadfbb0a796b71caedead5a2284f730e`。复现命令为 `python3 -B tools/reference_analysis/analyze_overlays.py`。

- `dxaml/xcp/dxaml/lib/MenuFlyout_Partial.cpp` 384–408 行将触发点交给 Flyout 定位。`MenuFlyoutPresenter_Partial.cpp` 95–192、1009–1020 行处理方向键循环、可聚焦项与初始焦点。`MenuFlyoutItem_Partial.cpp` 185–224 行在调用命令后关闭菜单。
- Gallery `WinUIGallery/Samples/ListView/ListViewPage.xaml` 421–442 行示范右键触发行级菜单。`WinUIGallery/Samples/MenuFlyout/MenuFlyoutPage.xaml` 同时示范动作、分隔、子项和图标。
- `controls/dev/CommonStyles/MenuFlyout_themeresources.xaml` 40–42、123–125、255–261 行给出桌面 Acrylic、普通与高对比边框及菜单最小高度 32。当前共用主窗口内的弹层表面与回退策略。
- 上游为已验证的 Flyout/OverlayHost、ItemPresenter、SelectableList、SelectionLogic 和输入原语。先完成并验证单层命令菜单、禁用、右键与键盘触发、初始焦点、上下循环、动作执行和焦点恢复。嵌套菜单、复杂项模板与真实移动端长按为 TODO(spec)。
