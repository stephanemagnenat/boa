use crate::{
    syntax::{
        ast::{keyword::Keyword, node::Node, punc::Punctuator, token::TokenKind},
        parser::{
            statement::Statement, AllowAwait, AllowReturn, AllowYield, Cursor, Expression,
            ParseError, ParseResult, TokenParser,
        },
    },
    Interner,
};

/// Do...while statement parsing
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/do...while
/// [spec]: https://tc39.es/ecma262/#sec-do-while-statement
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser::statement) struct DoWhileStatement {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    allow_return: AllowReturn,
}

impl DoWhileStatement {
    /// Creates a new `DoWhileStatement` parser.
    pub(in crate::syntax::parser::statement) fn new<Y, A, R>(
        allow_yield: Y,
        allow_await: A,
        allow_return: R,
    ) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
        R: Into<AllowReturn>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            allow_return: allow_return.into(),
        }
    }
}

impl TokenParser for DoWhileStatement {
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<'_>, interner: &mut Interner) -> ParseResult {
        cursor.expect(Keyword::Do, "do while statement", interner)?;

        let body = Statement::new(self.allow_yield, self.allow_await, self.allow_return)
            .parse(cursor, interner)?;

        let next_token = cursor
            .peek_skip_lineterminator()
            .ok_or(ParseError::AbruptEnd)?;

        if next_token.kind != TokenKind::Keyword(Keyword::While) {
            return Err(ParseError::expected(
                vec![Keyword::While.to_string()],
                next_token.display(interner).to_string(),
                next_token.pos,
                "do while statement",
            ));
        }

        cursor.skip(|tk| tk.kind == TokenKind::LineTerminator);

        cursor.expect(Keyword::While, "do while statement", interner)?;
        cursor.expect(Punctuator::OpenParen, "do while statement", interner)?;

        let cond =
            Expression::new(true, self.allow_yield, self.allow_await).parse(cursor, interner)?;

        cursor.expect(Punctuator::CloseParen, "do while statement", interner)?;
        cursor.expect_semicolon(true, "do while statement", interner)?;

        Ok(Node::do_while_loop(body, cond))
    }
}