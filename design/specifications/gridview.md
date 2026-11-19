# GridView

2026-11-19 核对 WinUI `main`（`07e0354d2e06572501ef48078a3cc91138749c9f`）和 WinUI Gallery `main`（`6de91feccadfbb0a796b71caedead5a2284f730e`）。

| 规格 | 本地参考 |
| --- | --- |
| 默认 ItemsWrapGrid 横向排布、纵向滚动；底部内距 10；焦点项滚入视口；单个 Tab 入口 | WinUI `dxaml/xcp/dxaml/themes/generic.xaml:9347–9405` |
| 行优先与列优先各沿垂直于排布的轴虚拟化。可用轴宽除以单元尺寸取整，至少一列或一行，可设置换行上限 | `dxaml/xcp/dxaml/lib/ItemsWrapGrid_Partial.cpp:51–153`、`dxaml/xcp/components/moco/lib/WrappingLayoutStrategyImpl.cpp:961–987` |
| 排布方向的方向键步进一项，达到末端停住；跨行或列以实际跨度步进，缺少同列或同行条目时停在当前项 | `WrappingLayoutStrategyImpl.cpp:486–504` |
| 单项最小 44×44、右下项间距各 4、内容居中。默认透明，使用 SubtleFill 悬停与按压，选中描边 2，选中颜色映射主题 AccentFill | `controls/dev/CommonStyles/GridViewItem_themeresources.xaml:5–45,48–88,137–157`、`generic.xaml:109–110` |
| None、Single、Multiple、Extended；项调用开关可与选择分开。Ctrl 单独移动焦点，Shift 扩展选区 | `dxaml/xcp/dxaml/lib/ListViewBase_Partial_Interaction.cpp:246–280,364–433,1240–1287` |
| 独立数据模板、换行上限、行列间距、选择与调用示例；SplitButton 第一例是 3 列、32×32 色块、可项调用的 GridView | Gallery `WinUIGallery/Samples/GridView/GridViewPage.xaml:20–54,126–163,182–214,225–299`、`WinUIGallery/Samples/SplitButton/SplitButtonPage.xaml:38–62` |

`ui/grid-view.slint` 复用 `ui/selection-model.slint` 的 SelectionModel、`ui/interaction.slint` 的焦点与指针，以及 `ui/scroll-view.slint` 和 `ui/scroll-bar.slint` 的原生 Flickable 和共享滚动条。两种排布共用同一虚拟化窗口，只读取可见项和前后各一条预留行或列。

内置模板显示 `ItemData.text`，也可通过 `item-content(index)` 提供居中的色块等有实尺寸的内容。内置模板由 GridView 创建并接线，保留 `ItemData.text` 作为无障碍名称。需要自定义子树时，可设置 `use-default-item-template: false`，在 `item-template` 插槽遍历 `realized-count`，用 `cell-at` 取得实际源索引、位置和状态。关闭内置模板会跳过其条目的实例化，避免 Slint 1.19 的具名插槽替换留下默认重复器。`GridViewItem` 提供外观、指针和无障碍语义，模板把项调用和勾选交给 `GridView`。Rust 宿主使用现有 `SelectionLogic` 回调并绑定 `GridNavigation.step` 来按行列跨度跳过禁用项，接线示例见 `apps/gallery/src/lib.rs`。

独立运行 `target/debug/fluent-gallery --smoke-test grid-view` 可验证模板、选择、键盘、滚动和虚拟化。macOS 26.4.1 arm64 的 Skia Debug 在背板缩放 2× 和 1.25× 下通过独立验证。去掉重复模板后，两种缩放下，十万项模型首次绘制均读取 32 次，完成滚轮、大距离定位、末项定位、横向翻页和触控板滚动后累计读取 221 次。读取计数包括布局与交互对同一项的重复访问。

本批覆盖规则网格、条目模板、选择、键盘、滚动和虚拟化。分组、拖放重排、SemanticZoom 与异尺寸换行留待独立支持；SplitButton 文本色板仍使用原有布局。实体触控、屏幕阅读器和 Windows 像素对照待对应环境验证。
