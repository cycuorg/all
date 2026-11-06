// Zola Traditional Chinese search.
// Read the Zola JSON index directly instead of elasticlunr.Index.load():
// Zola's zh index may serialize a "trimmer-zh" pipeline function, while
// the bundled browser elasticlunr.min.js does not register that function.
// Literal substring matching is suitable for Chinese, which has no spaces
// between words, and does not require the elasticlunr runtime at all.
(function () {
  "use strict";

  function normalize(value) {
    return String(value || "").normalize("NFKC").toLocaleLowerCase("zh-TW");
  }

  function fieldsOf(doc) {
    return [doc.title, doc.description, doc.body, doc.content]
      .filter(function (value) { return typeof value === "string" && value.length; })
      .map(normalize);
  }

  function searchDocuments(indexData, query) {
    var docs = indexData && indexData.documentStore && indexData.documentStore.docs;
    if (!docs) throw new Error("搜尋索引沒有 documentStore.docs；請確認 Zola 已啟用 include_content");
    var terms = normalize(query).split(/\s+/).filter(Boolean);
    var results = [];

    Object.keys(docs).forEach(function (ref) {
      var doc = docs[ref];
      if (!doc) return;
      var fields = fieldsOf(doc);
      if (!terms.every(function (term) {
        return fields.some(function (field) { return field.indexOf(term) !== -1; });
      })) return;

      var title = normalize(doc.title);
      var description = normalize(doc.description);
      var body = normalize(doc.body || doc.content);
      var score = 0;
      terms.forEach(function (term) {
        if (title.indexOf(term) !== -1) score += 8;
        if (description.indexOf(term) !== -1) score += 4;
        if (body.indexOf(term) !== -1) score += 1;
      });
      results.push({ ref: ref, doc: doc, score: score });
    });

    return results.sort(function (a, b) {
      return b.score - a.score || String(a.doc.title || "").localeCompare(String(b.doc.title || ""), "zh-TW");
    });
  }

  function makeTeaser(doc, query) {
    var body = String(doc.body || doc.content || doc.description || "");
    var plain = body.replace(/<[^>]*>/g, " ").replace(/\s+/g, " ").trim();
    if (!plain) return "";
    var at = normalize(plain).indexOf(normalize(query));
    if (at < 0) at = 0;
    var start = Math.max(0, at - 55);
    var end = Math.min(plain.length, start + 180);
    return (start ? "…" : "") + plain.slice(start, end) + (end < plain.length ? "…" : "");
  }

  function render(itemsBox, results, query) {
    itemsBox.replaceChildren();
    results.slice(0, 10).forEach(function (item) {
      var wrap = document.createElement("div");
      wrap.className = "search-results__item";

      var link = document.createElement("a");
      link.href = item.ref;
      link.textContent = item.doc.title || item.ref;

      var teaser = document.createElement("div");
      teaser.className = "teaser";
      teaser.textContent = makeTeaser(item.doc, query);

      wrap.appendChild(link);
      wrap.appendChild(teaser);
      itemsBox.appendChild(wrap);
    });
  }

  function init() {
    var input = document.getElementById("search");
    var resultsBox = document.getElementById("search-results");
    var itemsBox = resultsBox && resultsBox.querySelector(".search-results__items");
    var searchBox = input && input.closest(".search-box");
    if (!input || !resultsBox || !itemsBox || !searchBox) return;

    var indexPromise = null;
    var timer = null;
    var lastQuery = null;

    function loadIndex() {
      if (!indexPromise) {
        var url = searchBox.getAttribute("data-index-url");
        if (!url) throw new Error("Missing search index URL");
        indexPromise = fetch(url, { credentials: "same-origin" }).then(function (response) {
          if (!response.ok) throw new Error("Search index HTTP " + response.status);
          return response.json();
        });
      }
      return indexPromise;
    }

    async function runSearch() {
      var query = input.value.trim();
      if (query === lastQuery) return;
      lastQuery = query;
      if (!query) {
        itemsBox.replaceChildren();
        resultsBox.hidden = true;
        return;
      }

      try {
        var indexData = await loadIndex();
        var results = searchDocuments(indexData, query);
        render(itemsBox, results, query);
        resultsBox.hidden = results.length === 0;
      } catch (error) {
        console.error("Zola search failed:", error);
        itemsBox.replaceChildren();
        var message = document.createElement("div");
        message.className = "search-results__item";
        message.textContent = "搜尋索引載入失敗：" + error.message;
        itemsBox.appendChild(message);
        resultsBox.hidden = false;
      }
    }

    input.addEventListener("input", function () {
      clearTimeout(timer);
      timer = setTimeout(runSearch, 120);
    });
    input.addEventListener("keydown", function (event) {
      if (event.key === "Escape") resultsBox.hidden = true;
    });
    document.addEventListener("click", function (event) {
      if (!event.target.closest(".search-box")) resultsBox.hidden = true;
    });
    input.addEventListener("focus", function () {
      if (itemsBox.children.length) resultsBox.hidden = false;
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
