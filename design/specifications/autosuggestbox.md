# AutoSuggestBox

分析日期 2026-11-19。本轮先实现 P7 的 AutoSuggestBox，NavigationView 由另一会话维护。仅在当前 Mac 构建与验证。

## 官方参考

本地只读仓库 `references/microsoft-ui-xaml`，分支 `main`，提交 `07e0354d2e06572501ef48078a3cc91138749c9f`。Gallery 为 `references/WinUI-Gallery`，分支 `main`，提交 `6de91feccadfbb0a796b71caedead5a2284f730e`。

运行 `python3 -B tools/reference_analysis/analyze_autosuggestbox.py` 可重新提取资源与行为。采用常规模板 `DefaultAutoSuggestBoxStyle`。`_perf2026` 变体的发布范围尚未确认。

Gallery 的 `WinUIGallery/Assets/ControlImages/AutoSuggestBox.png` 是 72×72 的目录图标。控件界面的同主题、同缩放截图仍待提供。

| 规格 | 官方路径和位置 |
| --- | --- |
| 输入区复用 TextBox，最小高度 32、字体 14、控件圆角 4，弹层使用 Acrylic 和 Flyout 描边 | `controls/dev/AutoSuggestBox/AutoSuggestBox_themeresources.xaml` 29–44、250–286 行，相关资源沿用已有 TextBox 与 Overlay 规格 |
| 搜索图标 12，搜索按钮宽 32、高 28、左侧外距 2，内部外距 1,3，右侧独立间隔 4 | 同模板 8、24–28、101、242 行 |
| 清除按钮宽 32，内部上下各留 4，位于搜索按钮左侧，两个按钮均跳过 Tab | 同模板 25、54、91、241–242 行 |
| 列表最大高度 374，容器描边 1、上下内边距 2；列表横向外距 -1，与容器内侧边界对齐 | `dxaml/xcp/dxaml/themes/generic.xaml` 28、118–120 行，模板 283–286 行 |
| 默认列表条目高 40、文本左右内距 16,12、圆角 4；底板左右内缩 4、上下 2，选择指示条宽 3、高 16、左距 4、圆角 1.5，按下时缩短 6 | `controls/dev/CommonStyles/ListViewItem_themeresources.xaml` 209–212、234–243 行，`dxaml/xcp/core/core/elements/ListViewBaseItemChrome.cpp` 52–58、79 行 |
| 输入框与列表相接的一侧使用直角，向上展开时反转直角方向 | `controls/dev/AutoSuggestBox/AutoSuggestBoxHelper.cpp` 103–128 行 |
| 无软键盘时选用输入框上下空间较大的一侧，受可用视口约束 | `dxaml/xcp/dxaml/lib/AutoSuggestBox_Partial.cpp` 2021–2092 行 |
| 文本变更区分 UserInput、SuggestionChosen、ProgrammaticChange。过滤由调用方在 UserInput 回调中执行 | 原生实现 365–370、451–500、2274–2292、2361–2397 行；Gallery `WinUIGallery/Samples/AutoSuggestBox/AutoSuggestBoxPage.xaml.cs` 123–147、161–174 行 |
| 上下键预览候选并替换文本，越过端点回到用户原文；单个候选已选中时方向键保持该项。Esc 与打开列表时的 Tab 恢复用户原文并关闭，Tab 继续导航 | 原生实现 1028–1159 行 |
| Enter 提交当前文本及已选候选；点击候选先选择再提交；搜索按钮始终仅提交文本并清空候选参数 | 原生实现 777–792、857–874、1149–1159、2410–2440 行；Gallery 177–216 行 |

## 依赖准入与范围

| 依赖 | 实现路径 | 状态与用途 |
| --- | --- | --- |
| 主题、尺寸、圆角与减少动效 | `ui/tokens.slint`、`ui/motion.slint` | 已实现，复用输入和弹层令牌 |
| 编辑、IME、选区、焦点 | `ui/text-box.slint`、`ui/interaction.slint` | 已实现，编辑保持在 Slint TextInput，组合键在 preedit 非空时交还编辑器 |
| 候选选择、虚拟化与滚动条 | `ui/selection-model.slint`、`ui/selectable-list.slint` | 已实现，复用模型及条目呈现；候选模型可动态替换 |
| 弹层定位、关闭、材质与输入阻挡 | `ui/overlay-host.slint`、`ui/overlay-placement.slint` | 已实现；补充贴合锚点的最大空间定位，以及允许内联按钮延伸至锚点区域的内容裁剪选项 |
| 页面与演示状态 | `apps/gallery/ui/gallery.slint` | 已实现，新增独立 AutoSuggestBox 页面 |

上游为 TextBox、SelectionModel、SelectableList、OverlayHost。控件通过文本、候选模型和回调与调用方交互，Rust 示例负责过滤。无需 NavigationView 的层级、动画或布局模块。依赖不形成反向引用。

最小验证包含亮暗及高对比主题、空候选、禁用、搜索图标可选、用户输入与程序改值的回调原因、候选预览、Enter/搜索按钮/点击提交、Esc/Tab、预编辑输入保护、模型替换、失活关闭、窄窗贴合和长列表可见项读取。正式交付以这些原生入口通过为条件。

`TODO(spec)` 原版同主题同缩放截图、宿主字体与抗锯齿差异、Windows 原生弹层跨窗口边界、真实 IME 候选窗口避让、触屏软键盘及屏幕阅读器均需对应环境。当前模板未声明弹层进入/退出动画，保持共享弹层的即时显示行为。过滤示例采用 Unicode 小写转换和空白分词，区域相关语言排序由调用方决定。

## 实现与本机验证

控件位于 `ui/auto-suggest-box.slint`。`text` 可双向绑定，`items` 接受共享的 `ItemData` 模型，`text-changed` 提供变更原因，`suggestion-chosen` 提供候选索引，`query-submitted` 提供文本与候选索引。仅文本提交时索引为 -1。`update-text-on-select` 默认启用，`query-icon` 默认隐藏。公开方法支持打开、关闭与全选。宿主需像现有控件一样连接共享选择逻辑，并连接 `AutoSuggestText.byte-length`，本轮在 `apps/gallery/src/lib.rs` 接入该 UTF-8 光标换算。

`text-changed` 采用 Slint 同步回调。`TODO(spec)` WinUI 在原生实现 451–468、2274–2292 行使用计时器合并文本事件，并通过事件对象提供 `CheckCurrent`。当前未提供该合并时序和事件对象。调用方执行异步过滤时需在返回结果前检查查询文本是否仍为当前文本。

`TODO(spec)` 共享 PopupWindow 展开时捕获窗口指针，点击输入框正文或背景会先收起列表。内联搜索和清除按钮已能在展开态直接操作。保持候选展开时的正文鼠标选区，以及外部点击透传，仍需补充共享弹层能力与原生回归。

`apps/gallery/ui/auto-suggest-box-page.slint` 展示输入、查询结果、禁用、搜索图标可选和候选填充选项。`apps/gallery/src/auto_suggest.rs` 只承担该示例的过滤。移动端尺寸沿用项目的 `minimum-target` 策略，在窄窗回归中验证 44 逻辑像素目标。

2026-11-19 在 macOS 26.4.1 ARM64 使用 Skia 和 debug 构建执行完整 `--smoke-test`，分别设置 `SLINT_SCALE_FACTOR=1.25` 和 `SLINT_SCALE_FACTOR=2`。两轮均通过，覆盖目标 980×700 与 320×640 逻辑窗口，物理缓冲分别为 1226×876 / 400×800 和 1960×1400 / 640×1280。1.25× 的宽窗经过物理像素取整，启动日志的逻辑尺寸四舍五入为 981×701。

新增的 `apps/gallery/src/smoke/auto_suggest.rs` 验证亮暗和高对比填充、直角接缝、贴合与视口边界、悬停和按下、拖出取消、单次清除通知、禁用、用户和程序文本变更、Unicode 文本、选择后保持原文的选项、上下键循环、单候选、跳过禁用行、Enter/搜索按钮/候选点击提交、Esc/Tab 和焦点、空结果、稍后返回的候选模型、模型替换与离开页面关闭。十万条候选场景注入 12 次滚轮事件并通过 Up 定位末项，两种缩放均读取 40 行数据。该计数验证虚拟化范围，帧时间、触控板惯性与长期内存尚未测试。

2× 缩放另验证短视口向上展开，检查弹层底边与输入框相接处的直角，再恢复窗口继续完整回归。

共享工作区新增 `crates/fluent-materials` 后，重新通过格式检查、workspace check、全目标严格 clippy 和 workspace test。共 16 项单元测试及两个 crate 的文档测试通过。验证中将材质测试里的像素数组构造改为 `SharedPixelBuffer::new` 后写入 `make_mut_slice`，修正了 Slint 1.19 的类型不匹配。

并行开发仍在修改共享条目、令牌和材质，最终验证采用 `.artifacts/autosuggest-build-20261119/` 的源码快照。`source-manifest.json` 记录文件 SHA256，保留快照供复现。冒烟模式先等待 `SLINT_SCALE_FACTOR` 请求的缩放就绪，再执行同步场景，防止把默认 1× 当作高 DPI 结果。该快照重新通过全部检查、16 项测试、文档测试，以及 2× / 1.25× 的宽窄窗口原生回归。产物已放到 `target/debug/fluent-gallery` 并启动。

可从项目根目录复现构建。

```sh
cargo build --locked --manifest-path .artifacts/autosuggest-build-20261119/Cargo.toml --target-dir target -p fluent-gallery --bin fluent-gallery
```

交付二进制 SHA256 为 `757e56f528659161d0502e93c0e0c805b0faaa9ecbc59d61695c953a642cc589`。已清理快照中的重复验证二进制，保留统一路径的产物。

本轮临时输出盘点覆盖 `.artifacts/` 和 `tests/visual/tmp/`。AutoSuggestBox 无过期临时文件，保留另一会话的 NavigationView 验证产物和必要的 Cargo 增量缓存。两份官方参考仓库的工作树保持干净。
