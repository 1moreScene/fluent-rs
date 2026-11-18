# RadioButton 与 RadioGroup

分析日期 2026-11-18。WinUI 源码 `references/microsoft-ui-xaml`，分支 `main`，提交 `354bf6017e6443023214d34309dfdf0fe2d526b1`。WinUI Gallery `../WinUI-Gallery`，detached HEAD，提交 `7360ac322414e118eadfb0c7c2f4ecc31a218f59`。

`python -B tools/reference_analysis/analyze_radio.py --gallery-source ../WinUI-Gallery` 可重新提取模板、资源、方向键源码与 Gallery 示例。

## 依赖准入

- `ui/interaction.slint` 的 `FluentPressable` 已验证指针命中、释放激活、取消和键盘焦点。RadioButton 复用这些行为，使用 Space 激活。外观复用 `FluentSurface`、`FluentFocusRing`、`FluentText` 与通用颜色令牌。
- RadioGroup 按实例持有单选索引。RadioButton 不依赖 RadioGroup。RadioGroup 使用 Slint 焦点系统移动实际焦点，Rust 导航只计算目标索引。无需 Popup 或 F3 的通用列表框架。
- 最小验证覆盖两组独立互斥、选中与焦点分离、Tab、方向键、Space、禁用、亮暗与高对比、窄窗口和触控尺寸。真机触控与读屏待平台验证。

## 官方行为

| 项目 | 规格及证据 |
| --- | --- |
| 尺寸 | 20 逻辑像素外环，32 高的图形槽，内容左 8 上 6，最小宽 120，焦点横向外扩 7、纵向 3。 `controls/dev/CommonStyles/RadioButton_themeresources.xaml` 183–197、366–378 行 |
| 状态外观 | 未选中用强描边及 ControlAltFill，已选中用强调色外圈和 TextOnAccent 内点。选中内点常态 12、悬停 14、按下 10。未选中按下显示临时内点。高对比按 ButtonFace、ButtonText、Highlight、HighlightText 和 GrayText 映射。 `RadioButton_themeresources.xaml` 24–50、82–108、204–356 行 |
| 动效 | 悬停与按压的内点使用 250 ms 或 167 ms 的尺寸过渡及 `ControlFastOutSlowInKeySpline`。 `RadioButton_themeresources.xaml` 255–305、339–342 行，`Common_themeresources_any.xaml` 602–604 行 |
| 单项 | 只可激活为选中，重复激活保持选中，默认忽略 Enter。原生独立 RadioButton 的默认互斥范围为父容器，同名组以视觉根为范围，独立项的方向键扫描可回绕。 `dxaml/xcp/dxaml/lib/RadioButton_Partial.cpp` 25–31、184–205、507–522、563–685 行 |
| 组 | Gallery 使用的是 WinUI `RadioButtons` 容器。此容器单选，Tab 进入选中项，组内方向键默认随焦点选择，Ctrl+方向键只移动焦点。上下按列优先的索引顺序，左右按实际相邻列几何位置，外边界不回绕。 `controls/dev/RadioButtons/RadioButtons.cpp` 79–110、135–185、332–368、438–464 行及 `InteractionTests/RadioButtonsTests.cs` 219–317、613–632 行 |
| 组布局 | 列间距 7，行与标题下方间距 8。Gallery 展示了显式单选项，以及设定初始索引和最大列数的两个数据驱动组。 `controls/dev/RadioButtons/RadioButtons.xaml` 19–25 行及同目录 `RadioButtons_themeresources.xaml` 17–19 行，`WinUIGallery/Samples/RadioButton/RadioButtonPage.xaml` 21–61 行 |

项目的独立项提供可绑定的 checked；互斥和键盘组导航通过显式 RadioGroup 提供。RadioGroup 以位置索引绑定数据，`-1` 或越界时显示无选中项。窄宽度按最小单元宽度减列，横向导航优先最近可用列，再选最接近当前行的启用项。WinUI 对不等宽目标的空间导航排序待逐项对照。按减弱动效设置直接更新内点。 [微软 RadioButtons 设计说明](https://learn.microsoft.com/en-us/windows/apps/design/controls/radio-button)

应用为窗口的 `RadioNavigation.target` 注册 Rust 模型导航实现，Gallery 的绑定示例位于 `apps/gallery/src/lib.rs`，计算逻辑位于 `apps/gallery/src/radio_navigation.rs`。组件使用 Slint 的 radio-group / radio-button 可访问性语义和实际焦点。

## Windows 验证

Windows 11 Pro Build 26200 x64，Slint 1.19.0、Skia OpenGL。Rust 单元测试覆盖空组、全禁用、跳过禁用项、纵向边界和多列不等高时的目标选择。Windows 调试版在 100%、125%、150%、200% 缩放通过 720×620 与 320×640 逻辑像素的 Gallery 冒烟检查。

- 交互检查覆盖 Tab 进入已选项或首个启用项、Shift+Tab 重新进入、方向键选中、Ctrl 只移动焦点、Space 激活、Enter 保持原选择、禁止反复点击取消、指针拖出取消，以及两个组之间互不影响。
- Skia 像素检查覆盖亮色、暗色及高对比的选中内点和旧选项恢复未选中。窄窗检查自动减列与 44 逻辑像素的触控目标高度。
- 调试版默认页空闲采样在启动 4 秒后连续三次得到 46,379,008 字节 Private Bytes。帧时间、长期内存和真实触控开销未测试。
- Windows 10 1809、实际触控、读屏及官方截图对照未测试。分别需要 Build 17763 设备或虚拟机、触控设备、辅助技术，以及官方 Gallery 的同主题和同缩放截图。其他平台按仓库当前平台规则未构建。

参考 JSON、Skia 像素缓冲和内存采样均在内存中检查。已盘点临时目录，保留 Windows 调试版、正式分析脚本和现有构建缓存。
