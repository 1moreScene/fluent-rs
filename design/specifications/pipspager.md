# PipsPager

2026-11-19 核对本地 WinUI `main`（`07e0354d2e06572501ef48078a3cc91138749c9f`）及 WinUI Gallery `main`（`6de91feccadfbb0a796b71caedead5a2284f730e`）的常规模板与示例。

| 规格 | 官方依据 |
| --- | --- |
| 默认无限页、从 0 开始、显示 5 枚圆点、水平、前后按钮折叠。支持有限页、双方向、可见/悬停可见/折叠与循环 | `controls/dev/PipsPager/PipsPager.idl:6–19,42–59,83–87` |
| 横向圆点命中格 12×24，纵向 24×12，图标字号普通 4/选中和悬停 6。导航格 24×24，箭头字号 8，按下缩到 87.5% | `PipsPager_themeresources.xaml:92–109,198–274,280–289` |
| 高对比选中、悬停和按下使用 Highlight 背景与 HighlightText 前景。普通主题使用透明格及独立的 Strong/Secondary/Disabled 前景 | `PipsPager_themeresources.xaml:4–89` |
| 两端隐藏但保留空间，折叠取消占位；方向键移动焦点，进入圆点区聚焦选中项，按键激活后才换页。选中项尽量居中 | `PipsPager.xaml:15–95`，`PipsPager.cpp:161–239,240–289,418–445,522–564,590–610` |
| 与翻页内容双向联动及配置方向、按钮可见性和循环 | `WinUIGallery/Samples/PipsPager/PipsPagerPage.xaml:10–79` |

依赖已实现的 `FluentPressable`、`FluentFocusScope`、焦点环、Tooltip、图标与主题令牌。用有限可见区间呈现圆点，页码模型只保存整数，适用于大量页面和无限分页。Gallery 验证方向键预览焦点、激活、翻页、两端、循环、空集、禁用、尺寸变化及大页数。宿主可双向绑定 `page-index`，经边界校正的选择从 `selected-index` 读取，空集返回 -1。页数和前后按钮配置由宿主提供。平台待验证为实体触控、辅助技术和 Windows 参考像素，圆点与箭头用跨平台图形实现，待官方截图核对字形边缘。

## 本机验证

同批专项在 Retina 2× 和 1.25× 下通过。覆盖前后箭头、悬停占位、水平及垂直键盘焦点、Space / Enter 激活、循环、空集、禁用、外部赋值、100,000 页末端和无限模式。默认可见区最多五枚圆点，320 逻辑像素窄窗按 44 逻辑像素触控目标缩减可见数量。环境、复验命令和未测试项见 [SelectorBar 本机验证](selectorbar.md#本机验证)。
