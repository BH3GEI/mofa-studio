# Editor 子模块

使用大语言模型将 Personal News JSON 转成新闻联播式播报稿。

## 功能
- 接收已聚合的 JSON 数据（包含 inputs / config）。
- 生成结构化、可直接播报的纯文本稿件。
- 自动压缩输入（每源最多 3 条，字段截断）以避免上下文超限。
- Prompt 存放在 `personal-news/editor/prompt.txt`，可独立修改。

## 环境变量
- `LLM_API_KEY`：LLM 服务 API Key。
- `LLM_API_BASE`：可选，默认 `https://api.openai.com`。
- `LLM_MODEL`：可选，默认 `gpt-4o-mini`。

可写入 `personal-news/env.secret`。

## 配置项
- `llm_temperature`：可选，默认 0.2。

## CLI 使用

```sh
python personal-news/editor/client.py personal-news/temp.json
```

输入示例：

```json
{
  "config": {
    "mode": "morning",
    "language": "zh-CN"
  },
  "inputs": [
    {"source": "rss", "items": []},
    {"source": "x", "items": []},
    {"source": "weather", "items": []}
  ]
}
```
