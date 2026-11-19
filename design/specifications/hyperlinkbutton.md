# HyperlinkButton

实现与 Mac 验证已完成；结果、构建输入、并行开发边界和未测试项见 [本机验证记录](../../docs/p4-button-extensions-validation.md)。

研究日期 2026-11-19。与 RepeatButton 同批，二者互不依赖；上游是已经验证的共享输入、焦点、Button 尺寸、文字、主题与动效。

## 一级参考

只读 WinUI `main` / `07e0354d2e06572501ef48078a3cc91138749c9f`、Gallery `main` / `6de91feccadfbb0a796b71caedead5a2284f730e`，实际路径为项目 `references/` 下对应目录。分析脚本为 `tools/reference_analysis/analyze_repeat_hyperlink.py`。

| 规格 | 证据 |
| --- | --- |
| AccentTextFill Primary/Secondary/Tertiary/Disabled；Subtle 背景；透明描边；ButtonPadding；圆角；83ms 背景过渡 | `controls/dev/CommonStyles/HyperlinkButton_themeresources.xaml` 5–17、51–70 |
| Default/Light 的文字强调色来自系统色阶，不是 Accent 按钮上的黑白字 | `Common_themeresources_any.xaml` 93–96、297–300 |
| 常规文字不画下划线，高对比强制下划线；资源可以显式开启 | `Hyperlink_themeresources.xaml` 20；`dxaml/xcp/dxaml/lib/HyperlinkButton_Partial.cpp` 180–212 |
| 手型光标；先 Click 回调，然后读取 NavigateUri，再调用系统 Launcher | 上述原生 27–33、148–172 |
| Space/Enter 与 ButtonBase 一致，默认释放触发 | `components/controls/KeyDownUp/inc/ButtonBaseKeyProcess.h` 12–18、76–120 |
| 一例打开 Microsoft 主页，一例 Click 导航到 ToggleButton | `WinUIGallery/Samples/HyperlinkButton/HyperlinkButtonPage.xaml` 22–48、同名 `.cs` 17–20 |
| 原生辅助功能角色 Hyperlink | `HyperlinkButtonAutomationPeer_Partial.cpp` GetAutomationControlTypeCore |

## 依赖与平台边界

复用 `ui/interaction.slint` 的 FluentPressable，`ui/primitives.slint` 文本与焦点环，`ui/tokens.slint` 尺寸与颜色，`ui/motion.slint` 减少动画；均已实现。HyperlinkButton 的 Click 与 URI 请求分离，URI 请求交给宿主回调，控件层不直接调用平台 API。Gallery 宿主在 Mac 使用 NSWorkspace / NSURL；冒烟中注入记录器，避免自动打开外部浏览器。无 URI 的链接只发送 Click，用于页内导航。

新增 Accent 文字色阶输入。Mac 没有 Windows UISettings，默认采用官方 `SystemThemingInterop.cpp` 216–240 明确写出的跨平台测试覆盖色阶（Dark1 #005A9E、Dark2 #004275、Dark3 #002642、Light2 #76B9ED、Light3 #A6D8FF）。这是可追溯的固定宿主回退，**不冒称当前 Windows 用户默认强调色阶**。宿主可覆写完整色阶；原版运行时调色由 Windows UISettings 提供（原生 244–285）。

Slint 1.19 没有 Hyperlink accessible-role，当前以可聚焦 Button 语义和链接标签提供激活操作，记录角色差异，不制造自定义焦点引擎。下划线使用 Slint 字体引擎；自定义复杂内容模板尚未提供。

最小验证：普通与 URI 两种操作、Click 在 URI 请求之前且可清空 URI、禁用、鼠标拖出取消、Space/Enter、Tab 与指针焦点区别、亮暗/高对比及下划线、四状态像素、文本换行、125%/100% UI 与不同 backing scale、320px 窄窗。Windows 系统调色/实际默认浏览器唤起、真实触摸、屏幕阅读器及官方截图对照需对应环境。

高对比资源进一步按 `dxaml/xcp/dxaml/themes/generic.xaml` 2095、2099 行解析：PageBackgroundTransparent 实际是 WindowColor（本项目 contrast-surface），描边才是透明；不能把资源键名中的 Transparent 直接当作透明值。
