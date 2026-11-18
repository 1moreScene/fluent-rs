# P1 验证记录

日期 2026-11-18。使用 Windows 11 Pro Build 26200 x64、Core i5-1235U、Intel Iris Xe（驱动 32.0.101.7084），桌面分辨率 1920×1080，Rust 1.100.0、Slint 1.19.0、Skia OpenGL。官方资源和提交见 [P1 共享基础](../design/specifications/foundations.md)。

## 复现

```powershell
cargo fmt --all -- --check
cargo check --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo run --release --locked -p fluent-gallery --bin fluent-gallery -- --smoke-test
```

上述命令通过。Cargo 测试目标当前含 0 个 Rust 单元测试，Gallery 冒烟模式执行窗口内验证。默认 125% 缩放下输出 900×775 与 400×800 像素。另在独立进程用 `SLINT_SCALE_FACTOR` 分别覆盖 100%、150%、200%，验证同一组 720×620 和 320×640 逻辑像素场景。

| 范围 | 证据 |
| --- | --- |
| Button | 通过程序分发的鼠标和键盘事件检查按下与松开、拖出取消与拖回、Tab、Space/Enter、去除重复触发、禁用和恢复。Skia 像素检查确认了悬停、按压、禁用填充及焦点环出现和消失。 |
| TextBox | 通过程序分发的事件检查焦点、中文与英文字符输入、选择、编辑、单行提交、多行换行、只读、禁用和清除。Skia 像素检查覆盖亮暗主题焦点底边。 |
| Foundations | 通过实际页面按钮路径切换减少动效，验证高对比关闭动画；检查替换强调色所绘制的像素、放大文字后的 Button 高度和窄窗布局，以及 Mica/Acrylic 的不透明回退色。窄窗验证 44 逻辑像素的触控高度。 |
| Android | `cargo check --locked -p fluent-gallery --lib --target aarch64-linux-android` 与 `cargo apk2 build --target aarch64-linux-android --lib` 通过。调试 APK 为 ARM64，最低与目标 API 均为 35，16 KiB 对齐及 APK v3 签名经工具核验。 |

用发布版在默认 Button 页空闲采样。每次启动等待 4 秒，然后每 500 毫秒读取一次进程 Private Bytes，共三次。三次独立启动的读数范围为 44,208,128–54,616,064 字节。该范围包含启动间波动，尚未建立可比较的内存回归基线。

## 未测试

| 项目 | 所需环境或材料 |
| --- | --- |
| Android 15 安装、启动、触控、软键盘及生命周期 | ARM64 API 35 设备或模拟器。当前仅验证编译与打包。 |
| Windows 10 1809、macOS 15 与 26、iOS/iPadOS 16 | Windows 10 Build 17763 VM 或实机、相应 Apple 设备及 Xcode 构建环境。 |
| 实际触控、系统 IME 组合、系统剪贴板、读屏和系统偏好同步 | 需要设备上的人工操作与辅助技术。Gallery 当前的主题、强调色、字号和减少动效由应用设置驱动。 |
| 原生材质、动效曲线和官方像素差异 | 原生材质安排在 P5。需要同一主题、系统缩放及状态下的官方截图或录屏。 |
| 启动耗时、帧时间、持续重绘与长期内存趋势 | 需要固定场景和单独采样。 |

P1 代码和 Windows 冒烟验证已就绪。实际触控及最低平台验收仍待完成。

## 产物盘点

像素检查在内存中完成。清理了 `target/debug/apk/` 中被签名版替代的未对齐 APK，保留已签名 APK、签名侧车文件、Windows 发布版、增量缓存和现有验证脚本。已有调试版进程占用原 EXE，本轮使用独立的发布版完成窗口验证。
