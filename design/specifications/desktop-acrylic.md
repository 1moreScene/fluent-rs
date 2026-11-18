# 窗口 Desktop Acrylic

分析日期 2026-11-18。WinUI 源码位于 `references/microsoft-ui-xaml`，分支 `main`，提交 `354bf6017e6443023214d34309dfdf0fe2d526b1`。WinUI Gallery 位于 `../WinUI-Gallery`，detached HEAD，提交 `7360ac322414e118eadfb0c7c2f4ecc31a218f59`。

- `WinUIGallery/Samples/SystemBackdrops/SystemBackdropsPage.xaml.cs` 37–44 行展示 Desktop Acrylic 与 Thin 和纯色之间的切换。相应的 `Samples/SampleCode/SystemBackdrops/SystemBackdropsSampleDesktopAcrylicController.txt` 9–34 行先检查 Windows App SDK 支持情况，再配置窗口。
- 本项目用 Win32 DWM 的 `DWMSBT_TRANSIENTWINDOW` 请求 Windows 11 系统提供的窗口级 Acrylic。该属性从 Build 22621 起提供。Gallery 的 `Foundations → Desktop Acrylic` 在 Windows 构建号、DWM 合成和应用主题允许时启用它。DWM 接受后，Skia 使用透明客户区。系统决定具体材质合成，退出该模式时恢复原来的窗口主题。 [DWM 背景枚举](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/ne-dwmapi-dwm_systembackdrop_type) · [DWM 客户区扩展](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/nf-dwmapi-dwmextendframeintoclientarea)
- 旧版 Windows、DWM 调用失败、其他平台以及 Gallery 的高对比主题使用不透明回退。普通主题回退使用 `controls/dev/Materials/Acrylic/AcrylicBrush_themeresources.xaml` 43、95 行的 `AcrylicBackgroundFillColorDefaultBrush` 颜色，亮色 `#F9F9F9`，暗色 `#2C2C2C`。高对比跟随宿主背景色。

## 验证

Windows 11 Build 26200、Skia OpenGL。用同一测试进程的两扇窗口验证桌面合成。将底层测试窗口从洋红切换到纯绿，上层 Gallery 的无内容客户区采样点分别由 RGB (254, 143, 219) 变为 (125, 254, 85)。Python 只读取通过窗口所有权检查的三个像素，两个测试窗口随事件循环退出。

Gallery 冒烟检查覆盖 DWM 启用、暗色切换、高对比退出、恢复 Acrylic 和切回纯色。Windows 10 1809、其他 GPU 或渲染器、系统省电及辅助技术策略未测试。官方截图对照需要原版 Gallery 在相同窗口叠放及主题下的截图。

Windows 调试版在 100%、125%、150% 和 200% 缩放通过冒烟检查。合成采样在内存中完成，临时测试入口已移除，保留当前 Windows 调试版和常规冒烟检查。
