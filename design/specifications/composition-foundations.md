# P3 组合控件基础

分析日期 2026-11-18。参考仓库在项目的 `references/` 下，以只读方式分析。

| 仓库 | 分支 | 提交 |
| --- | --- | --- |
| Microsoft WinUI | `main` | [07e0354d2e06572501ef48078a3cc91138749c9f](https://github.com/microsoft/microsoft-ui-xaml/tree/07e0354d2e06572501ef48078a3cc91138749c9f) |
| Microsoft WinUI Gallery | `main` | [6de91feccadfbb0a796b71caedead5a2284f730e](https://github.com/microsoft/WinUI-Gallery/tree/6de91feccadfbb0a796b71caedead5a2284f730e) |

该 Gallery 提交的 `standalone.props` 使用 Windows App SDK 2.6.6 experimental。以下记录用于追踪源码中的行为与设计令牌。实机行为须按对应的发布版本验证。

## 官方证据

| 能力 | WinUI 源码 | WinUI Gallery |
| --- | --- | --- |
| 选择与滚动 | `dxaml/xcp/dxaml/lib/ListViewBase_Partial_Interaction.cpp` 395、468、1254、1287 行涉及多选与键盘定位。相邻的 `ListViewBase_Partial_DataVirtualization.cpp` 74 行区分数据总数和已实现的 UI 容器。 | `WinUIGallery/Samples/ListView/ListViewPage.xaml` 128、484、515 行分别展示选择、滚动位置恢复和滚动到条目。 |
| 列表视觉 | `controls/dev/CommonStyles/ListViewItem_themeresources.xaml` 14 行的 `ListViewItemMinHeight` 为 40。19–28 行给出按下、选中和指针经过时的资源键。 | 同页的选择示例提供不同选择模式。 |
| 锚定弹层 | `dxaml/xcp/dxaml/lib/Popup_Partial.cpp` 707–718 行区分外部点按关闭策略。相邻的 `FlyoutBase_partial.cpp` 62 行记录 Flyout 默认朝上，2257–2291 行依据输入来源确定焦点状态。 | `WinUIGallery/Samples/Popup/PopupPage.xaml` 19–23 行展示偏移量与外部点按关闭。相邻 `Flyout/FlyoutPage.xaml` 28 行展示按钮锚点。 |
| 模态 | `controls/dev/CommonStyles/ContentDialog_themeresources.xaml` 7、41、194、222 行记录遮罩与默认按钮状态。相邻的 `Common_themeresources_any.xaml` 59、263 行给出亮暗主题遮罩 `#4D000000`（ARGB）。 | `WinUIGallery/Samples/ContentDialog/ContentDialogPage.xaml` 展示对话框的触发和结果。 |
| 响应式页面 | `controls/dev/NavigationView/NavigationView.idl` 178–181 行定义紧凑和宽屏阈值。 | `WinUIGallery/Samples/NavigationView/NavigationViewPage.xaml` 76、123、586–590 行展示自动、顶部和响应式切换。 |

## 依赖与接口

| 模块 | 已实现的上游 | 下游 |
| --- | --- | --- |
| `ui/selection-model.slint` | Rust `selection.rs` 与 Slint `ModelRc`。选择用当前模型的行索引表示。切换模型或模式时清空选择。原位重排行时由宿主清空选择。 | 可选列表，后续可复用到选择类控件。 |
| `ui/selectable-list.slint` | `SelectionModel`、`ItemPresenter`、Slint `FocusScope` 和虚拟 `ListView`。列表行使用共享主题、指针和焦点环。 | Gallery 的 Lists 页，后续可为 ListBox 和 ComboBox 提供列表行为。 |
| `ui/overlay-host.slint` | Slint `PopupWindow`、窗口安全区、`FluentElevatedSurface` 和主题遮罩。锚定模式优先在触发器下方，空间不足时比较上下空间并夹在安全区内。模态模式由完整视口内的遮罩阻止背景输入。 | Lists 页的详情及清除确认，后续供弹层和对话框消费。 |
| `ui/page-host.slint` | 既有 Gallery 页面状态、共享紧凑宽度令牌和安全区。 | Gallery 页面容器，Lists 页随宽度调整操作区布局。 |

列表默认行高至少 40 个逻辑像素，有触控最小目标时至少 44 个逻辑像素。单选时方向键移动并更新选择，Ctrl 加方向键只移动焦点。多选时方向键移动焦点，Space、Enter 或点按切换单行。列表使用自己的整行选中色，包括高对比主题。操作区显示当前选中数，并保留离开页面前的选择。离开页面或窗口可用区域变化时关闭弹层。

Lists 页使用惰性 Rust 模型生成十万行，通过读取次数和末行定位检验可见范围实例化。冒烟验证同时覆盖列表焦点、外部点按、Esc、模态背景阻止、焦点恢复和宽窄窗口。正式组合控件的默认动作、输入路由和外观要按各自规格验证。

TODO(spec) 需要相同主题与缩放的 Windows Gallery 参考截图。真实触控、读屏、窗口安全区变化、系统后台与移动端焦点行为需要对应设备。多选范围扩展、不同弹层的动画以及 NavigationView 控件分别在各自用例验收。
