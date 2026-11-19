# TreeView

2026-11-19 核对本地 WinUI `main`（`07e0354d2e06572501ef48078a3cc91138749c9f`）和 Gallery `main`（`6de91feccadfbb0a796b71caedead5a2284f730e`）。

| 规格 | 源码 |
| --- | --- |
| 最小行高 28、行边距 4/2、内容最小高 20、12 像素的折叠图标、每层缩进 16 | `controls/dev/TreeView/TreeView_themeresources.xaml:33–43`、`TreeViewItem.xaml:130–148`、`TreeViewItem.cpp:515–525` |
| 悬停与按下用 Subtle 填充，中性圆角，单选使用 3×16 的指示条 | `TreeView_themeresources.xaml:5–36`、`TreeViewItem.xaml:16–91,130` |
| 左箭头折叠或移到父项，右箭头展开或移到子项，多选 Space 切换勾选 | `TreeViewItem.cpp:24–64,638–698` |
| 支持多选及父子选中状态 | `TreeViewItem.xaml:101–117`、`TreeViewItem.cpp:411–441,590–598`、`WinUIGallery/Samples/TreeView/TreeViewPage.xaml:62–75` |
| 多选时单击标签触发项目，通过复选框或 Space 更新选中集合 | `TreeView.cpp:159–163,241–244`，`TreeViewItem.cpp:54–61,411–441` |

虚拟列表、焦点和滚动条已由 SelectableList 提供，单选复用 SelectionModel，复选呈现复用 CheckBoxSurface。树投影、可见行到稳定源索引的转换，以及多选的父子合并是本批新增前置。数据按先序输入，父索引引用数据源；展开、折叠保留隐藏节点的已选状态。长列表仅创建视口可见的行，树关系在投影时迭代计算。

本批覆盖普通和多选树。拖放重排、异步加载、Windows 对照截图和触控设备留待各自验证。

## 本机验证

Apple Silicon macOS、Skia Debug。原生窗口回归覆盖指针展开、方向键进入子项和回到父项、深层选择在折叠后保留、禁用项、半选与级联勾选。4,000 节点示例折叠时对应 1,000 个根行，从初次显示到 End 跳转末行共读取 36 次行数据。模型单测也检查 10,000 层树的迭代投影。
