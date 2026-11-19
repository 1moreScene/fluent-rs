# NavigationView 自定义内容

Gallery 的 **NavigationView → Custom pane content** 展示实际的 Slint 子组件。侧栏和顶部的示例使用同一个通知计数作为状态。

| 内嵌左栏 | 顶部 | 位置 |
| --- | --- | --- |
| `pane-header` | `top-header` | 左栏标题行，或顶部主项之前 |
| `pane-custom-content` | `top-custom-content` | 左栏菜单上方，或顶部靠近尾部 |
| `pane-footer` | `top-footer` | 尾部菜单项之前 |

宿主用 `slot-name << Component { … }` 注入任意 Slint 子组件。布局使用内容的首选尺寸。空插槽归还空间，顶部在计算溢出时预留自定义内容宽度。两种位置由独立的实例承载，需保留的状态应绑定到宿主。

Gallery 的项目文件显式启用 Slint 1.19 的实验性 `named-slots`。该版本在编译阶段拒绝 PopupWindow 中的命名插槽。本轮任意组件插槽已接入内嵌左栏和顶部。Compact、Minimal 打开后的原生 PopupWindow 抽屉仍使用标准内容。
