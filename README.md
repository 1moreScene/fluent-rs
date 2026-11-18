# Fluent RS

使用 Rust 和 Slint 自研 Fluent 风格原生控件。Gallery 包含 Button、TextBox、ToggleButton 和 CheckBox 的交互与状态演示，以及用于验证共享视觉基础的 Foundations 页。

## 开发与验证

工作区锁定 Rust 1.100.0 与 Slint 1.19.0。安装 rustup 后，从仓库根目录运行以下命令。Windows 需安装 Visual Studio 的 MSVC x64 编译工具和 Windows SDK。

```sh
cargo run --locked -p fluent-gallery --bin fluent-gallery
cargo run --locked -p fluent-gallery --bin fluent-gallery -- --smoke-test

cargo fmt --all -- --check
cargo check --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

窗口顶部可切换页面及亮色、暗色、高对比主题。**Toggles** 展示 ToggleButton 和 CheckBox 的二态、三态及交互状态。**Foundations → Desktop Acrylic** 在 Windows 11 Build 22621 及以上启用系统窗口 Acrylic。该页也提供 Mica / Mica Alt 的纯色回退和局部应用内 Acrylic 模糊预览。**TextBox** 页包含单行、只读和多行示例。在已聚焦的可编辑单行框输入内容后，点击右侧的 **×** 可清空并继续输入。 [窗口材质规格](design/specifications/desktop-acrylic.md)

冒烟模式在桌面会话中验证控件输入、三态切换、主题和材质，并使用 Skia 像素缓冲检查交互样式及窄窗口布局，然后自动退出。

**Radio** 页展示独立 RadioButton、两个互不影响的 RadioGroup，以及已选中和未选中时的交互状态。用 Tab 进入组，用方向键移动并选择，按住 Ctrl 时只移动焦点。 [RadioButton 与 RadioGroup 规格](design/specifications/radio-button.md)

**Range** 页展示水平和垂直 Slider、ProgressBar 和 ProgressRing。可切换步长与刻度吸附、反转方向，并用 Slider 验证确定进度。页面也可切换不确定模式、暂停、错误和圆环停用。 [Slider](design/specifications/slider.md) · [ProgressBar](design/specifications/progressbar.md) · [ProgressRing](design/specifications/progressring.md) · [验证记录](docs/p2-range-validation.md)

## 平台构建

| 平台 | 最低版本 | 目标 |
| --- | --- | --- |
| Windows | Windows 10 1809，Build 17763 | `x86_64-pc-windows-msvc` |
| macOS | 15.0 | `aarch64-apple-darwin`、`x86_64-apple-darwin` |
| iOS / iPadOS | 16.0 | `aarch64-apple-ios` |
| Android | 15，API 35 | `aarch64-linux-android` |

在 Windows 使用 `cargo build --locked -p fluent-gallery --bin fluent-gallery --release` 生成 `target/release/fluent-gallery.exe`。部署机器需安装与构建工具兼容的 x64 VC++ v14 运行库。 [Microsoft Visual C++ 运行库](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist/)

在 macOS 安装 Xcode 命令行工具后，用 `rustup target add aarch64-apple-darwin x86_64-apple-darwin` 安装两个目标，再分别用 `cargo build --locked -p fluent-gallery --bin fluent-gallery --target <目标>` 构建。Intel 与 Apple Silicon 实机分别运行同架构产物进行验收。

### iOS

安装 Xcode 和 XcodeGen，在 macOS 的仓库根目录运行：

```sh
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
cd platforms/ios
xcodegen generate
open FluentGallery.xcodeproj
```

在 Xcode 中选择 iOS 16 或更新版本的设备或模拟器运行 `FluentGallery`。实体设备需配置签名团队。若构建 Intel 模拟器包，另安装 `x86_64-apple-ios`。Xcode 构建阶段调用 Cargo 构建同一个 Rust 入口并装入 `.app`。参见 [Slint iOS 平台说明](https://docs.slint.dev/latest/docs/slint/guide/platforms/mobile/ios/)。

### Android

准备 JDK 17、Android SDK Platform 35、Build Tools 35、Platform Tools、NDK r30 或更新版本，以及 ARM64 API 35 真机或模拟器。设置 `JAVA_HOME`、`ANDROID_HOME` 和 `ANDROID_NDK_ROOT`，并使 `adb` 和主机 `clang` 可从 PATH 调用。

```sh
rustup target add aarch64-linux-android
cargo install cargo-apk2 --version 1.5.0 --locked
cd apps/gallery
cargo apk2 run --target aarch64-linux-android --lib
```

构建发布 APK 使用 `cargo apk2 build --target aarch64-linux-android --lib --release`，并按打包工具要求提供发布签名。参见 [Slint Android 平台说明](https://docs.slint.dev/latest/docs/slint/guide/platforms/mobile/android/)。

共享 Slint 设计令牌、视觉原语和控件源码位于 `ui/`。资源映射与依赖见 [P1 共享基础](design/specifications/foundations.md)、[Button](design/specifications/button.md)、[TextBox](design/specifications/textbox.md)、[ToggleButton](design/specifications/togglebutton.md) 和 [CheckBox](design/specifications/checkbox.md) 的规格。测试证据见 [P0](docs/p0-validation.md)、[P1](docs/p1-validation.md) 和 [P2 验证记录](docs/p2-validation.md)。
