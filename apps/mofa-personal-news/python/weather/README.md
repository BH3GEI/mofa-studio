# Weather 子模块

用于获取天气信息并输出标准化的 `weather` source items。

## 功能
- 调用 wttr.in 当前天气接口（无需 API Key）。
- 输出摘要 `summary` 供主程序播报使用。

## 环境变量
- 无需 API Key。

## 配置项
- `city`：城市名称（如 `Beijing`）。
- `weather_provider`：可选，默认 `wttr`。

## CLI 使用

```sh
python personal-news/weather/client.py config.json
```

输出格式：

```json
[
  {
    "summary": "多云，最低气温12度，最高气温19度，风速约3.2米每秒。"
  }
]
```
