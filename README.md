# Parse, don't validate (in Rust)

[Parse, don't validate](https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/) is a truly excellent introduction to the idea of type-driven design. However, its examples are written in Haskell, a notoriously difficult language for people unfamiliar with its syntax or functional programming paradigms to understand.

This article is my port to Rust, with a few comments I'll add along the way. I will be paraphrasing some parts while directly quoting others.

I'm also adding several Rust projects in this repo which illustrate the concepts we'll go through.

## The realm of possibility

One of the wonderful things about static type systems is that they can make it possible, and sometimes even easy, to answer questions like “is it possible to write this function?” For an extreme example, consider the following Rust type signature:

```rust
fn foo(num: u8) -> Infallible;
```

Is it possible to implement foo? Trivially, the answer is no, as [Infallible](https://doc.rust-lang.org/std/convert/enum.Infallible.html) is a enum that contains no variants, so it’s impossible for any function to produce a value of type Infallible. That example is pretty boring, but the question gets much more interesting if we choose a more realistic example:

```rust
fn head<T>(slice: &[T]) -> &T;
```

This function returns the first element from a list. Is it possible to implement? It certainly doesn’t sound like it does anything very complicated, but if we attempt to implement it, the compiler won’t be satisfied (unless you `unwrap()`).

```rust
fn head<T>(slice: &[T]) -> &T {
    match slice.get(0) {
        Some(v) => v,
    }
}
```

Compilation error!

```bash
error[E0004]: non-exhaustive patterns: `None` not covered
   --> head/src/main.rs:9:11
    |
9   |     match slice.get(0) {
    |           ^^^^^^^^^^^^ pattern `None` not covered
    |
note: `Option<&T>` defined here
   --> /home/andrew/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:568:5
    |
564 | pub enum Option<T> {
    | ------------------
...
568 |     None,
    |     ^^^^ not covered
    = note: the matched value is of type `Option<&T>`
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern or an explicit pattern as shown
    |
10  ~         Some(v) => v,
11  ~         None => todo!(),
    |

For more information about this error, try `rustc --explain E0004`.
error: could not compile `head` (bin "head") due to previous error
```



