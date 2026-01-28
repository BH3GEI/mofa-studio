# MoFA Studio 插件接入指南（中文）

本指南说明两类接入方式：
- 内建 Rust App（需重编译）：可做原生 Makepad UI，亦可走 WebView + Rust HTTP。
- 外挂 WebView（无需重编译）：后端为 Python HTTP，前端为 HTML/JS（可由 React/Vue 构建后落地）。

## 模式总览

**App 与插件**
- 原生 App（编译内建）：Rust + Makepad UI
- WebView App（编译内建）：Rust HTTP + WebView UI
- WebView 插件（动态加载）：Python HTTP + WebView UI

**前端框架（非独立模式）**
- React/Vue 等仅是 WebView 内部的前端实现方式，可用于插件或内建 App。

## 一、内建 Rust App（需重编译）

### 1. 创建 app crate
在 `apps/` 下创建新 crate；workspace 已包含 `apps/*`，无需修改根 `Cargo.toml`（/Users/yao/Desktop/code/work/mofa-org/mofalaya/mofa-studio/Cargo.toml:1）。

依赖最小化配置示例：
```toml
[dependencies]
makepad-widgets = { workspace = true }
mofa-widgets = { path = "../../mofa-widgets" }
```

### 2. 实现 MofaApp
`MofaApp` 与 `AppInfo` 定义见 `/Users/yao/Desktop/code/work/mofa-org/mofalaya/mofa-studio/mofa-widgets/src/app_trait.rs:64`。

示例：
```rust
use makepad_widgets::Cx;
use mofa_widgets::{MofaApp, AppInfo};

pub struct MoFaMyApp;

impl MofaApp for MoFaMyApp {
    fn info() -> AppInfo {
        AppInfo {
            name: "My App",
            id: "mofa-myapp",
            description: "My custom MoFA app",
            ..Default::default()
        }
    }

    fn live_design(cx: &mut Cx) {
        crate::screen::live_design(cx);
    }
}
```

### 3. 创建 screen widget
用 `live_design!` 定义主界面 widget，并实现 `Widget`。

### 3.1 WebView + Rust HTTP Server 模式（无 Python）
若欲沿用“WebView + 本地 HTTP”之架构而去 Python，可在内建 Rust App 中自启 Rust HTTP Server，并由 WebView 加载：
- 监听 `127.0.0.1:0` 取空闲端口；
- 后台线程处理 `/api/*` 与静态文件；
- `WebViewContainer` 加载 `http://127.0.0.1:{port}/`；
- 页面切换时启停服务。

参考实现：`apps/mofa-hello-world-rust/`。

若用 React/Vue/Vite 等，先构建产物，再由 Rust 服务静态目录，并为前端路由加 `index.html` 回落。

### 4. 接入壳层（mofa-studio-shell）

1) 添加依赖与 feature：
- `mofa-studio-shell/Cargo.toml` 中新增 optional 依赖与 feature（/Users/yao/Desktop/code/work/mofa-org/mofalaya/mofa-studio/mofa-studio-shell/Cargo.toml:6）。

2) 在壳层注册应用：
- `LiveHook::after_new_from_doc` 注册 `AppInfo`（/Users/yao/Desktop/code/work/mofa-org/mofalaya/mofa-studio/mofa-studio-shell/src/app.rs:412）。
- `LiveRegister::live_register` 注册 `MofaApp::live_design`（/Users/yao/Desktop/code/work/mofa-org/mofalaya/mofa-studio/mofa-studio-shell/src/app.rs:492）。
- `live_design!` 中引入 screen 类型，满足 Makepad 编译期要求。

3) 路由与页面：
- 在 `PageId` 增加新页面，并补充 `tab_live_id` / `page_live_id` / `PageRouter::new`（/Users/yao/Desktop/code/work/mofa-org/mofalaya/mofa-studio/mofa-widgets/src/app_trait.rs:94）。
- 在 Dashboard 增加对应页面节点（/Users/yao/Desktop/code/work/mofa-org/mofalaya/mofa-studio/mofa-studio-shell/src/widgets/dashboard.rs:333）。
- 在 Sidebar 增加按钮与选择处理（/Users/yao/Desktop/code/work/mofa-org/mofalaya/mofa-studio/mofa-studio-shell/src/widgets/sidebar.rs:375）。

4) 定时器与资源管理（可选）：
若页面含轮询/动画，按需实现计时器启停，并在页面切换时调用。

### 5. 编译与运行
完成上述改动后，重编译应用。

## 二、外挂 WebView 插件（无需重编译）

### 1. 目录结构
插件目录位于 `~/.mofa-studio/plugins/`。示例结构：
```
~/.mofa-studio/plugins/
  my-plugin/
    manifest.json
    python/
      app.py
    static/
      index.html
```

若前端用 React/Vue/Vite，请将构建产物拷入 `static/`，并确保资源路径为相对路径（如 Vite 设 `base: './'`）。

### 2. manifest.json
字段定义见 `/Users/yao/Desktop/code/work/mofa-org/mofalaya/mofa-studio/mofa-widgets/src/plugins/manifest.rs:22`。

必填：`id`、`name`、`version`。建议指定 `type: "webview"`。
`python_entry` 默认 `python/app.py`，`static_dir` 默认 `static`。

示例：
```json
{
  "id": "my-plugin",
  "name": "My Plugin",
  "version": "1.0.0",
  "description": "A short description",
  "author": "Your Team",
  "type": "webview",
  "python_entry": "python/app.py",
  "static_dir": "static",
  "show_in_sidebar": true
}
```

### 3. Python 后端（HTTP Server）
要求：
- 端口从 argv 读取；
- 只绑定 `127.0.0.1`；
- 提供静态文件与 API。

最小示例：
```python
#!/usr/bin/env python3
import json
import sys
from http.server import HTTPServer, SimpleHTTPRequestHandler
from pathlib import Path

class Handler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        static_dir = Path(__file__).parent.parent / "static"
        super().__init__(*args, directory=str(static_dir), **kwargs)

    def do_GET(self):
        if self.path == "/api/info":
            body = json.dumps({"status": "ok"}).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(body)
        else:
            super().do_GET()

if __name__ == "__main__":
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8080
    HTTPServer(("127.0.0.1", port), Handler).serve_forever()
```

### 4. 前端页面（index.html）
要求：
- API 走相对路径（如 `/api/info`）；
- 实现 `window.setTheme(darkMode)`；
- `darkMode` 取值 0.0~1.0。

### 5. 运行机制
- 启动时扫描插件目录（/Users/yao/Desktop/code/work/mofa-org/mofalaya/mofa-studio/mofa-widgets/src/plugins/loader.rs:136）。
- `show_in_sidebar: true` 时显示在侧栏。
- 侧栏点击后启动 Python 服务，并加载 `http://127.0.0.1:{port}/`（/Users/yao/Desktop/code/work/mofa-org/mofalaya/mofa-studio/mofa-widgets/src/plugins/screen.rs:312）。

### 6. 常见问题
- 未显示：检查 `manifest.json` 语法与 `show_in_sidebar`。
- 无法启动：确认 `python_entry` 路径存在。
- 前端无数据：检查 API 路径与返回格式。

### 7. 说明（Rust 后端）
外挂 WebView 插件目前仅支持 `python_entry`。如需 Rust 后端，请用内建 Rust App 方案（见上节 3.1）。

## 选型建议
- 需要深度壳层整合或原生 UI：选内建 Rust App。
- 需要快速交付与前后端分离：选外挂 WebView 插件（Python 后端）。
