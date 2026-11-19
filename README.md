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

页面顶部可切换亮色、暗色和高对比主题。**Toggles** 展示 ToggleButton 和 CheckBox 的二态、三态及交互状态。**Foundations → Desktop Acrylic** 在 Windows 11 Build 22621 及以上启用系统窗口 Acrylic，在 macOS 使用 AppKit 合成窗口后方的内容。该页也提供 Mica / Mica Alt 的纯色回退和局部应用内 Acrylic 模糊预览。**TextBox** 页包含单行、只读和多行示例。在已聚焦的可编辑单行框输入内容后，点击右侧的 **×** 可清空并继续输入。 [窗口材质规格](design/specifications/desktop-acrylic.md)

冒烟模式在桌面会话中验证控件输入、三态切换、主题和材质，并使用 Skia 像素缓冲检查交互样式及窄窗口布局，然后自动退出。

**Radio** 页展示独立 RadioButton、两个互不影响的 RadioGroup，以及已选中和未选中时的交互状态。用 Tab 进入组，用方向键移动并选择，按住 Ctrl 时只移动焦点。 [RadioButton 与 RadioGroup 规格](design/specifications/radio-button.md)

**Range** 页展示水平和垂直 Slider、ProgressBar 和 ProgressRing。可切换步长与刻度吸附、反转方向，并用 Slider 验证确定进度。页面也可切换不确定模式、暂停、错误和圆环停用。 [Slider](design/specifications/slider.md) · [ProgressBar](design/specifications/progressbar.md) · [ProgressRing](design/specifications/progressring.md) · [验证记录](docs/p2-range-validation.md)

**ListBox** 页展示单选、多选和扩展范围选择，支持十万行虚拟滚动。方向键移动焦点，Home 和 End 跳转首尾，扩展模式使用 Shift 选择范围。 [ListBox 规格](design/specifications/listbox.md)

**ComboBox** 页可在少量工作区与十万行数据之间切换。**Font size** 演示可编辑版本。输入时补全匹配项，也可输入 8 至小于 100 的自定义字号。用 Enter 或 Tab 提交，Esc 恢复上一次提交的值。 [ComboBox 规格](design/specifications/combobox.md) · [P4 验证记录](docs/p4-selectors-validation.md)

**AutoSuggestBox** 页根据输入过滤控件名称。上下键预览候选，Esc 和 Tab 恢复输入原文。Enter 或点击候选可提交所选项，搜索按钮提交当前文本。页面可切换搜索图标、候选填充和禁用状态。 [AutoSuggestBox 规格](design/specifications/autosuggestbox.md)

**PasswordBox** 页展示 Peek、Hidden 和 Visible 模式。Peek 在本次聚焦期间从空密码输入后出现，按住眼睛或 Alt+F8 显示，释放隐藏。显隐按钮跳过 Tab，复制与剪切在显示模式中也受限制，粘贴沿用原生编辑器。 [PasswordBox 规格](design/specifications/passwordbox.md)

**ToggleSwitch** 页展示开关、长标签及各交互状态。点击状态文字或按 Space 切换，也可拖动滑块；标题不触发切换。 [ToggleSwitch 规格](design/specifications/toggleswitch.md)

**RepeatButton** 页演示按住连发、按下/释放/悬停三种触发方式和禁用。默认按下立即执行，500ms 后每 33ms 重复；Space 连发，Enter 执行一次。 [RepeatButton 规格](design/specifications/repeatbutton.md)

**HyperlinkButton** 页包含在默认浏览器打开 Microsoft 主页的链接，以及进入 ToggleButton 页的操作链接，可切换禁用和长标签。 [HyperlinkButton 规格](design/specifications/hyperlinkbutton.md)

`target/debug/fluent-gallery --smoke-test repeat-hyperlink` 单独验证这两个控件的主题、键盘/指针、连发时序及取消、URI 请求和窄窗。测试使用替代的 URI 接收器，不自动打开外部浏览器。

用 `target/debug/fluent-gallery --smoke-test password-switch` 可单独验证这批控件、共享文本框、遮蔽圆点和内联图标几何、系统剪贴板及窄窗布局。

**TreeView** 页演示折叠、展开、单选、父子级联多选、半选状态和 4,000 节点的虚拟滚动。左右键展开、折叠或移动到父子项，Space 在多选模式切换勾选。 [TreeView 规格](design/specifications/treeview.md)

**GridView** 页可切换行优先与列优先、四种选择模式以及十万项数据，演示自定义条目模板和双向布局的虚拟滚动。通过 `target/debug/fluent-gallery --smoke-test grid-view` 复验。 [GridView 规格](design/specifications/gridview.md)

**CommandBar** 页可切换宽度、标签位置及关闭时的显示模式。缩窄后，移入 More 的主命令保留原身份并排列在次命令之前。可用 Esc 关闭菜单，用 Preview 验证开关命令。 [CommandBar 规格](design/specifications/commandbar.md)

**Expander** 页演示向上和向下展开、居中标题与左对齐内容。方向从独立的下拉框选择，标题获得焦点后可用 Space 或 Enter 切换。 [Expander 规格](design/specifications/expander.md)

**SplitButton** 页展示紧凑的色块按钮和文字按钮。箭头区打开三列色板，Alt+↓ 或 F4 也可打开，Esc 关闭并恢复焦点。 [SplitButton 规格](design/specifications/splitbutton.md)

**DropDownButton** 页演示 Email 文本按钮、仅图标按钮和禁用状态。点击任意一侧或按 Space / Enter 打开邮件操作菜单，方向键选择，Esc 关闭。 [DropDownButton 规格](design/specifications/dropdownbutton.md)

**ToggleSplitButton** 页可用主区启用或取消列表格式，用箭头、F4 或 Alt+↓ 选择项目符号或罗马数字。用 `target/debug/fluent-gallery --smoke-test button-flyouts` 可在本机复验这两个控件。 [ToggleSplitButton 规格](design/specifications/togglesplitbutton.md)

**NumberBox** 页演示表达式输入、Hidden / Inline / Compact 步进按钮、范围、循环和小数格式。Enter 提交、Esc 恢复，方向键和 PageUp / PageDown 步进；只有编辑框聚焦时滚轮改变数值。 [NumberBox 规格](design/specifications/numberbox.md)

**PersonPicture** 页演示姓名首字母、照片、群组和数字 / 字形 / 图片徽章，以及不同头像尺寸。用 `target/debug/fluent-gallery --smoke-test number-person` 可在本机复验这两个控件。 [PersonPicture 规格](design/specifications/personpicture.md) · [本批验证记录](docs/number-person-validation.md)

**SelectorBar** 页展示可横向滚动的单选导航。Tab 进入后用左右键切换，Home 和 End 跳到首尾。 [SelectorBar 规格](design/specifications/selectorbar.md)

**PipsPager** 页可切换方向、箭头可见性和循环。方向键移动圆点焦点，Space 或 Enter 提交页码。用 `target/debug/fluent-gallery --smoke-test selector-pager` 可复验这两个控件。 [PipsPager 规格](design/specifications/pipspager.md)

**Overlays** 页展示 Flyout、ContextMenu 和 Tooltip。点按 Review changes 打开 Flyout，右键或 Shift+F10 打开 Workspace actions，在 Pin workspace 上悬停或用 Tab 聚焦可查看提示。 [弹层与选择器复验](docs/p4-overlays-validation.md)

**ContentDialog** 页展示默认命令、无默认命令和自定义输入三种用法。Tab 在模态窗口内循环，Esc 关闭并把焦点交还给打开它的控件。 [ContentDialog 规格](design/specifications/contentdialog.md)

**InfoBar** 页展示信息、成功、警告和错误通知，可切换长正文及关闭按钮，并重新打开已关闭的通知。 [InfoBar 规格](design/specifications/infobar.md)

Gallery 启动后通过 **NavigationView** 侧栏访问示例。**Overlays** 分组用 AnimatedIcon 播放展开和收起，**NavigationView** 条目通过 InfoBadge 显示示例中的 Mail 通知数。窄窗可通过左上角按钮打开侧栏。

**NavigationView** 示例用 Frame 和 Page 演示页面切换与前进后退。可以选择 Auto、Left、Top、LeftCompact 或 LeftMinimal，查看顶部溢出、分组与子菜单、尾部项及明暗材质。用 **Custom pane content** 添加 Mail 通知并查看自定义插槽。Gallery 侧栏与 NavigationView 示例中的 **Settings** 项使用官方动画齿轮。侧栏的 Settings 打开 Foundations 页面，该页还提供独立的 InfoBadge 和 AnimatedIcon 示例。 [NavigationView、Frame、Page 规格](design/specifications/navigationview.md) · [Fluent 图标来源与许可](ui/icons/fluent/README.md)

## 平台构建

| 平台 | 最低版本 | 目标 |
| --- | --- | --- |
| Windows | Windows 10 1809，Build 17763 | `x86_64-pc-windows-msvc` |
| macOS | 15.0 | `aarch64-apple-darwin`、`x86_64-apple-darwin` |
| iOS / iPadOS | 16.0 | `aarch64-apple-ios` |
| Android | 15，API 35 | `aarch64-linux-android` |

在 Windows 使用 `cargo build --locked -p fluent-gallery --bin fluent-gallery --release` 生成 `target/release/fluent-gallery.exe`。部署机器需安装与构建工具兼容的 x64 VC++ v14 运行库。 [Microsoft Visual C++ 运行库](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist/)

本机构建统一使用 `target/debug/` 和 `target/release/`。从仓库根目录运行 `cargo build --locked -p fluent-gallery --bin fluent-gallery` 即可生成 `target/debug/fluent-gallery`，上面的 `cargo run` 也使用这份产物。显式传入 `--target` 时，Cargo 使用 `target/<目标>/<构建模式>/`。

macOS Gallery 默认以 980×700 逻辑像素启动，界面使用 125% 的文字和控件尺寸，并随显示器的系统缩放渲染。

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
