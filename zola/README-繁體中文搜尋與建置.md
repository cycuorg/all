# Zola 網誌建置與繁體中文搜尋

## 搜尋修正內容
- `config.toml` 啟用 `build_search_index`，索引語言使用 `zh`。
- `templates/base.html` 將頁面語系標記為 `zh-Hant`，並將正確的索引 URL 傳給搜尋程式。
- `static/js/search.js` 同時使用 elasticlunr 相關性排序及文件全文字串比對。繁體中文不依賴空白切詞，因此可搜尋「借用」、「所有權」或「繁體中文」等連續字串。
- 搜尋結果以 DOM `textContent` 建立，避免將文章標題/摘要直接插入 HTML。

## 一、編譯新的 zola.exe（Windows / MSYS2 UCRT64）
確認 Rust/Cargo 與 MSYS2 UCRT64 建置工具已安裝。在解壓後的 Zola 原始碼根目錄執行：

```bash
cargo clean
cargo build --release --features indexing-zh
./target/release/zola.exe --version
```

`--features indexing-zh` 是關鍵：原始碼的 Zola 根 `Cargo.toml` 已宣告此 feature，會將 `search/indexing-zh` 傳遞給 `elasticlunr-rs/zh`。若不帶此 feature，`zh` 語言索引可能無法建立。

成功後執行檔位於 `target/release/zola.exe`。可複製到 Rust CMS 的工具目錄，例如：

```bash
cp target/release/zola.exe /r/projects/rustcms/tools/zola.exe
```

請依實際路徑調整。不要把原始碼專案目錄 `tools/zola/` 與執行檔 `tools/zola.exe` 混淆。

## 二、建立/更新網誌
將本 ZIP 的 `zola` 網誌專案放在 CMS 的工具目錄，例如 `R:\projects\rustcms\tools\zola`。`config.toml` 的 `output_dir = "../blog"` 會把產出寫到 `tools/blog`。

在 MSYS2 UCRT64：

```bash
cd /r/projects/rustcms/tools/zola
../zola.exe check
../zola.exe build
```

若 `zola.exe` 放在其他位置，請改成實際執行檔路徑。CMD 範例：

```bat
cd /d R:\projects\rustcms\tools\zola
..\zola.exe check
..\zola.exe build
```

建置後確認輸出目錄有以下檔案：
- `R:\projects\rustcms\tools\blog\search_index.zh.json`
- `R:\projects\rustcms\tools\blog\elasticlunr.min.js`
- `R:\projects\rustcms\tools\blog\js\search.js`

## 三、新增文章與重新產生內容
在 `content` 目錄新增 Markdown 檔，例如 `content/my-new-post.md`：

```toml
+++
title = "繁體中文搜尋測試"
date = 2026-11-05
description = "測試 Zola 網誌繁體中文關鍵字搜尋"
[taxonomies]
tags = ["Rust", "繁體中文"]
categories = ["程式設計"]
+++

這篇文章用來測試繁體中文搜尋。可以搜尋「繁體中文」、「關鍵字搜尋」或「Rust」。
```

儲存後重新產生靜態網頁與搜尋索引：

```bash
cd /r/projects/rustcms/tools/zola
../zola.exe build
```

本機預覽可執行：

```bash
../zola.exe serve --interface 127.0.0.1 --port 1111
```

依 `base_url` 與實際部署路徑開啟 `http://127.0.0.1:1111/blog/`。若 Zola serve 的本機掛載路徑與正式 `/blog/` 不同，請以終端機顯示的本機網址為準。

## 四、驗證搜尋
1. 確認 `blog/search_index.zh.json` 存在且不是 404。
2. 瀏覽器開發者工具 Network 中確認索引請求路徑為目前網站的 `/blog/search_index.zh.json`。
3. 在搜尋框輸入「繁體中文」、「所有權」、「借用」測試；也測試英文如 `Rust`。
4. 修改文章後一定要重新執行 `zola build`，否則靜態輸出及索引仍是舊版。

## 注意
這個前端搜尋會對已建立的索引文件做繁體中文連續字串比對，不要求中文詞語之間有空白。它是字串包含式搜尋，不是中文語意搜尋或同義詞搜尋。搜尋只涵蓋 Zola 索引內的頁面；在文章 front matter 中設定 `in_search_index = false` 的頁面不會被找到。
