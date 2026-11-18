# Slider

分析日期 2026-11-18。本地只读参考为 `references/microsoft-ui-xaml`（main，`354bf6017e6443023214d34309dfdf0fe2d526b1`）和 `../WinUI-Gallery`（detached HEAD，`7360ac322414e118eadfb0c7c2f4ecc31a218f59`）。取值来自 Python 对官方源码与 Gallery 示例的提取。官方对照截图待提供。

## 规格

| 项目 | WinUI 源码证据 |
| --- | --- |
| 水平／垂直最小横截面 32，滑块布局 18，外圈向外延伸 2、圆角 10，内圆基准 12 | `controls/dev/CommonStyles/Slider_themeresources.xaml` 161–173、192–199、400–434 |
| 轨道亮暗色厚 4、高对比厚 2，圆角 2。外侧刻度长 4，与轨道相隔 4 | 同文件 5–7、80–82、162、411–434 |
| Normal、PointerOver、Pressed、Disabled 内圆比例 0.86、1.167、0.71、1.167 | 同文件 206–250。公共资源 `ControlFastAnimationDuration` 167 ms、`ControlNormalAnimationDuration` 250 ms，`ControlFastOutSlowInKeySpline` 为 (0,0,0,1) |
| 外圆、轨道及选中段 | 同文件 `SliderOuterThumbBackground`、`SliderThumbBorderBrush`、`SliderTrackFill*`、`SliderTrackValueFill*`。颜色见 `controls/dev/CommonStyles/Common_themeresources_any.xaml` 22–24、226–228 |
| 交互 | `dxaml/xcp/dxaml/lib/Slider_Partial.cpp` 1722–1814、1993–2028。拖动以步长或刻度频率吸附到零点网格并包含区间端点，键盘步进取 SmallChange 或 TickFrequency |
| 键盘 | `dxaml/xcp/components/controls/KeyDownUp/inc/SliderKeyProcess.h` 26–69。四个方向键、Home、End，水平键受 RTL 与反向联合作用，垂直键受反向作用 |
| 刻度密度 | `dxaml/xcp/dxaml/lib/TickBar_Partial.cpp` 100–167、229–262 和同名头文件。刻度起点为区间最小值，单物理像素，过密时按 20 逻辑像素门槛跳过部分标记 |
| Gallery | `WinUIGallery/Samples/Slider/` 的 SliderSimple、SliderRange、SliderTicks、SliderVertical 示例，覆盖普通、范围步长、刻度、垂直 |

## 依赖与准入

- 已实现 `ui/interaction.slint` 的指针捕获区域、Slint FocusScope、`ui/primitives.slint` 的焦点环，以及 `ui/tokens.slint` 和 `ui/motion.slint` 的主题与动效。P1 和 P2 的 Gallery 冒烟测试已验证亮暗、键盘焦点、禁用及指针事件。
- Slider 与两个进度控件共用区间归一化。Gallery 绑定 Slider 数值驱动进度控件，控件之间无反向依赖。
- 本轮复现检查包含点击、拖动到区间外、禁用取消、焦点、键盘、步长、刻度吸附、亮暗与高对比渲染及窄窗命中尺寸。平台待验证包括实际触控、读屏和性能采样。
- TODO(spec) 官方拖动提示依赖 P3 弹层能力。Windows 系统高对比色、焦点边缘、外圈渐变和触控滚动竞争待用对应设备和参考画面核验。

[Gallery 与 Windows 验证记录](../../docs/p2-range-validation.md)
