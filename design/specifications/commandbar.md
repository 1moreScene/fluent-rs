# CommandBar

2026-11-19 核对本地 WinUI `main`（`07e0354d2e06572501ef48078a3cc91138749c9f`）和 Gallery `main`（`6de91feccadfbb0a796b71caedead5a2284f730e`）。

| 规格 | 源码 |
| --- | --- |
| 默认紧凑态高度 48，展开态高度 64，最小态 24。平常背景透明，展开使用应用内 Acrylic | `controls/dev/CommonStyles/CommandBar_themeresources.xaml:5–10,69–99`、`dxaml/xcp/dxaml/themes/generic.xaml:23–26` |
| 主命令右对齐，More 打开次命令，移入溢出的主命令在次命令之前 | `CommandBar_themeresources.xaml:795–875`、`dxaml/xcp/dxaml/lib/CommandBar_Partial.cpp:3205–3267` |
| 显式正溢出序号按升序成组移动，其余命令从右往左移动 | `CommandBar_Partial.cpp:2850–3058` |
| 主按钮宽 68，标签字号 12，支持图标下方、右侧和不显示标签。常规溢出最小宽 160，触控 240，最大宽 480 | `controls/dev/CommonStyles/AppBarButton_themeresources.xaml:33,113–130,153–246,359–377`、`CommandBar_themeresources.xaml:5–7` |
| 官方示例展示右侧标签、可变次命令和键盘加速键 | `WinUIGallery/Samples/CommandBar/CommandBarPage.xaml:44–120` |
| 展开使用常规时长、收起使用快速时长，随可用视口向下或向上揭开 | `CommandBar_themeresources.xaml:120–253`，沿用 FluentMotion 的对应时长和缓动 |

复用 FluentPressable、OverlayHost、SelectableList 与 SDK Acrylic。动态命令分区是本批新增前置，负责在可用宽度变化时稳定映射原始命令索引。首次复用已验证的弹层开关、滚动、外部点击、Esc 和焦点恢复。

本批支持普通和开关命令，快捷键注册、粘滞打开、嵌套子菜单和独立 AppBarSeparator 留待扩展。Windows 动画逐帧对照和真实触控设备未测试。

## 本机验证

Apple Silicon macOS、Skia Debug。原生窗口回归覆盖宽度变化时命令移入与移出、窄布局保留原命令索引、键盘跳过禁用项、Preview 调用、Esc 与外部点击关闭，以及展开中间帧和关闭后焦点恢复。暗色和高对比的溢出表面通过像素采样。

## More 浮层复验

沿用上述两份本地仓库版本。按 `CommandBar_Partial.cpp:2218–2259,2450–2457,2571–2611,2639–2697` 实现测宽、半视口限高、窄视口铺满宽度、右边缘对齐及向上回退。贴合边缘随工具栏展开移动，同时向上展开时保持原工具栏底边。普通溢出呈现沿用 8px 圆角和 1px 边框，窄视口保留远离工具栏的边框。对应 `CommandBar_themeresources.xaml:863–941`、`generic.xaml:325,2101,4250`。

`AppBarButton_themeresources.xaml:113–122,197–239,369–374` 提供 12/38/76px 标签起点、16px 图标和 5/8px 普通文字上下内距。触控内距为 9/12px。勾选图形的 14px 大小来自 `AppBarToggleButton_themeresources.xaml:481–488`。主次命令间的分隔线为 1px，上下各留 4px，取自 `AppBarSeparator_themeresources.xaml:21–24,53–57`。菜单高度和滚动范围都计入该分隔带。

## More 内容布局复验

2026-11-19 继续核对上述两个本地版本。官方 Gallery 的 `WinUIGallery/Samples/CommandBar/CommandBarPage.xaml:67–77` 用普通 `AppBarButton` 展示 Settings。用户截图中的三行文字墨迹始于 x=211–212，来自 Gallery 示例的 Preview 开关触发混合列布局。Gallery 将 Preview 改为普通命令，三个菜单项的图标起点恢复为距内容左边缘 12px，文字起点为 38px。`CommandBar_Partial.cpp:1581–1663` 的混合列规则仍用于 SDK 调用方显式提供开关命令的场景。

`CommandBar_themeresources.xaml:19–24,75,932–939` 将上下 4px 留白设置在 `ScrollViewer` 内的 `ItemsPresenter` 上。SDK 的共享列表把留白计入滚动内容，首尾留白随菜单滚动，并排除在菜单项命中范围之外。

`target/debug/fluent-gallery --smoke-test command-bar` 在 macOS 2× 和 1.25× 背板缩放下通过，覆盖默认界面比例下菜单宽度和三行像素位置，以及窄窗、鼠标、键盘、滚动留白、主题、实时材质与动画。该入口与全量窗口回归共用 CommandBar 验证逻辑。全量窗口回归在进入 CommandBar 前触发了 TextBox 清除图标居中检查失败。`.artifacts/command-bar/` 保留本次两份专项结果及两份全量失败日志。

## More 渲染性能

旧实现从 More 的输入处理直接调用窗口快照，读取整窗后才创建浮层。More 和展开的工具栏现通过 Skia 对背景实时模糊，使用原色调和 2% 噪声。色调使用源叠加，保留高对比和减少透明度的纯色回退。Windows Acrylic 的 Luminosity/Color 双混合仍保留平台合成差异。

macOS 原生窗口回归在 2× 和 1.25× 背板缩放下通过，覆盖菜单在宽窄视口的贴合、向上展开、长标签测宽和滚动、分隔带高度、明暗和高对比，以及展开后改变背板颜色时的实时采样。两次日志保存在 `.artifacts/command-bar/native-final.log` 和 `.artifacts/command-bar/scale-1.25.log`。

在当前 Mac 的 1960×1400 背板下，使用 `profile-graphics` 启用 Slint 1.19 的 Metal 渲染通知。每次运行打开 16 次，预热前两次，统计点击处理耗时、点击到首个 Skia 渲染完成通知及相邻通知间隔。测量包含动画、实时材质和实际输入分发。

同机连续复测的 P95 如下。点击处理包含对打开状态的读取。

| 指标 | 改动前 | 改动后 |
| --- | ---: | ---: |
| More 同步处理 | 95.9 ms | 15.1 ms |
| 点击到首个 Skia 绘制完成 | 108.0 ms | 30.5 ms |
| 打开期间的相邻绘制间隔 | 18.6 ms | 17.2 ms |

保留 `.artifacts/command-bar/before-repeat.log` 和 `.artifacts/command-bar/after-repeat.log`。首个绘制完成通知来自 Skia，屏幕最终扫描输出时间未测量。清理了该目录内的临时基线二进制、调试日志和重复回归日志。

```sh
cargo build --locked -p fluent-gallery --bin fluent-gallery --features profile-graphics
target/debug/fluent-gallery --command-bar-profile
```
