# SelectorBar

2026-11-19 核对本地 WinUI `main`（`07e0354d2e06572501ef48078a3cc91138749c9f`）及 WinUI Gallery `main`（`6de91feccadfbb0a796b71caedead5a2284f730e`）的常规模板与示例。

| 规格 | 官方依据 |
| --- | --- |
| 横排、Tab 一次进入，空间不足横向滚动。首次聚焦且没有可聚焦选择时选中当前或首个可用项 | `controls/dev/SelectorBar/SelectorBar.xaml:9–38`，`SelectorBar.cpp:56–65,98–127,153–172` |
| 左右移动切换单选；文字与图标可独立显示 | `InteractionTests/SelectorBarTests.cs:47–66`，`SelectorBarItem.cpp:58–94` |
| Bar 上下留 4，条目内边距 12/10/12/7，图文间距 8，图标缩放 0.8 及横向负边距 2，底部指示条高 3，宽 4 并在选中时横向放大四倍 | `SelectorBar_themeresources.xaml:26–34,96–104`，`SelectorBar.xaml:97–148,174–213` |
| 所有状态底色透明，普通/悬停/按下/禁用分别用主题文字色，选中使用强调色指示条。提示条动画 167 ms | `SelectorBar_themeresources.xaml:8–25,36–83`，`controls/dev/ComboBox/ComboBox_themeresources.xaml:330` |
| 图标标签、切页与数据切换示例 | `WinUIGallery/Samples/SelectorBar/SelectorBarPage.xaml:22–73` |

前置已实现的 `SelectionModel`、`SelectionLogic`、`FluentFocusScope`、`FluentScrollView`、`FluentSystemIcon` 与 Motion 由 ListBox、NavigationView 和既有焦点/滚动演示覆盖。SelectorBar 提供横向单选条目与选中变更供页面或应用模型消费，仅适用于少量导航选项。Gallery 验证单 Tab、键盘与鼠标选中、禁用、图标、窄窗选中项滚入视野及主题。按条目索引报告选中项，宿主可通过 `set-selected-index` 更新选择，传入 -1 清除。选中项变化后需要容器自动滚入可见区。实体触控、辅助技术及 Windows 参考像素留待相应设备验收。

## 本机验证

macOS 26.4.1 / Apple Silicon / Skia Debug 上运行 `target/debug/fluent-gallery --smoke-test selector-pager`。Retina 2× 和 `SLINT_SCALE_FACTOR=1.25` 下均验证首次 Tab 选中、左右键和禁用项、鼠标选择、亮暗及高对比指示条、动画后像素稳定、320 逻辑像素窄窗的首尾滚动。当前 Skia 后端未提供帧通知，未测空闲帧数及内存基线差异。实体触控、VoiceOver、其他目标平台和同版本 WinUI Gallery 截图对照未测试。

同台 Mac 的 DropDownButton / ToggleSplitButton、GridView、RepeatButton / HyperlinkButton 和 Expander / SplitButton 独立回归通过。Gallery 全量冒烟一度停在 NumberBox 的 Compact 聚焦断言，重新构建当前工作区后在 2× 通过。最终产物在两种缩放下均通过本批专项。
工作区格式、编译、Clippy 和 28 项工作区单元测试通过。
