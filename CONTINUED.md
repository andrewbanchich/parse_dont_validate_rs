# Parse, don't validate (in Rust!), continued

I'd like to distill the article to a few key points:

1. "Parsing" is caching validation logic in a static type system.
2. Domain invariants should be encoded in types as much as possible.
3. It should not be possible to construct a type in an invalid state (i.e. some of its invariants are incorrect). In Rust, this is commonly referred to as "correct by construction".

# All is vanity

![All is Vanity (1892) by Charles Allan Gilbert](https://github.com/andrewbanchich/parse_dont_validate_rs/assets/13824577/c1a08954-91ff-4cc3-b144-e71cb43c8dbb)

When we look at this picture we see two images - one very different from the other, but every stroke shared between them.
Every computer program is the same; one codebase, but two different programs. The program for users is called "runtime", while the program for developers is called "compile time".

To "move a problem to compile time" is to enhance the developer program. We saw the effects of this in the "Parse, don't validate" article. Adding type safety means moving certain problems to compile time - the check for the list length disappeared, some code became simpler, and other code disappeared.

This effect is no different than that of moving from a dynamically typed language to a statically typed one. In JavaScript, for example, it's common to see code like

```javascript
if (typeof userInput == 'number') {
  ...
}
```

to ensure the provided value is actually a number and not a string, array, object, etc. By using a statically typed language ("static" means compile time, while "dynamic" means runtime), we are "parsing, not validating" as soon as our program enters existence (compilation). And not all statically typed languages are the same! Rust does not have a concept of `null` or or exceptions, both of which are other examples of the same type safety issues addressed in "Parse, don't validate". In C#, you need:

```csharp
if (data != null)
```

since any reference type could be null, even though the type signature does not say so. This is because the language, like many others, did not originally allow developers to encode non-nullability into reference types. `null` and exceptions are "special" in that the type system does not catch them. TODO: yeah, some languages are doing stuff to fix this, but you get the point.

Rust adds other layers of default "language type safety" on top of this - thread safety, and ownership being two of the most important.

## Don't rely on default language type safety

But one of the major points of "Parse, don't validate" is that you should not simply rely on the default language data structures to enforce your domain invariants. You can (and should) use them whenever you can, but creating your own custom types which wrap native language types allow you to educate the compiler, and code for developers, even more.
