# ComboBox

2026-11-18 分析本地 WinUI `main` 的 `07e0354d2e06572501ef48078a3cc91138749c9f` 和 WinUI Gallery `main` 的 `6de91feccadfbb0a796b71caedead5a2284f730e`。运行 `python3 -B tools/reference_analysis/analyze_selectors.py` 重现。

- `dxaml/xcp/dxaml/lib/ComboBox_Partial.cpp` 2775–2833、2973–2979、3046–3116 行定义打开、导航、确认、Esc 与 Tab。IDL 中 `ComboBoxSelectionChangedTrigger` 从 `Committed` 开始，`Always` 在导航时写入选择，`Committed` 只更新候选视觉。
- `controls/dev/ComboBox/ComboBox_themeresources.xaml` 321–342 行给出 64×32 的触发器最小尺寸、80 的弹出层最小宽度和触控 240 的最小宽度，以及 15 项上限与单侧 7 项阈值。411–503 行给出主要视觉状态。577–593 行描述占位文字、箭头与弹出滚动。Gallery `WinUIGallery/Samples/ComboBox/ComboBoxPage.xaml` 22–59 行展示内联项和带初始选择的数据绑定，69 行起为可编辑示例。
- 前置依赖为 P1 的按压、焦点、主题令牌，P3 的共享选择、虚拟列表、OverlayHost、安全区和恢复焦点，以及独立验收的 P4 ListBox。两种模式复用共享选择和视图。下拉候选独立于已提交的索引，确认后更新触发器文字。宿主用 `set-selected-index()` 更新选择，显式写入会关闭下拉。
- 验证场景包括占位、初始与宿主选择、选择开关、禁用、空模型、指针和键盘、弹出层滚动与裁剪、焦点恢复、亮暗主题，以及 100,000 行的实际读取计数。真实触控、读屏和官方画面对照待目标平台验证。

## 可编辑模式

- `controls/dev/ComboBox/ComboBoxHelper.cpp` 72–121 行为当前打开方向筛选两个控件的圆角。向下展开时，弹层顶角和文本框底角置零。向上展开时，弹层底角和文本框顶角置零。闭合后文本框恢复四角圆角。弹层表面、圆角裁切、阴影与 Acrylic 使用同一组角半径。

- 同一版本的 WinUI 模板 `controls/dev/ComboBox/ComboBox_themeresources.xaml` 342–344、528–556、580–591 行定义内嵌 TextBox、`11,5,38,6` 文本边距、箭头悬停和按下底色、30 像素下拉按钮底色及四周 4 像素留边。项目复用通过 P2 验证的 `ui/text-box.slint` 和 Slint TextInput，右侧为已有箭头组件。
- `dxaml/xcp/dxaml/lib/ComboBox_Partial.cpp` 2377–2397、2441–2564、3009–3099 行定义焦点离开、Enter、Tab 提交和 Esc 恢复，以及自定义值的 `TextSubmitted`。3934–3996、4075–4165 行定义输入匹配、后缀补全和 Backspace 精确匹配。`dxaml/test/native/external/controls/combobox/ComboBoxIntegrationTests.cpp` 3366–3367 行确认默认使用 `Committed`。
- 设置 `editable: true` 即可使用原生编辑器。`edit-text` 为当前编辑内容，`selected-text` 为已提交值。接受自定义文本时 `selected-index` 为 `-1`。`text-submitted(string)` 的返回值用于宿主校验，默认接受有效的非空白文本。`ui/selectable-list.slint` 将候选保持在可见区域，编辑器继续接收键盘输入。
- `ComboBox_Partial.cpp` 4913–4973 行定义可编辑弹层的垂直定位。优先从输入框底边展开，底部越界且上方能容纳时翻到上方。应用在可用高度内裁切超长列表，并沿用共享 Acrylic、剪裁动画与减少动效策略。
- Gallery `WinUIGallery/Samples/ComboBox/ComboBoxPage.xaml` 68–86 行和 `ComboBoxPage.xaml.cs` 16–32、63–100 行提供 Font size 示例、默认第三项、即时大小预览和自定义值校验。项目的示例使用同一组字号，校验失败时恢复输入框内容。TODO(spec) 原版示例的提示对话框、Windows NLS 宽度/假名/变音等价匹配、IME 候选窗口避让、弹层边缘半像素对齐、Always 模式和 Windows 同场景截图。

## 焦点、定位与下拉动效

- `ComboBox_Partial.cpp` 872–905、3537 行区分 PointerFocused 与 Focused。模板 473–505 行仅在键盘焦点等适用状态启用外框。应用的鼠标打开和点选保持指针来源，关闭后恢复焦点而不附加键盘环。Tab 和键盘打开显示各自的焦点提示。
- `ComboBox_Partial.cpp` 4767–4908、5331–5362、5491–5505 行说明桌面普通列表的定位。下拉左贴显示区，当前选中行与显示区中心对齐。无选中项时以列表中项为定位基准。视口边缘限制宽高，最多展示 15 行。虚拟列表直接定位到可见区；较长的无选中列表以首个可见的可用项作为键盘入口。候选移动期间保持打开时的几何。移动端原版 Carousel 另行验证。
- 模板 517–526 行和 `ThemeAnimations.cpp` 596–721、733–867 行给出双向剪裁。下拉围绕显示区中心扩开，初始剪裁比例 0.50，展开 250ms，面板显示区全不透明，显示区文字在 83ms 内调至半透明。收起 167ms，目标比例 0.15，最后 83ms 才淡出。Esc、外部点按和提交使用收起过渡。Tab 导航、窗口可用区域变化及减少动效即时完成关闭。

## 视觉复核

- `controls/dev/ComboBox/ComboBox_themeresources.xaml` 324–345、411–505、567–593、614–738 行定义触发器资源、按压与焦点视觉、12 像素箭头、38 像素箭头列、焦点外边框，以及选中条目的低对比填充、3×16 指示条和按压时 0.625 的缩放。
- `controls/dev/AnimatedIcon/AnimatedVisuals/AnimatedChevronDownSmallVisualSource.h` 34–43 行标出箭头按下 150ms、回弹约 300ms。相邻 C++ 文件 324–346 行提供纵向位移与关键帧。`dxaml/xcp/dxaml/lib/SplitOpenThemeAnimation_Partial.h` 16–17 行给出 83ms 显示区文字过渡和 250ms 下拉展开。双向剪裁细节见上节。箭头回弹使用 Slint 缓动近似原轨迹，减少动效时直接显示最终帧。
- `controls/dev/Materials/Acrylic/AcrylicBrush_themeresources.xaml` 44、96 行提供下拉所用的应用内 Acrylic 资源。本实现由 Skia 对窗口内背景做模糊和着色，高对比或减少透明度时使用资源对应的纯色。Mac 启动时读取 AppKit 的减少透明度设置。WinUI 的亮度混合、噪声与平台合成仍待同场景对照。
