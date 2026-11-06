+++
title = "Rust 程式語言入門"
date = 2026-11-05
description = "認識 Rust 程式語言的核心特色：記憶體安全、Ownership 與並行處理"
[taxonomies]
tags = ["Rust", "Programming", "入門"]
categories = ["程式設計"]
+++

# Rust 程式語言入門

Rust 是一種重視**安全性**、**效能**與**並行處理**的系統程式語言，由 Mozilla 研發，現由 Rust Foundation 維護。

## 為什麼選擇 Rust？

傳統系統語言（如 C/C++）雖然效能出色，但容易出現記憶體錯誤（use-after-free、buffer overflow 等）。Rust 透過編譯期檢查大幅減少這類問題，同時不犧牲執行效率。

## 主要特色

1. **Memory Safety** — 無需 garbage collector，卻能在編譯期保證記憶體安全
2. **Ownership** — 獨特的所有權系統，明確資料的生命週期
3. **Borrowing** — 透過借用規則安全地共享資料
4. **Zero-cost Abstractions** — 高階抽象不會帶來執行期開銷
5. **Fearless Concurrency** — 編譯器協助避免資料競爭

## 簡單範例

```rust
fn main() {
    let mut s = String::from("你好，Rust");
    s.push_str("！");
    println!("{}", s);
}
```

## 適合的應用場景

- 系統程式與作業系統元件
- Web 後端（Actix、Axum、Rocket）
- 命令列工具
- WebAssembly
- 嵌入式與效能敏感服務

想深入學習，建議從官方 *The Rust Programming Language*（俗稱 The Book）開始。
