# X 子模块

用于获取关注账号的公开动态并输出标准化的 `x` source items。

## 功能
- 使用 X API v2 获取用户最新推文。
- 将互动数据映射为 `engagement` 字段。

## 环境变量
- `X_BEARER_TOKEN`：X API v2 Bearer Token。

也可写入 `personal-news/env.secret`。

## 配置项
- `x_priority_accounts`：关注账号列表（用户名或用户 ID）。

## CLI 使用

```sh
python personal-news/x/client.py config.json
```

输出格式：

```json
[
  {
    "author": "username",
    "text": "tweet text",
    "engagement": {
      "likes": 12,
      "retweets": 3
    },
    "created_at": "2026-01-20T08:00:00Z"
  }
]
```
