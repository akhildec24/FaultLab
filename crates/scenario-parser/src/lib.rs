//! Scenario parser — a small DSL for defining simulation scenarios.
//!
//! Full lexer/parser/AST implementation arrives on Day 22–23.
//! This crate currently provides JSON-based scenario loading as a
//! placeholder.

use simulation_core::Scenario;

/// Parse a JSON scenario string into a Scenario struct.
pub fn parse_json(input: &str) -> Result<Scenario, String> {
    serde_json::from_str(input).map_err(|e| format!("Parse error: {}", e))
}

/// Parse the FaultLab DSL (not yet implemented).
pub fn parse_dsl(_input: &str) -> Result<Scenario, String> {
    Err("DSL parser not yet implemented — arrives on Day 22".into())
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
            "traffic_start_rps": 10,
            "traffic_target_rps": 100,
            "traffic_ramp_seconds": 30,
            "seed": 42
        }"#;
        let scenario = parse_json(input).unwrap();
        assert_eq!(scenario.name, "test");
        assert_eq!(scenario.seed, 42);
    }

    #[test]
    fn parse_invalid_json_returns_error() {
        let result = parse_json("{ invalid }");
        assert!(result.is_err());
    }

    #[test]
    fn dsl_parser_not_yet_implemented() {
        let result = parse_dsl("service checkout {}");
        assert!(result.is_err());
    }
}
