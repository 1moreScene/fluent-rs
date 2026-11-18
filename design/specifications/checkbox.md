# CheckBox

分析日期 2026-11-18。WinUI 源码 `references/microsoft-ui-xaml`，分支 `main`，提交 `354bf6017e6443023214d34309dfdf0fe2d526b1`。WinUI Gallery `../WinUI-Gallery`，detached HEAD，提交 `7360ac322414e118eadfb0c7c2f4ecc31a218f59`。

`python -B tools/reference_analysis/analyze_toggles.py --gallery-source ../WinUI-Gallery` 可复现资源与状态提取。

## 前置依赖

- 已实现且经 Windows Gallery 验证的 `FluentPressable`、`FluentSurface`、`FluentText`、`FluentIcon`、`FluentFocusRing`、主题和 Motion。先建立与 ToggleButton 共用的三态值、切换行为和辅助语义，再接入 CheckBox。
- CheckBox 依赖共用切换基元并提供独立绘制。Gallery 消费同一绘制组件展示状态组合。无需 Popup、SelectionModel 或平台专属 API。
- 准入检查包括二态、三态、鼠标和键盘、禁用取消、亮暗与高对比、紧凑触控尺寸和选中标记渲染。触控硬件及读屏待平台验证。

## 官方证据

| 规格 | 证据 |
| --- | --- |
| 尺寸 | 最小 120×32、方框 20、图标字号 12、描边 1、内容左 8 上 5、焦点横向外扩 7 和纵向 3。 `controls/dev/CommonStyles/CheckBox_themeresources.xaml` 269–294、598–611 行 |
| 亮暗主题 | Unchecked 用 `ControlAltFillColorSecondary/Tertiary/Quarternary/Disabled` 和普通/禁用强描边。Checked 与 Indeterminate 使用同组强调色填充、描边和标记颜色。 `CheckBox_themeresources.xaml` 29–64、205–240 行 |
| 高对比 | 按值和交互状态映射 ButtonFace、ButtonText、Highlight、HighlightText 和 GrayText 等。 `CheckBox_themeresources.xaml` 93–152 行 |
| 输入 | 继承 ToggleButton 的切换；WinUI 二态额外识别数字键盘 Add/Subtract。Slint `KeyEvent` 未提供键盘区域，专用数字键盘操作待补。 `dxaml/xcp/dxaml/lib/CheckBox_Partial.cpp` 34–67、130–226 行 |
| 动效 | 方框颜色离散切换。按 60 fps 的笔画关键帧计，通常进入 19 帧（约 317 ms），持续按下时变为 Checked 则为 9 帧（150 ms），擦除 4 帧（约 67 ms）。进入曲线为 cubic-bezier(0.55, 0, 0, 1)，擦除近似线性。 `AnimatedAcceptVisualSource.cpp` 1443–1448、1464–1623 行；`AnimatedIcon.cpp` 420–449 行 |
| Gallery | 二态、三态、选择组的半选示例。 `WinUIGallery/Samples/CheckBox/CheckBoxPage.xaml` 21–85 行及同目录 `CheckBoxPage.xaml.cs` 49–98 行 |

图标用项目自绘路径。进入时沿笔画从起点绘制，取消时从起点擦除。按键松开或鼠标释放通常采用 19 帧；宿主在持续按下期间写入 Checked 时采用 9 帧。减少动效、高对比、禁用和半选直接切换。快速反向操作由 Slint 重新定向动画，WinUI 的跨状态排队细节及逐帧视觉对照待验证。Slint 1.19 将中间态通过辅助描述传递，原生 Mixed 和读屏验证仍待补。

Windows 调试版在 100%、125%、150%、200% 缩放下通过新增的动效像素检查。勾选约 170 ms 时检查笔画已进入末段，取消后等待 120 ms 检查标记已擦除。其余冒烟结果和平台环境见 [P2 验证记录](../../docs/p2-validation.md)。
