//! Tests for the parser.

use super::Parser;
use crate::{
    syntax::{ast::node::Node, ast::op::NumOp, lexer::Lexer},
    Interner,
};

#[allow(clippy::result_unwrap_used)]
pub(super) fn check_parser(js: &str, expr: &[Node], interner: Interner) {
    let mut lexer = Lexer::new_with_interner(js, interner);
    lexer.lex().expect("failed to lex");

    assert_eq!(
        Parser::new(&lexer.tokens, lexer.interner)
            .parse_all()
            .expect("failed to parse"),
        Node::statement_list(expr)
    );
}

pub(super) fn check_invalid(js: &str) {
    let mut lexer = Lexer::new(js);
    lexer.lex().expect("failed to lex");

    assert!(Parser::new(&lexer.tokens, lexer.interner)
        .parse_all()
        .is_err());
}

/// Should be parsed as `new Class().method()` instead of `new (Class().method())`
#[test]
fn check_construct_call_precedence() {
    let mut int = Interner::new();
    check_parser(
        "new Date().getTime()",
        &[Node::call(
            Node::get_const_field(
                Node::new(Node::call(
                    Node::local(int.get_or_intern("Date")),
                    Vec::new(),
                )),
                int.get_or_intern("getTime"),
            ),
            Vec::new(),
        )],
        int,
    );
}

#[test]
fn assing_operator_precedence() {
    let mut int = Interner::new();
    let a_sym = int.get_or_intern("a");

    check_parser(
        "a = a + 1",
        &[Node::assign(
            Node::local(a_sym),
            Node::bin_op(NumOp::Add, Node::local(a_sym), Node::const_node(1.0)),
        )],
        int,
    );
}