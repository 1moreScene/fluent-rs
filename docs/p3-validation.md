# P3 列表与弹层基础验证

2026-11-18，MacBook Air Apple M3，macOS 26.4.1（25E253），Rust 1.100.0，Slint 1.19.0，Skia 调试版。

## 复现

```sh
python3 -B tools/reference_analysis/analyze_composition.py
cargo fmt --all -- --check
cargo check --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo run --locked -p fluent-gallery --bin fluent-gallery -- --smoke-test
SLINT_SCALE_FACTOR=1.25 cargo run --locked -p fluent-gallery --bin fluent-gallery -- --smoke-test
```

两份官方仓库已浅克隆到 `references/`，工作树干净。Python 脚本按提交提取来源路径、关键词和行号。 [官方参考及依赖](../design/specifications/composition-foundations.md)

## 本轮结果

格式、编译、Clippy 和 4 项 Rust 单元测试通过。Gallery 冒烟使用 Slint 分发鼠标与键盘事件，两种缩放均覆盖 720×620 与 320×640 逻辑像素。200% 返回 1440×1240 与 640×1280 的 Skia 像素缓冲。125% 返回 900×776 与 400×800 的缓冲。

| 项目 | 验证证据 |
| --- | --- |
| 选择 | 点按单选、禁用条目忽略点按、方向键跳过禁用项、滚轮滚动保留选择、多选模式下独立移动焦点和 Space 切换。替换为空模型后选择与滚动复位。 |
| 虚拟滚动 | 惰性模型有 100,000 行。宽窗冒烟共调用 53 次行数据读取，包含重复读取。Home 和 End 确认首尾定位以及可见范围更新。 |
| 锚定弹层 | 点击外部与 Esc 关闭并恢复触发器焦点。模拟窗口尺寸变化、离开页面时自动关闭，回到页面保留选择。窄窗在触发器上方放置弹层。 |
| 模态 | 点按遮罩后弹层保持打开。Tab 在弹层内循环，Esc 和取消恢复触发器焦点。确认清除后焦点转到仍可用的详情操作。 |
| 视觉 | Skia 采样确认列表选中色覆盖亮色、暗色和高对比主题，锚定弹层在窄窗绘制亮暗两种面板底色。窄窗列表行高达到 44 个逻辑像素。 |
| 回归 | 现有 Button、TextBox、ToggleButton、CheckBox、RadioButton、Slider、进度控件和材质参加同一运行。 |

## 未测试

- 官方视觉对照需要同缩放的 Windows Gallery 截图和输入录屏。真实触屏、VoiceOver、窗口跨屏切换、移动安全区变化及后台恢复需要相应设备和会话。
- 内存回归、帧时间、功耗和重绘频率需要同机器、同场景的对照采样。
- macOS 15、Intel Mac、Windows、iOS、Android 未运行本轮改动。需要各自的系统与设备。

## 产物

参考分析与像素检查均在内存中完成。盘点 `.artifacts/`、`tests/visual/tmp/` 和 Python 缓存，本轮没有待清理的临时产物。保留官方参考仓库、当前构建结果和有效的增量缓存。
