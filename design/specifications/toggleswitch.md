# ToggleSwitch

分析日期 2026-11-19，与 PasswordBox 同批。只读参考 WinUI `main` 的 `07e0354d2e06572501ef48078a3cc91138749c9f` 和 Gallery `main` 的 `6de91feccadfbb0a796b71caedead5a2284f730e`，路径分别为 `references/microsoft-ui-xaml` 与 `references/WinUI-Gallery`。

运行 `python3 -B tools/reference_analysis/analyze_password_switch.py` 可提取资源与行为。采用常规 DefaultToggleSwitchStyle，perf2026 变体的发布范围未确认。

| 规格与行为 | 来源 |
| --- | --- |
| 最小宽 154，标题下距 4，开关区上下各 10，开关与状态文字间距 12 | `controls/dev/CommonStyles/ToggleSwitch_themeresources.xaml` 185–200、492–506 行 |
| 轨道 40×20、圆角 10，滑块槽 20×20、位移 20；圆点 12、悬停 14、按下 17×14 | 同模板 231–241、268–287、311–321、445、507–521 行 |
| Off 使用 ControlAltFill、ControlStrongStroke、TextFillSecondary；On 使用 AccentFill 和 TextOnAccentFill，单独映射禁用和高对比 | 同模板 4–38、64–99、124–159 行 |
| 状态颜色和圆点尺寸使用 83ms 与 0,0,0,1；位移使用 RepositionThemeAnimation | 同模板 231–321、374–438 行，`Common_themeresources_any.xaml` 602、606 行 |
| Space 松开后切换，Enter 不切换；拖动以位移中点决定状态，取消不提交 | `dxaml/xcp/dxaml/lib/ToggleSwitch_Partial.cpp` 594–617、805–864、1004–1008 行；`components/controls/KeyDownUp/inc/ToggleSwitchKeyProcess.h` |
| 标题不参与 Thumb 命中，正文状态文字可点击；正文尺寸同时预留 On 与 Off 文本 | 模板 492–524 行；原生 250–254 行将 Tap 绑定到 SwitchThumb |
| Gallery 提供默认标签及 Header、OnContent、OffContent，并绑定 ProgressRing | `WinUIGallery/Samples/ToggleSwitch/ToggleSwitchPage.xaml` 22–34 行 |

## 依赖准入

依赖 `ui/tokens.slint`、`ui/motion.slint`、`ui/interaction.slint` 的 FluentPressable、FluentPointerArea 与焦点反馈，以及 `ui/primitives.slint` 的文本和焦点环，均已有验证。给 FluentPressable 补充原始指针通知与点击提交开关，拖动继续复用 Slint TouchArea 的捕获。新控件提供二态 IsOn，程序赋值与用户输入使用同一属性。

验证覆盖 On/Off 的正常、悬停、按下、焦点、禁用，亮暗及高对比，Space/Enter、一次通知、点击正文、标题不切换、拖动过中点与取消、长标签、窄窗及不同缩放、减少动画，以及有限时间的位移和形变动画。公开控件和静态状态矩阵复用 ToggleSwitchSurface。

`TODO(spec)` 原生位移时序由 `ThemeAnimations.cpp` 972–995 行的 TAS_REPOSITION 系统主题生成，仓库不包含该系统曲线。当前位移沿用共享 167ms 控件过渡作为宿主回退，通过 FluentMotion 的可覆写属性区分该回退与官方参数。圆点形变和透明度的 83ms 来自官方资源。减少动画时立即定位。原版截图对照、Windows 系统时序、真实触摸和屏幕阅读器待对应环境验证。

## 本机验证记录（2026-11-19）

与 PasswordBox 共用上述源码快照及验证命令，见 [本机验证记录](passwordbox.md#本机验证记录2026-11-19)。macOS 26.4.1 ARM64 / Skia Debug 下，亮暗/高对比、正常/悬停/按下/焦点/禁用、Space 与 Enter 区别、标题不切换、状态文字点击、程序赋值通知、拖动中点提交与禁用取消、长标签及 320px 窄窗均通过。实际像素确认滑块正常 < 悬停 < 按下的宽度，300ms 后位移到相反槽位，减少动画时立即到位。该验证不替代官方 Windows 系统曲线测量，也未测量帧时间或移动端输入。
