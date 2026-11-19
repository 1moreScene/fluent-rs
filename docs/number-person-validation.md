# NumberBox / PersonPicture 本机验证

日期：2026-11-19。设备：Apple Silicon ARM64，macOS 26.4.1（25E253），Rust 1.100.0，Slint 1.19.0 / Skia。仅验证本机，没有执行其他目标构建。

## 参考与实现

先只读提取本地官方 WinUI `main` / `07e0354d2e06572501ef48078a3cc91138749c9f` 与 Gallery `main` / `6de91feccadfbb0a796b71caedead5a2284f730e`；规格位于 `design/specifications/{numberbox,personpicture}.md`，提取入口为 `tools/reference_analysis/analyze_number_person.py`。

NumberBox 使用共享 TextBox、RepeatButton、OverlayHost 和主题。新增无外部依赖的 fluent-core 数值与首字母模块。数值为 f64，经过规范字符串桥接 Slint，显示的十位有效数字舍入不改写 Value。修复 Slint 延迟属性通知导致范围/数值同时赋值时显示旧文本的问题。Compact 使用已有可编辑 ComboBox 的弹层焦点处理：打开后保留编辑器，Esc 仅恢复文本，Tab 提交并关闭。

共享 TextBox 将编辑角色集中到组件根节点，内部 TextInput 保留原生编辑引擎而不再重复暴露；NumberBox 由外层 Spinbox 提供唯一编辑角色。清除按钮列宽/偏移、滚轮回调及 RepeatButton 关闭默认表面的接口均保持既有默认行为。

PersonPicture 由 Slint 原生图像 cover、圆形裁剪、主题和文本组成。默认图标使用同版本 Microsoft Fluent System Icons 的 MIT 资产，不捆绑 Segoe 字体；测试照片为自制非方形 SVG。

## 已执行

- fluent-core 的 6 项测试通过：表达式优先级/错误、显示与实际精度、无效/空值/步进、范围/循环、名字分词与括号、Unicode 回落。
- 新控件原生专项在显示缩放 2× 和 1.25× 通过；每次覆盖 UI 125% / 100%、亮/暗/高对比、980×700 与 320×640、触控目标布局。
- NumberBox：编辑时不提前提交、Enter 松开提交、Esc 恢复、无效文本策略、空值 NaN、单次变化事件、Min/Max 联动、Wrap、f64 精度、小数分隔符、方向键/Page 键、聚焦全选、仅聚焦时滚轮、清除按钮位置、Inline/Compact 定位和焦点、连发达上界停用及切页取消。
- PersonPicture：Group/Photo/Initials/Person 优先级、显式首字母、括号/组合音标/非拉丁姓名、徽章图片/数字/字形优先级、99+ 与真实计数、负数、尺寸和徽章比例/偏移、矩形图像居中 cover 与圆形角落像素、跳过 Tab 和无点击状态。

完整 Gallery 回归在候选二进制上通过（2× 显示缩放、宽窄窗口），包括共享 TextBox / PasswordBox / ComboBox / 弹层、所有控件、主题及材质。固定源码工作区测试 28 项通过（fluent-core 6、Gallery 19、materials 3）。最终二进制在 2× 与 1.25× 的原生专项均再次通过，包含新增的外部点击提交/关闭/失焦断言。固定源码的 `cargo fmt --all -- --check`、`cargo check --locked --workspace`、`cargo clippy --locked --workspace --all-targets -- -D warnings` 全部通过，无警告。`cargo test --locked --workspace` 与 Gallery debug 构建通过。

## 验证边界

没有 Windows 官方 Gallery 运行截图，尚未完成逐像素跨平台验收；没有测试 Windows、Intel macOS、macOS 15、iOS、Android、真实触摸、真实 IME 或屏幕阅读器。事件注入和 Skia 像素检查不替代这些验收。

NumberBox 外部首击用于关闭/提交而不透传；共享 Slint 阴影与原生 ThemeShadow 的像素对应待对照。默认十进制格式支持 . / , 与最低小数位；完整区域/货币格式由宿主 NumberFormatter 扩展。Compact 受共享根窗口弹层约束，边缘限位；WinUI 模板允许弹层超出根窗口。PersonPicture 不接入 Windows 联系人服务，补充平面名字安全回落通用头像，默认人形向量轮廓与 Segoe 原字形仍需对照。

没有进行 FPS、Private Bytes 或最低配置性能采样；PersonPicture 无计时器，NumberBox 复用 RepeatButton 的按需计时器，取消与达界停止已测。

## 产物

交付入口统一为 `target/debug/fluent-gallery`。并行会话仍修改工作区，因此最终验证保留 `.artifacts/number-person-build-20261119/source`、SHA-256 清单和官方参考提取；后续并行改动不属于这份产物的验收范围，根目录源码未被快照覆盖。

最终 SHA-256：`c421e10a3003b850b1745daccae508c9ff3fa6b878575a8e8fe8542943fa3d0e`。记录：该目录内 `source-manifest.json`、`final-checks.json`、`native-validation.json`、`official-reference.json`。

清理：删除本轮重复参考 JSON、单独核心 lint 缓存、旧候选二进制和安装后重复二进制；保留约 1.8 MB 的固定源码及有效证据。未清理共享 target 缓存、其他会话产物或官方参考仓库。
