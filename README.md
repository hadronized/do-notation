<!-- cargo-sync-readme start -->

# Doo, the monadic `do` notation brought to Rust.

This crate provides the `doo!` macro, which provides the Haskell monadic syntactic sugar `do`.
The syntax is very similar to what you find in Haskell:

- You use the `doo!` macro; in Haskell, you use the `do` keyword. `do` is currently a reserved keyword
  in Rust.
- The `<-` operator is the _bind_ operator: it binds its left hand side to the monadic right hand side
  by _entering_ the right side via a closure.
- Like almost any statement in Rust, you must end your statement with a `;`.
- The last line must be absent of `;` or contains the `return` keyword.
- You can use `return` nowhere but on the last line.
- A line containing a single expression is a valid statement and has the same effect as `_ <- expr`.

# How do I make my monad works with `doo`?

You have to implement two traits: [`Pointed`] and [`Bind`]. Feel free to have a look at their
documentation for further information.

# First example: fallible code

One of the first monadic application that people learn is the _fallible_ effect — `Maybe` in Haskell.
In `Rust`, it’s `Option`. `Option` is an interesting monad as it allows you to fail early.

```rust
use doo::doo;

let r = doo! {
  x <- Some("Hello, world!");
  y <- Some(3);
  Some(x.len() * y)
};

assert_eq!(r, Some(39));
```

The `binding <- expr` syntax unwraps the right part and binds it to `binding`, making it available to
next calls. The final line re-enter the structure (here, `Option`) explicitly.

Note that it is possible to re-enter the structure without having to specify how (with `Option`, you
re-enter with `Some`). You can use the `return` keyword, that will automatically lift the value into
the right structure:

```rust
use doo::doo;

let r = doo! {
  x <- Some(1);
  y <- Some(2);
  z <- Some(3);
  return [x, y, z];
};

assert_eq!(r, Some([1, 2, 3]));
```

<!-- cargo-sync-readme end -->
