# ProgressRing

分析日期 2026-11-18。本地只读参考为 `references/microsoft-ui-xaml`（main，`354bf6017e6443023214d34309dfdf0fe2d526b1`）和 `../WinUI-Gallery`（detached HEAD，`7360ac322414e118eadfb0c7c2f4ecc31a218f59`）。取值来自 Python 对官方源码与 Gallery 示例的提取。官方对照截图待提供。

## 规格

| 项目 | WinUI 源码证据 |
| --- | --- |
| 默认 32×32，最小 16×16，无焦点或点击。Inactive 不可见 | `controls/dev/ProgressRing/ProgressRing.xaml` 4–32 |
| 默认透明底环、强调色前景；高对比使用系统色 | `controls/dev/ProgressRing/ProgressRing_themeresources.xaml` 4–15 |
| 确定圆环 | `AnimatedVisuals/ProgressRingDeterminate.cpp` 61、168–237、269–277。32 画布中心线半径 8×1.77，线宽 1.5×1.77，圆头；按进度裁剪。`ProgressRing.cpp` 305–323 中向前播放，回退直接定位 |
| 不确定圆环 | `AnimatedVisuals/ProgressRingIndeterminate.cpp` 51、158–243、289–319。80 画布中心线半径 35、线宽 7.5、圆头，2 秒内转 900°，弧长先增至半圈再缩回 |
| Gallery | `WinUIGallery/Samples/ProgressRing/` 的 DeterminateProgressRing 和 IndeterminateProgressRing 示例含 60×60、启停及可选背景 |

## 依赖与准入

- 已实现 `FluentIcon` / Slint Path 原语与 Skia 渲染，P2 CheckBox 已验证 Path 圆头笔划。复用既有主题、`FluentMotion` 与区间归一化；各 Progress 控件独立。
- Gallery 中复现确定空／部分／满进度、默认尺寸和大尺寸、不确定循环、停用、减少动效、恢复和亮暗／高对比。无焦点或指针操作。平台待验证包括实际后台生命周期、读屏与帧时间。
- 使用 Slint ArcTo 绘制圆头几何。减少动效和宿主停用动画使用静态弧。TODO(spec) 原有确定动画的首尾关键帧细节、圆弧起点对齐和官方画面对照待补。

[Gallery 与 Windows 验证记录](../../docs/p2-range-validation.md)
