# Parse, don't validate (in Rust!)

[Parse, don't validate](https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/) is a truly excellent introduction to the idea of type-driven design. However, its examples are written in Haskell, a notoriously difficult language for people unfamiliar with its syntax or functional programming paradigms to understand.

This article is my port to Rust; the words are those of the original author (Alexis King) - only the language terminology and code examples have been changed.

I'm also adding several Rust projects in this repo which illustrate the concepts we'll go through.

## The realm of possibility

One of the wonderful things about static type systems is that they can make it possible, and sometimes even easy, to answer questions like “is it possible to write this function?” For an extreme example, consider the following Rust type signature:

```rust
fn foo(num: u8) -> Infallible;
```

Is it possible to implement foo? Trivially, the answer is no, as [Infallible](https://doc.rust-lang.org/std/convert/enum.Infallible.html) is an enum that contains no variants, so it’s impossible to even construct. That example is pretty boring, but the question gets much more interesting if we choose a more realistic example:

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

```rust
fn main() {
    let config_dirs = get_configuration_directories();

    match head(&config_dirs) {
        Some(cache_dir) => initialize_cache(cache_dir),
        None => panic!("should never happen; already checked configDirs is non-empty"),
    }
}

fn get_configuration_directories() -> Vec<PathBuf> {
    let config_dirs_string =
        env::var("CONFIG_DIRS").expect("CONFIG_DIRS environment variable must be set");

    let config_dirs_list: Vec<_> = config_dirs_string.split(',').map(|s| s.into()).collect();

    if config_dirs_list.is_empty() {
        panic!("CONFIG_DIRS cannot be empty")
    }

    config_dirs_list
}
     
```

When `get_configuration_directories()` retrieves a `Vec` of file paths from the environment, it proactively checks that it is non-empty. However, when we use `head()` in `main()` to get the first element of the `Vec`, the `Option<PathBuf>` result still requires us to handle a `None` case that we know will never happen! This is terribly bad for several reasons:

- First, it’s just annoying. We already checked that the `Vec` is non-empty, why do we have to clutter our code with another redundant check?

- Second, it has a potential performance cost. Although the cost of the redundant check is trivial in this particular example, one could imagine a more complex scenario where the redundant checks could add up, such as if they were happening in a tight loop.

- Finally, and worst of all, this code is a bug waiting to happen! What if `get_configuration_directories()` were modified to stop checking that the list is empty, intentionally or unintentionally? The programmer might not remember to update main, and suddenly the “impossible” error becomes not only possible, but probable.

The need for this redundant check has essentially forced us to punch a hole in our type system. If we could statically prove the `None` case impossible, then a modification to `get_configuration_directories()` that stopped checking if the `Vec` was empty would invalidate the proof and trigger a compile-time failure. However, as-written, we’re forced to rely on a test suite or manual inspection to catch the bug.

### Paying it forward

Clearly, our modified version of head leaves some things to be desired. Somehow, we’d like it to be smarter: if we already checked that the `Vec` was non-empty, `head()` should unconditionally return the first element without forcing us to handle the case we know is impossible. How can we do that?

Let’s look at the original (partial) type signature for `head()` again:

```rust
fn head<T>(slice: &[T]) -> &T;
```

The previous section illustrated that we can turn that partial type signature into a total one by weakening the promise made in the return type. However, since we don’t want to do that, there’s only one thing left that can be changed: the argument type (in this case, `&[T]`). Instead of weakening the return type, we can strengthen the argument type, eliminating the possibility of `head()` ever being called on an empty slice in the first place.

To do this, we need a type that represents non-empty slices. Fortunately this type is simple to implement ourselves:

```rust
struct NonEmptyVec<T>(T, Vec<T>);
```

Note that `NonEmptySlice` a is really just a tuple of a `&T` and an ordinary, possibly-empty `&[T]`. This conveniently models a non-empty slice by storing the first element of the slice separately from the slice’s tail: even if the `&[T]` component is `&[]`, the `&T` component must always be present. This makes head completely trivial to implement:

```rust
fn main() {
    let config_dirs = get_configuration_directories();
    initialize_cache(&head(config_dirs));
}

fn get_configuration_directories() -> NonEmptyVec<PathBuf> {
    let config_dirs_string = env::var("CONFIG_DIRS").unwrap_or_default();
    let mut config_dirs_list: Vec<_> = config_dirs_string.split(',').map(|s| s.into()).collect();

    match config_dirs_list.pop() {
        Some(head) => NonEmptyVec(head, config_dirs_list),
        None => panic!("CONFIG_DIRS cannot be empty"),
    }
}

struct NonEmptyVec<T>(T, Vec<T>);

fn head<T>(vec: NonEmptyVec<T>) -> T {
    vec.0
}

fn initialize_cache(cache_dir: &Path) {
    todo!("just imagine this does something")
}

```

Note that the redundant check in `main()` is now completely gone! Instead, we perform the check exactly once, in `get_configuration_directories()`. It constructs a `NonEmptyVec` a from a `Vec<T>` using the [`Vec::pop()`](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.pop) function, which has the following signature:

```rust
fn pop(&mut self) -> Option<T>
```

The `Option` is still there, but this time, we handle the `None` case very early in our program: right in the same place we were already doing the input validation. Once that check has passed, we now have a `NonEmptyVec<PathBuf>` value, which preserves (in the type system!) the knowledge that the `Vec` really is non-empty. Put another way, you can think of a value of type `NonEmptyVec<T>` as being like a value of type `&[T]`, plus a proof that the list is non-empty.

By strengthening the type of the argument to `head()` instead of weakening the type of its result, we’ve completely eliminated all the problems from the previous section:

- The code has no redundant checks, so there can’t be any performance overhead.

- Furthermore, if `get_configuration_directories()` changes to stop checking that the list is non-empty, its return type must change, too. Consequently, `main()` will fail to typecheck, alerting us to the problem before we even run the program!

// TODO - left out section I'm not understanding

## The power of parsing

You may be wondering what the above example has to do with the title of this blog post. After all, we only examined two different ways to validate that a list was non-empty; no parsing in sight. That interpretation isn’t wrong, but I’d like to propose another perspective: in my mind, the difference between validation and parsing lies almost entirely in how information is preserved. Consider the following pair of functions:

```rust
fn validate_non_empty<T>(vec: Vec<T>) -> Result<(), String> {
    if vec.is_empty() {
        Err("Slice was empty".to_string())
    } else {
        Ok(())
    }
}

fn parse_non_empty<T>(mut vec: Vec<T>) -> Result<NonEmptyVec<T>, String> {
    match vec.pop() {
        None => Err("Vec was empty".to_string()),
        Some(head) => Ok(NonEmptyVec(head, vec)),
    }
}
```

These two functions are nearly identical: they check if the provided list is empty, and if it is, they return an error. The difference lies entirely in the return type: `validate_non_empty()` always returns () if successful. The type that contains no information, but `parse_non_empty()` returns `NonEmptyVec<T>`, a refinement of the input type that preserves the knowledge gained in the type system. Both of these functions check the same thing, but `parse_non_empty()` gives the caller access to the information it learned, while `validate_non_empty()` just throws it away.

These two functions elegantly illustrate two different perspectives on the role of a static type system: `validate_non_empty()` obeys the typechecker well enough, but only `parse_non_empty()` takes full advantage of it. If you see why `parse_non_empty()` is preferable, you understand what I mean by the mantra “parse, don’t validate.” Still, perhaps you are skeptical of `parse_non_empty()`’s name. Is it really parsing anything, or is it merely validating its input and returning a result? While the precise definition of what it means to parse or validate something is debatable, I believe `parse_non_empty()` is a bona-fide parser (albeit a particularly simple one).

Consider: what is a parser? Really, a parser is just a function that consumes less-structured input and produces more-structured output. By its very nature, a parser is a partial function — some values in the domain do not correspond to any value in the range — so all parsers must have some notion of failure. Often, the input to a parser is text, but this is by no means a requirement, and `parse_non_empty()` is a perfectly cromulent parser: it parses lists into non-empty lists, signaling failure by terminating the program with an error message.

Under this flexible definition, parsers are an incredibly powerful tool: they allow discharging checks on input up-front, right on the boundary between a program and the outside world, and once those checks have been performed, they never need to be checked again! Haskellers (and Rustaceans) are well-aware of this power, and they use many different types of parsers on a regular basis:

- The [`serde_json`](https://docs.rs/serde_json/latest/serde_json/) library provides functions that can be used to parse JSON data into domain types.

- Likewise, [`clap`](https://docs.rs/clap/latest/clap/) provides various ways of parsing command-line arguments.

- Database libraries [`SQLx`](https://github.com/launchbadge/sqlx) have mechanisms for parsing values held in an external data store.

- The [`axum`](https://docs.rs/axum/latest/axum/) web server parses Rust datatypes from path components, query parameters, HTTP headers, and more.

The common theme between all these libraries is that they sit on the boundary between your Rust application and the external world. That world doesn’t speak in structs and enums, but in streams of bytes, so there’s no getting around a need to do some parsing. Doing that parsing up front, before acting on the data, can go a long way toward avoiding many classes of bugs, some of which might even be security vulnerabilities.

One drawback to this approach of parsing everything up front is that it sometimes requires values be parsed long before they are actually used. In a dynamically-typed language, this can make keeping the parsing and processing logic in sync a little tricky without extensive test coverage, much of which can be laborious to maintain. However, with a static type system, the problem becomes marvelously simple, as demonstrated by the `NonEmptyVec` example above: if the parsing and processing logic go out of sync, the program will fail to even compile.

## The danger of validation

Hopefully, by this point, you are at least somewhat sold on the idea that parsing is preferable to validation, but you may have lingering doubts. Is validation really so bad if the type system is going to force you to do the necessary checks eventually anyway? Maybe the error reporting will be a little bit worse, but a bit of redundant checking can’t hurt, right?

Unfortunately, it isn’t so simple. Ad-hoc validation leads to a phenomenon that the [language-theoretic security](http://langsec.org/) field calls *shotgun parsing*. In the 2016 paper, [The Seven Turrets of Babel: A Taxonomy of LangSec Errors and How to Expunge Them](http://langsec.org/papers/langsec-cwes-secdev2016.pdf), its authors provide the following definition:

> *Shotgun parsing is a programming antipattern whereby parsing and input-validating code is mixed with and spread across processing code—throwing a cloud of checks at the input, and hoping, without any systematic justification, that one or another would catch all the “bad” cases.*

They go on to explain the problems inherent to such validation techniques:

> *Shotgun parsing necessarily deprives the program of the ability to reject invalid input instead of processing it. Late-discovered errors in an input stream will result in some portion of invalid input having been processed, with the consequence that program state is difficult to accurately predict.*

In other words, a program that does not parse all of its input up front runs the risk of acting upon a valid portion of the input, discovering a different portion is invalid, and suddenly needing to roll back whatever modifications it already executed in order to maintain consistency. Sometimes this is possible — such as rolling back a transaction in an RDBMS — but in general it may not be.

It may not be immediately apparent what shotgun parsing has to do with validation — after all, if you do all your validation up front, you mitigate the risk of shotgun parsing. The problem is that validation-based approaches make it extremely difficult or impossible to determine if everything was actually validated up front or if some of those so-called “impossible” cases might actually happen. The entire program must assume that raising an exception anywhere is not only possible, it’s regularly necessary.

Parsing avoids this problem by stratifying the program into two phases — parsing and execution — where failure due to invalid input can only happen in the first phase. The set of remaining failure modes during execution is minimal by comparison, and they can be handled with the tender care they require.

# Parsing, not validating, in practice

So far, this blog post has been something of a sales pitch. “You, dear reader, ought to be parsing!” it says, and if I’ve done my job properly, at least some of you are sold. However, even if you understand the “what” and the “why,” you might not feel especially confident about the “how.”

My advice: focus on the datatypes.

Suppose you are writing a function that accepts a list of tuples representing key-value pairs, and you suddenly realize you aren’t sure what to do if the list has duplicate keys. One solution would be to write a function that asserts there aren’t any duplicates in the list:

```rust
fn check_no_duplicate_keys<K: PartialEq, V>(slice: &[(K, V)]) -> Result<(), ValidationError>
```

However, this check is fragile: it’s extremely easy to forget. Because its return value is unused, it can always be omitted, and the code that needs it would still typecheck. A better solution is to choose a data structure that disallows duplicate keys by construction, such as a `HashMap`. Adjust your function’s type signature to accept a `HashMap` instead of a list of tuples, and implement it as you normally would.

Once you’ve done that, the call site of your new function will likely fail to typecheck, since it is still being passed a list of tuples. If the caller was given the value via one of its arguments, or if it received it from the result of some other function, you can continue updating the type from list to `HashMap`, all the way up the call chain. Eventually, you will either reach the location the value is created, or you’ll find a place where duplicates actually ought to be allowed. At that point, you can insert a call to a modified version of `check_no_duplicate_keys()`:

```rust
fn check_no_duplicate_keys<K: Eq + Hash, V>(slice: &[(K, V)]) -> Result<HashMap<K, V>, ValidationError>
```

Now the check cannot be omitted, since its result is actually necessary for the program to proceed!

This hypothetical scenario highlights two simple ideas:

- Use a data structure that makes illegal states unrepresentable. Model your data using the most precise data structure you reasonably can. If ruling out a particular possibility is too hard using the encoding you are currently using, consider alternate encodings that can express the property you care about more easily. Don’t be afraid to refactor.

- Push the burden of proof upward as far as possible, but no further. Get your data into the most precise representation you need as quickly as you can. Ideally, this should happen at the boundary of your system, before any of the data is acted upon.

- If one particular code branch eventually requires a more precise representation of a piece of data, parse the data into the more precise representation as soon as the branch is selected. Use generic types judiciously to allow your datatypes to reflect and adapt to control flow.

In other words, **write functions on the data representation you wish you had, not the data representation you are given**. The design process then becomes an exercise in bridging the gap, often by working from both ends until they meet somewhere in the middle. Don’t be afraid to iteratively adjust parts of the design as you go, since you may learn something new during the refactoring process!

Here are a handful of additional points of advice, arranged in no particular order:

- Let your datatypes inform your code, don’t let your code control your datatypes. Avoid the temptation to just stick a Bool in a record somewhere because it’s needed by the function you’re currently writing. Don’t be afraid to refactor code to use the right data representation—the type system will ensure you’ve covered all the places that need changing, and it will likely save you a headache later.

- Treat functions that return `Result<(), E>` with deep suspicion. Sometimes these are genuinely necessary, as they may perform an imperative effect with no meaningful result, but if the primary purpose of that effect is raising an error, it’s likely there’s a better way.

- Don’t be afraid to parse data in multiple passes. Avoiding shotgun parsing just means you shouldn’t act on the input data before it’s fully parsed, not that you can’t use some of the input data to decide how to parse other input data. Plenty of useful parsers are context-sensitive.

- Avoid denormalized representations of data, especially if it’s mutable. Duplicating the same data in multiple places introduces a trivially representable illegal state: the places getting out of sync. Strive for a single source of truth.

- Keep denormalized representations of data behind abstraction boundaries. If denormalization is absolutely necessary, use encapsulation to ensure a small, trusted module holds sole responsibility for keeping the representations in sync.

- Use abstract datatypes to make validators “look like” parsers. Sometimes, making an illegal state truly unrepresentable is just plain impractical given the tools Rust provides, such as ensuring an integer is in a particular range. In that case, use an abstract newtype with a smart constructor to “fake” a parser from a validator.

As always, use your best judgement. It probably isn’t worth breaking out singletons and refactoring your entire application just to get rid of a single error "impossible" call somewhere — just make sure to treat those situations like the radioactive substance they are, and handle them with the appropriate care. If all else fails, at least leave a comment to document the invariant for whoever needs to modify the code next.
