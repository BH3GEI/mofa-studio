# RSS 子模块

用于读取 RSS 源并输出标准化的 `rss` source items。

## 功能
- 拉取配置中的 RSS 源。
- 解析标题、摘要与发布时间。
- 访问文章链接获取正文摘要（默认启用）。

## 配置项
- `rss_sources`：RSS URL 列表。
- `rss_fetch_full_text`：是否抓取正文摘要，默认 `false`。

## CLI 使用

```sh
python personal-news/rss/client.py config.json
```

输出格式：

```json
[
  {
    "title": "Story A",
    "summary": "Summary A",
    "published_at": "Mon, 20 Jan 2026 08:00:00 GMT",
    "source_name": "Example News"
  }
]
```

## RSS源列表

### 📰 综合新闻（国际 / 时政）

- **BBC News**
  - RSS：https://feeds.bbci.co.uk/news/rss.xml
  - 特点：全球视角强，结构清晰，播报型文本很友好
- **Reuters**
  - RSS：https://www.reuters.com/rssFeed/topNews
  - 特点：事实密度高，偏“新闻稿体”，非常适合 TTS
- **The New York Times**
  - RSS：https://rss.nytimes.com/services/xml/rss/nyt/HomePage.xml
  - 特点：叙事感强，适合“深度新闻联播”

---

### 💻 科技 / 创业 / 开发者

- **Hacker News**
  - RSS：https://news.ycombinator.com/rss
  - 特点：技术人必备，适合“今日科技要闻”
- **TechCrunch**
  - RSS：https://techcrunch.com/feed/
  - 特点：创业、AI、产品发布
- **The Verge**
  - RSS：https://www.theverge.com/rss/index.xml
  - 特点：科技 + 社会 + 消费电子，语言偏口语化

---

### 🤖 AI / 工程 / 研究向

- **MIT Technology Review**
  - RSS：https://www.technologyreview.com/feed/
  - 特点：AI、前沿科技，适合“深度解读板块”
- **ArXiv**
  - AI 分类 RSS：https://arxiv.org/rss/cs.AI
  - 特点：可以做“今日论文速览”

---

### 🇨🇳 中文内容（质量相对稳定）

- **36氪**
  - RSS：https://36kr.com/feed
  - 特点：中文科技 / 创业
- **少数派**
  - RSS：https://sspai.com/feed
  - 特点：工具、效率、数码，文本质量高
- **知乎日报**
  - RSS：https://daily.zhihu.com/rss
  - 特点：适合“今日观点精选”
