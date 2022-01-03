<!-- cargo-sync-readme start -->

# `do-notation`, the monadic `do` notation brought to Rust.

This crate provides the `m!` macro, which provides the Haskell monadic syntactic sugar `do`.

> Note: it is not possible to use the `do!` syntax as `do` is a reserved keyword in Rust.

The syntax is very similar to what you find in Haskell:

- You use the `m!` macro; in Haskell, you use the `do` keyword.
- The `<-` syntactic sugar binds its left hand side to the monadic right hand side
  by _entering_ the right side via a closure.
- Like almost any statement in Rust, you must end your statement with a semicolon (`;`).
- The last line must be absent of `;` or contains the `return` keyword.
- You can use `return` nowhere but on the last line.
- A line containing a single expression with a semicolon is a valid statement and has the same effect as `_ <- expr`.
- `let` bindings are allowed in the form `let <pattern> = <expr>;` and have the regular Rust meaning.
- The `do` notation syntax does not extend into inner code blocks; however, it can have its own `m!` block. For example:
  `m! { outer_do... if exp { m! { inner_do... } } else { ... } ... }`.

## How do I make my monad works with `m!`?

Because monads are higher-kinded types, it is not possible to define the monadic do-notation in a fully type-system
elegant way. However, this crate is based on the rebindable concept in Haskell (i.e. you can change what the `>>=`
operator’s types are), so `m!` has one type-system requirement and one syntactic requirement.

First, you have to implement one trait: [`Lift`], which allows to _lift_ a value `A` into a _monadic structure of
`A`_. For instance, lifting a `A` into the `Option` monad yields an `Option<A>`.

Then, you have to provide an `and_then` method, which is akin to Haskell’s `>>=` operator. The choice of using
`and_then` and not a proper name like `flat_map` or `bind` is due to the current state of the standard-library —
monads like `Option` and `Result<_, E>` don’t have `flat_map` defined on them but have `and_then`. The type signature
is not enforced, but:

- `and_then` must be a binary function taking a type `A`, a closure `A -> Monad<B>` and returns `Monad<B>`, where
  `Monad` is the monad you are adding `and_then` for. For instance, if you are implementing it for `Option`,
  `and_then` takes an `A`, a closure `A -> Option<B>` and returns an `Option<B>`.
- `and_then` must move its first argument, which has to be `self`. The type of `Self` is not enforced.
- `and_then`’s closure must take `A` with a `FnOnce` closure.

## Meaning of the `<-` operator

The `<-` syntactic sugar is not strictly speaking an operator: it’s not valid vanilla Rust. Instead, it’s a trick
defined in the `m!` allowing to use both [`Lift::lift`] and `and_then`. When you look at code inside a do-notation
block, every monadic statements (separated with `;` in this crate) can be imagined as a new level of nesting inside
a closure — the one passed to `and_then`, indeed.

## First example: fallible code

One of the first monadic application that people learn is the _fallible_ effect — `Maybe` in Haskell.
In `Rust`, it’s `Option`. `Option` is an interesting monad as it allows you to fail early.

```rust
use do_notation::m;

let r = m! {
  x <- Some("Hello, world!");
  y <- Some(3);
  Some(x.len() * y)
};

assert_eq!(r, Some(39));
```

The `binding <- expr` syntax unwraps the right part and binds it to `binding`, making it available to
next calls — remember, nested closures. The final line re-enters the structure (here, `Option`) explicitly.

Note that it is possible to re-enter the structure without having to specify how / knowing the structure
(with `Option`, you re-enter with `Some`). You can use the `return` keyword, that will automatically lift the
value into the right structure:

```rust
use do_notation::m;

let r = m! {
  x <- Some(1);
  y <- Some(2);
  z <- Some(3);
  return [x, y, z];
};

assert_eq!(r, Some([1, 2, 3]));
```

<!-- cargo-sync-readme end -->
