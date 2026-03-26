//! Scenario parser — a small DSL for defining simulation scenarios.
//!
//! Provides:
//! - `parse_json` — parse a JSON scenario string into a `Scenario` struct
//! - `parse_dsl` — parse a FaultLab DSL source string into a `Scenario` struct
//! - `parse_dsl_with_failures` — parse DSL and return both scenario and scheduled failures

mod ast;
mod lexer;
mod parser;

use simulation_core::{FailureInjection, Scenario};

pub use ast::AstScenario;
pub use lexer::{LexError, Token, TokenKind};
pub use parser::{ParseError, Parser};

/// Parse a JSON scenario string into a Scenario struct.
pub fn parse_json(input: &str) -> Result<Scenario, String> {
    serde_json::from_str(input).map_err(|e| format!("Parse error: {}", e))
}

/// Parse the FaultLab DSL into a Scenario struct.
/// Scheduled failures are not included (they are injected at runtime).
pub fn parse_dsl(input: &str) -> Result<Scenario, String> {
    let ast = Parser::parse(input).map_err(|e| e.to_string())?;
    validate(&ast).map_err(|e| e.to_string())?;
    Ok(ast.to_scenario())
}

/// Parse the FaultLab DSL and return both the Scenario and scheduled failures.
/// Each scheduled failure is (time_ms, FailureInjection).
pub fn parse_dsl_with_failures(
    input: &str,
) -> Result<(Scenario, Vec<(u64, FailureInjection)>), String> {
    let ast = Parser::parse(input).map_err(|e| e.to_string())?;
    validate(&ast).map_err(|e| e.to_string())?;
    Ok((ast.to_scenario(), ast.scheduled_failures()))
}

/// Parse the DSL and return the full AST (for tools that need position info).
pub fn parse_dsl_ast(input: &str) -> Result<AstScenario, String> {
    let ast = Parser::parse(input).map_err(|e| e.to_string())?;
    validate(&ast).map_err(|e| e.to_string())?;
    Ok(ast)
}

/// Semantic validation — checks that references are valid, no duplicate IDs, etc.
fn validate(ast: &AstScenario) -> Result<(), ParseError> {
    use std::collections::HashSet;

    // Check for duplicate node IDs
    let mut seen_ids = HashSet::new();
    for node in &ast.nodes {
        if !seen_ids.insert(&node.id) {
            return Err(ParseError {
                message: format!("duplicate node id: \"{}\"", node.id),
                line: 0,
                col: 0,
            });
        }
    }

    // Check edge references
    let node_ids: HashSet<&str> = ast.nodes.iter().map(|n| n.id.as_str()).collect();
    for edge in &ast.edges {
        if !node_ids.contains(edge.from.as_str()) {
            return Err(ParseError {
                message: format!(
                    "edge references unknown node: \"{}\"",
                    edge.from
                ),
                line: 0,
                col: 0,
            });
        }
        if !node_ids.contains(edge.to.as_str()) {
            return Err(ParseError {
                message: format!(
                    "edge references unknown node: \"{}\"",
                    edge.to
                ),
                line: 0,
                col: 0,
            });
        }
    }

    // Check failure references
    for failure in &ast.failures {
        match &failure.failure {
            FailureInjection::Crash { node_id }
            | FailureInjection::Recover { node_id }
            | FailureInjection::AddLatency { node_id, .. }
            | FailureInjection::ReduceCapacity { node_id, .. } => {
                if !node_ids.contains(node_id.as_str()) {
                    return Err(ParseError {
                        message: format!(
                            "failure references unknown node: \"{}\"",
                            node_id
                        ),
                        line: 0,
                        col: 0,
                    });
                }
            }
            FailureInjection::Disconnect { from, to }
            | FailureInjection::AddPacketLoss { from, to, .. } => {
                if !node_ids.contains(from.as_str()) {
                    return Err(ParseError {
                        message: format!(
                            "failure references unknown node: \"{}\"",
                            from
                        ),
                        line: 0,
                        col: 0,
                    });
                }
                if !node_ids.contains(to.as_str()) {
                    return Err(ParseError {
                        message: format!(
                            "failure references unknown node: \"{}\"",
                            to
                        ),
                        line: 0,
                        col: 0,
                    });
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_json() {
        let input = r#"{
            "name": "test",
            "nodes": [],
            "connections": [],
            "traffic": {
                "start_rps": 10,
                "target_rps": 100,
                "ramp_seconds": 30
            },
            "seed": 42
        }"#;
        let scenario = parse_json(input).unwrap();
        assert_eq!(scenario.name, "test");
        assert_eq!(scenario.seed, 42);
        assert_eq!(scenario.traffic.start_rps, 10);
        assert_eq!(scenario.traffic.target_rps, 100);
    }

    #[test]
    fn parse_invalid_json_returns_error() {
        let result = parse_json("{ invalid }");
        assert!(result.is_err());
    }

    #[test]
    fn dsl_parser_works() {
        let src = r#"scenario "test" { seed: 42 }"#;
        let result = parse_dsl(src);
        assert!(result.is_ok());
        let scenario = result.unwrap();
        assert_eq!(scenario.name, "test");
        assert_eq!(scenario.seed, 42);
    }

    #[test]
    fn dsl_parser_validation_duplicate_ids() {
        let src = r#"
            scenario "test" {
              nodes {
                service "a" { capacity: 100, latency: 10ms }
                service "a" { capacity: 100, latency: 10ms }
              }
            }
        "#;
        let result = parse_dsl(src);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("duplicate node id"));
    }

    #[test]
    fn dsl_parser_validation_unknown_edge_ref() {
        let src = r#"
            scenario "test" {
              nodes {
                service "a" { capacity: 100, latency: 10ms }
              }
              edges {
                "a" -> "b" { latency: 5ms }
              }
            }
        "#;
        let result = parse_dsl(src);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown node"));
    }

    #[test]
    fn dsl_parser_with_failures() {
        let src = r#"
            scenario "test" {
              nodes {
                service "a" { capacity: 100, latency: 10ms }
              }
              failures {
                at 5s: crash "a"
                at 10s: recover "a"
              }
            }
        "#;
        let result = parse_dsl_with_failures(src);
        assert!(result.is_ok());
        let (scenario, failures) = result.unwrap();
        assert_eq!(scenario.name, "test");
        assert_eq!(failures.len(), 2);
        assert_eq!(failures[0].0, 5000);
    }

    #[test]
    fn dsl_parser_validation_unknown_failure_ref() {
        let src = r#"
            scenario "test" {
              nodes {
                service "a" { capacity: 100, latency: 10ms }
              }
              failures {
                at 5s: crash "b"
              }
            }
        "#;
        let result = parse_dsl(src);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown node"));
    }

    #[test]
    fn dsl_parser_full_scenario_round_trip() {
        let src = r#"
            # Retry storm scenario
            scenario "retry-storm" {
              seed: 42
              nodes {
                client "client" {
                  name: "Customer"
                  capacity: 1000
                  latency: 5ms
                  timeout: 5000ms
                }
                service "checkout-api" {
                  name: "Checkout API"
                  capacity: 120
                  latency: 40ms
                  error_rate: 1%
                  timeout: 800ms
                  retry: exponential { base: 100ms, max: 5000ms, max_retries: 3, jitter: 0.2 }
                  shed: reject
                }
                database "orders-db" {
                  name: "Orders DB"
                  capacity: 80
                  latency: 25ms
                  error_rate: 0.5%
                  timeout: 2000ms
                }
              }
              edges {
                "client" -> "checkout-api" { latency: 10ms }
                "checkout-api" -> "orders-db" { latency: 5ms }
              }
              traffic {
                start: 20 rps
                target: 500 rps
                ramp: 30s
              }
              failures {
                at 20s: crash "orders-db"
                at 35s: recover "orders-db"
              }
            }
        "#;
        let (scenario, failures) = parse_dsl_with_failures(src).unwrap();
        assert_eq!(scenario.name, "retry-storm");
        assert_eq!(scenario.nodes.len(), 3);
        assert_eq!(scenario.connections.len(), 2);
        assert_eq!(scenario.traffic.start_rps, 20);
        assert_eq!(scenario.traffic.target_rps, 500);
        assert_eq!(scenario.traffic.ramp_seconds, 30);
        assert_eq!(failures.len(), 2);

        // Verify the scenario can be serialized to JSON
        let json = serde_json::to_string(&scenario).unwrap();
        assert!(json.contains("retry-storm"));
        assert!(json.contains("checkout-api"));
    }
}
