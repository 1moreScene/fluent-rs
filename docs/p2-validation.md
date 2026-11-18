# P2 ToggleButton 与 CheckBox 验证

日期 2026-11-18。Windows 11 Pro Build 26200 x64，Core i5-1235U、Intel Iris Xe（驱动 32.0.101.7084），1920×1080，Rust 1.100.0、Slint 1.19.0、Skia OpenGL。

## 复现

```powershell
python -B tools/reference_analysis/analyze_toggles.py --gallery-source ../WinUI-Gallery
cargo fmt --all -- --check
cargo check --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo run --release --locked -p fluent-gallery --bin fluent-gallery -- --smoke-test
```

参考分析成功提取两个模板各 12 个交互状态，以及对应 Gallery 示例。四项 Cargo 检查通过，`cargo test` 目标当前含 0 个 Rust 测试。Windows 调试版通过默认 125% 缩放冒烟检查，发布版在 100%、125%、150%、200% 缩放下通过 720×620 和 320×640 逻辑像素两组场景。另三种缩放通过 `SLINT_SCALE_FACTOR` 独立启动进程复现。

| 项目 | 验证证据 |
| --- | --- |
| 输入状态 | 运行时分发鼠标和键盘事件。两个控件均通过 hover、按下暂不触发、拖出取消、松开切换、Tab 焦点、Space/Enter、长按重复拦截、切换模式、禁用期间取消和再启用。 |
| 绑定与选中值 | 三态按 Unchecked → Checked → Indeterminate → Unchecked 循环，二态按 Unchecked ↔ Checked 切换。宿主写入 Indeterminate 不发用户事件，二态从该值激活会回到 Unchecked。 |
| 视觉与布局 | Skia 像素检查覆盖悬停、按下、禁用、键盘焦点环、亮暗与高对比选中填充、CheckBox 勾号与短横线。窄窗检查 44 逻辑像素命中尺寸与长标签换行。 |
| 材质 | 检查 Mica / Mica Alt 的亮暗回退、高对比回退、局部 Acrylic 的亮暗固色。在应用内模糊开启时验证样片确实柔化底图硬边，高对比时恢复固色。 |
| 现有控件 | Button、TextBox 和原有 Foundations 的互动与主题冒烟检查均继续通过。 |

发布版默认 Button 页启动 4 秒后，每 500 毫秒读取一次 Private Bytes，三次均为 45,207,552 字节。这是单次启动的空闲快照。帧时间、启动耗时、长时间切换和 Acrylic 增量开销未测试。

## 未测试

- 实际触控、Windows 辅助技术和官方视觉对照仍需实体触控设备、读屏软件及同主题同缩放的官方截图或录屏。Slint 1.19 的 AccessKit 桥只传布尔选中值，中间态以辅助描述表达；数字键盘专用加减键待上游事件提供键区信息。
- Windows 10 1809 需要 Build 17763 环境。macOS、iOS 与 Android 本轮依第 13 节限定在 Windows 开发平台，后续需对应设备与构建环境。
- P2 初次验收时已验证局部预览的模糊和半透明着色。后续已验证 Windows 窗口 Acrylic。原生 Mica 与应用内 Acrylic 的噪声及排除混合仍待补。 [窗口 Acrylic 验证](../design/specifications/desktop-acrylic.md)

## 产物

Skia 像素缓冲与参考 JSON 均在内存中检查。已盘点临时目录，未产生待清理的截图或一次性脚本。保留 Windows 调试版、发布版、现有增量缓存和过去的签名 APK。
