# WebView App 创建指南

本文档记录创建基于 WebView 的内建应用的完整流程，以 `mofa-converter` 为例。

## 架构概述

WebView 应用采用 Rust + Python 混合架构：
- **Rust 部分**：提供 WebView 容器，管理 Python 进程生命周期
- **Python 部分**：HTTP 服务器，提供 API 和静态文件服务
- **前端部分**：HTML/CSS/JS，运行在 WebView 中

## 文件结构

```
apps/mofa-converter/
├── Cargo.toml              # Rust crate 配置
├── src/
│   ├── lib.rs             # 应用入口，实现 MofaApp trait
│   └── screen/
│       └── mod.rs         # WebView 容器实现
└── python/web/
    ├── app.py             # Python HTTP 服务器
    └── static/
        └── index.html     # 前端页面
```

## 步骤详解

### 1. 创建 Crate

```bash
mkdir -p apps/mofa-converter/src/screen
mkdir -p apps/mofa-converter/python/web/static
```

**Cargo.toml**:
```toml
[package]
name = "mofa-converter"
version = "0.1.0"
edition = "2021"
description = "A simple content converter"

[dependencies]
makepad-widgets = { workspace = true }
mofa-widgets = { path = "../../mofa-widgets" }
log = "0.4"
serde_json = "1.0"
dirs = "5.0"
```

### 2. 实现 MofaApp (src/lib.rs)

```rust
pub mod screen;

use makepad_widgets::*;
use mofa_widgets::{AppInfo, MofaApp};

pub struct MoFaConverterApp;

impl MofaApp for MoFaConverterApp {
    fn info() -> AppInfo {
        AppInfo {
            name: "Converter",
            id: "mofa-converter",
            description: "Convert between formats",
            tab_id: Some(live_id!(converter_tab)),
            page_id: Some(live_id!(converter_page)),
            show_in_sidebar: true,
            ..Default::default()
        }
    }

    fn live_design(cx: &mut Cx) {
        screen::live_design(cx);
    }
}
```

### 3. WebView 容器 (src/screen/mod.rs)

核心要点：
- 使用 `live_design!` 定义 UI
- 使用 `TcpListener` 找可用端口
- 使用 `std::process::Command` 启动 Python
- 实现 `WebViewAction` 事件处理
- 提供 `start_server` / `stop_server` 方法
- 提供 `update_dark_mode` 方法

参考实现：复制 `mofa-hello-world/src/screen/mod.rs` 并修改：
- 结构体重命名为 `ConverterScreen`
- 日志标签改为 `[Converter]`
- 路径查找逻辑改为 `apps/mofa-converter/python/web`

### 4. Python 后端 (python/web/app.py)

最小实现：
```python
from http.server import HTTPServer, SimpleHTTPRequestHandler
import json

class Handler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        import pathlib
        self.directory = str(pathlib.Path(__file__).parent / "static")
        super().__init__(*args, directory=self.directory, **kwargs)

    def do_GET(self):
        if self.path == "/api/info":
            self._json_response({"name": "Converter", "version": "1.0.0"})
        else:
            super().do_GET()

    def _json_response(self, data, status=200):
        body = json.dumps(data).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(body)

def run_server(port=8080):
    server = HTTPServer(("127.0.0.1", port), Handler)
    print(f"Server running on http://127.0.0.1:{port}")
    server.serve_forever()

if __name__ == "__main__":
    import sys
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8080
    run_server(port)
```

### 5. Shell 集成

#### 5.1 Cargo.toml 添加特性

```toml
[features]
default = ["...", "mofa-converter"]
mofa-converter = ["dep:mofa-converter"]

[dependencies]
mofa-converter = { path = "../apps/mofa-converter", optional = true }
```

#### 5.2 app.rs 导入和注册

导入：
```rust
use mofa_converter::MoFaConverterApp;
use mofa_converter::screen::ConverterScreenWidgetRefExt;
```

`after_new_from_doc` 中注册：
```rust
self.app_registry.register(MoFaConverterApp::info());
```

`live_register` 中注册：
```rust
<MoFaConverterApp as MofaApp>::live_design(cx);
```

#### 5.3 dashboard.rs 添加页面

导入 screen：
```rust
use mofa_converter::screen::ConverterScreen;
```

在 `content` 中添加：
```rust
converter_page = <ConverterScreen> {
    width: Fill, height: Fill
    visible: false
}
```

#### 5.4 sidebar.rs 添加导航

添加 `SidebarSelection` 枚举项：
```rust
pub enum SidebarSelection {
    // ...
    Converter,
    // ...
}
```

添加按钮（在 `live_design!` 中）：
```rust
converter_tab = <SidebarMenuButton> {
    text: "内容转换器"
    draw_icon: {
        svg_file: dep("crate://self/resources/icons/start.svg")
    }
}
```

添加点击处理（在 `handle_event` 中）：
```rust
if self.view.button(ids!(main_content.converter_tab)).clicked(actions) {
    self.handle_selection(cx, SidebarSelection::Converter);
}
```

添加选择处理（在 `handle_selection` 中）：
```rust
SidebarSelection::Converter => {
    self.view.button(ids!(main_content.converter_tab))
        .apply_over(cx, live! { draw_bg: { selected: 1.0 } });
    self.pinned_app_name = None;
    // ...
}
```

更新 `clear_all_selections` 宏调用，添加 `converter_tab`。

#### 5.5 app.rs 页面生命周期

停用（切换出页面时）：
```rust
if old_page == Some(PageId::Converter) {
    self.ui.web_view_container(ids!(...converter_page...))
        .set_active(cx, false);
}
```

激活（切换入页面时）：
```rust
if page == PageId::Converter {
    self.ui.web_view_container(ids!(...converter_page...))
        .set_active(cx, true);
    self.ui.converter_screen(ids!(...converter_page...))
        .start_server(cx);
}
```

可见性（在 `update_page_visibility` 中）：
```rust
self.ui.view(ids!(...converter_page...))
    .apply_over(cx, live!{ visible: (current == Some(PageId::Converter)) });
```

标题（在 `update_hero_title` 中）：
```rust
PageId::Converter => ("内容转换器", "在音频、视频与文稿之间自由转换"),
```

暗黑模式（在 `apply_dark_mode_screens_with_value` 中）：
```rust
self.ui.converter_screen(ids!(...converter_page...))
    .update_dark_mode(cx, dm);
```

#### 5.6 mofa-widgets 添加 PageId

在 `app_trait.rs` 中：
1. `PageId` 枚举添加 `Converter`
2. `tab_live_id()` 添加 `PageId::Converter => live_id!(converter_tab)`
3. `page_live_id()` 添加 `PageId::Converter => live_id!(converter_page)`
4. `PageRouter::new()` 的 pages 向量添加 `PageId::Converter`

## 测试检查清单

- [ ] `cargo build` 成功编译
- [ ] 侧边栏显示应用按钮
- [ ] 点击按钮切换到应用页面
- [ ] "Start Server" 启动 Python 服务
- [ ] WebView 加载前端页面
- [ ] 暗黑模式切换正常工作
- [ ] 切换页面时服务器正常启停

## 常见问题

**Q: Python 文件找不到？**
A: 检查 `get_python_path()` 中的路径是否正确，开发环境和打包环境路径不同。

**Q: WebView 不加载？**
A: 确保 `start_server` 返回后等待足够时间（如 1.5 秒）再调用 `load_url`。

**Q: 侧边栏按钮没反应？**
A: 检查 `clear_all_selections` 是否包含了新按钮，以及 `handle_selection` 是否正确设置选中状态。
