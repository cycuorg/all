+++
title = "深入理解 Ownership 與 Borrowing"
date = 2026-11-03
description = "用實際例子說明 Rust 最核心的所有權與借用規則"
[taxonomies]
tags = ["Rust", "Ownership", "進階"]
categories = ["程式設計"]
+++

# 深入理解 Ownership 與 Borrowing

Ownership 是 Rust 最獨特的設計。它讓編譯器能在**編譯期**追蹤每個值的生命週期，進而保證記憶體安全。

## Ownership 三大規則

1. 每個值都有一個擁有者（owner）
2. 同一時間只能有一個擁有者
3. 當擁有者離開作用域，值會被 drop

```rust
{
    let s = String::from("hello"); // s 是擁有者
} // 作用域結束，s 被 drop，記憶體釋放
```

## 移動（Move）語意

```rust
let s1 = String::from("hello");
let s2 = s1; // s1 的所有權移動到 s2
// println!("{}", s1); // 編譯錯誤！s1 已失效
```

## 借用（Borrowing）

不想轉移所有權時，可以使用參考：

```rust
fn calculate_length(s: &String) -> usize {
    s.len()
}

let s = String::from("hello");
let len = calculate_length(&s);
println!("{} 的長度是 {}", s, len); // s 仍然有效
```

### 可變借用規則

- 同一時間只能有一個 `&mut T`，或任意多個 `&T`
- 不能同時存在可變與不可變借用

這些規則由編譯器強制執行，是 Rust 「Fearless Concurrency」的基礎。
