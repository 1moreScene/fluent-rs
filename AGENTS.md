# AGENTS.md — Rust + Slint Fluent UI 项目开发规范

> 本文件是仓库内 AI 编码代理与贡献者的工程约束。开始工作前先阅读本文件，再检查实际仓库结构和现有代码。除非用户明确修改决策，不得擅自替换技术栈、降低兼容要求或引入第三方 WinUI 仿制主题。

## 1. 项目目标

使用 **Rust + Slint** 自研一套高保真、跨平台的 **Microsoft WinUI 3 / Fluent Design 风格**组件与材质系统，供原生桌面和移动应用复用。

优先级（从高到低）：

1. **跨平台视觉一致性与高保真**：以 Microsoft 官方 WinUI 3 / Windows Design 规范及可合法获取的 WinUI Gallery 行为为参照，复刻控件的尺寸、布局、颜色、状态、阴影、材质与动画。
2. **性能与资源效率**：原生编译、GPU 加速、合理的内存与 CPU 占用；避免浏览器运行时与不必要的大型依赖。
3. **可维护性**：共享设计令牌、可复用组件、清晰的平台抽象、可测量的性能和视觉回归。
4. **可用性与无障碍**：键盘导航、焦点状态、触控、文本缩放、屏幕阅读器等，在平台能力允许时尽量一致。

本项目**不是** WinUI 3 官方运行时的移植，也**不是**通过给某个现成主题换色实现的外观模仿。上层 UI 由自研组件构成；不要求不同系统具有相同的原生底层窗口能力。

## 2. 不可更改的技术决策

- **应用与组件实现语言**：Rust；UI 声明使用 `.slint`。
- **UI 框架**：Slint，初始基线 **1.19.x**。首次引入时精确锁定已验证版本，并提交 `Cargo.lock`；升级必须通过平台回归测试。
- **首选渲染器**：Skia；允许 Skia 等底层依赖使用 C/C++，但**不以 C++ 编写本项目的功能实现**。如需要原生桥接，优先使用 Rust 的平台 API 封装；平台要求不得不用少量其他语言的启动/构建胶水时，先说明原因和最小范围。
- **备选渲染器**：FemtoVG，仅在明确需要比较内存、兼容性或帧时间且具备测量结果时引入；不得无依据地替换默认渲染器。
- **禁止以社区 WinUI / Fluent 仿制主题或控件库替代自研组件**。可以参考其实现思路，但不能作为本项目视觉层的直接依赖。
- **禁止 Web UI 技术路线**：不使用 Electron、基于 WebView 的 UI、Tauri 前端、Chromium 或 HTML/CSS/JS 来实现主要界面。
- **不得重写整个渲染引擎或布局系统**；优先使用 Slint 提供的布局、输入、动画、渲染与事件能力。
- 不考虑 Linux **桌面**发布。Android Rust 目标名包含 `linux` 是正常的，不属于 Linux 桌面支持。

## 3. 正式支持平台与最低版本

| 平台 | 最低版本 | 架构 / 构建目标 | 备注 |
| --- | --- | --- | --- |
| Windows | **Windows 10 1809 (Build 17763)** | `x86_64-pc-windows-msvc` | 同时支持 Windows 11；Win11 专有 DWM 属性必须做运行时能力判断 |
| macOS | **macOS Sequoia 15.0（2024）** | `aarch64-apple-darwin`、`x86_64-apple-darwin` | 以 15.0 为最低部署目标，验证 Intel/Apple Silicon 双架构 |
| iOS | **iOS 16.0** | `aarch64-apple-ios`；模拟器另行配置 | 包含 iPadOS 对应版本，适配安全区域、触控与生命周期 |
| Android | **Android 15（API 35）** | 首选 `aarch64-linux-android` | `minSdk = 35`，验证实体设备或 API 35 模拟器 |

**明确不支持**：Windows 7/8/8.1、Windows 10 1809 以前版本、macOS 15 以前版本、iOS 16 以前版本、Android 15 以前版本、Linux 桌面。

特别提醒：在 macOS 26 / Windows 11 开发机上通过编译或运行，**不代表**已验证最低系统版本。必须在对应最低版本的测试环境中实际运行与验收。设置 Deployment Target 也不等于依赖自动兼容。

## 4. 已有开发设备与职责

- **MacBook（M3 / macOS 26）**：macOS ARM64 开发、macOS Intel 交叉编译、iOS 构建与模拟器、Apple 平台材质和 Metal 路径测试、签名与打包。
- **Windows 11 PC**：Fluent 视觉基准、Win32 / DWM 集成、Windows 构建、Windows 10 1809 虚拟机测试、性能采样。
- **Android 15 模拟器或实体机**：安装与启动、GPU 渲染、触控、虚拟键盘、暂停恢复和旋转测试。
- **Intel macOS 15 测试环境（待准备）**：验证 x86_64 程序的真实部署；Rosetta 运行不能完全替代目标 Intel 机器的系统兼容测试。

不要声称已经在某个平台“测试通过”，除非确实在该平台执行过测试并记录证据。

## 5. 微软官方源码与 Gallery：最高优先级参考资料

项目所有与 **WinUI 3 视觉、材质、UI 样式、控件状态、交互、动画** 有关的复刻工作，必须首先查看以下两份**微软官方仓库**（用户会自行下载到本地）：

| 参考仓库 | 官方地址 | 主要用途 |
| --- | --- | --- |
| **WinUI 官方源码**（`microsoft-ui-xaml`） | <https://github.com/microsoft/microsoft-ui-xaml> | 控件定义、默认 Style/ControlTemplate、ThemeResource、VisualState、动画、资源字典及实现原理 |
| **WinUI Gallery 官方仓库**（`WinUI-Gallery`） | <https://github.com/microsoft/WinUI-Gallery> | 官方控件示例、用法、页面布局、各种交互状态、响应式案例与展示行为 |

这两份仓库是本项目的**一级主要参考**，不是可选的补充材料，也不得用社区 WinUI/Fluent 仿制库替代。`Microsoft Learn` / Windows Design 官方文档可作为官方 API 契约、设计解释、辅助功能与版本差异的补充。只有当官方资料确实不足时，才能考虑其他来源，且须注明来源和不确定性。

### 5.1 本地参考仓库的定位与保护

- 用户自行下载仓库；**先检查本地实际路径**，不得擅自假设它们已经存在，不得虚构文件、目录、类名或源码内容。
- 推荐放在项目根目录的 `references/microsoft-ui-xaml/` 与 `references/WinUI-Gallery/`，但用户已有其他路径时，以真实路径为准；可通过环境变量 `WINUI_SOURCE_DIR`、`WINUI_GALLERY_DIR` 指定。
- `references/` 目录应加入 `.gitignore`，避免把整个官方源码仓库提交到本项目，降低仓库体积与许可证管理成本。
- 两份官方仓库视为**只读资料**。不得在其中直接修改、格式化、清理、重命名或提交文件；不得把它们作为本项目运行时依赖。
- 记录两个仓库实际的 Git Commit SHA、分支与分析日期；再次分析前检查是否已更新。**不能仅根据 `main` 分支名称断定版本稳定**。
- Gallery 可能展示实验性 Windows App SDK API；必须区分已稳定发布行为与实验性行为，不得直接把实验行为当作全部平台的正式规范。

### 5.2 强制工具规则：禁止 Computer Use；Python 优先

**绝对禁止使用 Computer Use / computer-use / 云端图形化电脑操作工具查看、浏览、搜索、阅读、分析或操作两份官方参考仓库及项目源码。** 不可为了方便截图、查找文件、点击网页、打开 IDE 或运行 Gallery 而绕过此禁令。不要先使用 Computer Use 再补做 Python 验证。

源码研究与资料抽取的工具优先顺序：

1. **Python 脚本（首选、默认）**：`pathlib` 遍历文件、`re`/字符串搜索、`xml.etree.ElementTree` 或适当的解析器分析 XAML、`json`/`csv` 输出结构化数据；跨平台可复现。
2. **本地 Git / 命令行工具（辅助）**：`git grep`、`git log`、`git rev-parse`、`rg`、构建工具；用于验证脚本发现、定位历史和获取版本信息。仍以 Python 自动化分析为先。
3. **官方网页或官方文档（补充）**：只在本地参考缺失、需要验证最新版本、说明差异或交叉核查时使用。不得使用 Computer Use 浏览网页。

对于视觉研究：使用用户提供或以非 Computer Use 方式取得的**合法截图/参考图片**，优先用 Python（Pillow、OpenCV、NumPy 等）进行尺寸测量、像素采样、对齐、颜色分析、裁切和图像差异。截图不存在时，应报告缺少什么材料，**不要编造实测像素值或假称已检查界面**。禁止用 Computer Use 捕获或查看 Gallery 界面。用户本人手动运行 Gallery 或提供截图，不属于代理使用 Computer Use。

如果使用 Python 脚本分析时无法获取完整答案，应指出限制并选择可审计的非 Computer Use 方案；不得悄悄切换为图形化自动操作。不得依赖浏览器 DevTools、点击式自动化或网页 UI 来代替可复现的本地源码分析。

### 5.3 每个控件的标准研究—实现流程

1. **定位官方源码**：使用 Python 在 `microsoft-ui-xaml` 搜索控件名及对应 XAML 样式、资源键、视觉状态、动画和模板；再在 `WinUI-Gallery` 查找对应示例与交互说明。
2. **提取设计规格**：形成结构化记录，包括尺寸、内外边距、字重、字体、颜色、透明度、圆角、描边、阴影、焦点、悬停/按下/禁用状态、过渡时长、easing、主题差异等。只记录实际有依据的值。
3. **记录证据**：在 `design/specifications/<control>.md` 写入参考仓库、相对文件路径、行号或资源键、Git Commit、截图来源与分析日期；不确定内容标记 `TODO(spec)`。
4. **独立实现**：使用 Rust + `.slint` 与项目自身 Design Tokens 实现控件；官方 XAML/C#/C++ 仅作为研究资料，不直接作为项目 UI 实现或依赖。
5. **可复现验证**：为 Gallery 添加全部状态的演示，对照官方截图、规格表和交互行为。视觉检查优先调用 Python 脚本生成差异报告，人工复核边缘抗锯齿、字体渲染和平台合成差异。
6. **回归与版本追踪**：官方仓库版本变化时，先生成设计差异记录，再决定是否修改令牌、控件与测试基准。

建议将自动化参考分析脚本集中在 `tools/reference_analysis/`，将其生成的结构化数据和简短报告存放于 `design/specifications/` 或 `docs/reference-analysis/`。脚本应接受命令行参数或环境变量指定两个仓库目录，**不得在代码中写死某个人的绝对路径**。默认只读；任何缓存或输出都写入本项目指定的输出目录，而非官方仓库。

### 5.4 版权、许可与引用边界

- 参考官方源码**不等于允许无限制复制内容**。使用每个仓库自己的许可证与第三方资产说明，保留必要的许可证及著作权信息；有不确定性时先报告。
- 不得直接批量复制官方 XAML 模板、C#/C++ 实现、字体、图标和图片到本项目并忽略版权或再分发要求。
- 优先根据提取出的设计规格与观察到的交互行为进行**独立 Rust + Slint 实现**。
- 在项目设计文档里引用**来源路径、资源键和 Git Commit**，而不是把大量官方源码粘贴进项目文档。

## 6. 建议的 Cargo Workspace 结构

以下是新仓库的目标结构；**如仓库已经存在，先阅读实际布局，尽量做增量修改，不要因为本示意而删除或大规模移动现有文件。**

```text
fluent-rs/
├── AGENTS.md
├── references/             # 本地只读官方参考仓库（gitignore，不提交）
│   ├── microsoft-ui-xaml/
│   └── WinUI-Gallery/
├── tools/
│   └── reference_analysis/ # Python 参考源码与截图分析脚本
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── crates/
│   ├── fluent-core/          # 应用无关的模型、状态和能力定义
│   ├── fluent-tokens/        # 颜色、排版、尺寸、圆角、阴影、动效令牌
│   ├── fluent-controls/      # 自研 .slint 基础控件
│   ├── fluent-navigation/    # 导航和复杂布局控件
│   ├── fluent-materials/     # Mica/Acrylic 材质抽象与降级策略
│   ├── fluent-motion/        # 动效规范及通用动画行为
│   └── fluent-platform/      # 与具体系统隔离的平台能力接口
├── platforms/
│   ├── windows/              # Win32 / DWM 适配
│   ├── macos/                # AppKit / CoreGraphics 等适配
│   ├── ios/                  # iOS 包装与生命周期
│   └── android/              # Android cdylib / Activity / 包装
├── apps/
│   └── gallery/              # 控件和状态演示、视觉对照
├── tests/
│   ├── visual/
│   ├── performance/
│   └── compatibility/
├── design/
│   ├── references/          # 经过许可的参考截图、索引，不含官方仓库副本
│   └── specifications/     # 每个控件的可追溯参考规格
└── docs/
```

Cargo workspace 里存放实际可编译 crate；`platforms/` 中哪些目录作为独立 crate、哪些仅提供原生工程包装，应依据 Slint 官方 Android/iOS 构建方式确定。不要为了符合目录示意而制造无用 crate。

## 7. 分层原则

1. **Design Tokens 层**：唯一的颜色、字号、字重、间距、圆角、描边、阴影和动画时长来源。禁止同类控件反复硬编码相近常量。
2. **控件层**：只负责 UI 外观、输入和状态表达；不得直接读写数据库、执行网络请求或调用平台专属 API。
3. **应用状态层**：Rust 负责业务状态、异步任务和数据模型；通过明确的属性、回调、模型与 Slint 交互。
4. **材质/平台层**：抽象背景材质、窗口透明、标题栏、DPI、安全区域、输入法、触控、生命周期等操作系统差异。
5. **Gallery 层**：每个公开控件都必须有演示页，覆盖主要状态、明暗主题、不同尺寸和键盘/触控交互。

优先使用静态类型的接口、组合式组件和清晰的单向数据流。禁止为赶进度把平台判断散落到各个控件中。

### 7.1 控件依赖优先：先基础设施，再组合控件（强制）

**实施顺序必须由前置依赖决定，而不是由控件在 Gallery 中的展示顺序、视觉复杂度或名称决定。** 可以并行开发彼此独立的分支，但不得在依赖未准备好时通过复制临时代码、硬编码数据或绕过统一状态机制“先做出外观”。

将共享能力按以下层次管理。层次是**逻辑模块**，并不要求每层都新增一个 Cargo crate；优先在既有 `fluent-core`、`fluent-controls`、`fluent-navigation`、`fluent-motion`、`fluent-platform` 内使用清晰的子模块：

| 层次 | 首先建立的能力 | 主要消费者 |
| --- | --- | --- |
| **F0：设计与运行基础** | Design Tokens、Theme/Accent、排版/图标资源、缩放与 DPI、Rust/Slint 状态桥接、Gallery 测试基座 | 所有控件 |
| **F1：交互与视觉原语** | `Surface`/`Text`/`Icon`/`Border`、状态与视觉反馈、指针/键盘/触控抽象、`FocusScope`/焦点环、禁用语义、基础动效、文本输入和 IME 能力验证 | 基础和组合控件 |
| **F2：基础控件** | Button/ToggleButton、CheckBox、RadioButton/RadioGroup、Slider、ProgressBar、ProgressRing、TextBox/PasswordBox | Dialog、InfoBar、表单和导航 |
| **F3：组合基础设施** | `SelectionModel`、`ItemPresenter`、数据模型适配、`ScrollView`/`ListView` 封装、`Popup`/`Overlay` 容器、位置计算、关闭规则、焦点恢复、页面宿主和导航状态 | ComboBox、菜单、Dialog、NavigationView、TreeView 等 |
| **F4：组合控件** | ListBox、ComboBox、Flyout/ContextMenu/Tooltip、ContentDialog、InfoBar、NavigationView | 应用完整页面与复杂交互 |
| **F5：高级控件与优化** | TreeView、DataGrid/TreeDataGrid、AutoSuggestBox、CommandBar、复杂虚拟化和编辑行为 | 后续扩展与大数据场景 |

注意：

- **优先复用 Slint 引擎已经提供的能力**：例如 `TextInput`、`FocusScope`、`PopupWindow`、`ScrollView`、`ListView`。本项目重新实现的是 Fluent 视觉、控件外部接口及缺失的交互语义，**不是重新造一套焦点引擎、文本输入法、滚动器或通用渲染器**。
- `FocusScope`/文本输入、键盘快捷键、触控和弹出层的能力边界，必须在 F1/F3 阶段用最小原型验证，尤其是移动端虚拟键盘、弹出层定位和关闭事件。不能把未经确认的 Slint 行为当作既有接口。
- 复杂容器的数据模型、选择状态、键盘导航、可访问性语义和虚拟化需要设计成共享基础设施；`ScrollView` 会实例化全部子项，长列表应优先验证 `ListView` 的可见项实例化机制，不得以 `ScrollView` 直接承载十万级列表。
- **动效基础设施在 F1 就应存在**（状态切换、时间参数、easing、可减少动画的偏好等）；F5 或后续阶段只是进一步打磨高级转场。不得等控件全部完成才统一补动画状态接口。
- **材质抽象必须在 F0/F1 确定语义与纯色降级接口**；平台 Mica/Acrylic 与昂贵的背景模糊效果可以在组合控件开发期间并行完善，不能成为所有基础控件的阻塞依赖。

### 7.2 关键组合控件的明确前置依赖

| 目标控件 | 必需前置能力 | 不可跳过的验证 |
| --- | --- | --- |
| **TextBox / PasswordBox** | 文本编辑引擎（优先 Slint `TextInput`）、焦点、IME、选择/光标、输入状态与主题 | 中英文输入法、复制粘贴、选择、禁用、虚拟键盘、密码遮蔽 |
| **CheckBox / RadioButton / ToggleSwitch** | 输入/焦点、状态反馈、可点击区域、Button/Toggle 基元；RadioGroup 需互斥状态 | 键盘操作、Checked/Indeterminate（适用时）、组间互斥、无障碍语义 |
| **ProgressRing** | Shape/绘制原语、Motion 基础、Reduced Motion 降级 | 动画频率、后台暂停、性能开销 |
| **ListBox / 可选列表** | `SelectionModel`、条目呈现、列表/滚动、焦点与键盘导航 | 单/多选、列表为空、长列表、可见项复用或创建策略 |
| **Popup / Flyout / Tooltip** | 弹层宿主、定位与裁剪、层级、关闭策略、焦点转移/恢复 | Esc、点击外部、窗口缩放、DPI、移动端触控 |
| **ComboBox** | Button/显示区、Popup、ListBox/SelectionModel、焦点与键盘导航、滚动 | 展开/收起、上下键/Enter/Esc、选中同步、长列表、移动端 |
| **ContentDialog** | Overlay/模态层、Button、焦点初始值/约束/恢复、键盘事件、响应式布局 | 模态阻塞、默认/取消操作、Esc、窗口缩放、安全区域 |
| **InfoBar** | Surface、Text/Icon、Button、状态主题；关闭动画为可选增强 | 级别样式、关闭、文本换行、无障碍通知 |
| **NavigationView** | 条目/选择模型、滚动/列表、页面宿主、导航状态、焦点/快捷键、响应式断点 | 选中同步、展开/折叠、历史状态、窄屏布局、不同 DPI |
| **TreeView** | 分层数据模型、展开状态、选择模型、焦点与方向键、虚拟化策略 | 深层树、节点折叠、定位、可访问性 |
| **DataGrid / TreeDataGrid** | 列/行模型、视口与虚拟化、选择、编辑提交、排序、滚动同步、焦点/IME | 10 万行数据、行列虚拟化、编辑、键盘交互、内存与帧时间 |

此表为工程依赖基线，**不是**微软控件内部实现结构的断言。每个 WinUI 目标控件的精确行为仍以第 5 节规定的本地官方源码/Gallery 分析为准。

### 7.3 每次新增控件的依赖准入门槛（强制）

开发一个控件之前，先在 `design/specifications/<control>.md` 或关联的任务记录里列明：

1. **依赖清单**：前置 Tokens、交互原语、平台能力、共享组件、状态模型及其实现路径；标明“已实现 / 待实现 / 平台待验证”。
2. **依赖关系**：说明上游 API、下游消费者，若存在循环依赖须先拆分公共原语，不允许使用跨 crate 双向依赖或复制实现规避。
3. **最小可运行测试**：包含基础状态、焦点/键盘/触摸（适用时）、异常关闭/恢复、至少亮暗两主题和初步性能检查。
4. **准入条件**：必要依赖在 Gallery 或自动化测试中经过验证后，才实施正式组合控件；允许做明确标记的技术探索性原型，但不得将其伪装成已完成的正式组件。
5. **完成条件**：确认组合控件不重复实现已有 Popup、Selection、Focus、Scroll、Motion 逻辑，且未破坏其他依赖方；新增/修改的临时验证产物遵守第 13.1 节清理规则。


## 8. Fluent 高保真规范

- **参考基准**：本文件第 5 节指定的微软 WinUI 官方源码与官方 WinUI Gallery（一级参考），以及 Microsoft Learn 和项目 `design/specifications/` 中经确认、带来源追踪的规格。
- **禁止主观猜测**颜色、尺寸和动画参数后宣称“1:1 还原”。不确定时标为 `TODO(spec)`，写清测量与验证方式。
- **首批控件（候选范围，不表示实现顺序）**：Button、TextBox、PasswordBox、CheckBox、RadioButton、ToggleSwitch、Slider、ComboBox、NavigationView、ContentDialog、InfoBar、ProgressBar、ProgressRing。**实际顺序严格遵守第 7.1–7.3 节的依赖门槛与第 14 节里程碑。**
- **各状态必须覆盖**：Normal、PointerOver、Pressed、Focused、Disabled，以及控件特有的 Selected / Checked / Error 等。
- **主题**：亮色、暗色、强调色、高对比需求；优先复用令牌，不允许在组件内堆叠平台专属颜色分支。
- **排版**：定义字体栈、字重映射和基线；使用合法可分发或系统提供的字体，不擅自捆绑微软专有字体与图标资产。
- **图标**：优先自制或有明确授权的 SVG / 向量资源；标明第三方资产的版权与许可证。
- **动画**：准确模拟进入、离开、悬停、按下、导航和弹出层动效；避免纯粹为了“好看”而添加不符合原规范的动画。
- **辅助功能**：控件需具有合理的焦点反馈、语义、标签、可触达目标和禁用状态；涉及 Slint 无法直接覆盖的功能时记录平台限制。

**视觉目标**：高保真是持续验证的工程目标，不得在尚无对比截图、规格数据和交互测试时标称“像素级一致”。

## 9. Mica、Acrylic 与窗口材质

- 将材质定义为**视觉语义**（如 `Mica`、`Acrylic`、`SolidFallback`），而非直接等同于某个 DWM API。
- Windows 11：检测具体 DWM 能力后才请求系统背景材质；**不要**在 Windows 10 直接调用 Win11 专有属性并假定有效。
- Windows 10：按操作系统和窗口合成能力选择可用原生效果或自研模拟效果。
- macOS / iOS：使用对应平台的窗口/视图合成机制；目标是接近一致的结果，而不是依赖 Windows API。
- Android：考虑系统模糊限制、功耗、API 级别以及应用内背景模糊的实际性能。
- 应用内部 Acrylic：优先验证 Slint + Skia 的背景模糊、半透明叠层、着色与噪声；不得把应用内 blur 与系统级跨窗口模糊混为一谈。
- 材质失败必须有不闪烁、可读、可操作的纯色降级路径。
- 所有模糊、阴影和透明窗口特效都需在最低目标系统和低性能 GPU 上测量开销。

## 10. 多平台交互与响应式设计

- **共用设计语言，允许布局因设备类型变化**：桌面可采用侧边 NavigationView；手机可以使用紧凑导航或底部导航，视觉令牌和控件状态仍应统一。
- 桌面端：鼠标悬停、右键、键盘焦点、快捷键、窗口缩放、不同 DPI/多显示器。
- 移动端：触控命中区域、滚动惯性、安全区域、横竖屏、软键盘、后台/恢复、内存压力。
- 不以桌面 `hover` 作为移动端关键操作的唯一入口。
- 不得以同一套固定像素布局强行覆盖手机与桌面；使用可测量的断点和适配规则。

## 11. 性能要求与测量

性能优先，但避免凭语言或框架推断实际内存用量。维护可复现的基准，包含：

- 冷启动耗时、首次可交互耗时；
- 空闲/峰值 **Private Bytes 或等效私有内存**、工作集、GPU 资源占用；
- 60 Hz 与 120 Hz 的 P50/P95/P99 帧时间、动画卡顿；
- 复杂列表与树形结构的虚拟化和滚动性能；
- Acrylic / 阴影 / 半透明窗口的增量开销；
- 长时间运行、页面反复切换后的泄漏趋势；
- Android / iOS 的启动、切后台恢复、功耗和内存压力表现。

**初期参考目标**（不是框架官方保证，也不是未测试时的通过结论）：桌面轻量 Gallery 空闲私有内存争取不超过 **120 MB**；60 Hz 动画 P95 帧时间争取不超过 **16.7 ms**。数据必须带设备、OS、构建模式、分辨率、渲染器、场景和测量方法。

性能优化顺序：先确定可复现瓶颈 → 采样/剖析 → 做最小改动 → 对比测量。不要仅为“性能更好”增加不必要的 `unsafe`、复杂缓存或并行架构。

## 12. 代码质量与安全

- 使用稳定 Rust，工具链版本写入 `rust-toolchain.toml`；在仓库中锁定 Slint 和关键依赖版本。
- 所有公开的 Rust API 应有用途明确的文档与错误处理；优先 `Result`，避免在正常路径使用 `unwrap()` / `expect()`。
- 限制 `unsafe` 的范围。必须写清安全不变量、生命周期和平台前置条件，并提供对应测试。
- 异步任务不能阻塞 UI 事件循环；跨线程更新 Slint 状态使用 Slint 认可的事件循环机制。
- 避免不必要的复制、长期持有大型资源、重复加载字体和纹理。
- 不得引入隐含的网络遥测、跟踪或用户数据上传。
- 添加依赖前核查维护状态、许可证、二进制大小和各目标平台可构建性。
- 处理平台能力失败时明确降级，并提供可诊断日志；不要吞掉关键错误。

## 13. 验证命令与验收门槛

在仓库完成初始化且相关 crate 存在后，至少执行：

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

常规任务在用户当前开发平台执行适用的编译、启动与打包验证。**除非用户明确要求验证其他平台，否则不得为开发环境之外的平台执行目标构建、交叉编译、打包或签名。** 例如在 Windows 上开发时，只执行 Windows 构建和验证；即使本机已安装 Android SDK，也不生成 Android APK。跨平台兼容目标保持在第 3 节，未执行的目标验证按实际情况记录。

工作区 Cargo 检查只能验证所使用的构建目标。最低版本兼容性需要在相应系统或设备上实际运行；Android 和 iOS 的验收也需要对应目标的真实构建与运行。

每次新增或修改 UI 组件，至少检查：

1. 亮色/暗色、正常/悬停/按下/焦点/禁用状态的视觉输出；
2. 鼠标/键盘和相关平台触摸行为；
3. 不同窗口尺寸及高 DPI 下布局无裁切；
4. 无明显内存回归或不必要的持续重绘；
5. 官方参考与当前实现的截图或可复现对照记录。

如果某项测试无法运行，报告中必须明确写 **“未测试”**、具体原因及下一次验证所需的设备/环境；禁止虚构成功结果。


### 13.1 构建与验证产物的定期清理（强制）

**每次完成构建、调试、控件验证或视觉/性能回归后，都要检查本次产生的临时产物，并清理确认已过时、无须保留的文件。** 使用 Python 脚本优先完成盘点、过期判断和受控清理；不得仅为省事而无限积累截图、临时脚本和构建缓存。

清理范围包括但不限于：

- **过时的验证截图**：临时截图、同一控件/状态下被新验证替代的截图、调试裁切图、差异热力图、失败重试生成的重复图片。临时截图优先集中于 `tests/visual/tmp/` 或 `.artifacts/visual/`。
- **一次性验证脚本**：仅服务某次调试的临时 Python/Rust/Shell/PowerShell 脚本、废弃的实验脚本及不再被调用的辅助工具。应优先复用或更新 `tools/` 下的正式脚本，避免累积 `test_final2.py` 一类临时文件。
- **构建与测试产物**：不再需要的旧二进制、`target/` 中冗余构建输出、临时打包文件、覆盖率中间件、性能采样缓存、测试日志、生成的对照图以及过期报告。对于增量编译缓存优先按空间占用和必要性判断，**不要每次构建后无条件执行 `cargo clean`**。

执行规则：

1. **先盘点再删除**：优先用 Python `pathlib`、文件时间戳、引用关系和 `git status` 识别候选文件；核对文件是否仍被测试、脚本、文档或报告使用，不得仅凭文件名或创建时间武断判定过时。
2. **保留有效证据**：保留当前获批准的视觉基准图（golden/baseline）、可复现的性能基线、失败分析所需的最小复现文件、正式验证脚本，以及所有仍被引用的报告。新基准必须经验证后才可替换旧基准，不能把删除差异截图当作测试通过。
3. **限定删除范围**：只自动清理项目内明确归属的临时输出目录（例如 `.artifacts/`、`tests/visual/tmp/`）；对已纳入 Git 的文件、共享参考资料、正式脚本、基准图或不明确归属的文件，必须逐项核实用途，必要时先询问项目所有者。**严禁清理或修改 `references/microsoft-ui-xaml/`、`references/WinUI-Gallery/` 中的任何内容。**
4. **保护用户数据**：清理前检查 `git status --short`；禁止使用无范围的 `rm -rf`、`git clean -fdx`、`git reset --hard` 等可能误删源码或未提交工作的操作。任何批量清理须先提供 dry-run 清单，确认目标仍位于允许的目录内后再执行。
5. **自动化与忽略规则**：优先在 `tools/maintenance/` 编写可跨平台运行的 Python 清理脚本，默认 `--dry-run`，显式 `--apply` 才真正删除；为临时截图、日志、分析缓存和打包中间产物配置 `.gitignore`，但不得忽略应纳入版本控制的正式测试基准。
6. **清理后复查**：确认正式基准、源码和验证脚本仍完整；如清理影响测试发现或构建流程，应重新执行相关验证。交付报告应简述清理了哪些类型、所在目录及是否保留未清理项；不得夸大为已完成所有环境清理。

原则：**只删除确认过时且可重建的产物；保留可追溯的验证证据；清理过程可审计、可预览、范围受控。**

## 14. 依赖驱动的工作顺序与里程碑

**以下 P0–P7 是前置依赖与交付门槛，不是所有任务必须串行执行的时间表。** 同一阶段内互不依赖的分支可并行；一个控件的必要前置能力未完成时，不得因为其所属阶段已经开始就直接实现该控件。每阶段都执行第 13 节验证、第 13.1 节产物清理，而不是最后集中补测。

### P0 — 可运行性与参考基准（最先完成）

- 初始化 Cargo Workspace、Slint 1.19、Skia、Gallery；在 Windows 11 和 M3 macOS 26 启动。
- 按第 13 节规定的任务范围，检查 Windows 10 1809、macOS 15.0、iOS 16、Android 15 的目标构建和实际运行可行性；对未能测试的环境明确标记。
- 确认两个微软官方参考仓库的真实路径、Commit SHA，搭建 Python 分析与控件规格记录流程。
- 使用最小原型验证文本输入/IME、键盘/指针/触控、弹出层、Skia 材质和移动端生命周期的技术边界，不必提前实现正式外观。
- **通过门槛**：有最小可运行示例、兼容性风险清单和可追溯参考；不得声称尚未实测的旧版 OS 已兼容。

### P1 — 设计令牌与共享交互原语（所有正式控件的前置）

- 实现颜色、排版、字号、间距、描边、圆角、阴影、明暗/强调色主题及响应式基础规范。
- 建立 `Surface`/`Text`/`Icon`、状态样式、点击命中区域、焦点环、禁用/悬停/按压反馈；复用 Slint 既有输入与焦点能力。
- 建立 Motion 基础约定：状态过渡、动画时长、easing、动画减弱/关闭策略；不要把通用 Motion 拖到后期。
- 建立 Rust/Slint 共享状态与事件桥接，材质语义接口和可靠的纯色降级路径。
- **通过门槛**：基本状态、主题切换、键盘焦点与触控在 Gallery 原型中可验证，基础 API 稳定。

### P2 — 基础控件（无 Popup/复杂容器前置）

- 先完成 Button/ToggleButton、CheckBox、RadioButton/RadioGroup、Slider，以及 ProgressBar/ProgressRing；ToggleSwitch 依赖既有状态与交互原语。
- 在 TextInput/IME 验证通过后，实现 TextBox、PasswordBox，完整验证选中、光标、输入法、剪贴板和移动端输入。
- 每个控件同步完成 Normal/PointerOver/Pressed/Focused/Disabled 等适用状态及动画，不使用临时硬编码状态方案。
- **通过门槛**：基础控件有官方来源规格、Gallery 全状态演示、基础功能测试和适用平台的输入验证。

### P3 — 组合控件的公共基础设施（必须先于 P4）

- 建立 `SelectionModel`、`ItemPresenter`、列表模型适配、`ListView`/滚动封装；验证长列表只实例化可见项或具有等效策略。
- 建立 Popup/Overlay 能力：锚点/窗口位置、裁剪、层级、点击外部/Esc 关闭、焦点转移/恢复；优先复用 Slint `PopupWindow`，调查而不是假定其跨平台行为。
- 建立页面宿主、导航状态和窄屏/宽屏响应式基础；模态对话框所需的阻止背景输入、焦点约束等行为应先有可运行原型。
- **通过门槛**：独立 Gallery 验证页证明弹层、列表选择、滚动、焦点与页面切换可正常协作，再开发依赖它们的控件。

### P4 — 组合与导航控件（严格检查前置依赖）

- 在 Popup + 选择/列表完成后实现 ComboBox、Flyout、ContextMenu、Tooltip；不要先编写独立于共享 Popup 的专用弹出实现。
- 在 Overlay + Button + 焦点规则完成后实现 ContentDialog；在基础控件完成后实现 InfoBar。
- 在选择/列表 + 页面宿主 + 导航状态完成后实现 NavigationView，并验证桌面与移动端自适应模式。
- **通过门槛**：每个组合控件的依赖记录完整，键盘/触控、打开关闭、焦点恢复、响应式和异常路径均通过验证。

### P5 — 高级材质、动效打磨与平台一致性（可与 P2–P4 并行）

- 平台能力检测与跨平台 Mica/Acrylic、系统级/应用内模糊、降级策略；不要把所有控件锁死在原生 Mica 可用的前提上。
- 在 P1 Motion API 基础上完善导航、弹层、入场/退出、弹性动效；检查减少动画偏好和功耗。
- 使用 Python 进行参考截图与本项目 Gallery 截图对齐/差异分析，记录字体渲染、DPI、合成器差异。
- **通过门槛**：各平台可优雅降级，功能正确，性能/视觉数据可复现；昂贵材质和动效有独立性能基线。

### P6 — v0.1 跨平台与性能验收

- 在 Windows 10 1809+/Windows 11、macOS 15+、iOS 16+、Android 15+ 的目标环境完成实际运行测试；不可用的最低版本测试如实列出缺口。
- 进行帧时间、启动、内存、触控/键盘、IME、长期运行、暂停恢复、窗口与 DPI 压力测试。
- 验证打包、签名、许可证与文档，保留批准的截图/性能基线并清理过时产物。
- **通过门槛**：Gallery 演示和可复用首批基础/组合控件达到约定验收条件后发布 v0.1；未通过的功能不标记完成。

### P7 — 高级容器与扩展控件（不阻塞 v0.1）

- 在列表/选择/滚动/焦点/模型基础设施成熟后，再引入 TreeView、DataGrid/TreeDataGrid、AutoSuggestBox、CommandBar 等高级控件。
- 其中 DataGrid/TreeDataGrid 还必须先验证行列虚拟化、可编辑单元格、排序、列宽/滚动同步和大数据性能；不可用简单 `ScrollView` 全量创建海量单元格。
- 仅根据正式控件需求增加模块或 crate；不得为满足目录图而提前拆出大量空模块。

**依赖典型路径（非穷举）**：

```text
Tokens / Theme / Typography / Motion 基础
  ├─ Input / Focus / TextInput ──> TextBox / PasswordBox
  ├─ Surface / Button / Toggle ──> CheckBox / RadioButton / ToggleSwitch
  ├─ Shape / Motion ────────────> ProgressBar / ProgressRing
  ├─ SelectionModel + ListView ─> ListBox ───────────────┐
  ├─ Popup / Overlay + Focus ────────────────────────────┼─> ComboBox / Flyout
  ├─ Overlay + Button + Focus ──> ContentDialog          │
  └─ PageHost + NavigationState + Selection + Scroll ────> NavigationView

ListView + Selection + Hierarchy ──> TreeView
Virtualization + Rows/Columns + Selection + Editing ──> DataGrid
```

任何计划调整都应先更新依赖记录与相应回归测试。先完成 P0 的证据，再投入大量时间开发复杂控件或材质；遇到兼容性障碍不得未经确认擅自提高最低 OS 要求。

## 15. AI Agent 的执行规则

接到任务后，按如下顺序工作。**凡是涉及 WinUI 视觉复刻或源码研究，必须优先执行第 5 节的官方源码 Python 分析流程，绝不使用 Computer Use**：

1. **检查现状**：查看 `AGENTS.md`、`Cargo.toml`、工具链文件、现有组件和相关测试；若是视觉任务还要识别两个官方参考仓库的实际位置及 Commit；不要假定示意目录已创建。
2. **限定范围并检查依赖**：明确变更目标、受影响平台、需要验证的 UI 状态及兼容风险；按第 7.1–7.3 节列出前置能力与依赖状态，必要时先补齐共享基础设施。
3. **遵循现有约定**：优先复用已经实现的 Token、组件和抽象；避免重复发明接口。
4. **按依赖小步实现**：先验证上游模块，再形成最小可运行闭环；禁止为赶工复制临时 Popup、焦点、选择或滚动逻辑。
5. **实际验证**：优先执行格式检查、编译、单元测试，再按第 13 节的任务范围运行 Gallery、截图及性能测试。
6. **交付报告**：说明改了哪些文件、为何这样设计、运行了哪些命令、哪些平台已经测试、哪些尚未测试以及遗留风险。
7. **清理过时产物**：依照第 13.1 节，优先通过 Python 脚本盘点并清理已确认废弃的验证截图、一次性验证脚本和构建/测试输出；保留有效基准与可复现证据，说明清理结果。

**严禁**：
- 未经授权更换 Rust + Slint 技术栈；
- 引入社区 WinUI 仿制主题代替自研控件；
- 用 C++ 编写主要 UI/业务功能（底层依赖允许）；
- 使用 WebView/Electron 等浏览器 UI；
- 私自扩大平台范围到 Linux 桌面或 Windows 7；
- 私自提高上述最低系统版本；
- 在没有测试证据时声称视觉 1:1、性能达标或四平台全部兼容；
- 用虚构的 benchmark、伪造截图、跳过失败测试来宣称完成。
- **使用 Computer Use 查看、浏览或操作参考仓库、项目源码，或捕获 Gallery 参考画面。**
- **在可使用 Python 脚本分析源码或截图时，优先用不透明的点击操作替代可复现的分析。**
- 把官方仓库作为可修改的项目目录，或将整个参考仓库提交到本项目。

## 16. 维护与参考资料

- **微软 WinUI 官方源码（一级参考）**：<https://github.com/microsoft/microsoft-ui-xaml>
- **微软 WinUI Gallery 官方仓库（一级参考）**：<https://github.com/microsoft/WinUI-Gallery>
- Slint 官方文档：<https://docs.slint.dev/latest/>
- Slint Rust API：<https://docs.slint.dev/latest/docs/rust/slint/>
- Slint Focus / FocusScope：<https://docs.slint.dev/latest/docs/slint/guide/development/focus/>、<https://docs.slint.dev/latest/docs/slint/reference/keyboard-input/focusscope/>
- Slint PopupWindow：<https://docs.slint.dev/latest/docs/slint/reference/window/popupwindow/>
- Slint ListView / ScrollView：<https://docs.slint.dev/latest/docs/slint/reference/std-widgets/views/listview/>、<https://docs.slint.dev/latest/docs/slint/reference/std-widgets/views/scrollview/>
- Slint iOS：<https://docs.slint.dev/latest/docs/slint/guide/platforms/mobile/ios/>
- Slint Android：<https://docs.slint.dev/latest/docs/slint/guide/platforms/mobile/android/>
- Windows 设计规范：<https://learn.microsoft.com/en-us/windows/apps/design/>
- WinUI 3 设计与控件参考：<https://learn.microsoft.com/en-us/windows/apps/winui/winui3/>

更新本文件中的 OS 基线、语言、UI 框架、渲染策略、官方参考仓库优先级或 Computer Use 禁令，需要先经项目所有者明确同意，并同步修改构建配置、测试矩阵与发布文档。
