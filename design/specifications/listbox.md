# ListBox

2026-11-18 在本地分析 WinUI `main` 的 `07e0354d2e06572501ef48078a3cc91138749c9f` 和 WinUI Gallery `main` 的 `6de91feccadfbb0a796b71caedead5a2284f730e`。运行 `python3 -B tools/reference_analysis/analyze_selectors.py` 重现。Gallery 的示例 XAML 没有直接使用 ListBox，行为以 WinUI 的 ListBox 实现为准。Gallery 的 `WinUIGallery/Samples/ListView/ListViewPage.xaml` 128、515 行用于参考选择模式与滚动的演示方式。

- `dxaml/xcp/dxaml/lib/ListBox_Partial.cpp` 274–345 行处理 Enter、Space、Home、End、PageUp、PageDown 和多选模式的 Ctrl+A。366–399、467–508、684–788 行定义单选随焦点、多选切换、扩展选择的锚点与范围。 `dxaml/xcp/core/core/elements/DefaultValues.cpp` 54–57 行默认启用单选随焦点。
- `controls/dev/CommonStyles/ListBox_themeresources.xaml` 5、33、61 行的边框粗细分别为 0、2、0。198–228 行启用纵向滚动与焦点带入视口，采用 VirtualizingStackPanel。自研列表用 Slint 的虚拟重复器、矩形主题表面和焦点环。
- 已验证的上游为 P3 的 `SelectionModel`、`SelectableList`、`ItemPresenter`、Slint `FocusScope` 和 `Flickable`，以及共享主题令牌。ListBox 通过只读索引和 `set-selected-index()` 提供选择状态，内部用共享视图呈现选项。替换模型或改变项数会清空选择，原位重排序由宿主清空或重设。ComboBox 后续复用同一模型与视图。
- 准入与验证场景包括三种选择模式、指针和按键、禁用与空模型、主题像素、窄窗和 100,000 行的首尾定位与实际读取计数。真实触控、读屏和官方画面对照为平台待验证。滚动条外观与原始像素对照标为 TODO(spec)。

## 视觉复核

按 ListBoxItem 自身模板的 89、118–189 行实施。正常行透明，悬停和按下用 SubtleFillColorSecondary/Tertiary，选中用 SystemControlHighlightListAccentLowBrush，选中悬停和按下分别用 Medium/High。原模板使用零时刻离散切换。 `dxaml/xcp/dxaml/themes/generic.xaml` 222、300–302、4147、4225–4227 行给出暗亮容器 `#2B2B2B` / `#F2F2F2`，蓝色强调叠层暗色用 60%/80%/90%、亮色用 40%/60%/70%。系统强调色在本实现可由宿主提供。

TODO(spec) 固定高度虚拟化列表以单行文本呈现，待用官方 ListBox 同字体、同 DPI 对照校准行高、边缘和滚动条。
