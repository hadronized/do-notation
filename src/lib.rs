//! # Doo, the monadic `do` notation brought to Rust.
//!
//! This crate provides the `doo!` macro, which provides the Haskell monadic syntactic sugar `do`.
//! The syntax is very similar to what you find in Haskell:
//!
//! - You use the `doo!` macro; in Haskell, you use the `do` keyword. `do` is currently a reserved keyword
//!   in Rust.
//! - The `<-` operator is the _bind_ operator: it binds its left hand side to the monadic right hand side
//!   by _entering_ the right side via a closure.
//! - Like almost any statement in Rust, you must end your statement with a `;`.
//! - The last line must be absent of `;` or contains the `return` keyword.
//! - You can use `return` nowhere but on the last line.
//! - A line containing a single expression is a valid statement and has the same effect as `_ <- expr`.
//!
//! # How do I make my monad works with `doo`?
//!
//! You have to implement two traits: [`Pointed`] and [`Bind`]. Feel free to have a look at their
//! documentation for further information.
//!
//! # First example: fallible code
//!
//! One of the first monadic application that people learn is the _fallible_ effect — `Maybe` in Haskell.
//! In `Rust`, it’s `Option`. `Option` is an interesting monad as it allows you to fail early.
//!
//! ```rust
//! use doo::doo;
//!
//! let r = doo! {
//!   x <- Some("Hello, world!");
//!   y <- Some(3);
//!   Some(x.len() * y)
//! };
//!
//! assert_eq!(r, Some(39));
//! ```
//!
//! The `binding <- expr` syntax unwraps the right part and binds it to `binding`, making it available to
//! next calls. The final line re-enter the structure (here, `Option`) explicitly.
//!
//! Note that it is possible to re-enter the structure without having to specify how (with `Option`, you
//! re-enter with `Some`). You can use the `return` keyword, that will automatically lift the value into
//! the right structure:
//!
//! ```rust
//! use doo::doo;
//!
//! let r = doo! {
//!   x <- Some(1);
//!   y <- Some(2);
//!   z <- Some(3);
//!   return [x, y, z];
//! };
//!
//! assert_eq!(r, Some([1, 2, 3]));
//! ```

#[macro_export]
macro_rules! doo {
  // return
  (return $r:expr ;) => {
    $crate::Pointed::point($r)
  };

  // const-bind
  (_ <- $x:expr ; $($r:tt)*) => {
    $crate::Bind::bind($x, |_| { doo!($($r)*) })
  };

  // bind
  ($binding:ident <- $x:expr ; $($r:tt)*) => {
    $crate::Bind::bind($x, |$binding| { doo!($($r)*) })
  };

  // const-bind
  ($e:expr ; $($a:tt)*) => {
    $crate::Bind::bind($e, |_| doo!($($a)*))
  };

  // pure
  ($a:expr) => {
    $a
  }
}

/// Pointed functors.
///
/// In simple terms, a pointed functor lifts a value into an object that adds a _default_ structure
/// to it.
pub trait Pointed<A> {
  /// Lift a value into a default structure.
  fn point(a: A) -> Self;
}

impl<A> Pointed<A> for Option<A> {
  fn point(a: A) -> Self {
    Some(a)
  }
}

impl<A, E> Pointed<A> for Result<A, E> {
  fn point(a: A) -> Self {
    Ok(a)
  }
}

/// Monadic bind.
///
/// The monadi bind is a simple function that temporarily removes the structure and exposes a value to a
/// lambda (λ). That λ then must re-enter the structure, eventually changing the type of the inner value.
/// When the the lambda is called, it is passed as argument the previously unwrapped value, which then is
/// _bound_ for the whole lifetime of the lambda.
///
/// Because Rust lacks kinds, the definition of [`Bind`] is a bit convoluted:
///
/// - The associated `A` type variable represents the value that will be unwrapped and presented to the
///   λ.
/// - The `B` value is the type the λ can switch to for the inner value.
/// - The `Output` associated type variable is the final type after re-entering the structure. Rust doesn’t
///   allow us to state that it must be the same structure as `Self`, but if we had kinds, you can imagine
///   that the lambda is passed the `A` from `Self<A>` and `Output` is akin to `Self<B>`.
/// - The λ, called `F`, is pretty straight-forward.
pub trait Bind<B> {
  type A;
  type Output;

  fn bind<F>(self, f: F) -> Self::Output
  where
    F: FnOnce(Self::A) -> Self::Output;
}

impl<A, B> Bind<B> for Option<A> {
  type A = A;
  type Output = Option<B>;

  fn bind<F>(self, f: F) -> Self::Output
  where
    F: FnOnce(Self::A) -> Self::Output,
  {
    self.and_then(f)
  }
}

impl<A, B, E> Bind<B> for Result<A, E> {
  type A = A;
  type Output = Result<B, E>;

  fn bind<F>(self, f: F) -> Self::Output
  where
    F: FnOnce(Self::A) -> Self::Output,
  {
    self.and_then(f)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn option() {
    let r: Option<i32> = doo! {
      v <- Some(3);
      Some(v)
    };

    assert_eq!(r, Some(3));

    let r: Option<i32> = doo! {
      v <- r;
      x <- Some(10);
      Some(v * x)
    };

    assert_eq!(r, Some(30));

    let n: Option<i32> = None;
    let r: Option<i32> = doo! {
      v <- Some(314);
      x <- n;
      Some(v * x)
    };

    assert_eq!(r, None);

    let r = doo! {
      _ <- Some("a");
      b <- Some("b");
      _ <- Option::<&str>::None;
      Some(b)
    };

    assert_eq!(r, None);

    let r = doo! {
      _ <- Some("a");
      return "b";
    };

    assert_eq!(r, Some("b"));
  }

  #[test]
  fn result() {
    let r: Result<i32, &str> = doo! {
      v <- Ok(3);
      Ok(v)
    };

    assert_eq!(r, Ok(3));

    let r: Result<i32, &str> = doo! {
      v <- r;
      x <- Ok(10);
      Ok(v * x)
    };

    assert_eq!(r, Ok(30));

    let n: Result<i32, &str> = Err("error");
    let r: Result<i32, &str> = doo! {
      v <- Ok(314);
      x <- n;
      Ok(v * x)
    };

    assert_eq!(r, Err("error"));

    let r = doo! {
      _ <- Result::<&str, &str>::Ok("a");
      b <- Ok("b");
      _ <- Result::<&str, &str>::Err("nope");
      Ok(b)
    };

    assert_eq!(r, Err("nope"));

    fn guard<E>(cond: bool, err: E) -> Result<(), E> {
      if cond {
        Ok(())
      } else {
        Err(err)
      }
    }

    let r = doo! {
      x <- Ok(true);
      _ <- guard(1 == 2, "meh");
      Ok(x)
    };

    assert_eq!(r, Err("meh"));
  }

  #[test]
  fn instruction_counter() {
    struct IC<A> {
      count: usize,
      value: A,
    }

    impl<A> IC<A> {
      fn new(value: A) -> Self {
        IC { count: 1, value }
      }

      fn value(&self) -> &A {
        &self.value
      }

      fn count(&self) -> usize {
        self.count
      }
    }

    impl<A> Pointed<A> for IC<A> {
      fn point(a: A) -> Self {
        IC::new(a)
      }
    }

    impl<A, B> Bind<B> for IC<A> {
      type A = A;
      type Output = IC<B>;

      fn bind<F>(self, f: F) -> Self::Output
      where
        F: FnOnce(Self::A) -> Self::Output,
      {
        let r = f(self.value);

        IC {
          count: self.count + r.count,
          value: r.value,
        }
      }
    }

    let ic = doo! {
      a <- IC::new(10);
      b <- IC::new(2);
      IC::new(a + b)
    };

    assert_eq!(ic.value(), &12);
    assert_eq!(ic.count(), 3);

    let ic = doo! {
      _ <- IC::new("a");
      return [1, 2, 3];
    };

    assert_eq!(ic.value(), &[1, 2, 3]);
    assert_eq!(ic.count(), 2);
  }
}
