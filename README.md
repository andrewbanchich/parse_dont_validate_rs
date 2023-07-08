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

This function returns the first element from a list. Is it possible to implement? It certainly doesn’t sound like it does anything very complicated, but if we attempt to implement it, the compiler won’t be satisfied (unless you panic).

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

This message is helpfully pointing out that our function is partial, which is to say it is not defined for all possible inputs. Specifically, it is not defined when the input is `&[]`, an empty slice. This makes sense, as it isn’t possible to return the first element of a slice if the slice is empty; there’s no element to return! So, remarkably, we learn this function isn’t possible to implement (without panicking), either.

## Turning partial functions total

To someone coming from a dynamically-typed background, this might seem perplexing. If we have a list, we might very well want to get the first element in it. And indeed, the operation of “getting the first element of a list” isn’t impossible in Rust (without panicking), it just requires a little extra ceremony. There are two different ways to fix the `head()` function, and we’ll start with the simplest one.

### Managing expectations

As established, `head()` is partial because there is no element to return if the slice is empty: we’ve made a promise we cannot possibly fulfill. Fortunately, there’s an easy solution to that dilemma: we can weaken our promise. Since we cannot guarantee the caller an element of the slice, we’ll have to practice a little expectation management: we’ll do our best return an element if we can, but we reserve the right to return nothing at all. In Rust, we express this possibility using the [`Option`](https://doc.rust-lang.org/std/option/enum.Option.html) type:

```rust
fn head<T>(slice: &[T]) -> Option<&T>;
```

This buys us the freedom we need to implement `head()`; it allows us to return `None` when we discover we can’t produce a value of type `T` after all:

```rust
fn head<T>(slice: &[T]) -> Option<&T> {
    slice.get(0)
}
```

Problem solved, right? For the moment, yes... but this solution has a hidden cost.

Returning `Option` is undoubtably convenient when we’re implementing `head()`. However, it becomes significantly less convenient when we want to actually use it! Since `head()` always has the potential to return `None`, the burden falls upon its callers to handle that possibility, and sometimes that passing of the buck can be incredibly frustrating. To see why, consider the following code:

```
getConfigurationDirectories :: IO [FilePath]
getConfigurationDirectories = do
  configDirsString <- getEnv "CONFIG_DIRS"
  let configDirsList = split ',' configDirsString
  when (null configDirsList) $
    throwIO $ userError "CONFIG_DIRS cannot be empty"
  pure configDirsList

main :: IO ()
main = do
  configDirs <- getConfigurationDirectories
  case head configDirs of
    Just cacheDir -> initializeCache cacheDir
    Nothing -> error "should never happen; already checked configDirs is non-empty"
```
