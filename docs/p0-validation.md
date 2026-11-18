# P0 验证记录

日期 2026-11-18。Windows 主机为 Windows 11 Pro Build 26200，x64，Intel Core i5-1235U，Intel Iris Xe 驱动 `32.0.101.7084`，桌面分辨率 1920×1080。工具链为 Rust 1.100.0、Slint 1.19.0、`skia-bindings` 0.157.0。

| 项目 | 状态 | 证据或验收环境 |
| --- | --- | --- |
| Windows 11 工作区质量门槛 | 通过 | `cargo fmt --all -- --check`、`cargo check --locked --workspace`、`cargo clippy --locked --workspace --all-targets -- -D warnings`、`cargo test --locked --workspace`。当前只有启动冒烟检查，Cargo 测试目标包含 0 个用例。 |
| Windows 11 Gallery | 通过，Skia OpenGL | x64 调试版和发布版分别连续三次退出码为 0，收到 OpenGL 绘制通知并生成包含非均匀内容的 900×600 像素缓冲。发布版 EXE 为 19,563,520 字节。 |
| Windows 11 Direct3D 12 | 未通过 | 初次使用 Slint 默认 Windows 配置时，Direct3D 12 适配器探测失败，同一窗口经 Skia CPU 回退成功绘制。Gallery 的 Windows 目标改用 Slint 的 `renderer-skia-opengl`，并在同一主机上验证了 OpenGL 绘制。 |
| Windows 10 1809 | 未测试 | 需要 Build 17763 的 x64 实机或虚拟机，运行同一 Gallery 二进制并记录适配器及回退行为。 |
| macOS 26 和 15.0 | 未测试 | 当前主机无 Apple SDK。需要 MacBook 上的 Xcode 构建环境，并在 macOS 15.0 的 Apple Silicon 和 Intel 环境分别运行。 |
| iOS / iPadOS 16.0 | 未测试 | 用 Git Bash 检查脚本语法，并用替身命令核对实体设备、ARM64 模拟器和双架构模拟器的目标映射与输出路径。需要在 Xcode 生成并构建真实 `.app`，再部署到 iOS 或 iPadOS 16 设备或模拟器。 |
| Android 15 ARM64 编译检查 | 通过 | `cargo check --locked -p fluent-gallery --lib --target aarch64-linux-android`，SDK Platform 35、Build Tools 35.0.0、JDK 17.0.21。 |
| Android 15 ARM64 调试 APK | 通过 | `cargo apk2 build --target aarch64-linux-android --lib`，NDK r30。APK 安装清单的最低与目标 API 均为 35，包含 ARM64 NativeActivity。原生库的 ELF 加载段和 APK 中的 `.so` 都通过 16 KiB 对齐检查，APK v3 调试签名验证通过。 |
| Android 15 启动 | 未测试 | 需要 ARM64 API 35 真机或模拟器安装并验证启动、渲染及生命周期。 |
| 视觉及性能基线 | 未测试 | 冒烟检查验证像素缓冲有实际绘制。控件对照需要可用的官方截图，性能验收需要采样场景和运行设备。 |

Windows 冒烟检查可用 `cargo run --locked -p fluent-gallery -- --smoke-test` 复现。OpenGL 成功输出形如 `Fluent RS Gallery: Skia rendered 900x600 (OpenGL)`，尺寸取决于窗口与缩放设置。

## 官方参考资料

2026-11-18 以 Python 检查了本地参考路径，并用 Git 核验来源和版本。`D:/vscode/WinUI-Gallery` 的 `origin` 指向微软官方仓库，当前为 detached HEAD，提交 `7360ac322414e118eadfb0c7c2f4ecc31a218f59`。未发现本地微软官方 `microsoft-ui-xaml` 仓库。远端默认分支 `main` 的查询结果为 `354bf6017e6443023214d34309dfdf0fe2d526b1`，仅供后续定位。首次研究控件时需重新核验两份仓库版本。 [WinUI Gallery](https://github.com/microsoft/WinUI-Gallery) · [WinUI 官方源码](https://github.com/microsoft/microsoft-ui-xaml)

## 产物盘点

冒烟检查仅在内存中读取像素缓冲。已清理 `.artifacts/` 下的 DXC 探测文件、iOS 包装脚本试验产物及 Android 工具下载压缩包，以及 `target/` 下的一次性打包工具编译缓存和未对齐 APK。保留 Windows 二进制、已签名的 Android 调试 APK、项目增量缓存及供复测的隔离 Android 工具链。
