# PersonPicture

研究日期 2026-11-19，与 NumberBox 同批；前置是已验证的 Tokens、文本、图像和视觉基元，不依赖选择/弹层。

只读 WinUI `main` / `07e0354d2e06572501ef48078a3cc91138749c9f`，Gallery `main` / `6de91feccadfbb0a796b71caedead5a2284f730e`，实际位于项目 `references/`。提取脚本：`tools/reference_analysis/analyze_number_person.py`。

| 规格/行为 | 来源（WinUI 相对根目录） |
| --- | --- |
| 默认 96×96、半粗、非 Tab 目标，静态呈现无悬停/按压/禁用动效 | `controls/dev/PersonPicture/PersonPicture.xaml` 5–14、20–43 |
| Group 优先于 Photo，再 Initials、通用 Person；Initials 显式覆盖 DisplayName 生成结果；图像 UniformToFill | `PersonPicture.cpp` 108–178；模板 24–42、65–67 |
| 字号为直径 42%，忽略文字缩放；徽章直径为 50%、文字为徽章的 60%；Badge 位于右上并向右/上各超出 4 | `PersonPicture.cpp` 563–615；模板 66、68；主题资源 13–15 |
| BadgeImage 优先，其次非零 BadgeNumber，再 BadgeGlyph；负数隐藏徽章，不回落 Glyph；大于 99 显示 99+ | `PersonPicture.cpp` 185–289 |
| 灰底 ControlAltFill Quarternary、CardStroke 描边 1；徽章 Accent、TextOnAccent、描边 2；高对比单独映射 Window/HighlightText/ButtonFace/ButtonText | `PersonPicture_themeresources.xaml` 5–41；`Common_themeresources_any.xaml` 32、236 |
| 名字取首词/末词首字符，保留组合附加符号，去结尾括号；首三字符中 Glyph/Symbolic 优先，CJK/Greek/Arabic 等返回通用图像 | `InitialsGenerator.cpp` 73–123、126–240、242–583；25 个空格分隔段上限见 `.h` 84 |
| 辅助功能角色 Text，名称与徽章信息合并 | `PersonPictureAutomationPeer.cpp` 18–20；`PersonPicture.cpp` 293–365 |

Gallery `WinUIGallery/Samples/PersonPicture/PersonPicturePage.xaml` 14–36 演示 ProfileImage、Jane Doe 和显式 SB。

## 依赖、实现与验证门槛

`ui/tokens.slint`、`ui/primitives.slint`、`ui/fluent-system-icon.slint` 已实现。圆形裁剪和 cover 由 Slint 图像/裁剪能力实现，需用非方形自有图像在 Gallery 最小例中验证角落透明、中心裁剪和缩放。纯 Rust 首字母模块放在 `crates/fluent-core`，通过 `PersonPictureLogic` 注册，供其他头像消费者复用。与 NumberBox 无循环依赖。

公开 size 保证常规正圆；外部强制非方形布局时内部取短边，不改写调用者的布局约束。原生代码会写回 Width/Height，此差异属于宿主布局接口。Contact/PreferSmallImage 涉及 Windows 联系人服务，本轮只接受调用方提供的 DisplayName / Initials / ProfilePicture / BadgeImageSource，不接入通讯录或网络。

不捆绑 Segoe 专有字体。默认 person/people 使用项目已采用的微软 Fluent System Icons 同版本 MIT 资产，作为 E77B/E716 的合法向量替代；原版字形轮廓对照为 `TODO(spec)`，不得宣称相同轮廓。首字母按官方已列出的 Unicode 区段处理；补充平面安全回落通用图像，Rust/Windows 的区域大小写边界需进一步验证。

验证覆盖四种呈现优先级及来回切换、显式首字母、单词/多词/括号/组合音标/非拉丁、徽章数值/图像/字符优先级及 99+、半径/字号/徽章位置、矩形图像 cover 和圆角像素、无交互焦点、亮暗/高对比、不同尺寸和两种显示缩放、动态值及空图像回落。静态控件不增加动画或常驻计时器；实际触摸、屏幕阅读器及 Windows 截图对照未测试时如实记录。
