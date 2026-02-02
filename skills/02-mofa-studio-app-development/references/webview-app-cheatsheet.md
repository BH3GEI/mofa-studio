# WebView App 创建速查表

## 快速创建新应用（假设应用名为 `mofa-myapp`）

### 1. 创建目录结构
```bash
mkdir -p apps/mofa-myapp/src/screen
mkdir -p apps/mofa-myapp/python/web/static
touch apps/mofa-myapp/python/web/static/index.html
```

### 2. 复制并修改文件

从 `mofa-hello-world` 复制并全局替换：
- `HelloWorld` → `MyApp`
- `hello_world` → `myapp`
- `hello-world` → `myapp`

需要修改的文件：
- `Cargo.toml`
- `src/lib.rs`
- `src/screen/mod.rs`（注意日志标签和路径）
- `python/web/app.py`
- `python/web/static/index.html`

### 3. Shell 集成（6个文件）

| 文件 | 修改内容 |
|------|----------|
| `mofa-studio-shell/Cargo.toml` | 添加 feature 和 dependency |
| `mofa-studio-shell/src/app.rs` | import, `after_new_from_doc`, `live_register` |
| `mofa-studio-shell/src/widgets/dashboard.rs` | import screen, 添加 page |
| `mofa-studio-shell/src/widgets/sidebar.rs` | 添加 tab 按钮、selection 枚举、点击处理、选中样式 |
| `mofa-widgets/src/app_trait.rs` | 添加 `PageId`, `tab_live_id()`, `page_live_id()`, `PageRouter::new()` |

### 4. app.rs 关键修改点

导入：
```rust
use mofa_myapp::MoFaMyApp;
use mofa_myapp::screen::MyAppScreenWidgetRefExt;
```

生命周期（复制 WebViewPlaceholder 的块，替换名称）：
- 停用 WebView（old_page）
- 激活 WebView + start_server（page）
- 可见性（update_page_visibility）
- 标题（update_hero_title）
- 暗黑模式（apply_dark_mode_screens_with_value）

### 5. sidebar.rs 关键修改点

- `SidebarSelection` 枚举添加变体
- `live_design!` 添加 `myapp_tab` 按钮
- `handle_event` 添加点击检测
- `handle_selection` 添加选中处理（复制 WebViewPlaceholder 块）
- `restore_selection_state` 添加恢复逻辑
- `clear_all_selections` 宏调用添加按钮路径

### 6. 编译测试

```bash
cargo build --release 2>&1 | head -50
```

常见错误：
- `live_id!` 未定义 → 检查是否添加到了 `PageRouter`
- 路径找不到 → 检查 `get_python_path()` 中的路径
- Widget 未找到 → 检查 dashboard.rs 中的 import

## 文件清单（共需修改/创建 11 个文件）

应用内部（5个）：
```
apps/mofa-myapp/Cargo.toml
apps/mofa-myapp/src/lib.rs
apps/mofa-myapp/src/screen/mod.rs
apps/mofa-myapp/python/web/app.py
apps/mofa-myapp/python/web/static/index.html
```

Shell 集成（6个）：
```
mofa-studio-shell/Cargo.toml
mofa-studio-shell/src/app.rs
mofa-studio-shell/src/widgets/dashboard.rs
mofa-studio-shell/src/widgets/sidebar.rs
mofa-widgets/src/app_trait.rs
```
