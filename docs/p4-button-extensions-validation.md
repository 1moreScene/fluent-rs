# RepeatButton 与 HyperlinkButton 本机验证

2026-11-19，Mac / ARM64 / macOS 26.4.1，Rust 1.100.0、Slint 1.19.0、Skia、Debug。本批只实现 RepeatButton 与 HyperlinkButton，前者补齐后续 NumberBox 的按住步进前置。规格分别见 [RepeatButton](../design/specifications/repeatbutton.md)、[HyperlinkButton](../design/specifications/hyperlinkbutton.md)。

## 可追溯输入与构建

官方源码与 Gallery 的路径、分支、SHA、资源键和行号均列于规格。`tools/reference_analysis/analyze_repeat_hyperlink.py` 默认只读分析，支持两个仓库的路径参数或环境变量。当前提取结果保存在 `.artifacts/repeat-hyperlink-build-20261119/official-reference.json`。

本轮存在并行开发。保留完整工作区改动，在 `.artifacts/repeat-hyperlink-build-20261119/` 固定本批验证输入，`source-manifest.json` 记录源码、配置及资产的 SHA-256。冻结时两个并行新页面 DropDownButton / ToggleSplitButton 存在编译问题，仅从私有 Gallery 的导入和路由中排除，原工作区页面保持不变。保留当时已可编译的共享 ButtonSurface / SplitButton / ContextMenu / MenuPresenter 更新，完整回归覆盖它们的既有消费者。后续其他会话的更新不属于该快照，边界见 `integration-boundary.json`。

从项目根目录可重现本轮构建：

```sh
cargo build --locked --manifest-path .artifacts/repeat-hyperlink-build-20261119/Cargo.toml --target-dir target -p fluent-gallery --bin fluent-gallery
SLINT_SCALE_FACTOR=2 target/debug/fluent-gallery --smoke-test
SLINT_SCALE_FACTOR=1.25 target/debug/fluent-gallery --smoke-test repeat-hyperlink
```

交付的 `target/debug/fluent-gallery` SHA-256 为 `e8d4d0d9935da621e2f5fe7bf71906911fa026a7410579ea24f1e42c4f7f01bc`，对应检查结果在快照的 `verification.json` 中。

## 结果

在上述固定源码上执行 `cargo fmt --all -- --check`、locked workspace check、clippy all-targets（`-D warnings`）、workspace test 和 Gallery build，均通过。Rust 单元测试共 21 项通过。

最终可执行文件在 backing scale 2 完整 Gallery 回归通过，宽窗 1960×1400 backing、320×640 逻辑像素窄窗为 640×1280 backing。覆盖共享 Button、文本框、密码框、选择器、滚动列表、弹层、ContentDialog、InfoBar、TreeView、CommandBar、Expander、SplitButton、NavigationView 及主题/材质。

`--smoke-test repeat-hyperlink` 在 backing scale 2 和 1.25 验证了本批控件；专项每次覆盖默认 125% 与 100% UI，再验证 320px 窄窗。最终 1.25 运行的宽/窄输出为 1226×876、400×800 backing。

- RepeatButton：按下立即执行；释放无额外 Click；Space 使用内部重复、OS 自动重复不叠加；Enter 只执行一次；其他键取消键盘按住；Release / Hover 模式；指针移出暂停、重新进入延迟恢复；失焦、禁用、Click 内禁用和页面离开停止。非法时序不启动定时器。亮暗及高对比、焦点来源、正常/悬停/按下/禁用像素和触控尺寸均通过。
- 连发直接记录实际回调的单调时钟时间，不在按住后的每一步插入同步渲染。完整回归中默认 500ms 的首次回调为 501.11ms，180ms 的恢复和键盘首次回调为 183.20ms、183.63ms；1.25 专项分别为 502.20ms、181.78ms、183.51ms。停止后等待确认计数稳定且 Timer 不运行。这些是功能时序记录，不是帧时间性能结论。
- HyperlinkButton：鼠标、Space、Enter 与 Tab；拖出取消和禁用；先 Click 后读取 URI，Click 可清空 URI；无 URI 链接进入 ToggleButton 页；宿主错误显示；亮暗及高对比的字色、状态和下划线规则；实际字色像素、长标签换行、紧凑窗口均通过。自动化使用 URI 记录器，不打开浏览器。

## 验证边界与清理

Windows 官方运行截图对照、最低系统版本、真实触摸、屏幕阅读器、Mac 默认浏览器实际唤起均**未测试**。前几项需要对应设备/环境；浏览器唤起需手动点击 URI 示例验收。Mac 的强调文字色阶采用规格中明确记录的官方测试覆盖色阶，Windows UISettings 运行时色阶尚待宿主接入。Slint 缺少 Hyperlink 辅助功能角色，当前沿用 Button 语义。帧时间与长期内存基准**未测试**，仅验证空闲没有功能连发定时器。

本轮像素检查直接读取 Skia 内存缓冲，未生成临时截图。盘点后仅清理本轮 `.artifacts/repeat-hyperlink-build-20261119/` 下被最终产物替代的两份临时可执行文件，合计约 305 MiB；保留源码快照、官方提取、边界说明和验证记录。共享构建缓存、其他会话产物、有效基准和两个官方仓库均保留。
