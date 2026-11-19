# ContentDialog

2026-11-18 核对本地 WinUI `main` 的 `07e0354d2e06572501ef48078a3cc91138749c9f` 与 WinUI Gallery `main` 的 `6de91feccadfbb0a796b71caedead5a2284f730e`。运行 `python3 -B tools/reference_analysis/analyze_overlays.py` 提取主要证据。

- `controls/dev/CommonStyles/ContentDialog_themeresources.xaml` 5–53、60–66、221–258 行定义 320–548 × 184–756 的面板范围、24 逻辑像素留白、8 逻辑像素按钮间距、20 逻辑像素半粗体标题和 12 逻辑像素标题下距。标题最多两行。内容区使用 `LayerFillColorAltBrush`，底部命令区使用 `SolidBackgroundFillColorBaseBrush`，两区以 1 逻辑像素卡片描边分隔。面板使用 `OverlayCornerRadius`、普通 1 逻辑像素与高对比 2 逻辑像素描边，以及 `SmokeFillColorDefaultBrush`。颜色见同仓库 `Common_themeresources_any.xaml` 50–68、254–272 行。
- 同一模板 74–114 行和 `Common_themeresources_any.xaml` 603–606 行给出中心缩放和淡入淡出。打开从 105% 缩放至 100%，历时 250 ms，遮罩与面板用 83 ms 淡入。关闭的缩放反向运行 167 ms，淡出 83 ms。项目复用共享动效偏好和弹层关闭计时。
- `dxaml/xcp/dxaml/lib/ContentDialog_Partial.cpp` 398–441、577–595、1655–1685、1746–1794、2000–2101 行规定焦点及结果。优先聚焦正文控件，其次是可用的默认按钮，再退回到第一个可用命令。正文或已聚焦的其他按钮先处理 Enter，未处理的 Enter 可触发可用的默认按钮。焦点离开默认按钮转到另一命令时移除默认强调色。Esc 返回 `None`，关闭后恢复打开前的控件。
- `dxaml/test/native/external/controls/contentdialog/ContentDialogIntegrationTests.cpp` 1339–1430、2907–3292、3344 行起覆盖 Tab 环绕、Enter 传播、默认按钮样式和初始焦点。Gallery 的 `WinUIGallery/Samples/ContentDialog/ContentDialogPage.xaml` 12–39 行和 `ContentDialogPage.xaml.cs` 26–93 行展示有默认按钮和无默认按钮的不同用法。

## 依赖与实现

- 复用已验证的 `ui/overlay-host.slint`、`ui/button.slint`、`ui/interaction.slint` 和 `ui/scroll-view.slint`。共享 PopupWindow 隔离背景指针与焦点遍历，使用窗口可用区域居中，关闭时走既有焦点恢复。正文先收到按键，Dialog 容器随后拦截未处理按键。窗口缩放时保留已打开的对话框和焦点。
- `ui/content-dialog.slint` 提供 `show()`、`hide()`、Primary/Secondary/None 结果、同步关闭拦截和正文插槽。自定义正文可设置 `content-takes-initial-focus`，由插槽内的控件在初始化时调用 `focus()`。宿主可指定请求尺寸，实际尺寸受官方范围和安全区限制。正文视口保留默认隐藏滚动条，命令区独立于正文。
- Slint 1.19 使用带标题的 GroupBox 辅助角色。原版跨窗口模态、ShowAsync 延迟关闭、系统辅助技术、移动端软键盘及 Windows 同场景实录列为 TODO(spec)。

## 本机验证

Apple M3 Mac，macOS 26.4.1，arm64 Debug 与 Skia。通过 2× Retina 和 1.25× 渲染缩放的宽窄窗口冒烟。验证背景点击和快捷键阻塞、正反向 Tab 环绕、默认命令、聚焦命令优先、禁用与无默认命令、文本框处理 Enter、Esc 和同步关闭拦截、焦点回到实际打开者，以及缩放时保留模态与焦点。像素检查覆盖明暗和高对比主题的内容区与命令区。闭合动效期间保持背景阻塞，完成时单次报告结果并恢复焦点。

格式检查、工作区检查、Clippy 和 8 项 Rust 测试通过。构建产物为 `target/debug/fluent-gallery`。像素缓冲保留在内存中，检查临时输出目录与 Python 缓存后没有本轮临时文件。其他平台、Windows 官方实机画面对照及系统读屏未测试。
