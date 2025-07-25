// Copyright 2024-2025 Golem Cloud
//
// Licensed under the Golem Source License v1.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://license.golem.cloud/LICENSE
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use combine::{
    between,
    parser::char::{char as char_, spaces},
    ParseError, Parser,
};

use crate::expr::Expr;
use crate::parser::errors::RibParseError;
use crate::rib_source_span::GetSourcePosition;

pub fn multi_line_block<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: combine::Stream<Token = char>,
    RibParseError: Into<
        <Input::Error as ParseError<Input::Token, Input::Range, Input::Position>>::StreamError,
    >,
    Input::Position: GetSourcePosition,
{
    spaces().with(between(
        char_('{').skip(spaces()),
        char_('}').skip(spaces()),
        internal::block().skip(spaces()),
    ))
}

mod internal {
    use combine::parser::char::{char, spaces};
    use combine::{sep_by, ParseError, Parser};

    use crate::parser::errors::RibParseError;
    use crate::parser::rib_expr::rib_expr;
    use crate::rib_source_span::GetSourcePosition;
    use crate::Expr;

    // A block is different to a complete rib-program that the it may not be the end of the stream
    pub fn block<Input>() -> impl Parser<Input, Output = Expr>
    where
        Input: combine::Stream<Token = char>,
        RibParseError: Into<
            <Input::Error as ParseError<Input::Token, Input::Range, Input::Position>>::StreamError,
        >,
        Input::Position: GetSourcePosition,
    {
        spaces().with(sep_by(rib_expr(), char(';').skip(spaces())).and_then(
            |expressions: Vec<Expr>| {
                if expressions.len() >= 2 {
                    Ok(if expressions.len() == 1 {
                        expressions.first().unwrap().clone()
                    } else {
                        Expr::expr_block(expressions)
                    })
                } else {
                    Err(RibParseError::Message(
                        "block should have at least two elements separated by semicolons"
                            .to_string(),
                    ))
                }
            },
        ))
    }
}
#[cfg(test)]
mod tests {
    use bigdecimal::BigDecimal;
    use test_r::test;

    use crate::expr::Expr;
    use crate::function_name::DynamicParsedFunctionName;
    use crate::{ArmPattern, MatchArm};

    #[test]
    fn test_block_parse() {
        let rib_expr = r#"
          {
            let x = 1;
            let y = 2;
            foo(x);
            foo(y)
          }
        "#;

        let expr = Expr::from_text(rib_expr).unwrap();

        let expected = Expr::expr_block(vec![
            Expr::let_binding("x", Expr::number(BigDecimal::from(1)), None),
            Expr::let_binding("y", Expr::number(BigDecimal::from(2)), None),
            Expr::call_worker_function(
                DynamicParsedFunctionName::parse("foo").unwrap(),
                None,
                None,
                vec![Expr::identifier_global("x", None)],
                None,
            ),
            Expr::call_worker_function(
                DynamicParsedFunctionName::parse("foo").unwrap(),
                None,
                None,
                vec![Expr::identifier_global("y", None)],
                None,
            ),
        ]);

        assert_eq!(expr, expected);
    }

    #[test]
    fn test_block_in_if_expr() {
        let rib_expr = r#"
          if true then {
            let x = 1;
            let y = 2;
            foo(x);
            foo(y)
          } else 1
        "#;

        let expr = Expr::from_text(rib_expr).unwrap();

        let expected = Expr::cond(
            Expr::boolean(true),
            Expr::expr_block(vec![
                Expr::let_binding("x", Expr::number(BigDecimal::from(1)), None),
                Expr::let_binding("y", Expr::number(BigDecimal::from(2)), None),
                Expr::call_worker_function(
                    DynamicParsedFunctionName::parse("foo").unwrap(),
                    None,
                    None,
                    vec![Expr::identifier_global("x", None)],
                    None,
                ),
                Expr::call_worker_function(
                    DynamicParsedFunctionName::parse("foo").unwrap(),
                    None,
                    None,
                    vec![Expr::identifier_global("y", None)],
                    None,
                ),
            ]),
            Expr::number(BigDecimal::from(1)),
        );

        assert_eq!(expr, expected);
    }

    #[test]
    fn test_block_in_match_expr() {
        let rib_expr = r#"
          match foo {
           some(x) => {
              let x = 1;
              let y = 2;
              foo(x);
              foo(y)
            }
          }
        "#;

        let expr = Expr::from_text(rib_expr).unwrap();

        let expected = Expr::pattern_match(
            Expr::identifier_global("foo", None),
            vec![MatchArm::new(
                ArmPattern::Constructor(
                    "some".to_string(),
                    vec![ArmPattern::Literal(Box::new(Expr::identifier_global(
                        "x", None,
                    )))],
                ),
                Expr::expr_block(vec![
                    Expr::let_binding("x", Expr::number(BigDecimal::from(1)), None),
                    Expr::let_binding("y", Expr::number(BigDecimal::from(2)), None),
                    Expr::call_worker_function(
                        DynamicParsedFunctionName::parse("foo").unwrap(),
                        None,
                        None,
                        vec![Expr::identifier_global("x", None)],
                        None,
                    ),
                    Expr::call_worker_function(
                        DynamicParsedFunctionName::parse("foo").unwrap(),
                        None,
                        None,
                        vec![Expr::identifier_global("y", None)],
                        None,
                    ),
                ]),
            )],
        );

        assert_eq!(expr, expected);
    }

    #[test]
    fn test_nested_block() {
        let rib_expr = r#"
          let foo = some(1);
          match foo {
           some(x) => {
              let x = 1;
              let y = 2;
              foo(x);
              foo(y)
            }
          }
        "#;

        let expr = Expr::from_text(rib_expr).unwrap();

        let expected = Expr::expr_block(vec![
            Expr::let_binding(
                "foo",
                Expr::option(Some(Expr::number(BigDecimal::from(1)))),
                None,
            ),
            Expr::pattern_match(
                Expr::identifier_global("foo", None),
                vec![MatchArm::new(
                    ArmPattern::Constructor(
                        "some".to_string(),
                        vec![ArmPattern::Literal(Box::new(Expr::identifier_global(
                            "x", None,
                        )))],
                    ),
                    Expr::expr_block(vec![
                        Expr::let_binding("x", Expr::number(BigDecimal::from(1)), None),
                        Expr::let_binding("y", Expr::number(BigDecimal::from(2)), None),
                        Expr::call_worker_function(
                            DynamicParsedFunctionName::parse("foo").unwrap(),
                            None,
                            None,
                            vec![Expr::identifier_global("x", None)],
                            None,
                        ),
                        Expr::call_worker_function(
                            DynamicParsedFunctionName::parse("foo").unwrap(),
                            None,
                            None,
                            vec![Expr::identifier_global("y", None)],
                            None,
                        ),
                    ]),
                )],
            ),
        ]);

        assert_eq!(expr, expected);
    }
}
