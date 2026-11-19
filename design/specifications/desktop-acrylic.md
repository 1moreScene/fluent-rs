# 窗口 Desktop Acrylic

分析日期 2026-11-18。原 Windows 分析使用 WinUI 源码 `references/microsoft-ui-xaml` 的 `main` 分支和 `../WinUI-Gallery` 的 detached HEAD。macOS 分析时本地未找到这两份仓库，复核了两个相同提交的公开源码。 [WinUI 源码 354bf60](https://github.com/microsoft/microsoft-ui-xaml/blob/354bf6017e6443023214d34309dfdf0fe2d526b1/controls/dev/Materials/Acrylic/AcrylicBrush_themeresources.xaml) · [WinUI Gallery 7360ac3](https://github.com/microsoft/WinUI-Gallery/blob/7360ac322414e118eadfb0c7c2f4ecc31a218f59/WinUIGallery/Samples/SystemBackdrops/SystemBackdropsPage.xaml.cs)

- `WinUIGallery/Samples/SystemBackdrops/SystemBackdropsPage.xaml.cs` 37–44 行展示 Desktop Acrylic 与 Thin 和纯色之间的切换。相应的 `Samples/SampleCode/SystemBackdrops/SystemBackdropsSampleDesktopAcrylicController.txt` 9–34 行先检查 Windows App SDK 支持情况，再配置窗口。
- 本项目用 Win32 DWM 的 `DWMSBT_TRANSIENTWINDOW` 请求 Windows 11 系统提供的窗口级 Acrylic。该属性从 Build 22621 起提供。Gallery 的 `Foundations → Desktop Acrylic` 在 Windows 构建号、DWM 合成和应用主题允许时启用它。DWM 接受后，Skia 使用透明客户区。系统决定具体材质合成，退出该模式时恢复原来的窗口主题。 [DWM 背景枚举](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/ne-dwmapi-dwm_systembackdrop_type) · [DWM 客户区扩展](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/nf-dwmapi-dwmextendframeintoclientarea)
- macOS 使用 AppKit `NSVisualEffectView` 的 `popover` 材质和 `behindWindow` 混合，在 Slint 内容后合成窗口后方的内容。亮暗外观跟随 Gallery 主题，窗口活动状态交给 AppKit。Slint 只在 AppKit 材质启用时叠加主题调色。 [AppKit 材质](https://developer.apple.com/documentation/appkit/nsvisualeffectview/material-swift.enum) · [微软 Acrylic 设计规范](https://learn.microsoft.com/en-us/windows/apps/design/style/acrylic)
- 旧版 Windows、窗口合成不可用、macOS 减少透明度、移动平台以及 Gallery 的高对比主题使用不透明回退。普通主题回退使用 `controls/dev/Materials/Acrylic/AcrylicBrush_themeresources.xaml` 43、95 行的 `AcrylicBackgroundFillColorDefaultBrush` 颜色，亮色 `#F9F9F9`，暗色 `#2C2C2C`。高对比跟随宿主背景色。

## 验证

Windows 11 Build 26200、Skia OpenGL。用同一测试进程的两扇窗口验证桌面合成。将底层测试窗口从洋红切换到纯绿，上层 Gallery 的无内容客户区采样点分别由 RGB (254, 143, 219) 变为 (125, 254, 85)。Python 只读取通过窗口所有权检查的三个像素，两个测试窗口随事件循环退出。

Gallery 冒烟检查覆盖 DWM 启用、暗色切换、高对比退出、恢复 Acrylic 和切回纯色。Windows 10 1809、其他 GPU 或渲染器、系统省电及辅助技术策略未测试。官方截图对照需要原版 Gallery 在相同窗口叠放及主题下的截图。

Windows 调试版在 100%、125%、150% 和 200% 缩放通过冒烟检查。合成采样在内存中完成，临时测试入口已移除，保留当前 Windows 调试版和常规冒烟检查。

macOS 26.4.1、Apple M3、Skia，使用同一进程的两扇窗口采样。后方测试窗口从洋红变为纯绿时，前方窗口空白客户区同一采样区域的 RGB 中位数从 (227, 215, 233) 变为 (208, 230, 215)。后方保持绿色时，暗色主题为 (36, 97, 49)，高对比为 (0, 0, 0)。采样前检查了两个窗口的所有权和普通窗口层的叠放顺序。

最终调试版的完整冒烟检查在 arm64 原生和 x86_64 Rosetta 下各通过两次，分别使用 1× 和 2× 缩放。覆盖亮暗 Acrylic、高对比与纯色切换。临时合成采样入口、截图及转换文件已清理。

macOS 15.0、Intel 实机、系统减少透明度选项切换、合成功耗和与官方 Gallery 的同场景像素对照未测试。

## macOS 色调与模糊校准

2026-11-18 重新读取本地两份官方仓库。WinUI Gallery `main` 提交 `6de91feccadfbb0a796b71caedead5a2284f730e` 中，`WinUIGallery/SampleSupport/SamplePages/SampleSystemBackdropsWindow.xaml.cs` 153–170 行由系统控制器提供 `Base` 和 `Thin` 两种变体，无自定义调色。WinUI 源码 `main` 提交 `07e0354d2e06572501ef48078a3cc91138749c9f` 的 `controls/dev/Materials/Acrylic/AcrylicBrush_themeresources.xaml` 43、95 行提供同系列 AcrylicBrush 的暗色和亮色调色参考 `#2C2C2C` 与 `#FCFCFC`。Gallery 的 `DesktopAcrylicController` 使用自己的默认参数。微软公开亮色示例图中，无控件的底部带（x=18–659，y=427、438、449）像素 RGB 中位数为 (215, 219, 235)。图中缺少原背景配对帧，因此该读数用于比较明亮、平滑的视觉趋势。 [Gallery 样例](https://github.com/microsoft/WinUI-Gallery/blob/6de91feccadfbb0a796b71caedead5a2284f730e/WinUIGallery/SampleSupport/SamplePages/SampleSystemBackdropsWindow.xaml.cs#L151) · [WinUI 同系列主题资源](https://github.com/microsoft/microsoft-ui-xaml/blob/07e0354d2e06572501ef48078a3cc91138749c9f/controls/dev/Materials/Acrylic/AcrylicBrush_themeresources.xaml) · [微软示例图](https://learn.microsoft.com/en-us/windows/apps/develop/ui/images/materials-desktop-acrylic-backdrop.png)

亮色使用 `#FCFCFC`、20/255 的不透明度，保留原生滤镜对背景的响应并略微提亮。暗色使用 `#2C2C2C`、115/255 的不透明度，用实测黑白背景校准纯白侧亮度，接近原来暗色的明度。两项不透明度为本项目的 macOS 调校值。用 AppKit 公开预设改变实际边缘过渡，模糊半径仍由 AppKit 决定。

同一台 M3、macOS 26.4.1、2× 缩放下，对进程自有前后测试窗口的横向采样带转为 sRGB。黑白背景交界的 10%–90% 宽度按相对色阶计算，平台合成与调色均包含在结果中。

| 主题 | 黑侧 / 白侧 RGB 中位数，旧 → 新 | 黑白过渡宽度，旧 → 新 |
| --- | --- | --- |
| 亮色 | (208, 213, 215) / (227, 231, 233) → (167, 172, 174) / (235, 240, 241) | 45.0 → 65.5 点 |
| 暗色 | (38, 47, 49) / (92, 99, 102) → (37, 41, 43) / (98, 101, 103) | 70.5 → 71.0 点 |

亮色的洋红与绿色背景采样分别从 (227, 215, 233) 与 (208, 230, 215) 变为 (236, 176, 241) 与 (165, 237, 174)，背景色差保留更多。降低原生材质视图的不透明度会露出黑白交界的硬边，`Sheet` 预设在本次测试中呈纯白。macOS 15.0、Intel 实机和官方 Gallery 同背景同缩放的测量未测试。

校准后的两版调试构建在 1×、2× 各通过一次完整冒烟检查，x86_64 使用 Rosetta。已清理一次性对照脚本、截图和测量程序产物。

## SDK 应用内 Acrylic 的内存与打开耗时

2026-11-19 在 Apple M3、macOS 26.4.1（25E253）及 arm64 Debug 下测量，窗口为 980×700 逻辑像素、1960×1400 背板像素。`fluent-materials` 接收宿主窗口与材料参数。带模糊边距的区域沿用原始图像行跨度，由 Skia 持有快照所有权。最终合成直接写进 Slint 共用像素缓冲。模糊半径、遮罩、光度、色调、噪声、动画和分辨率使用原参数。

```sh
target/debug/fluent-gallery --acrylic-memory-check
target/debug/fluent-gallery --renderer-memory-check
```

两版 Acrylic 检查都经由原生窗口的指针与键盘事件打开、关闭 NavigationView 抽屉 36 次。抽屉打开耗时取后 32 次指针分发的同步耗时。内存使用 macOS `phys_footprint_peak`，以十进制 MB 记录。该指标包括进程所拥有的图形分配。帧时间、长时间运行的极端峰值及其他平台未在本轮验证。

| 指标 | 改动前 | 改动后 |
| --- | ---: | ---: |
| 进程内存峰值 | 281.6 MB | 259.5 MB |
| 抽屉打开耗时 P50 | 237.2 ms | 189.6 ms |
| 抽屉打开耗时 P95 | 273.2 ms | 215.4 ms |

原始记录保存在 `.artifacts/memory/acrylic-before.log`、`.artifacts/memory/acrylic-after-final.log` 和 `.artifacts/memory/renderer-floor.log`。

同后端、同分辨率的诊断窗口只绘制一行文字，记录到 152.6 MB 峰值和 57.1 MB 采样占用。采样里有 22.4 MB IOSurface，另有可回收的图形页。单独优化 SDK 控件和 Acrylic 合成后，50 MB 峰值目标仍待解决。
