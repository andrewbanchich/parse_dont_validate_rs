# Parse, don't validate (in Rust!), continued

I'd like to distill the article to a few key points:

1. "Parsing" is caching validation logic in a static type system.
2. Domain invariants should be encoded in types as much as possible.
3. It should not be possible to construct a type in an invalid state (i.e. some of its invariants are incorrect). In Rust, this is commonly referred to as "correct by construction".

# All is vanity

![All is Vanity (1892) by Charles Allan Gilbert](https://github.com/andrewbanchich/parse_dont_validate_rs/assets/13824577/c1a08954-91ff-4cc3-b144-e71cb43c8dbb)

When we look at this picture we see two images - one very different from the other, but every stroke shared between them.
Every computer program is the same; one for users and one for developers.

