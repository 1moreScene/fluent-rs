# InfoBar

2026-11-18 核对本地 WinUI `main` 的 `07e0354d2e06572501ef48078a3cc91138749c9f` 与 WinUI Gallery `main` 的 `6de91feccadfbb0a796b71caedead5a2284f730e`。运行 `python3 -B tools/reference_analysis/analyze_overlays.py` 提取主要证据。

- `controls/dev/InfoBar/InfoBar.xaml` 3–41、94–127、175–179 行定义普通控件圆角、严重级别表面、左侧图标、中部内容与右侧关闭按钮。`InfoBar_themeresources.xaml` 5–59 行映射四种级别的填充和图标颜色。底色、信息强调色及卡片描边见 `controls/dev/CommonStyles/Common_themeresources_any.xaml` 46–81、165–174、250–288、369–378 行。
- `InfoBar_themeresources.xaml` 62–94 行给出 48 逻辑像素最小内容高度、16 像素图标、38 像素关闭按钮及 5 像素边距。标题与正文均为 14 像素，标题半粗体。内容列从左侧 16 像素起，图标后间距 14 像素，正文横排与标题间距 12 像素。另有纵排正文顶部间距 4 像素与动作间距 12 像素。
- `InfoBarPanel.cpp` 19–84、89–151 行按内容的自然宽高选择布局。单项或内容较宽较高时使用纵排，多项能在行内排下时横排。`InfoBar.xaml` 113–115 行使用整词换行。Gallery `WinUIGallery/Samples/InfoBar/InfoBarPage.xaml` 20–130 行和 `InfoBarPage.xaml.cs` 23–90 行提供级别选择、长短正文、动作、图标与关闭按钮的示例。
- `InfoBar.idl` 6–20、62–85、99–101 行定义四种级别、默认关闭、默认可关闭与显示图标。`InfoBar.cpp` 81–137、175–217 行为关闭按钮和程序关闭报告不同原因，允许同步取消，关闭后折叠，无额外关闭动画。`InfoBarAutomationPeer.cpp` 20–54 行定义通知优先级。

## 依赖和实现边界

- 复用已验证的 `FluentSurface`、`FluentText`、`FluentIcon`、`Button`、主题令牌、共享指针与键盘焦点。通知条直接参与页面布局。Gallery 用既有滚动视口验证窄窗。
- 实现标题、消息、四种级别、默认图标及其显隐、文字动作按钮、关闭按钮及其显隐、`is-open` 和同步 `closing` / `closed`。级别符号用自绘几何，技术规格采用官方样式资源。提示区域使用 Slint 的实时通知语义。
- 自定义图标、任意附加内容和定制动作模板为后续扩展。官方 Windows 字形和实机场景截图、VoiceOver 通知与实体触控需要目标环境验证。

## 本机验证

- Apple M3、macOS 26.4.1、arm64 Debug 与 Skia。通过原生 2× Retina 和 1.25× 渲染缩放的宽窄窗口冒烟。逐个检查四种级别在亮色、暗色和高对比中的底色与图标。宽窗验证短文横排，长文场景验证纵向排列和高度随实际换行增长。320 逻辑像素宽窗口验证通知条保持在视口宽度内。
- 验证鼠标和键盘关闭入口、Esc 保留非模态通知、取消关闭、关闭后释放焦点和折叠布局、隐藏关闭按钮、程序关闭原因、重复关闭和重新打开。新增导航项后缩小窄窗页头与按钮间距，并复验 ComboBox、Tooltip 等既有控件。
- 产物位于 `target/debug/fluent-gallery`。像素检查在内存进行，临时输出目录和 Python 缓存没有新增文件。Windows 画面对照、系统读屏和实体触控未测试。
