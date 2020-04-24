#![deny(
    unused_qualifications,
    clippy::all,
    unused_qualifications,
    unused_import_braces,
    unused_lifetimes,
    unreachable_pub,
    trivial_numeric_casts,
    rustdoc,
    missing_debug_implementations,
    missing_copy_implementations,
    deprecated_in_future,
    meta_variable_misuse,
    non_ascii_idents,
    rust_2018_compatibility,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style
)]
#![warn(clippy::perf, clippy::single_match_else, clippy::dbg_macro)]
#![allow(
    clippy::missing_inline_in_public_items,
    clippy::cognitive_complexity,
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::as_conversions
)]

pub mod builtins;
pub mod environment;
pub mod exec;
pub mod realm;
pub mod syntax;
#[cfg(feature = "wasm-bindgen")]
mod wasm;

#[cfg(feature = "wasm-bindgen")]
pub use crate::wasm::*;
use crate::{
    builtins::value::ResultValue,
    exec::{Executor, Interpreter},
    realm::Realm,
    syntax::{ast::node::Node, lexer::Lexer, parser::Parser},
};
use gc::{Finalize, Trace};
use std::{num::NonZeroUsize, usize};
use string_interner::{StringInterner, Symbol};

#[cfg(feature = "serde-ast")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "serde-ast")]
pub use serde_json;

/// Internal type for the string interner.
type Interner = StringInterner<InternerSym>;

/// Symbol used for the internal string interner.
#[cfg_attr(feature = "serde-ast", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct InternerSym {
    val: NonZeroUsize,
}

impl Symbol for InternerSym {
    /// Creates an `InternerSym` from the given `usize`.
    ///
    /// # Panics
    ///
    /// If the given `usize` is `usize::MAX`.
    fn from_usize(val: usize) -> Self {
        assert!(
            val != usize::MAX,
            "symbol value {} is too large and not supported by `InternerSym` type",
            val
        );
        InternerSym {
            val: NonZeroUsize::new(val + 1).unwrap_or_else(|| {
                unreachable!("should never fail because `val + 1` is nonzero and `<= usize::MAX`")
            }),
        }
    }

    fn to_usize(self) -> usize {
        self.val.get() - 1
    }
}

impl Finalize for InternerSym {}
unsafe impl Trace for InternerSym {
    #[inline]
    unsafe fn trace(&self) {}
    #[inline]
    unsafe fn root(&self) {}
    #[inline]
    unsafe fn unroot(&self) {}
    #[inline]
    fn finalize_glue(&self) {
        Finalize::finalize(self)
    }
}

fn parser_expr(src: &str) -> Result<Node, String> {
    let mut lexer = Lexer::new(src);
    lexer.lex().map_err(|e| format!("SyntaxError: {}", e))?;
    let tokens = lexer.tokens;
    Parser::new(&tokens, lexer.interner)
        .parse_all()
        .map_err(|e| format!("ParsingError: {}", e))
}

/// Execute the code using an existing Interpreter
/// The str is consumed and the state of the Interpreter is changed
pub fn forward(engine: &mut Interpreter, src: &str) -> String {
    // Setup executor
    let expr = match parser_expr(src) {
        Ok(v) => v,
        Err(error_string) => {
            return error_string;
        }
    };
    let result = engine.run(&expr);
    match result {
        Ok(v) => v.to_string(),
        Err(v) => format!("{}: {}", "Error", v.to_string()),
    }
}

/// Execute the code using an existing Interpreter.
/// The str is consumed and the state of the Interpreter is changed
/// Similar to `forward`, except the current value is returned instad of the string
/// If the interpreter fails parsing an error value is returned instead (error object)
pub fn forward_val(engine: &mut Interpreter, src: &str) -> ResultValue {
    // Setup executor
    match parser_expr(src) {
        Ok(expr) => engine.run(&expr),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

/// Create a clean Interpreter and execute the code
pub fn exec(src: &str) -> String {
    // Create new Realm
    let mut interner = Interner::new();
    let realm = Realm::create(&mut interner);
    let mut engine: Interpreter = Executor::new(realm, interner);
    forward(&mut engine, src)
}