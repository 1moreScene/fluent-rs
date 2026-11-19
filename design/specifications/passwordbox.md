# PasswordBox

分析日期 2026-11-19。本批两控件为 PasswordBox 与 ToggleSwitch，仅在当前 Mac 构建和验证。

## 一级参考

只读 WinUI `references/microsoft-ui-xaml`，`main`，`07e0354d2e06572501ef48078a3cc91138749c9f`。Gallery `references/WinUI-Gallery`，`main`，`6de91feccadfbb0a796b71caedead5a2284f730e`。运行 `python3 -B tools/reference_analysis/analyze_password_switch.py` 提取证据。

| 规格与行为 | 来源 |
| --- | --- |
| TextBox 公共尺寸、文字、主题描边和圆角，标签下留 8，窥视图标 12 | `controls/dev/CommonStyles/PasswordBox_themeresources.xaml` 8–26、100、189–193 行 |
| 窥视按钮不进 Tab，初始宽 30，布局后等于输入区实际高度；内部上、右、下各留 4 | 上述模板 36、193 行，`dxaml/xcp/dxaml/lib/PasswordBox_Partial.cpp` 817–835 行；`TextBoxInnerButtonMargin` |
| 默认遮蔽字符为 U+25CF 黑圆 | `dxaml/xcp/core/native/text/Controls/PasswordBox.cpp` 333–340 行；Slint 1.19 `items/text.rs` 的 `PASSWORD_CHARACTER` 为同一字符 |
| Peek、Hidden、Visible；Peek 按下显示、释放隐藏；Alt+F8 按下显示，F8 释放隐藏 | 原生 `PasswordBox_Partial.cpp` 349–385、919–939 行，`PasswordBox.cpp` 534–561 行 |
| 聚焦已有密码不出现窥视钮；本次焦点期间从空内容输入才允许显示；失焦、清空、宽度不超过 5em 时隐藏 | 原生 `PasswordBox.cpp` 234–248、367–378、432–435、574–578、619–626 行 |
| Gallery 覆盖基本输入、标题和占位符、设置 PasswordChar、切换 Hidden/Visible | `WinUIGallery/Samples/PasswordBox/PasswordBoxPage.xaml` 22–57 行 |

## 依赖准入

| 依赖 | 实现与状态 |
| --- | --- |
| 主题、尺寸与输入外观 | `ui/tokens.slint`、`ui/text-box.slint` 的 TextBoxSurface，已实现 |
| 编辑、选区、横向滚动、输入法 | 复用同一个 Slint TextInput，已实现；真实输入法及移动软键盘待平台验证 |
| 指针捕获与焦点 | `ui/interaction.slint`，已实现；窥视钮使用不进入 Tab 的指针区域 |
| Gallery 与状态回归 | 已实现，新增独立 PasswordBox 页及原生输入、像素检查 |

TextBox 增加敏感输入、显示状态和输入法提示接口，PasswordBox 组合该编辑器及共享外观。窥视不会重新创建 TextInput。复制和剪切快捷键在显示与遮蔽模式均拦截，粘贴和选区沿用编辑引擎，辅助功能值不输出密码。示例输出编辑通知次数，不输出密码。

本机 CoreText 测量 14px 的 U+25CF，系统字体包围盒约 11.04×11.03，Helvetica Neue 为 5.96×5.96。Mac 宿主设置 `FluentTheme.password-mask-font-family` 为 Helvetica Neue。该测量属于本机字体数据，原版截图尺寸仍待核验。共享 TextBoxActionSurface 明确导出内缩区域尺寸，清除与窥视图标使用该尺寸居中，避免插槽的外层 parent 坐标造成偏移。

准入与完成检查包括亮暗及高对比、正常/悬停/焦点/禁用、键盘 Tab、输入后窥视、拖出取消、Alt+F8、失焦隐藏、预填值不显示窥视钮、三种显示模式、复制/剪切保护、粘贴、选区和长内容滚动。不同窗口和缩放下检查按钮列、图标居中与输入框边界。

`TODO(spec)` Slint 1.19 的遮蔽字符固定为 U+25CF，自定义 PasswordChar 尚未提供。真实 IME、移动安全键盘、屏幕阅读器、原版同主题同缩放截图仍需对应环境。密码保存在 Slint 字符串状态中，未实现 WinUI 原生的内存保护策略。

## 本机验证记录（2026-11-19）

macOS 26.4.1 / ARM64 / Slint 1.19.0 / Skia，Debug。固定源码快照为 `.artifacts/password-switch-build-20261119`，`source-manifest.json` 记录每个源码与资源的 SHA-256。

- `cargo fmt --all -- --check`、locked workspace check、clippy all-targets（拒绝警告）、workspace test 均通过；21 个 Rust 单元测试通过。仅当前 Mac 构建。
- `SLINT_SCALE_FACTOR=2 target/debug/fluent-gallery --smoke-test` 完整 Gallery 通过，宽窗 1960×1400 backing、窄窗 640×1280 backing。包含共享 TextBox、ComboBox、弹层、对话框和导航回归。
- `--smoke-test password-switch` 在 backing scale 2 与 1.25 分别通过；每次先验证宿主默认 125% UI，再验证 100% UI，最后验证 320×640 逻辑像素触控布局。覆盖主题、输入、按住窥视/拖出/释放、Alt+F8、Tab、预填值与失焦、禁用、长文本行首行尾滚动、系统剪贴板及 ToggleSwitch 动效。行首行尾采用 Mac 的原生快捷键。剪贴板测试保留并恢复原有全部条目/格式，检测到外部新写入时不覆盖。
- 从实际 Skia 缓冲提取圆点暗像素高度，按 UI scale 归一化后为 5.76–6.50 逻辑像素；眼睛图标暗像素中心与内缩绘制区中心差小于 0.8 逻辑像素。共享文本框清除图标及按钮右侧留白命中检查也通过。
- 本次像素检查使用内存缓冲，未产生临时截图。保留约 1.1MB 源码快照用于复现；构建缓存继续复用。

Windows 官方运行截图对照、其他系统/最低版本、真实输入法、屏幕阅读器及触摸设备均**未测试**；本次不作为跨平台或像素级一致的验收结论。
