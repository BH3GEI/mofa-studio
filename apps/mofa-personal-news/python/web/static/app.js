const page = document.body.dataset.page;
const configInput = document.querySelector("#config-input");
const inputsInput = document.querySelector("#inputs-input");
const output = document.querySelector("#output");
const readme = document.querySelector("#readme");
const testResults = document.querySelector("#test-results");
const testsInput = document.querySelector("#tests-input");

const defaultConfig = {
  max_duration_seconds: 240,
  language: "zh-CN",
  mode: "morning",
  city: "Beijing",
};

const defaultInputs = [
  {
    source: "rss",
    items: [
      {
        title: "AI policy update",
        summary: "Regulators发布新的AI政策框架。",
        published_at: "2026-01-20T08:30:00Z",
        source_name: "TechDaily",
      },
    ],
  },
  {
    source: "x",
    items: [
      {
        author: "founderA",
        text: "我们刚发布新版本，细节见链接。",
        engagement: { likes: 120, retweets: 30 },
        created_at: "2026-01-20T09:00:00Z",
      },
    ],
  },
  { source: "weather", items: [{ summary: "多云转小雨，最高气温18度。" }] },
];

if (configInput) {
  configInput.value = JSON.stringify(defaultConfig, null, 2);
}
if (inputsInput) {
  inputsInput.value = JSON.stringify(defaultInputs, null, 2);
}

function renderMarkdown(markdown) {
  let html = markdown
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");

  html = html.replace(/```([\s\S]*?)```/g, (match, code) => {
    return `<pre><code>${code.trim()}</code></pre>`;
  });

  html = html.replace(/^### (.*)$/gm, "<h3>$1</h3>");
  html = html.replace(/^## (.*)$/gm, "<h2>$1</h2>");
  html = html.replace(/^# (.*)$/gm, "<h1>$1</h1>");
  html = html.replace(/`([^`]+)`/g, "<code>$1</code>");
  html = html.replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");
  html = html.replace(/^- (.*)$/gm, "<li>$1</li>");
  html = html.replace(/(<li>[\s\S]*?<\/li>)/g, "<ul>$1</ul>");
  html = html.replace(/\n{2,}/g, "</p><p>");
  return `<p>${html}</p>`;
}

async function loadReadme(path = "README.md") {
  try {
    const response = await fetch(`/api/readme?path=${encodeURIComponent(path)}`);
    const data = await response.json();
    if (!response.ok) {
      if (readme) {
        readme.textContent = data.error || "README 加载失败。";
      }
      return;
    }
    const content = data.content || "README 加载失败。";
    if (readme) {
      if (readme.dataset.mode === "markdown") {
        readme.innerHTML = renderMarkdown(content);
      } else {
        readme.textContent = content;
      }
    }
  } catch (error) {
    if (readme) {
      readme.textContent = "README 加载失败。";
    }
  }
}

async function loadModuleReadme(target) {
  const path = target.dataset.path;
  const output = target.closest(".module-item").querySelector(".module-content");
  if (!path || !output) {
    return;
  }
  if (output.dataset.loaded === "true") {
    output.classList.toggle("open");
    return;
  }
  output.textContent = "加载中...";
  output.classList.add("open");
  try {
    const response = await fetch(`/api/readme?path=${encodeURIComponent(path)}`);
    const data = await response.json();
    if (!response.ok) {
      output.textContent = data.error || "README 加载失败。";
      return;
    }
    output.innerHTML = renderMarkdown(data.content || "README 加载失败。");
    output.dataset.loaded = "true";
    output.classList.add("open");
  } catch (error) {
    output.textContent = "README 加载失败。";
  }
}

async function runGenerate() {
  if (!output) {
    return;
  }
  output.textContent = "生成中...";
  try {
    const payload = {
      config: JSON.parse(configInput.value),
      inputs: JSON.parse(inputsInput.value),
    };
    const response = await fetch("/api/generate", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });
    const data = await response.json();
    if (!response.ok) {
      output.textContent = data.error || "生成失败。";
      return;
    }
    output.textContent = data.script || "";
  } catch (error) {
    output.textContent = "JSON 解析失败，请检查输入格式。";
  }
}

async function runTests() {
  if (!testResults) {
    return;
  }
  testResults.innerHTML = "";
  let payload = {};
  if (testsInput) {
    try {
      payload = { tests: JSON.parse(testsInput.value) };
    } catch (error) {
      const item = document.createElement("li");
      item.className = "fail";
      item.textContent = "测试 JSON 解析失败。";
      testResults.appendChild(item);
      return;
    }
  }
  const response = await fetch("/api/run-tests", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  });
  const data = await response.json();
  const results = data.results || [];
  if (!results.length) {
    const item = document.createElement("li");
    item.className = "fail";
    item.textContent = "未返回测试结果。";
    testResults.appendChild(item);
    return;
  }
  results.forEach((result) => {
    const item = document.createElement("li");
    item.className = result.passed ? "pass" : "fail";
    item.textContent = `${result.passed ? "通过" : "失败"} - ${result.name}: ${result.detail}`;
    testResults.appendChild(item);
  });
}

function loadDefaultTests() {
  if (!testsInput) {
    return;
  }
  const sampleTests = [
    {
      name: "section-order",
      config: { max_duration_seconds: 240 },
      inputs: defaultInputs,
      order: [
        ["今日要闻", "与个人相关的动态"],
        ["与个人相关的动态", "生活服务"],
      ],
      contains: ["个人新闻联播"],
    },
    {
      name: "dedupe-rss-x",
      config: {},
      inputs: [
        {
          source: "rss",
          items: [
            {
              title: "SpaceX launches new rocket",
              summary: "SpaceX launches new rocket successfully.",
              published_at: "2026-01-20T08:00:00Z",
              source_name: "SpaceNews",
            },
          ],
        },
        {
          source: "x",
          items: [
            {
              author: "spacex",
              text: "SpaceX launches new rocket successfully.",
              engagement: { likes: 500, retweets: 200 },
              created_at: "2026-01-20T08:10:00Z",
            },
          ],
        },
      ],
      contains: ["与个人相关的动态中"],
      not_contains: ["今日要闻方面"],
    },
  ];
  testsInput.value = JSON.stringify(sampleTests, null, 2);
}

const runBtn = document.querySelector("#run-btn");
if (runBtn) {
  runBtn.addEventListener("click", runGenerate);
}

const testBtn = document.querySelector("#test-btn");
if (testBtn) {
  testBtn.addEventListener("click", runTests);
}

if (page === "docs") {
  loadReadme();
}

if (page === "module") {
  const path = document.body.dataset.readmePath;
  if (path) {
    loadReadme(path);
  }
}

if (page === "run") {
  loadReadme();
}

if (page === "tests") {
  loadDefaultTests();
}
