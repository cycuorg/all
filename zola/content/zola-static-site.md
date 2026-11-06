+++
title = "用 Zola 打造極速中文部落格"
date = 2026-11-01
description = "介紹 Zola 靜態站產生器的優點，以及如何開啟搜尋、分類與中文支援"
[taxonomies]
tags = ["Zola", "Rust", "靜態網站"]
categories = ["網站開發"]
+++

# 用 Zola 打造極速中文部落格

[Zola](https://www.getzola.org/) 是用 Rust 寫成的靜態網站產生器，單一執行檔、速度快、功能齊全。

## 為什麼選 Zola？

- **單一 binary**：無需 Node.js 或一堆依賴
- **內建搜尋索引**：支援 elasticlunr 與 fuse
- **語法高亮、Sass、短代碼** 都內建
- **Taxonomies**（標籤 / 分類）原生支援
- 建置速度極快

## 開啟搜尋的關鍵設定

```toml
build_search_index = true

[search]
include_title = true
include_description = true
include_content = true
index_format = "elasticlunr_json"
```

建置後會產生 `search_index.zh-TW.json` 與 `elasticlunr.min.js`。前端再搭配簡單的 JavaScript 即可完成即時搜尋。

## 中文注意事項

預設官方 binary 未內建中文分詞（為控制體積）。若需要更好的中文搜尋效果，可自行編譯：

```bash
cargo install --git https://github.com/getzola/zola.git --features indexing-zh zola
```

語言代碼建議使用 `zh` 或 `zh-TW`，並在模板中正確載入對應的索引檔。

## 本站實作重點

本站已整合：

- Elasticlunr 全文搜尋
- 標籤與分類頁面
- Atom Feed
- 響應式現代介面
- 程式碼高亮

非常適合用來記錄技術筆記與個人部落格。
