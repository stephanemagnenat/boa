//! Call expression parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Functions
//! [spec]: https://tc39.es/ecma262/#prod-CallExpression

use super::arguments::Arguments;
use crate::syntax::lexer::TokenKind;
use crate::{
    syntax::{
        ast::{
            node::{
                field::{GetConstField, GetField},
                Call, Node,
            },
            Punctuator,
        },
        parser::{
            expression::Expression, AllowAwait, AllowYield, Cursor, ParseError, ParseResult,
            TokenParser,
        },
    },
    BoaProfiler,
};

use std::io::Read;

/// Parses a call expression.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-CallExpression
#[derive(Debug)]
pub(super) struct CallExpression {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    first_member_expr: Node,
}

impl CallExpression {
    /// Creates a new `CallExpression` parser.
    pub(super) fn new<Y, A>(allow_yield: Y, allow_await: A, first_member_expr: Node) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            first_member_expr,
        }
    }
}

impl<R> TokenParser<R> for CallExpression
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("CallExpression", "Parsing");

        let tk = cursor.peek();

        println!("Parse Call Expression Token: {:?}", tk);

        let mut lhs = match tk {
            Some(_) if tk.unwrap()?.kind() == &TokenKind::Punctuator(Punctuator::OpenParen) => {
                println!("Open paran");
                let args = Arguments::new(self.allow_yield, self.allow_await).parse(cursor)?;
                println!("After args: {:?}", cursor.peek());

                Node::from(Call::new(self.first_member_expr, args))
            }
            _ => {
                println!("Not open paran");
                let next_token = cursor.next().ok_or(ParseError::AbruptEnd)?;
                return Err(ParseError::expected(
                    vec![TokenKind::Punctuator(Punctuator::OpenParen)],
                    next_token?.clone(),
                    "call expression",
                ));
            }
        };

        println!("Parse Call Expression Finish Token: {:?}", cursor.peek());

        while let Some(tok) = cursor.peek() {
            let token = tok?.clone();
            println!("Call expression parsing... token: {:?}", token);
            match token.kind() {
                TokenKind::Punctuator(Punctuator::OpenParen) => {
                    let args = Arguments::new(self.allow_yield, self.allow_await).parse(cursor)?;
                    lhs = Node::from(Call::new(lhs, args));
                }
                TokenKind::Punctuator(Punctuator::Dot) => {
                    cursor.next().ok_or(ParseError::AbruptEnd)??; // We move the parser forward.

                    match &cursor.next().ok_or(ParseError::AbruptEnd)??.kind() {
                        TokenKind::Identifier(name) => {
                            lhs = GetConstField::new(lhs, name.clone()).into();
                        }
                        TokenKind::Keyword(kw) => {
                            lhs = GetConstField::new(lhs, kw.to_string()).into();
                        }
                        _ => {
                            return Err(ParseError::expected(
                                vec![TokenKind::identifier("identifier")],
                                token,
                                "call expression",
                            ));
                        }
                    }
                }
                TokenKind::Punctuator(Punctuator::OpenBracket) => {
                    let _ = cursor.next().ok_or(ParseError::AbruptEnd)?; // We move the parser.
                    let idx =
                        Expression::new(true, self.allow_yield, self.allow_await).parse(cursor)?;
                    cursor.expect(Punctuator::CloseBracket, "call expression")?;
                    lhs = GetField::new(lhs, idx).into();
                }
                _ => break,
            }
        }
        Ok(lhs)
    }
}
