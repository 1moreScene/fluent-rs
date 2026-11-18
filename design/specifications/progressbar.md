# ProgressBar

分析日期 2026-11-18。本地只读参考为 `references/microsoft-ui-xaml`（main，`354bf6017e6443023214d34309dfdf0fe2d526b1`）和 `../WinUI-Gallery`（detached HEAD，`7360ac322414e118eadfb0c7c2f4ecc31a218f59`）。取值来自 Python 对官方源码与 Gallery 示例的提取。官方对照截图待提供。

## 规格

| 项目 | WinUI 源码证据 |
| --- | --- |
| 指示段高 3、圆角 1.5，底轨高 1、圆角 0.5 | `controls/dev/ProgressBar/ProgressBar_themeresources.xaml` 29–32，`ProgressBar.xaml` 155–173 |
| 强调色、暂停色、错误色，底轨为 ControlStrongStrokeColorDefault | 主题资源 4–26。公共资源 `SystemFillColorCaution` 暗 #FCE100、亮 #9D5D00，`SystemFillColorCritical` 暗 #FF99A4、亮 #C42B1C |
| 确定进度按区间归一化，零跨度显示空。错误优先于暂停。无焦点 | `ProgressBar.cpp` 的 UpdateStates 和 SetProgressBarIndicatorWidth，`ProgressBar.xaml` 10 |
| 不确定进度 | `ProgressBar.cpp` 168–200、223–232，`ProgressBar.xaml` 94–111。周期 2 秒，40% 段从 −40% 移至 120%，60% 段在 750 ms 后从 −90% 移至 99.6%，曲线 (0.4,0,0.6,1)。暂停或错误时全宽着色 |
| Gallery | `WinUIGallery/Samples/ProgressBar/` 的 DeterminateProgressBar 和 IndeterminateProgressBar 示例 |

## 依赖与准入

- 已实现矩形绘制、主题 Tokens、`FluentMotion` 偏好。在 P1 Gallery 中已验证 Skia 渲染和减少动效接口。本控件与 Slider、ProgressRing 共用区间归一化。
- Gallery 中复现 0、部分、满进度、暂停、错误、恢复、不确定循环、宿主暂停动效及亮暗／高对比。无焦点或指针操作。性能验证包含检查静态模式停止循环。
- 减少动效时采用居中的静态段。不确定暂停／错误直接切为全宽状态。TODO(spec) WinUI 状态切换的位移动效及官方画面对照待补。

[Gallery 与 Windows 验证记录](../../docs/p2-range-validation.md)
