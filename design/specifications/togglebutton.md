# ToggleButton

分析日期 2026-11-18。WinUI 源码 `references/microsoft-ui-xaml`，分支 `main`，提交 `354bf6017e6443023214d34309dfdf0fe2d526b1`。WinUI Gallery `../WinUI-Gallery`，detached HEAD，提交 `7360ac322414e118eadfb0c7c2f4ecc31a218f59`。

`python -B tools/reference_analysis/analyze_toggles.py --gallery-source ../WinUI-Gallery` 可复现资源与状态提取。

## 前置依赖

- 已实现并经 Windows Gallery 验证的 `ui/interaction.slint`（指针捕获、Space/Enter 和焦点），`ui/button-surface.slint`（普通和强调色表面），`ui/tokens.slint`、`ui/motion.slint`、`ui/primitives.slint`（主题、过渡和焦点环）。
- 本轮先补可共享的三态值与切换行为，再供 ToggleButton 和 CheckBox 调用。值与输入视觉状态独立。Gallery 通过绑定读取状态。共用层不依赖 Gallery。
- Gallery 最小准入检查覆盖亮暗主题、Tab 焦点、按下与松开、拖出取消、禁用、重入及紧凑触控尺寸。真实触控与读屏待平台验证。

## 官方证据

| 行为 | 证据 |
| --- | --- |
| Normal / PointerOver / Pressed / Disabled | 每个选中值都有一组状态，禁用优先，随后是按下与悬停。焦点独立。 `dxaml/xcp/dxaml/lib/ToggleButton_Partial.cpp` 71–171 行 |
| 尺寸与动画 | `DefaultToggleButtonStyle` 复用 `ButtonPadding`、14 逻辑像素、4 圆角、1 描边，焦点外扩 3，背景过渡 83 ms。 `controls/dev/CommonStyles/ToggleButton_themeresources.xaml` 180–201 行 |
| 亮暗主题 | Unchecked 和 Indeterminate 使用普通控件填充，Checked 使用强调色填充及强调色描边。 `ToggleButton_themeresources.xaml` 7–42、123–158 行 |
| 高对比 | 模板按选中值配置高对比配色，Checked 悬停用 ButtonText 底与 ButtonFace 前景，按下用 HighlightText 底与 Highlight 前景。 `ToggleButton_themeresources.xaml` 62–100 行 |
| 键盘和切换 | 默认接受 Enter；点击先切换选中值。三态为 Unchecked → Checked → Indeterminate → Unchecked。二态的 Checked 返回 Unchecked，宿主写入的 Indeterminate 也返回 Unchecked。 `ToggleButton_Partial.cpp` 25–32、177–182、265–296 行 |
| Gallery | 绑定当前状态并提供禁用入口。 `WinUIGallery/Samples/ToggleButton/ToggleButtonPage.xaml` 21–35 行 |

`value` 可双向绑定，`toggled(value)` 在用户激活且值更新后触发。高对比色沿用宿主可配置调色板。中间态通过状态描述暴露；Slint 1.19 的 AccessKit 桥只公开布尔选中状态。官方像素对照待截图。

Windows 冒烟结果和仍需的平台环境见 [P2 验证记录](../../docs/p2-validation.md)。
