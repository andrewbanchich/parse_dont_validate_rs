# Parse, don't validate (in Rust)

[Parse, don't validate](https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/) is a truly excellent introduction to the idea of type-driven design. However, its examples are written in Haskell, a notoriously difficult language for people unfamiliar with its syntax or functional programming paradigms to understand.

This article is my port to Rust, with a few comments I'll add along the way. I will be paraphrasing some parts while directly quoting others.

I'm also adding several Rust projects in this repo which illustrate the concepts we'll go through.

## The realm of possibility

One of the wonderful things about static type systems is that they can make it possible, and sometimes even easy, to answer questions like “is it possible to write this function?” For an extreme example, consider the following Rust type signature:

```
fn foo(num: u8) -> Infallible
```

Is it possible to implement foo? Trivially, the answer is no, as [Infallible](https://doc.rust-lang.org/std/convert/enum.Infallible.html) is a enum that contains no variants, so it’s impossible for any function to produce a value of type Infallible. That example is pretty boring, but the question gets much more interesting if we choose a more realistic example:

```
fn head<T>(slice: &[T]) -> T
```

This function returns the first element from a list. Is it possible to implement? It certainly doesn’t sound like it does anything very complicated, but if we attempt to implement it, the compiler won’t be satisfied:

