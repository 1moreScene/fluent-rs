# RepeatButton

实现与 Mac 验证已完成；结果、构建输入、并行开发边界和未测试项见 [本机验证记录](../../docs/p4-button-extensions-validation.md)。

研究日期 2026-11-19。本批先做 RepeatButton，再做独立依赖既有按钮原语的 HyperlinkButton；均未在当前仓库实现。RepeatButton 是后续 NumberBox 步进按钮的前置，不提前实现 NumberBox。

## 一级参考与规格

只读 `references/microsoft-ui-xaml` / main / `07e0354d2e06572501ef48078a3cc91138749c9f`，`references/WinUI-Gallery` / main / `6de91feccadfbb0a796b71caedead5a2284f730e`。用 `python3 -B tools/reference_analysis/analyze_repeat_hyperlink.py` 复现资源与行号提取。使用常规 Default 样式，perf2026 变体未作为稳定基线。

| 规格 | 证据 |
| --- | --- |
| 外观、ButtonPadding、字号、ControlCornerRadius、1px 描边、83ms 背景过渡均与普通 Button 共用 | `controls/dev/CommonStyles/RepeatButton_themeresources.xaml` 5–17、78–99 |
| 默认 ClickMode=Press，按下立即 Click，释放不额外 Click | `dxaml/xcp/dxaml/lib/RepeatButton_Partial.cpp` 24–29；`components/controls/KeyDownUp/inc/ButtonBaseKeyProcess.h` 40–56、102–115 |
| Delay=500ms、Interval=33ms；Delay 必须非负、Interval 必须正 | `dxaml/xcp/components/DependencyObject/DependencyProperty.cpp` 714–720；RepeatButton 原生 149–177 |
| Space 使用内部定时器；Enter 按 ButtonBase 接受规则立即 Click，但不启动 Space 连发 | RepeatButton 原生 200–249；ButtonBaseKeyProcess 12–18、36–56 |
| 默认 Press 模式下，指针按住、移出暂停/再进入恢复；释放、失焦、禁用停止；首次 tick 后改用 Interval | RepeatButton 原生 184–194、251–267、308–465、493–574 |
| 支持 Hover/Release ClickMode 的重复行为 | 原生 308–379、410–465；ButtonBaseKeyProcess 32–35、53–56、102–115 |
| Gallery 按住累计次数、禁用、辅助功能通知 | `WinUIGallery/Samples/RepeatButton/RepeatButtonPage.xaml` 21–37、同名 `.cs` 17–24 |

高对比映射使用 RepeatButton 资源本身，不套用普通 Button 的高对比反转色。Default/Light 外观复用 ButtonSurface。

## 依赖准入

| 能力 | 实现 / 状态 |
| --- | --- |
| 颜色、尺寸、83ms、减少动画 | `ui/tokens.slint`、`ui/motion.slint`，已实现并经 Mac 原生回归 |
| 按钮外观与焦点 | `ui/button-surface.slint`、`ui/interaction.slint`，已实现；扩展共享 ClickMode 与指针捕获输出，默认 Release 保持原调用方式 |
| 持续按压与重复调度 | 复用 Slint TouchArea 捕获、FocusScope 与 Timer；在正式组件前通过最小 Gallery 验证 |
| 下游 | NumberBox / 需要按住步进的组合控件；不与任何组合控件形成循环依赖 |

最小验证覆盖即刻触发、延迟与后续重复、Space/Enter 差别、移出/进入、释放、取消、失焦、页面离开、禁用、参数变化、Hover/Release、亮暗和高对比、焦点来源、触控最小尺寸及窄窗。只在按压请求期间运行 Timer，空闲无重复调度。非法时序值禁用重复并暴露 timing-valid，避免忙循环；Slint 属性没有 WinRT setter 抛错契约，宿主应先验证输入。

完成门槛：共享 Button/Toggle/SplitButton/PasswordBox 的回归通过，实际有限时间检查证明停止后计数不再增加；官方 Windows 截图、真实触摸及屏幕阅读器未测试时明确记录。

Slint Flickable 的鼠标拖动平移会暂存下发给子项的 press。示例页沿用 ToggleSwitch / Slider 页的 `mouse-drag-pan-enabled: false`，使按下立即生效；滚轮和原生触摸滚动继续由 Slint 处理。真实触摸暂存/取消时序尚待设备验证。

Delay=0 通过 Slint 的下一 UI 调度（1ms 最小定时值）执行；不把功能连发当作视觉动效，减少动画不会取消按住操作。时序回归直接记录 Click 回调的单调时钟时间，分别检查默认 500ms 首次延迟和 180ms 配置的恢复/键盘延迟，允许 5ms 采样误差；避免将长时间的同步快照渲染算作“90ms 时仍未触发”的观察点。
