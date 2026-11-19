# NavigationView、Frame 与 Page

## 官方源码

2026-11-18 核对本机的两个微软仓库。

| 仓库 | 分支 | Commit |
| --- | --- | --- |
| Microsoft UI XAML | main | `07e0354d2e06572501ef48078a3cc91138749c9f` |
| WinUI Gallery | main | `6de91feccadfbb0a796b71caedead5a2284f730e` |

| 规格 | 证据 |
| --- | --- |
| Auto 在 641 逻辑像素以下进入 Minimal，641 到 1007 进入 Compact，1008 起进入 Expanded。侧栏默认宽度是 48 和 320。Left、LeftCompact 和 LeftMinimal 可覆盖 Auto。 | `controls/dev/NavigationView/NavigationView.idl:174–200`、`NavigationView.cpp:1549–1627` |
| Minimal 使用 Overlay，Compact 使用 CompactOverlay，Expanded 使用 CompactInline。浮层里调用叶子项后收起侧栏。响应式切回 Expanded 时保留显式收起状态。 | `NavigationView.cpp:1598–1627,2905–2918,5101–5107` |
| 侧栏按钮为 40×36，条目至少高 36，外边距为 4×2。图标为 16，图标列宽 40。指示条为 3×16，圆角为 2。左侧模式的内容表面左上圆角为 8，Minimal 为直角。方向键默认只移动焦点。 | `NavigationView_themeresources.xaml:205–239,263–265,599–614`、`NavigationView.idl:271–272` |
| 选中项用 SubtleFillColorSecondaryBrush，悬停时切换到 Tertiary，指示条用 AccentFillColorDefaultBrush。内嵌侧栏为透明，弹层使用应用内 Acrylic。页面表面使用 LayerFillColorDefaultBrush，弹层外遮罩使用 SystemAltMediumColor。 | `NavigationView_themeresources.xaml:4–49,73–115,139–181`、`controls/dev/CommonStyles/Common_themeresources_any.xaml:60,264`、`dxaml/xcp/dxaml/themes/generic.xaml:204,313,4129` |
| 内嵌侧栏开关为 200ms，曲线为 (0, .35, .15, 1)。浮层打开为 350ms，收起为 120ms，曲线为 (.1, .9, .2, 1)。Compact 从保留的图标栏边缘露出，Minimal 从左侧滑入。 | `controls/dev/SplitView/SplitView_themeresources.xaml:52–197,223–255,355–404`、`NavigationView.xaml:94–119` |
| 左侧选中指示条沿 Y 轴拉伸 200ms，随后收缩 400ms。从离开项发出的指示条在收缩时淡出。 | `NavigationView.cpp:2120–2123,2190–2227,2298–2356` |
| 左侧导航推荐 EntranceNavigationTransitionInfo。前进先用 150ms 淡出旧页，再用 300ms 让新页从下方 140 逻辑像素滑入。后退先让旧页向下滑出，再淡入新页。 | `NavigationView.cpp:3598–3623`、`dxaml/phone/lib/ThemeTransitions.cpp:3180–3222` |
| Gallery 把选中事件连接到 Frame.Navigate。Frame 管理导航和历史，Page 定义内容及导航生命周期。 | `WinUIGallery/Samples/NavigationView/NavigationViewPage.xaml:82–97,546–563`、`NavigationViewPage.xaml.cs:67–105`、`dxaml/xcp/dxaml/lib/Frame_Partial.cpp:229–373`、`dxaml/xcp/tools/XCPTypesAutoGen/Modules/Controls/Page.cs:11–36` |

## 依赖和实现边界

- 复用 `ui/selection-model.slint` 的选择和方向查找、`ui/interaction.slint` 的键鼠和焦点、`ui/{primitives,tokens,material}.slint` 的表面和 Acrylic、`ui/scroll-view.slint` 的滚动、`ui/overlay-host.slint` 的外部点按取消和焦点恢复，以及 `ui/page-host.slint` 的可用内容区域。Gallery 中通过已有控件的回归验证上游能力。
- 先提供共享的侧栏、指示条和页面过渡，再组合 NavigationView、Frame 和 Page。NavigationView 发出选中和调用事件，Frame 保存当前路由与前进后退历史，Page 承载预先声明的 Slint 内容。Gallery 在历史回退和前进时同步侧栏选中项。
- Fluent 图标使用微软 Fluent System Icons 1.1.346 的 9 个 20px Regular SVG，由主题驱动颜色。位置和渲染尺寸来自 NavigationView 模板。 [图标来源和 MIT 授权](../../ui/icons/fluent/README.md)
- Frame 使用声明式 Page 和内存历史。Gallery 将历史限制为 64 条，Frame 的默认值仍保留完整历史。动态页面类型、导航参数、缓存策略和历史序列化超出该示例。先前仅覆盖平面左侧导航，以下记录本轮补齐的模式、条目和动效。
- TODO(spec) 本地 Gallery 的控件图是 72×72 资源图。源码库另有 766×825 的层级设计图。当前没有匹配缩放和窗口尺寸的 Windows Gallery 截图。后续需在 Windows 提供对照截图，用于量化字形、阴影、Acrylic 和过渡画面的差异。

## 首轮本机验证

2026-11-19，Apple M3，macOS 26.4.1（25E253），arm64 Debug 与 Skia。格式检查、工作区检查、严格 Clippy 和 9 个单元测试通过。

```sh
cargo fmt --all -- --check
cargo check --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo build --locked -p fluent-gallery --bin fluent-gallery
target/debug/fluent-gallery --smoke-test
SLINT_SCALE_FACTOR=1.25 target/debug/fluent-gallery --smoke-test
```

- 原生 2× Retina 与 1.25× 缩放均通过。默认 125% UI 缩放下测得侧栏图标区 60、条目步长 50、浮层宽 400 逻辑像素。测试以 Slint 分发键鼠事件，并采样 Skia 像素缓冲。
- 验证 640/641 和 1007/1008 的模式切换、320 逻辑像素宽窗适配、展开与收起、紧凑浮层保持内容位置、浮层选中后关闭、Esc 与外部点击及焦点恢复。
- 验证选中项和 Frame 页面同步，像素采样确认 Home 与 Library 实际替换，方向键跳过禁用项。前进、后退和改走新路线时截断旧的前进历史均通过。
- 验证侧栏、指示条和 Frame 的过渡中间状态及结束状态。检查亮暗主题选中条和 Fluent SVG、高对比窄窗以及减少动效偏好。
- 两份本地参考仓库的工作树干净。像素检查在内存执行，临时输出目录没有新增文件。其他系统、实体触控、VoiceOver 和 Windows 画面对照未测试。


## 2026-11-19 第二轮源码对照

本轮仍使用上表的两个本地 Commit。

| 部分 | 官方证据及已接入的行为 |
| --- | --- |
| 显示方式 | `NavigationView.idl:24–31` 定义 Auto、Left、Top、LeftCompact、LeftMinimal。Gallery 加入可切换的模式选择器及 Pane 显隐。 |
| 顶部模板 | `NavigationView.xaml:212–286`、`NavigationView_themeresources.xaml:50–62,209–266,348–402,723–906` 定义透明 48 高顶部栏、常规/仅文本/仅图标条目、40×40 更多按钮和 16×3 底部选中指示条。顶部内容边界为直角。尾部条目位于右侧，Settings 使用图标。 |
| 溢出与选中 | `NavigationView.cpp:3916–3980,3991–4072,4166–4318` 以可用宽度和条目实际宽度分配主栏与溢出列表，选择溢出项后将其移至主栏，恢复全显示时预留 5 逻辑像素。菜单从 More 下方靠右展开，使用 `FlyoutPresenter_themeresources.xaml:5–34` 的 Acrylic 和圆角。 |
| 页切换 | `NavigationView.cpp:3598–3653` 和 `dxaml/phone/lib/ThemeTransitions.cpp:1543–1595`。顶部按左右方向滑动，溢出项从右进入。出页 150ms 位移 150，入页 300ms 起始位移 200。回退沿记录的反向位移。选中指示条沿 X 轴采用同一 200/400ms 拉伸收缩阶段。 |
| 层级与辅助条目 | `NavigationViewItem.cpp:784,1042–1084`、`NavigationViewItemBase.h:66`、`NavigationView.xaml:449–487,509–600`。扩展左栏按 31 逻辑像素每层缩进，使用原位展开。闭合图标栏和顶部条目使用共享弹层。支持分组标题、分隔线、可调用但不选中的父项、数值徽标与右侧或底部尾部入口。 |
| 展开图标 | `AnimatedChevronUpDownSmallVisualSource.cpp:104,789–796` 的两个状态段各约 483ms。当前用授权的 Fluent SVG 拟合该段的翻转，Windows AnimatedIcon 的内部路径变形未移植。 |
| 分组标题 | `NavigationView.xaml:515–545` 规定紧凑/展开切换的 100ms 延迟与 100ms 淡入、收起时 100ms 淡出。 |
| 浮层过渡 | `FlyoutBase_partial.cpp:2042–2079` 提供随方向的 50 像素位移，`LayoutTransition_partial.cpp:1124–1166` 连接系统的弹层动画，`ThemeGenerator.cpp:694–746` 从 Windows 主题读取具体轨道。Mac 复用 Fluent 的入场和淡出时长，系统主题轨道需在 Windows 抽取对照。 |
| Acrylic | `controls/dev/Materials/Acrylic/AcrylicBrush_themeresources.xaml:44,96,148`、`AcrylicBrush.h:64–65`、`AcrylicBrush.cpp:500–545`。沿用源码的明暗降级颜色、30 逻辑像素高斯模糊和 2% 微噪点。Skia 目前用普通透明色合成近似 Windows 的 Luminosity/Color 混合，噪点为本项目生成的 64×64 平铺纹理。 |

选中指示条使用单个连续几何体表达 Windows 同图层的重叠路径。WinUI 的快速溢出回收顺序、顶栏原生系统主题轨道、AnimatedIcon 路径和光度混合仍需 Windows 逐帧对照。当前 Gallery 使用固定数据演示。官方示例中的 AutoSuggestBox 与任意自定义 Pane 内容插槽依赖独立控件与界面组合，本轮未提供同名 API。


## 第二轮本机验证

2026-11-19，Apple M3、macOS 26.4.1（25E253）、arm64 Debug。格式检查、工作区检查、严格 Clippy 和 12 个单元测试通过。二进制为 `target/debug/fluent-gallery`。

- Retina 2× 和 1.25× 两次 Skia 原生窗口回归通过。覆盖五种模式、窄窗及 641/1008 断点、顶部溢出选中回迁、前进后退方向、左栏原位展开、紧凑子菜单、尾部入口、Esc 和焦点恢复。
- 采样确认 Acrylic 与纯色降级的颜色有差别，页面水平位移及选中指示条存在中间帧并稳定到终值。包含上一轮的侧栏、控件主题和高对比回归。
- Windows 逐帧对照、实体触控和 VoiceOver 未测试。未生成 Windows 或移动端构建。

## 反复切页的内存检查

早期 Mac 测试每轮访问 Gallery 的 12 个示例，其中一次进入 NavigationView 示例并切换其目标页。该记录适用于 Gallery 外层切页场景。当前 NavigationView 页内的复现命令见文末。

`footprint` 读取 macOS 物理占用，`ps` 读取驻留内存，`leaks` 另跑一轮扫描堆对象。堆扫描启用分配跟踪，其占用与普通运行分开看。每轮各指令访问 12 页，175 轮共 2,100 次页面访问。

- 改动前，普通运行第 25 到 175 轮的物理占用仅增加 32 KiB，Frame 历史从 26 增至 176 条。单独运行系统堆扫描，结果为 0 个泄漏、0 字节。
- 最终构建的固定 Auto 场景，第 75 到 175 轮物理占用在 184.89–189.05 MiB 内波动。第 150 和 175 轮均为 186.81 MiB，历史保持 64 条。
- 接入新模式后的普通运行，从第 75 到 175 轮轮换五种模式，物理占用在 196.80–199.99 MiB 内波动，首尾差 176 KiB。驻留内存同期增加 176 KiB。Frame 的示例历史保持 64 条。
- 最终产物轮换五种模式的独立堆扫描为 0 个泄漏、0 字节。启用分配跟踪时，第 100 到 175 轮物理占用保持 463,111,368 字节，驻留占用保持 386,608 KiB。同一次进程的 `vmmap -summary` 显示性能工具数据驻留 240.6 MiB，而活动堆分配约 17.7 MiB。普通运行的占用记录独立统计。
- 样例将历史上限设为 64，裁去最早条目。Frame 仍允许业务宿主按需要保留完整历史或指定上限。测试验证前进分支被正确截断，历史中的过渡方向保留。

当前原始记录集中在 `.artifacts/navigation-memory/`，保留基线扫描、普通 Auto、普通五种模式与最终扫描。无临时截图，保留调试产物及增量编译缓存。


## 2026-11-19 前置控件与材质对照

参考仓库 SHA 与上表相同。

| 前置或能力 | 已核对依据及本轮行为 |
| --- | --- |
| InfoBadge | `controls/dev/InfoBadge/InfoBadge.cpp:29–102` 为数值、图标、字体图标、点四类状态的优先级与圆角。色板、4–16 高度、11 字号与数值 4,0,4,2 边距来自 `InfoBadge_themeresources.xaml:5–84`。独立的 `ui/info-badge.slint` 供 NavigationView 和 Foundations 复用。 |
| AnimatedIcon | `controls/dev/AnimatedIcon/AnimatedIcon.cpp:225–273,315–454` 提供标记优先顺序、QueueOne 和小于 20ms 直达规则。Chevron 从 `AnimatedChevronUpDownSmallVisualSource.cpp:350–367,420–494,589–659,784–825` 提取 18 段标记、旋转和位移轨道及 4 条曲线。用独立 `ui/animated-icon.slint` 播放，仍用官方 Fluent System Icons 的 12px Chevron 字形。WinUI Composition 的描边与字体替代在 Mac 上未对照。 |
| Acrylic | `AcrylicBrush.cpp:380–411,472–545` 先将背景压到不透明底色，30 逻辑像素高斯模糊，再依次做 Luminosity 和 Color，最后加入 2% 噪点。亮色色值 #FCFCFC、光度 0.85、色调 0，暗色 #2C2C2C、光度 0.96、色调 0.15，取自 `AcrylicBrush_themeresources.xaml:44,96,148`。共享 OverlayHost 对 NavigationView 开启 Skia 混合。当前从打开前的窗口帧取样，关闭释放纹理，主题或位置变化使旧采样失效。噪点改用同版本 WinUI 的 256×256 资源，按 `controls/dev/Lights/MaterialHelper.cpp:194–210` 顶左对齐，以物理像素平铺。持续变化的实时背景未接入。 |
| 浮层动画 | 源码 `FlyoutBase_partial.cpp:68,2041–2079` 给出随实际放置方向的 50 逻辑像素位移。时长、延迟、透明度和曲线由 `LayoutTransition_partial.cpp:1124–1179` 和 `ThemeGenerator.cpp:644–791` 从 Windows 活动主题读取。可供宿主注入的 `FluentPopupMotion.profile` 接入平移、显示和关闭透明度轨道。本机没有 Windows 主题数据，顶部 More 与子菜单默认使用终态。SplitView 抽屉保留上表可从源码确认的动画。 |
| Pane 插槽 | `NavigationView.xaml:212–286,303–389` 提供 Header、CustomContent 和 Footer 各位置。Gallery 源码 `NavigationViewPage.xaml:541–561` 注入链接和布局。使用 Slint 命名插槽暴露内嵌左栏和 Top，PopupWindow 的同类插槽被 Slint 1.19 编译器拒绝。 |

WinUI Chevron 轨道复现命令如下。生成结果为数值数据，独立播放代码不依赖 C++ 模板。

```sh
python3 -B tools/reference_analysis/extract_navigation_chevron.py references/microsoft-ui-xaml --output apps/gallery/src/chevron_tracks.rs
```

[Pane 插槽用法](../../docs/navigation-pane-slots.md) · [Windows 主题轨道 API](https://learn.microsoft.com/en-us/windows/win32/api/uxtheme/nf-uxtheme-getthemeanimationproperty)

第三轮在同一台 Mac 上验证。格式检查、工作区检查、严格 Clippy 和 15 个单元测试通过。2× Retina 与 1.25× Skia 窗口回归覆盖光度与纯色对照、采样释放、左栏插槽点击、顶部插槽占位与状态保留、无系统配置时的浮层关闭，以及注入测试轨道后的延迟关闭。测试轨道的数值仅用于验证时序接口。Windows 主题实时轨道、触控和 VoiceOver 未测试。

同一产物完成 175 轮、2,100 次切页的常规内存回归。第 100–175 轮物理占用在 202.69–204.91 MiB 内波动，首尾差 160 KiB。该场景轮换五种模式，未循环开启 Acrylic 浮层。原始数据保留在 `.artifacts/navigation-memory/after-acrylic-slots.log`。用于确认图形回退的临时 SVG 已移除，本轮没有落盘截图。

## 2026-11-19 Gallery 侧栏和页内导航

本地参考仓库均为 `main`。WinUI 源码为 `07e0354d2e06572501ef48078a3cc91138749c9f`，Gallery 为 `6de91feccadfbb0a796b71caedead5a2284f730e`。

- `WinUIGallery/MainWindow.xaml:138–217` 用 NavigationView 承载 Frame，并让分组项在展开时保留当前选择。Fluent RS Gallery 复用 NavigationView 和已有 PageHost，保留 Button 作为启动示例。900px 展开阈值和 240px 基础侧栏宽度是 Fluent RS Gallery 的布局设置。独立 NavigationView 维持原有阈值和宽度。
- `NavigationView_themeresources.xaml:514–627` 核对左栏 InfoBadge 和展开箭头的状态。紧凑布局中的徽标距右上角各 2px。Top 样式的徽标按该文件的 `InfoBadgePresenter` 模板垂直居中。Shell 的 NavigationView 条目与示例的 Mail 条目共享同一个通知计数。
- 左栏条目按源模型索引保持实例身份。展开分组只更新可见性和布局，保持分组条目原有的焦点和 AnimatedIcon 播放状态。

页内内存测试从 Gallery 的 NavigationView 示例开始，固定在同一个展开的 Left Pane。每 16ms 经由原生窗口指针事件依次选择 Library、Mail、Home，每一步验证 NavigationView 选中项与 Frame 路由一致。共执行 2,100 次，前 300 次为预热，每 300 次读取窗口帧、物理占用和驻留内存，最后核对 Frame 历史上限 64 条。

```sh
target/debug/fluent-gallery --navigation-memory-check
/usr/bin/leaks --noContent --atExit -- ./target/debug/fluent-gallery --navigation-memory-check
```

实测环境为 Apple M3、macOS 26.4.1（25E253）、arm64 Debug、Skia。普通运行的窗口为 1960×1400 背板像素（2×），预热后物理占用在 217.80–219.89 MiB 之间，末两次采样相同。独立的 `leaks` 运行使用 980×700 背板像素（1×），完成同样的 2,100 次页内切换，退出扫描报告 0 个泄漏、0 字节。两次运行的 Frame 日志均稳定在 64 条。原始日志保留在 `.artifacts/navigation-memory/internal-left-final.log` 与 `.artifacts/navigation-memory/internal-left-final-leaks.log`。

Skia 窗口回归覆盖侧栏的鼠标和键盘选中、分组展开、窄窗抽屉及 Esc 关闭。还检查了两处 InfoBadge 的像素随通知数更新，以及同一条目上的 AnimatedIcon 连续绘制折叠、中间和展开三种姿态。Windows 逐帧对照、移动端触控和 VoiceOver 未测试。

## 2026-11-19 Settings 图标

WinUI `controls/dev/NavigationView/NavigationView.cpp:1392–1419` 为 Settings 项设置 `AnimatedIcon`、`AnimatedSettingsVisualSource` 和静态齿轮回退。左侧模板 `NavigationView_themeresources.xaml:454–498,610–615` 用 16px 图标盒并传递正常、悬停和按下状态。Top 保留图标与自动化名称。Gallery 外层使用这个内建 Settings 项，出处为 `WinUIGallery/MainWindow.xaml:138–149` 和该窗口的 Settings 导航逻辑。

`controls/dev/AnimatedIcon/AnimatedVisuals/AnimatedSettingsVisualSource.cpp:184–349,365–412,478–557,710–749` 给出四条轮廓、48×48 画布、60 fps 的 120 帧时间轴及两条曲线。按下段把齿轮转到 −20°，松开段用 39 帧转到 360°。Normal 与 PointerOver 的静止姿态一致。该矢量和动画段由共享 `AnimatedSettingsIcon` 播放，`NavigationItemDetails.icon-kind` 用于选择此图标。Top 的尾部条目在路线改变时保留实例，并继续播放释放动画。
