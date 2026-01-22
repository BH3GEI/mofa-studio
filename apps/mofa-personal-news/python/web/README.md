# 辅助网站

此目录提供本地辅助网站，用于文档、快速运行与测试用例。

## 运行

在仓库根目录执行：

```sh
python personal-news/web/app.py
```

然后访问：

```
http://127.0.0.1:5173
```

## 页面
- `/docs.html`：渲染 README.md，支持 Markdown。
- `/run.html`：编辑 JSON config + inputs 并生成脚本。
- `/tests.html`：编辑并运行测试用例，支持断言。
- `/about.html`：可复用的建站 Prompt。

文档页列出当前启用的子模块 README 路径，便于快速定位。

## 说明
- 服务端仅使用 Python 标准库。
- 不需要任何网络访问。
