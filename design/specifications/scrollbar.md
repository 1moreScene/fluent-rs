# 滚动条

2026-11-18 核对本地 WinUI `main` 的 `07e0354d2e06572501ef48078a3cc91138749c9f` 与 WinUI Gallery `main` 的 `6de91feccadfbb0a796b71caedead5a2284f730e`。运行 `python3 -B tools/reference_analysis/analyze_selectors.py` 提取相关资源和行为。

- `controls/dev/CommonStyles/ScrollBar_themeresources.xaml` 26–39、172–198、557–595、694–715 行给出 `ControlStrongFillColorDefaultBrush`、12 逻辑像素操作区、展开滑块最小 30 逻辑像素、静止滚动指示条 2 逻辑像素和右侧距离 2 逻辑像素。展开或收起使用 167 ms 宽度过渡，背景和箭头使用 83 ms 淡化。减少动效时立即更新。
- 模板 447–462、530–591、694–715 行和 `dxaml/xcp/dxaml/lib/ScrollBar_Partial.cpp` 1469–1528、1556–1576 行区分鼠标和触控指示。鼠标收起与展开都预留两端按钮，使用同一条纵向轨道和同一个滑块长度。收起时滑块模板宽 8、横移 2，展开时宽 12、横移归零。触控指示单独使用全高轨道及 32 逻辑像素的滑块下限。
- 模板 15–38、180–198、394–409、561–583 行给出鼠标滑块 6 逻辑像素的透明描边与箭头资源。轨道的 6 逻辑像素圆角来自同一模板 193–194、702 行的 2 倍换算。Gallery 的轨道使用共享 Acrylic 色调合成，在减少透明度时使用纯色回退。合成不采样滚动背景。页面滚动视口常设 12 逻辑像素的独立纵向滚动条位置，保持换行宽度稳定并避免遮住 TextBox 的边框和清空按钮。
- `controls/dev/ComboBox/ComboBox_themeresources.xaml` 368–369、590–593 行和 `controls/dev/CommonStyles/ListBox_themeresources.xaml` 201–202、226 行给出纵向 Auto、横向禁用。Gallery `WinUIGallery/Samples/ScrollViewer/ScrollViewerPage.xaml` 173–183 行展示独立控制横向和纵向可见性。项目现有需要显示的滚动区域为纵向。
- `controls/dev/CommonStyles/TextBox_themeresources.xaml` 187–191、337 行给出 TextBox 默认隐藏滚动条及模板绑定。项目保留默认隐藏，Gallery 多行示例显式启用纵向 Auto。单行编辑仍由原生 TextInput 跟踪光标。
- 前置依赖为 P1 的主题令牌、简化动效和指针命中，以及 P3 的 Slint Flickable、ListView 可视范围和行定位，均已有验证。`ui/scroll-bar.slint` 接受内容长度、可见长度、当前偏移，向视口请求移动。`ui/scroll-view.slint` 管理 Gallery 页面和可溢出的弹层内容。`SelectableList` 对跨越整个视口的滑块移动直接定位目标行，保留虚拟化。文本视口沿用现有光标定位和滚动偏移。
- 滚动条随内容溢出显示。鼠标可拖动滑块、点按轨道翻页或点按端点滚动一行。滚轮、滑块与键盘导航共用同一视口位置。Slint 1.19 的辅助功能枚举使用带名称的纵向范围控件表达滚动条。TODO(spec) WinUI 端点长按连发、系统级滚动条偏好、真实触控和屏幕阅读器，以及原版截图对照。

2026-11-18 根据用户的 1960×1464 截图复核 Gallery 页滚动条，使用 125% 界面缩放测量 TextBox 边缘与轨道间距，并验证悬停改变像素输出。列表和 Gallery 原生 Flickable 为滚动条区域继续接收滚轮事件。十万项 ListBox 上，指针停于滚动条时注入 12 次滚轮事件，共读取 12 行数据，既有选择保留。实体触控板手势和帧时间未测试。
