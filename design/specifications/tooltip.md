# Tooltip

2026-11-18 核对本地 WinUI `main` 的 `07e0354d2e06572501ef48078a3cc91138749c9f` 与 WinUI Gallery `main` 的 `6de91feccadfbb0a796b71caedead5a2284f730e`。复现命令为 `python3 -B tools/reference_analysis/analyze_overlays.py`。

- `dxaml/xcp/dxaml/lib/ToolTipService_Partial.cpp` 1405–1444、1686–1776、1795–1834 行区分悬停与键盘焦点及离开行为。鼠标和键盘首次触发使用系统悬停时间的两倍，鼠标悬停默认回退值为 400ms。相邻头文件的回退显示时长为 5 秒，WinUI 允许 Windows 系统设置覆盖。`ToolTip_Partial.cpp` 129、1118–1122 行禁用命中测试，并在键盘打开时以顶部替代鼠标位置。
- `controls/dev/CommonStyles/ToolTip_themeresources.xaml` 5–38、50–52、76–77 行定位 12 逻辑像素文字、1 描边、控制圆角、内距 9/6/9/8 和最大宽度 320。Gallery `WinUIGallery/Samples/ToolTip/ToolTipPage.xaml` 24、28–51 行展示简短文本、偏移和右侧定位。
- 上游是共享 Overlay 位置计算、主题表面、FluentPressable 的悬停和键盘焦点、Slint Timer。提示渲染在宿主提供的视觉层，不参与命中测试，也不获取焦点。默认计时采用 WinUI 的回退值，供宿主覆盖。
- 检验悬停延时、键盘焦点、退出、超时、点击、Esc、窄窗与主题绘制。真实触控、读屏、Windows 系统计时读取与相邻提示的快速重显为 TODO(spec)。
