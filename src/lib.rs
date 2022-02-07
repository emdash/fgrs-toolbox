// The MIT License (MIT)
//
// Copyright © 2022 <Brandon Lewis>
//
// Permission is hereby granted, free of charge, to any person
// obtaining a copy of this software and associated documentation
// files (the “Software”), to deal in the Software without
// restriction, including without limitation the rights to use, copy,
// modify, merge, publish, distribute, sublicense, and/or sell copies
// of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be
// included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS
// BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN
// ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
// CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
//
// Fork this project to create your own MIT license that you can
// always link to.


/**
 * What follows is a hairbrained re-inrerpretation of both the SML
 * code listing and the formal algorithm presented in the PDF.
 *
 * A straight-forward port of the SML was just too cumbersome to write
 * in safe rust.
 */

use core::fmt::Debug;


/**
 * Should this be its own crate? Or a macro?
 */
fn debug<T: Debug>(prefix: &str, value: T) {
    eprintln!("{}: {:?}", prefix, value);
}


/**
 * Captures values and operations external to pure lambda calculus.
 *
 * See tests for an examples of how this is used.
 */
pub trait SigmaRules: Sized {
    type Error: Sized + Debug;
    fn apply(f: Self, x: Self) -> Result<Self, Self::Error>;
}


/**
 * A container for various trait bounds.
 *
 * This gives us some parametricity without having where clauses
 * proliferate everywhere.
 */
pub trait Types {
    // A type which represents a "constant" value in the lambda calc.
    type Val: Debug + Clone + SigmaRules;
    // A type which represents a "symbol" in the lambda calc, usually
    // String. But if you want to replace this with an integer, or a
    // custom type, you can.
    type Sym: Debug + Clone + PartialEq;
}


/**
 * This is the abstract I/O format: Flat token sequences which
 * represent a postfix encoding lambda calculus. Postfix is used here
 * for the usual reasons: it is unambiguous, compact, and trivial to
 * evaluate.
 *
 * V is the value type, for constant values. S is the "symbol" type,
 * for identifiers.
 *
 * Example: `\x.x` becomes the sequence `[Id("x"), Id("x"), Lambda]`,
 * if S is `&'static str`.
 */
#[derive(Clone, Debug)]
pub enum Token<T: Types>
{
    Id(T::Sym),
    Val(T::Val),
    Lambda,
    Apply,
}

impl<T: Types> Token<T> {
    pub fn id<B>(name: B) -> Token<T> where B: Into<T::Sym> {
        Token::Id(name.into())
    }
}


/**
 * Just to get oriented, we start with a simple lambda expression
 * parser and evaluator.
 */
mod expr {

use core::iter::Iterator;
use core::fmt::Debug;
use super::{Token, Types};


/**
 * This ADT abstracts over classic lambda expression trees.
 *
 */
#[derive(Clone, Debug, PartialEq)]
pub enum Expr<T: Types> {
    Lambda(T::Sym, Box<Expr<T>>),
    Val(T::Val),
    Var(T::Sym),
    App(Box<Expr<T>>, Box<Expr<T>>)
}

#[derive(Debug)]
pub enum ParseError<T: Types> {
    Unexpected(Token<T>),
    Mismatched,
    Underflow,
    NotAVar,
    EOF
}


/**
 * Abstract over different ways of implementing an environment.
 */
pub trait Env<T: Types> {
    fn subst(&self, name: T::Sym) -> Expr<T>;
}


type Result<V, T> = core::result::Result<V, ParseError<T>>;


impl<'a, T: 'a> Expr<T> where T: Types + Clone {
    pub fn val<B>(v: B) -> Box<Self>
    where B: Into<T::Val> {
        Box::new(Expr::Val(v.into()))
    }

    pub fn lambda<B>(arg: B, body: Box<Self>) -> Box<Self>
    where B: Into<T::Sym> {
        Box::new(Expr::Lambda(arg.into(), body))
    }

    pub fn var<B>(name: B) -> Box<Self>
    where B: Into<T::Sym> {
        Box::new(Expr::Var(name.into()))
    }

    pub fn apply(func: Box<Self>, arg: Box<Self>) -> Box<Self> {
        Box::new(Expr::App(func, arg))
    }

    pub fn beta_reduce(self) -> Box<Self> {
        match self {
            Self::App(f, x) => if let Self::Lambda(a, b) = *f {
                b.subst(a, x)
            } else {
                panic!("not a function!");
            }
            _ => panic!("not reducible"),
        }
    }

    pub fn subst(self, var: T::Sym, exp: Box<Self>) -> Box<Self> {
        match self {
            Self::Var(v)       if v == var => exp.clone(),
            Self::Lambda(a, _) if a == var => {panic!("Identifier conflic");},
            Self::Lambda(a, b)             => Box::new(Self::Lambda(a, b.subst(var, exp))),
            Self::App(f, x)                => Box::new(Self::App(
                f.subst(var.clone(), exp.clone()),
                x.subst(var, exp))),
            x                              => Box::new(x)
        }
    }

    pub fn parse(
        input: impl Iterator<Item = &'a Token<T>>
    ) -> Result<Box<Self>, T> {
        let mut stack: Vec<Box<Self>> = Vec::new();

        for token in input { match token {
            // XXX: suspicious use of clone.
            Token::Val(v) => stack.push(Self::val(v.clone())),
            Token::Id(s)  => stack.push(Expr::var(s.clone())),
            Token::Lambda => {
                let body = stack.pop().ok_or(ParseError::Underflow)?;
                let arg = stack.pop().ok_or(ParseError::Underflow)?;
                // XXX: suspicious suspicious move.
                if let Expr::Var(s) = *arg {
                    stack.push(Expr::lambda(s, body));
                } else {
                    return Err(ParseError::NotAVar);
                }
            },
            Token::Apply  => {
                let arg = stack.pop().unwrap();
                let func = stack.pop().unwrap();
                stack.push(Expr::apply(func, arg));
            }
        } }

        if stack.len() == 1 {
            Ok(stack.pop().ok_or(ParseError::Underflow)?)
        } else {
            // If we got here and there's not exactly one value on the
            // stack, the program is incomplete
            Err(ParseError::EOF)
        }
    }
}

} /* mod expr */


#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use super::*;
    use super::expr::*;

    /* This shows how to implement Types for this crate */
    #[derive(Clone, Debug, PartialEq)]
    struct MyTypes;

    impl Types for MyTypes {
        type Val = i32;
        type Sym = String;
    }

    impl SigmaRules for i32 {
        type Error = ();
        // applying one int to another is nonsense
        fn apply(f: i32, x: i32) -> Result<i32, Self::Error> {
            Err(())
        }
    }

    type Tok = Token<MyTypes>;
    type Exp = Expr<MyTypes>;

    impl Env<MyTypes> for HashMap<String, Expr<MyTypes>> {
        fn subst(&self, name: String) -> Expr<MyTypes> {
            if let Some(val) = self.get(&name) {
                val.clone()
            } else {
                Expr::Var(name)
            }
        }
    }

    #[test]
    fn test_parse_simple0() {
        let got = Expr::parse(vec![
            Tok::id("x"),
            Tok::id("y"),
            Tok::Apply
        ].iter()).unwrap();

        let expected = Expr::apply(Expr::var("x"), Expr::var("y"));
        assert_eq!(got, expected);
    }

    #[test]
    fn test_parse_simple1() {
        let got = Expr::parse(vec![
            Tok::id("x"),
            Tok::id("y"),
            Tok::Lambda,
        ].iter()).unwrap();

        let expected = Expr::lambda("x", Expr::var("y"));
        assert_eq!(got, expected);
    }

    #[test]
    fn test_parse_simple2() {
        let got = Expr::parse(vec![
            Tok::id("x"),
            Tok::id("y"),
            Tok::Lambda,
            Tok::id("z"),
            Tok::Apply,
        ].iter()).unwrap();

        let expected = Expr::apply(
            Expr::lambda(
                "x".to_string(),
                Expr::var("y".to_string())
            ),
            Expr::var("z".to_string())
        );

        assert_eq!(got, expected);
    }

    #[test]
    fn test_beta_reduction() {
        type E = Exp;

        // (\x.x) 0 -b-> 0
        assert_eq!(
            E::apply(E::lambda("x", E::var("x")), E::val(0)).beta_reduce(),
            E::val(0)
        );

        // (\x.(\y.x)) 0 -b-> (\y.0)
        assert_eq!(
            E::apply(
                E::lambda("x",
                          E::lambda("y",
                                    E::var("x"))),
                E::val(0)).beta_reduce(),
            E::lambda("y", E::val(0))
        );

        // (\f.f 0) (\x.x) -b-> (\x.x) 0 -b-> 0
        assert_eq!(
            E::apply(
                E::lambda("f", E::apply(E::var("f"), E::val(0))),
                E::lambda("x", E::var("x")))
                .beta_reduce()
                .beta_reduce(),
            E::val(0)
        )
    }
}
