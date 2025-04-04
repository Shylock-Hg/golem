// Copyright 2024-2025 Golem Cloud
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::Expr;

// We assign unique variable identifiers to the identifiers present in the match arm literals,
// and ensuring they get propagated to the usage sites within resolution expressions. Here
// we make sure to replace global variable or local variable identifiers with match-arm identifiers (VariableId enum)
// to prevent conflicts with other local let bindings
// or global variables, thereby maintaining clear variable scoping and avoiding unintended clashes.
pub fn bind_variables_of_pattern_match(expr: &mut Expr) {
    internal::bind_variables(expr, 0, &mut []);
}

mod internal {
    use crate::{ArmPattern, Expr, ExprVisitor, MatchArm, MatchIdentifier, VariableId};
    use std::collections::VecDeque;

    pub(crate) fn bind_variables(
        expr: &mut Expr,
        previous_index: usize,
        match_identifiers: &mut [MatchIdentifier],
    ) -> usize {
        let mut index = previous_index;
        let mut queue = ExprVisitor::top_down(expr);
        let mut shadowed_let_binding = vec![];

        // Start from the end
        while let Some(expr) = queue.pop_front() {
            match expr {
                Expr::PatternMatch { match_arms, .. } => {
                    for arm in match_arms {
                        // We increment the index for each arm regardless of whether there is an identifier exist or not
                        index += 1;
                        let latest = process_arm(arm, index);
                        // An arm can increment the index if there are nested pattern match arms, and therefore
                        // set it to the latest max.
                        index = latest
                    }
                }
                Expr::Let { variable_id, .. } => {
                    shadowed_let_binding.push(variable_id.name());
                }
                Expr::Identifier { variable_id, .. } => {
                    let identifier_name = variable_id.name();
                    if let Some(x) = match_identifiers.iter().find(|x| x.name == identifier_name) {
                        if !shadowed_let_binding.contains(&identifier_name) {
                            *variable_id = VariableId::MatchIdentifier(x.clone());
                        }
                    }
                }

                _ => {}
            }
        }

        index
    }

    fn process_arm(match_arm: &mut MatchArm, global_arm_index: usize) -> usize {
        let match_arm_pattern = &mut match_arm.arm_pattern;

        pub fn go(
            arm_pattern: &mut ArmPattern,
            global_arm_index: usize,
            match_identifiers: &mut Vec<MatchIdentifier>,
        ) {
            match arm_pattern {
                ArmPattern::Literal(expr) => {
                    let new_match_identifiers =
                        update_all_identifier_in_lhs_expr(expr, global_arm_index);
                    match_identifiers.extend(new_match_identifiers);
                }

                ArmPattern::WildCard => {}
                ArmPattern::As(name, arm_pattern) => {
                    let match_identifier = MatchIdentifier::new(name.clone(), global_arm_index);
                    match_identifiers.push(match_identifier);

                    go(arm_pattern, global_arm_index, match_identifiers);
                }

                ArmPattern::Constructor(_, arm_patterns) => {
                    for arm_pattern in arm_patterns {
                        go(arm_pattern, global_arm_index, match_identifiers);
                    }
                }

                ArmPattern::TupleConstructor(arm_patterns) => {
                    for arm_pattern in arm_patterns {
                        go(arm_pattern, global_arm_index, match_identifiers);
                    }
                }

                ArmPattern::ListConstructor(arm_patterns) => {
                    for arm_pattern in arm_patterns {
                        go(arm_pattern, global_arm_index, match_identifiers);
                    }
                }

                ArmPattern::RecordConstructor(fields) => {
                    for (_, arm_pattern) in fields {
                        go(arm_pattern, global_arm_index, match_identifiers);
                    }
                }
            }
        }

        let mut match_identifiers = vec![];

        // Recursively identify the arm within an arm literal
        go(match_arm_pattern, global_arm_index, &mut match_identifiers);

        let resolution_expression = &mut *match_arm.arm_resolution_expr;

        // Continue with original pattern_match_name_binding for resoution expressions
        // to target nested pattern matching.
        bind_variables(
            resolution_expression,
            global_arm_index,
            &mut match_identifiers,
        )
    }

    fn update_all_identifier_in_lhs_expr(
        expr: &mut Expr,
        global_arm_index: usize,
    ) -> Vec<MatchIdentifier> {
        let mut identifier_names = vec![];
        let mut queue = VecDeque::new();
        queue.push_front(expr);

        while let Some(expr) = queue.pop_front() {
            match expr {
                Expr::Identifier { variable_id, .. } => {
                    let match_identifier =
                        MatchIdentifier::new(variable_id.name(), global_arm_index);
                    identifier_names.push(match_identifier);
                    let new_variable_id =
                        VariableId::match_identifier(variable_id.name(), global_arm_index);
                    *variable_id = new_variable_id;
                }

                _ => {
                    expr.visit_children_mut_top_down(&mut queue);
                }
            }
        }

        identifier_names
    }
}

#[cfg(test)]
mod pattern_match_bindings {
    use test_r::test;

    use crate::{Expr, InferredType};
    use expectations::*;

    #[test]
    fn test_simple_pattern_match_name_binding() {
        // The first x is global and the second x is a match binding
        let expr_string = r#"
          match some(x) {
            some(x) => x,
            none => 0
          }
        "#;

        let mut expr = Expr::from_text(expr_string).unwrap();

        expr.bind_variables_of_pattern_match();

        assert_eq!(expr, expected_match(1));
    }

    #[test]
    fn test_simple_pattern_match_name_binding_with_shadow() {
        // The first x is global and the second x is a match binding
        let expr_string = r#"
          match some(x) {
            some(x) => {
              let x = 1;
              x
            },
            none => 0
          }
        "#;

        let mut expr = Expr::from_text(expr_string).unwrap();

        expr.bind_variables_of_pattern_match();

        assert_eq!(expr, expected_match_with_let_binding(1));
    }

    #[test]
    fn test_simple_pattern_match_name_binding_block() {
        // The first x is global and the second x is a match binding
        let expr_string = r#"
          match some(x) {
            some(x) => x,
            none => 0
          };

          match some(x) {
            some(x) => x,
            none => 0
          }
        "#;

        let mut expr = Expr::from_text(expr_string).unwrap();

        expr.bind_variables_of_pattern_match();

        let first_expr = expected_match(1);
        let second_expr = expected_match(3); // 3 because first block has 2 arms

        let block = Expr::expr_block(vec![first_expr, second_expr])
            .with_inferred_type(InferredType::Unknown);

        assert_eq!(expr, block);
    }

    #[test]
    fn test_nested_simple_pattern_match_binding() {
        let expr_string = r#"
          match ok(some(x)) {
            ok(x) => match x {
              some(x) => x,
              none => 0
            },
            err(x) => 0
          }
        "#;

        let mut expr = Expr::from_text(expr_string).unwrap();

        expr.bind_variables_of_pattern_match();

        assert_eq!(expr, expected_nested_match());
    }

    mod expectations {
        use crate::{ArmPattern, Expr, InferredType, MatchArm, MatchIdentifier, VariableId};
        use bigdecimal::BigDecimal;

        pub(crate) fn expected_match(index: usize) -> Expr {
            Expr::pattern_match(
                Expr::option(Some(Expr::identifier_global("x", None)))
                    .with_inferred_type(InferredType::Option(Box::new(InferredType::Unknown))),
                vec![
                    MatchArm {
                        arm_pattern: ArmPattern::constructor(
                            "some",
                            vec![ArmPattern::literal(Expr::identifier_with_variable_id(
                                VariableId::MatchIdentifier(MatchIdentifier::new(
                                    "x".to_string(),
                                    index,
                                )),
                                None,
                            ))],
                        ),
                        arm_resolution_expr: Box::new(Expr::identifier_with_variable_id(
                            VariableId::MatchIdentifier(MatchIdentifier::new(
                                "x".to_string(),
                                index,
                            )),
                            None,
                        )),
                    },
                    MatchArm {
                        arm_pattern: ArmPattern::constructor("none", vec![]),
                        arm_resolution_expr: Box::new(Expr::number(BigDecimal::from(0))),
                    },
                ],
            )
        }

        pub(crate) fn expected_match_with_let_binding(index: usize) -> Expr {
            let let_binding = Expr::let_binding("x", Expr::number(BigDecimal::from(1)), None);
            let identifier_expr =
                Expr::identifier_with_variable_id(VariableId::Global("x".to_string()), None);
            let block = Expr::expr_block(vec![let_binding, identifier_expr]);

            Expr::pattern_match(
                Expr::option(Some(Expr::identifier_global("x", None))),
                vec![
                    MatchArm {
                        arm_pattern: ArmPattern::constructor(
                            "some",
                            vec![ArmPattern::literal(Expr::identifier_with_variable_id(
                                VariableId::MatchIdentifier(MatchIdentifier::new(
                                    "x".to_string(),
                                    index,
                                )),
                                None,
                            ))],
                        ),
                        arm_resolution_expr: Box::new(block),
                    },
                    MatchArm {
                        arm_pattern: ArmPattern::constructor("none", vec![]),
                        arm_resolution_expr: Box::new(Expr::number(BigDecimal::from(0))),
                    },
                ],
            )
        }

        pub(crate) fn expected_nested_match() -> Expr {
            Expr::pattern_match(
                Expr::ok(
                    Expr::option(Some(Expr::identifier_with_variable_id(
                        VariableId::Global("x".to_string()),
                        None,
                    )))
                    .with_inferred_type(InferredType::Option(Box::new(InferredType::Unknown))),
                    None,
                )
                .with_inferred_type(InferredType::Result {
                    ok: Some(Box::new(InferredType::Option(Box::new(
                        InferredType::Unknown,
                    )))),
                    error: Some(Box::new(InferredType::Unknown)),
                }),
                vec![
                    MatchArm {
                        arm_pattern: ArmPattern::constructor(
                            "ok",
                            vec![ArmPattern::literal(Expr::identifier_with_variable_id(
                                VariableId::MatchIdentifier(MatchIdentifier::new(
                                    "x".to_string(),
                                    1,
                                )),
                                None,
                            ))],
                        ),
                        arm_resolution_expr: Box::new(Expr::pattern_match(
                            Expr::identifier_with_variable_id(
                                VariableId::MatchIdentifier(MatchIdentifier::new(
                                    "x".to_string(),
                                    1,
                                )),
                                None,
                            ),
                            vec![
                                MatchArm {
                                    arm_pattern: ArmPattern::constructor(
                                        "some",
                                        vec![ArmPattern::literal(
                                            Expr::identifier_with_variable_id(
                                                VariableId::MatchIdentifier(MatchIdentifier::new(
                                                    "x".to_string(),
                                                    5,
                                                )),
                                                None,
                                            ),
                                        )],
                                    ),
                                    arm_resolution_expr: Box::new(
                                        Expr::identifier_with_variable_id(
                                            VariableId::MatchIdentifier(MatchIdentifier::new(
                                                "x".to_string(),
                                                5,
                                            )),
                                            None,
                                        ),
                                    ),
                                },
                                MatchArm {
                                    arm_pattern: ArmPattern::constructor("none", vec![]),
                                    arm_resolution_expr: Box::new(Expr::number(BigDecimal::from(
                                        0,
                                    ))),
                                },
                            ],
                        )),
                    },
                    MatchArm {
                        arm_pattern: ArmPattern::constructor(
                            "err",
                            vec![ArmPattern::literal(Expr::identifier_with_variable_id(
                                VariableId::MatchIdentifier(MatchIdentifier::new(
                                    "x".to_string(),
                                    4,
                                )),
                                None,
                            ))],
                        ),
                        arm_resolution_expr: Box::new(Expr::number(BigDecimal::from(0))),
                    },
                ],
            )
        }
    }
}
