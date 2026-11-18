# P1 共享基础

分析日期 2026-11-18。WinUI 源码位于 `references/microsoft-ui-xaml`，分支 `main`，提交 `354bf6017e6443023214d34309dfdf0fe2d526b1`。WinUI Gallery 位于 `../WinUI-Gallery`，detached HEAD，提交 `7360ac322414e118eadfb0c7c2f4ecc31a218f59`。

| 上游能力 | 状态与依赖 | 消费方及最小验证 |
| --- | --- | --- |
| 颜色、排版、DPI | 已有 `ui/tokens.slint` 和 Skia。补齐文本比例、响应式度量及表面令牌。 | Gallery、Button、TextBox。切换主题和文本比例，检查窄窗与高 DPI。 |
| 表面、文字、图标和焦点 | Slint 的 Rectangle、Text、Path 已可用。共享外观使用令牌和控件提供的状态。 | Button、TextBox 与 Gallery Foundations 页。检查颜色、描边、清除图标及键盘焦点环。 |
| 指针和键盘 | 现有 Button 已使用 TouchArea 与 FocusScope。TextBox 编辑复用 TextInput。实际移动触控及系统 IME 仍需平台验证。 | 复用原有冒烟测试覆盖点击、离开取消、焦点、启禁和文本编辑。在移动端复验触控、软键盘和选区。 |
| Motion | 已有 Button 83 ms 背景过渡。补齐曲线和关闭策略。 | Button 和 Gallery。检查正常、减少动效和高对比模式。 |
| Rust/Slint 与材质 | Gallery Rust 入口已实现。补齐类型化设置请求与不透明材质回退；平台合成能力待 P5。 | 在 Gallery 修改主题、强调色、文字大小、动效和材质，用 Skia 像素缓冲检查回退。 |

依赖从令牌、材质和 Slint 原语指向控件，再指向 Gallery。Gallery 通过组件属性及回调桥接 Rust，控件可以独立使用。

宿主可将窗口的 `default-font-size` 绑定到 `FluentMetrics.scaled-font-size`，并通过 `FluentTheme` 配置主题、强调色、正数文本比例和减少动效。`FluentPressable` 为按钮类控件提供 `hovered`、`pressed`、`focus-visible` 和 `activated`。文本编辑等需要独立焦点的控件可复用 `FluentPointerArea` 的边界状态。`FluentMaterials.fallback-color` 接收材质语义并返回当前主题下的不透明背景色，可在窗口第一帧直接绑定。

## 官方证据

- `controls/dev/CommonStyles/TextBlock_themeresources.xaml` 3–49 行定义 Caption/Body/Body Strong/Title 等字号和字重，Gallery 的 `WinUIGallery/Samples/Typography/TypographyTypeRamp.txt` 4–12 行展示对应梯度。14 逻辑像素作为 `1rem` 基准。沿用宿主系统字体及字体回退，Windows Gallery 优先使用 Segoe UI，其他平台由 Slint 选择默认字体。字形和基线对照待官方截图。
- `controls/dev/CommonStyles/Common_themeresources_any.xaml` 的 `CardBackgroundFillColorDefault`、`CardStrokeColorDefault`、`SolidBackgroundFillColorBase`、`FocusStrokeColorOuter/Inner` 提供表面及焦点颜色。`CornerRadius_themeresources.xaml` 5–14 行定义 4 与 8 逻辑像素圆角。逻辑像素由 Slint 按设备 DPI 映射，44 逻辑像素触控目标与 Gallery 紧凑宽度属于项目策略。
- Gallery 的 `WinUIGallery/Samples/ThemeShadow/ThemeShadowPage.xaml` 11–44 行以 Border 和 Z 位移展示投影。`dxaml/xcp/components/graphics/ProjectedShadowManager.cpp` 27–38 行记录环境光与方向光参数。`TODO(spec)`：单层 Skia 阴影是可关闭的 P1 近似，投影外观和性能待对照。
- Button 的 83 ms 过渡与 TextBox 的离散外观切换见两份控件规格。共享填充过渡沿用线性曲线，`TODO(spec)`：用 WinUI 录屏验证插值及合成差异。Gallery 的 `WinUIGallery/Samples/EasingFunction/StandardEasingFunction.txt` 5–8 行展示的 CircleEase 针对位移动画。
- Gallery 的 `WinUIGallery/Samples/SystemBackdrops/SystemBackdropsMicacontroller.txt` 12–37 行在创建 Mica 前检查能力。`WinUIGallery/Samples/SystemBackdrops/SystemBackdropsPage.xaml` 28–34 行区分 Mica（壁纸取样）、Mica Alt 和 Desktop Acrylic（窗后实时模糊）。Mica 与 Mica Alt 分别用 `SolidBackgroundFillColorBase` 和 `SolidBackgroundFillColorBaseAlt` 回退；亮暗值见 `controls/dev/CommonStyles/Common_themeresources_any.xaml` 68、75、272、279 行。高对比使用宿主对比度背景色。 [微软 Mica 指南](https://learn.microsoft.com/en-us/windows/apps/design/style/mica)
- `WinUIGallery/Samples/Acrylic/DefaultAppAcrylicBrush.txt` 引用 `AcrylicInAppFillColorDefaultBrush`，其亮暗回退、着色与亮度参数见 `controls/dev/Materials/Acrylic/AcrylicBrush_themeresources.xaml` 44、96 行。Gallery 将它用在局部示例，默认显示固色，高对比强制固色。预览以 Skia 应用内模糊和半透明着色近似该资源，模糊标准差取 12 逻辑像素。噪声、排除混合与原生 Mica 壁纸采样仍待补。 [微软 Acrylic 指南](https://learn.microsoft.com/en-us/windows/apps/design/style/acrylic) · [窗口 Desktop Acrylic](desktop-acrylic.md)

官方截图待提供。Windows 11 的本轮验证结果与其他平台所需环境见 [P1 验证记录](../../docs/p1-validation.md)。
