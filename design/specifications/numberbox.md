# NumberBox

研究日期 2026-11-19。先验证上一批 RepeatButton，再组合 NumberBox；同批 PersonPicture 与之独立。避开已有并行 GridView、DropDownButton、ToggleSplitButton 实现。

## 一级参考

只读 `references/microsoft-ui-xaml` / main / `07e0354d2e06572501ef48078a3cc91138749c9f`；`references/WinUI-Gallery` / main / `6de91feccadfbb0a796b71caedead5a2284f730e`。用 `python3 -B tools/reference_analysis/analyze_number_person.py` 复现证据提取。采用常规模板，预览 InputScope/TextAlignment API 和 perf2026 样式不作为稳定基线。

| 规格/行为 | 官方文件（WinUI 相对根目录） |
| --- | --- |
| Hidden 默认，Compact、Inline；空值 NaN；SmallChange=1、LargeChange=10；默认覆盖无效输入、关闭 Wrap/Expression | `controls/dev/NumberBox/NumberBox.idl` 6–19、35–52、80–96 |
| 复用 TextBox；Inline 最小宽 120，尾列 72；两个 32 宽 RepeatButton，边距 4，12 号上下箭头，不进入 Tab | `NumberBox.xaml` 28–39、113–115、167–189、335–365；`NumberBox_themeresources.xaml` 29–35 |
| Compact 聚焦即打开；失焦提交并关闭；聚焦全选 | `NumberBox.cpp` 414–438 |
| Compact 弹层相对尾列偏移 -21/-27，内距 6、描边 1、圆角 Overlay，两个 36×36、16 号箭头，上下间距 4，Acrylic | `NumberBox.xaml` 115–164、193–203；主题资源 5–26、31–33 |
| 方向键小步、PageUp/PageDown 大步；Enter 松开提交、Esc 松开恢复；仅输入框有焦点时滚轮步进 | `NumberBox.cpp` 533–596 |
| 步进先提交文本，空值不步进；越界夹取或跳至另一端；Disabled validation 保留无效文本且允许越界 | `NumberBox.cpp` 459–519、599–628、680–708 |
| 值变化通知只发一次，程序赋值也通知；Min 改动抬高 Max，Max 改动降低 Min | `NumberBox.cpp` 243–289、441–475 |
| 默认区域数字格式，最低小数位 0，显示前舍入 10 位有效数字；实际值不因显示舍入丢失精度 | `NumberBox.cpp` 81–114、216、633–655 |
| 表达式 + - * / ^、括号、负数；相同优先级左结合（2^3^2=64）；除零结果 NaN，与语法错误不同 | `NumberBoxParser.cpp` 9、23–63、77、95–107、122–137、182–252 |

Gallery `WinUIGallery/Samples/NumberBox/NumberBoxPage.xaml` 22–65：表达式、Inline/Compact、小步 10/大步 100、货币格式示例。

## 依赖准入与接口

| 前置 | 路径 / 状态 |
| --- | --- |
| 编辑、光标、IME、选区、清除按钮及主题 | `ui/text-box.slint` 已实现并经本机回归；增加通用滚轮回调、清除按钮列宽接口，默认保持原行为 |
| 重复动作、指针捕获和取消 | `ui/repeat-button.slint`、`ui/interaction.slint` 已验证；增加关闭默认表面的组合接口，不另写连发计时器 |
| 弹层、定位、材质、Esc/外部关闭 | `ui/overlay-host.slint` 已验证；Compact 使用同一宿主，保持编辑焦点 |
| 数值模型 | 新增实际有消费者的 `crates/fluent-core`，纯 Rust 模块负责 f64 解析/格式/表达式/范围。无 UI 或平台依赖，供本控件和后续数值编辑消费者复用 |

Slint 的 float 为 f32；本控件 Value、Minimum、Maximum、SmallChange、LargeChange 使用规范数字字符串桥接 Rust f64，空 Value 对应 NaN。值和编辑 Text 分离，避免把输入先截断为 f32。Rust 模型公开 f64 类型；宿主注册 `NumberBoxLogic`，可覆写区域数字的解析/格式。默认小数点 `.`，可配置小数分隔符和最低小数位；Windows Globalization 的所有区域/货币格式不在此轮支持，明确属于宿主接口，不能声称已完整移植。

准入先跑 RepeatButton 专项与 Rust 模型测试，再验证 Hidden/Inline/Compact，表达式、空值、无效文本、单次通知、Min/Max、Wrap、键盘、滚轮、连发停止、聚焦全选、Esc、关闭恢复、禁用、亮暗/高对比和窄窗/缩放。复用现有输入和弹层逻辑，不引入第二套焦点或滚动引擎。真实 IME/触摸及 Windows 官方运行截图对照仍需对应环境。

辅助功能采用 Spinner/Spinbox 角色及增减/设值操作，共享 TextBox 将编辑辅助功能集中到可覆盖的根节点，本控件由外层 Spinbox 暴露唯一编辑角色；Slint 仍从原生 TextInput 提供文本运行、选区和编辑操作。Slint 的辅助功能范围元数据为 f32，超范围时仅该元数据限幅；实际 Value 与文本读写仍走 f64 规范字符串。真实屏幕阅读器需另行验收。

Compact 的 Esc 只恢复文本，不关闭步进弹层（`NumberBox.cpp` 560–574）；Tab 离开时提交并关闭。源码 Popup 明确允许超出根窗口（模板 115），本项目共享弹层受根窗口约束并在边缘限位，属于 Slint 宿主差异。

共享非模态弹层的外部首击用于关闭并提交，不透传到背后控件；关闭后清除编辑焦点，避免 Slint Popup 关闭时恢复旧焦点而重新展开。该首击策略与原生失焦路由的完整等价性仍需 Windows 对照。

`NumberBox.xaml` 200–203 的 ThemeShadow Translation.Z=16 为官方深度依据；当前复用共享 Slint 阴影令牌（模糊 8、下偏 2），两种渲染器的阴影像素对应仍为 `TODO(spec)`，不能把深度值直接当作 blur 半径。
