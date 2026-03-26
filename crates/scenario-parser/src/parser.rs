//! Parser — recursive descent parser that consumes tokens from the
//! lexer and produces an AST with line/column error tracking.

use crate::ast::*;
use crate::lexer::{Lexer, Token, TokenKind};
use simulation_core::{
    ComponentKind, FailureInjection, ReplicationRole, RetryStrategy, SheddingPolicy,
};
use std::fmt;

/// A parse error with position information.
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error at {}:{}: {}", self.line, self.col, self.message)
    }
}

type Result<T> = std::result::Result<T, ParseError>;

/// The parser — holds a token stream and a position cursor.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    /// Parse a full scenario from source text.
    pub fn parse(src: &str) -> Result<AstScenario> {
        let tokens = Lexer::new(src)
            .tokenize()
            .map_err(|e| ParseError {
                message: e.message,
                line: e.line,
                col: e.col,
            })?;
        let mut parser = Parser::new(tokens);
        parser.parse_scenario()
    }

    // --- Token helpers ---

    fn current(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn peek_kind(&self) -> &TokenKind {
        &self.tokens[self.pos].kind
    }

    fn advance(&mut self) -> Token {
        let t = self.tokens[self.pos].clone();
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        t
    }

    fn expect(&mut self, expected: &TokenKind) -> Result<Token> {
        let t = self.current().clone();
        if &t.kind == expected {
            self.advance();
            Ok(t)
        } else {
            Err(ParseError {
                message: format!("expected {}, found {}", expected, t.kind),
                line: t.line,
                col: t.col,
            })
        }
    }

    fn expect_string(&mut self) -> Result<String> {
        let t = self.current().clone();
        match &t.kind {
            TokenKind::String(s) => {
                self.advance();
                Ok(s.clone())
            }
            _ => Err(ParseError {
                message: format!("expected string, found {}", t.kind),
                line: t.line,
                col: t.col,
            }),
        }
    }

    fn expect_integer(&mut self) -> Result<u64> {
        let t = self.current().clone();
        match &t.kind {
            TokenKind::Integer(n) => {
                self.advance();
                Ok(*n)
            }
            _ => Err(ParseError {
                message: format!("expected integer, found {}", t.kind),
                line: t.line,
                col: t.col,
            }),
        }
    }

    fn expect_duration(&mut self) -> Result<u64> {
        let t = self.current().clone();
        match &t.kind {
            TokenKind::Duration(d) => {
                self.advance();
                Ok(*d)
            }
            _ => Err(ParseError {
                message: format!("expected duration, found {}", t.kind),
                line: t.line,
                col: t.col,
            }),
        }
    }

    fn expect_percent(&mut self) -> Result<f64> {
        let t = self.current().clone();
        match &t.kind {
            TokenKind::Percent(p) => {
                self.advance();
                Ok(*p)
            }
            _ => Err(ParseError {
                message: format!("expected percent, found {}", t.kind),
                line: t.line,
                col: t.col,
            }),
        }
    }

    fn expect_number(&mut self) -> Result<f64> {
        let t = self.current().clone();
        match &t.kind {
            TokenKind::Integer(n) => {
                self.advance();
                Ok(*n as f64)
            }
            TokenKind::Float(f) => {
                self.advance();
                Ok(*f)
            }
            _ => Err(ParseError {
                message: format!("expected number, found {}", t.kind),
                line: t.line,
                col: t.col,
            }),
        }
    }

    // --- Grammar rules ---

    fn parse_scenario(&mut self) -> Result<AstScenario> {
        self.expect(&TokenKind::Scenario)?;
        let name = self.expect_string()?;
        self.expect(&TokenKind::LBrace)?;

        let mut scenario = AstScenario {
            name,
            ..AstScenario::default()
        };

        while self.peek_kind() != &TokenKind::RBrace {
            match self.peek_kind() {
                TokenKind::Nodes => {
                    self.advance();
                    scenario.nodes = self.parse_nodes()?;
                }
                TokenKind::Edges => {
                    self.advance();
                    scenario.edges = self.parse_edges()?;
                }
                TokenKind::Traffic => {
                    self.advance();
                    scenario.traffic = Some(self.parse_traffic()?);
                }
                TokenKind::Failures => {
                    self.advance();
                    scenario.failures = self.parse_failures()?;
                }
                TokenKind::Seed => {
                    self.advance();
                    self.expect(&TokenKind::Colon)?;
                    scenario.seed = self.expect_integer()?;
                }
                TokenKind::Eof => {
                    let t = self.current().clone();
                    return Err(ParseError {
                        message: "unexpected end of input, expected '}'".into(),
                        line: t.line,
                        col: t.col,
                    });
                }
                other => {
                    let t = self.current().clone();
                    return Err(ParseError {
                        message: format!(
                            "expected section (nodes, edges, traffic, failures, seed), found {}",
                            other
                        ),
                        line: t.line,
                        col: t.col,
                    });
                }
            }
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(scenario)
    }

    fn parse_nodes(&mut self) -> Result<Vec<AstNode>> {
        self.expect(&TokenKind::LBrace)?;
        let mut nodes = Vec::new();
        while self.peek_kind() != &TokenKind::RBrace {
            nodes.push(self.parse_node()?);
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(nodes)
    }

    fn parse_node(&mut self) -> Result<AstNode> {
        let kind = self.parse_node_kind()?;
        let id = self.expect_string()?;
        self.expect(&TokenKind::LBrace)?;

        let mut node = AstNode {
            id,
            kind,
            name: None,
            capacity: None,
            latency_ms: None,
            error_rate: None,
            timeout_ms: None,
            queue_limit: None,
            cache_hit_rate: None,
            replication_role: None,
            replication_lag_ms: None,
            retry_policy: None,
            shed_policy: None,
        };

        while self.peek_kind() != &TokenKind::RBrace {
            self.parse_node_prop(&mut node)?;
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(node)
    }

    fn parse_node_kind(&mut self) -> Result<ComponentKind> {
        let t = self.current().clone();
        let kind = match &t.kind {
            TokenKind::Client => ComponentKind::Client,
            TokenKind::Service => ComponentKind::Service,
            TokenKind::Queue => ComponentKind::Queue,
            TokenKind::Cache => ComponentKind::Cache,
            TokenKind::Database => ComponentKind::Database,
            TokenKind::ExternalApi => ComponentKind::ExternalApi,
            _ => {
                return Err(ParseError {
                    message: format!(
                        "expected node kind (client, service, queue, cache, database, external_api), found {}",
                        t.kind
                    ),
                    line: t.line,
                    col: t.col,
                })
            }
        };
        self.advance();
        Ok(kind)
    }

    fn parse_node_prop(&mut self, node: &mut AstNode) -> Result<()> {
        match self.peek_kind().clone() {
            TokenKind::Name => {
                self.advance();
                self.expect(&TokenKind::Colon)?;
                node.name = Some(self.expect_string()?);
                self.skip_comma();
            }
            TokenKind::Capacity => {
                self.advance();
                self.expect(&TokenKind::Colon)?;
                let v = self.expect_integer()?;
                node.capacity = Some(v as u32);
                self.skip_comma();
            }
            TokenKind::Latency => {
                self.advance();
                self.expect(&TokenKind::Colon)?;
                node.latency_ms = Some(self.expect_duration()?);
                self.skip_comma();
            }
            TokenKind::ErrorRate => {
                self.advance();
                self.expect(&TokenKind::Colon)?;
                node.error_rate = Some(self.expect_percent()?);
                self.skip_comma();
            }
            TokenKind::Timeout => {
                self.advance();
                self.expect(&TokenKind::Colon)?;
                node.timeout_ms = Some(self.expect_duration()?);
                self.skip_comma();
            }
            TokenKind::QueueLimit => {
                self.advance();
                self.expect(&TokenKind::Colon)?;
                let v = self.expect_integer()?;
                node.queue_limit = Some(v as u32);
                self.skip_comma();
            }
            TokenKind::CacheHitRate => {
                self.advance();
                self.expect(&TokenKind::Colon)?;
                node.cache_hit_rate = Some(self.expect_percent()?);
                self.skip_comma();
            }
            TokenKind::Replication => {
                self.advance();
                self.expect(&TokenKind::Colon)?;
                node.replication_role = Some(self.parse_replication_role()?);
                self.skip_comma();
            }
            TokenKind::ReplicationLag => {
                self.advance();
                self.expect(&TokenKind::Colon)?;
                node.replication_lag_ms = Some(self.expect_duration()?);
                self.skip_comma();
            }
            TokenKind::Retry => {
                self.advance();
                self.expect(&TokenKind::Colon)?;
                node.retry_policy = Some(self.parse_retry_policy()?);
                self.skip_comma();
            }
            TokenKind::Shed => {
                self.advance();
                self.expect(&TokenKind::Colon)?;
                node.shed_policy = Some(self.parse_shed_policy()?);
                self.skip_comma();
            }
            other => {
                let t = self.current().clone();
                return Err(ParseError {
                    message: format!("expected node property, found {}", other),
                    line: t.line,
                    col: t.col,
                });
            }
        }
        Ok(())
    }

    fn parse_replication_role(&mut self) -> Result<ReplicationRole> {
        let t = self.current().clone();
        let role = match &t.kind {
            TokenKind::Standalone => ReplicationRole::Standalone,
            TokenKind::Leader => ReplicationRole::Leader,
            TokenKind::Replica => ReplicationRole::Replica,
            _ => {
                return Err(ParseError {
                    message: format!(
                        "expected replication role (standalone, leader, replica), found {}",
                        t.kind
                    ),
                    line: t.line,
                    col: t.col,
                })
            }
        };
        self.advance();
        Ok(role)
    }

    fn parse_shed_policy(&mut self) -> Result<SheddingPolicy> {
        let t = self.current().clone();
        let policy = match &t.kind {
            TokenKind::Drop => SheddingPolicy::Drop,
            TokenKind::Reject => SheddingPolicy::Reject,
            TokenKind::Backpressure => SheddingPolicy::Backpressure,
            _ => {
                return Err(ParseError {
                    message: format!(
                        "expected shed policy (drop, reject, backpressure), found {}",
                        t.kind
                    ),
                    line: t.line,
                    col: t.col,
                })
            }
        };
        self.advance();
        Ok(policy)
    }

    fn parse_retry_policy(&mut self) -> Result<AstRetryPolicy> {
        let strategy_kind = self.current().clone();
        let strategy_name = match &strategy_kind.kind {
            TokenKind::Immediate => "immediate",
            TokenKind::Fixed => "fixed",
            TokenKind::Exponential => "exponential",
            _ => {
                return Err(ParseError {
                    message: format!(
                        "expected retry strategy (immediate, fixed, exponential), found {}",
                        strategy_kind.kind
                    ),
                    line: strategy_kind.line,
                    col: strategy_kind.col,
                })
            }
        };
        self.advance();
        self.expect(&TokenKind::LBrace)?;

        let mut max_retries: u32 = 3;
        let mut jitter: f64 = 0.0;
        let mut budget: Option<u32> = None;
        let mut delay_ms: u64 = 0;
        let mut base_ms: u64 = 0;
        let mut max_delay_ms: u64 = 0;

        while self.peek_kind() != &TokenKind::RBrace {
            let prop = self.current().clone();
            self.advance();
            self.expect(&TokenKind::Colon)?;
            match &prop.kind {
                TokenKind::String(s) if s == "max_retries" => {
                    max_retries = self.expect_integer()? as u32;
                    self.skip_comma();
                }
                TokenKind::String(s) if s == "jitter" => {
                    jitter = self.expect_number()?;
                    self.skip_comma();
                }
                TokenKind::String(s) if s == "delay" => {
                    delay_ms = self.expect_duration()?;
                    self.skip_comma();
                }
                TokenKind::String(s) if s == "base" => {
                    base_ms = self.expect_duration()?;
                    self.skip_comma();
                }
                TokenKind::String(s) if s == "max" => {
                    max_delay_ms = self.expect_duration()?;
                    self.skip_comma();
                }
                TokenKind::String(s) if s == "budget" => {
                    budget = Some(self.expect_integer()? as u32);
                    self.skip_comma();
                }
                _ => {
                    return Err(ParseError {
                        message: format!("expected retry parameter, found {}", prop.kind),
                        line: prop.line,
                        col: prop.col,
                    });
                }
            }
        }
        self.expect(&TokenKind::RBrace)?;

        let strategy = match strategy_name {
            "immediate" => RetryStrategy::Immediate,
            "fixed" => RetryStrategy::Fixed { delay_ms },
            "exponential" => RetryStrategy::Exponential {
                base_ms,
                max_delay_ms,
            },
            _ => unreachable!(),
        };

        Ok(AstRetryPolicy {
            strategy,
            max_retries,
            jitter,
            budget,
        })
    }

    fn parse_edges(&mut self) -> Result<Vec<AstEdge>> {
        self.expect(&TokenKind::LBrace)?;
        let mut edges = Vec::new();
        while self.peek_kind() != &TokenKind::RBrace {
            edges.push(self.parse_edge()?);
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(edges)
    }

    fn skip_comma(&mut self) {
        if self.peek_kind() == &TokenKind::Comma {
            self.advance();
        }
    }

    fn parse_edge(&mut self) -> Result<AstEdge> {
        let from = self.expect_string()?;
        self.expect(&TokenKind::Arrow)?;
        let to = self.expect_string()?;

        let mut edge = AstEdge {
            from,
            to,
            latency_ms: None,
            packet_loss: None,
            bandwidth_rps: None,
        };

        // Optional property block
        if self.peek_kind() == &TokenKind::LBrace {
            self.advance();
            while self.peek_kind() != &TokenKind::RBrace {
                self.parse_edge_prop(&mut edge)?;
            }
            self.expect(&TokenKind::RBrace)?;
        }

        Ok(edge)
    }

    fn parse_edge_prop(&mut self, edge: &mut AstEdge) -> Result<()> {
        let prop = self.current().clone();
        self.advance();
        self.expect(&TokenKind::Colon)?;
        match &prop.kind {
            TokenKind::Latency => {
                edge.latency_ms = Some(self.expect_duration()?);
                self.skip_comma();
            }
            TokenKind::ErrorRate => {
                edge.packet_loss = Some(self.expect_percent()?);
                self.skip_comma();
            }
            TokenKind::String(s) if s == "packet_loss" => {
                edge.packet_loss = Some(self.expect_percent()?);
                self.skip_comma();
            }
            TokenKind::String(s) if s == "bandwidth" => {
                let v = self.expect_integer()?;
                edge.bandwidth_rps = Some(v as u32);
                self.skip_comma();
            }
            _ => {
                return Err(ParseError {
                    message: format!("expected edge property (latency, packet_loss, bandwidth), found {}", prop.kind),
                    line: prop.line,
                    col: prop.col,
                });
            }
        }
        Ok(())
    }

    fn parse_traffic(&mut self) -> Result<AstTraffic> {
        self.expect(&TokenKind::LBrace)?;
        let mut traffic = AstTraffic::default();
        while self.peek_kind() != &TokenKind::RBrace {
            let prop = self.current().clone();
            self.advance();
            self.expect(&TokenKind::Colon)?;
            match &prop.kind {
                TokenKind::Start => {
                    traffic.start_rps = self.expect_integer()? as u32;
                    if self.peek_kind() == &TokenKind::Rps {
                        self.advance();
                    }
                    self.skip_comma();
                }
                TokenKind::Target => {
                    traffic.target_rps = self.expect_integer()? as u32;
                    if self.peek_kind() == &TokenKind::Rps {
                        self.advance();
                    }
                    self.skip_comma();
                }
                TokenKind::Ramp => {
                    let d = self.expect_duration()?;
                    traffic.ramp_seconds = d / 1000;
                    self.skip_comma();
                }
                _ => {
                    return Err(ParseError {
                        message: format!("expected traffic property (start, target, ramp), found {}", prop.kind),
                        line: prop.line,
                        col: prop.col,
                    });
                }
            }
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(traffic)
    }

    fn parse_failures(&mut self) -> Result<Vec<AstFailure>> {
        self.expect(&TokenKind::LBrace)?;
        let mut failures = Vec::new();
        while self.peek_kind() != &TokenKind::RBrace {
            failures.push(self.parse_failure()?);
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(failures)
    }

    fn parse_failure(&mut self) -> Result<AstFailure> {
        self.expect(&TokenKind::At)?;
        let at_ms = self.expect_duration()?;
        self.expect(&TokenKind::Colon)?;

        let failure_type = self.current().clone();
        self.advance();

        let failure = match &failure_type.kind {
            TokenKind::Crash => {
                let node_id = self.expect_string()?;
                FailureInjection::Crash { node_id }
            }
            TokenKind::Recover => {
                let node_id = self.expect_string()?;
                FailureInjection::Recover { node_id }
            }
            TokenKind::AddLatency => {
                let node_id = self.expect_string()?;
                let latency_ms = self.expect_duration()?;
                FailureInjection::AddLatency { node_id, latency_ms }
            }
            TokenKind::Disconnect => {
                let from = self.expect_string()?;
                self.expect(&TokenKind::Arrow)?;
                let to = self.expect_string()?;
                FailureInjection::Disconnect { from, to }
            }
            TokenKind::AddPacketLoss => {
                let from = self.expect_string()?;
                self.expect(&TokenKind::Arrow)?;
                let to = self.expect_string()?;
                let rate = self.expect_percent()?;
                FailureInjection::AddPacketLoss { from, to, rate }
            }
            TokenKind::ReduceCapacity => {
                let node_id = self.expect_string()?;
                let new_capacity = self.expect_integer()? as u32;
                FailureInjection::ReduceCapacity {
                    node_id,
                    new_capacity,
                }
            }
            _ => {
                return Err(ParseError {
                    message: format!(
                        "expected failure type (crash, recover, add_latency, disconnect, add_packet_loss, reduce_capacity), found {}",
                        failure_type.kind
                    ),
                    line: failure_type.line,
                    col: failure_type.col,
                })
            }
        };

        Ok(AstFailure { at_ms, failure })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> Result<AstScenario> {
        Parser::parse(src)
    }

    #[test]
    fn parse_minimal_scenario() {
        let src = r#"scenario "test" { seed: 42 }"#;
        let ast = parse(src).unwrap();
        assert_eq!(ast.name, "test");
        assert_eq!(ast.seed, 42);
        assert!(ast.nodes.is_empty());
    }

    #[test]
    fn parse_full_scenario() {
        let src = r#"
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
                  retry: immediate { max_retries: 3, jitter: 0.0 }
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
        let ast = parse(src).unwrap();
        assert_eq!(ast.name, "retry-storm");
        assert_eq!(ast.seed, 42);
        assert_eq!(ast.nodes.len(), 3);
        assert_eq!(ast.edges.len(), 2);
        assert!(ast.traffic.is_some());
        assert_eq!(ast.failures.len(), 2);

        // Check node details
        let svc = ast.nodes.iter().find(|n| n.id == "checkout-api").unwrap();
        assert_eq!(svc.capacity, Some(120));
        assert_eq!(svc.latency_ms, Some(40));
        assert_eq!(svc.error_rate, Some(0.01));
        assert!(svc.retry_policy.is_some());
        assert_eq!(svc.shed_policy, Some(SheddingPolicy::Reject));
    }

    #[test]
    fn parse_cache_replication_scenario() {
        let src = r#"
            scenario "cache-repl" {
              seed: 7
              nodes {
                cache "redis" {
                  capacity: 500
                  latency: 2ms
                  cache_hit_rate: 80%
                }
                database "primary" {
                  capacity: 100
                  latency: 30ms
                  replication: leader
                  replication_lag: 0ms
                }
                database "replica" {
                  capacity: 100
                  latency: 30ms
                  replication: replica
                  replication_lag: 300ms
                }
              }
            }
        "#;
        let ast = parse(src).unwrap();
        assert_eq!(ast.nodes.len(), 3);

        let cache = ast.nodes.iter().find(|n| n.id == "redis").unwrap();
        assert_eq!(cache.cache_hit_rate, Some(0.8));

        let primary = ast.nodes.iter().find(|n| n.id == "primary").unwrap();
        assert_eq!(primary.replication_role, Some(ReplicationRole::Leader));

        let replica = ast.nodes.iter().find(|n| n.id == "replica").unwrap();
        assert_eq!(replica.replication_role, Some(ReplicationRole::Replica));
        assert_eq!(replica.replication_lag_ms, Some(300));
    }

    #[test]
    fn parse_exponential_retry() {
        let src = r#"
            scenario "test" {
              nodes {
                service "svc" {
                  capacity: 100
                  latency: 10ms
                  retry: exponential { base: 100ms, max: 5000ms, max_retries: 3, jitter: 0.3, budget: 100 }
                }
              }
            }
        "#;
        let ast = parse(src).unwrap();
        let svc = &ast.nodes[0];
        let retry = svc.retry_policy.as_ref().unwrap();
        match &retry.strategy {
            RetryStrategy::Exponential { base_ms, max_delay_ms } => {
                assert_eq!(*base_ms, 100);
                assert_eq!(*max_delay_ms, 5000);
            }
            _ => panic!("expected exponential strategy"),
        }
        assert_eq!(retry.max_retries, 3);
        assert!((retry.jitter - 0.3).abs() < 0.001);
        assert_eq!(retry.budget, Some(100));
    }

    #[test]
    fn parse_all_failure_types() {
        let src = r#"
            scenario "test" {
              nodes {
                service "a" { capacity: 100, latency: 10ms }
                service "b" { capacity: 100, latency: 10ms }
              }
              failures {
                at 5s: crash "a"
                at 10s: recover "a"
                at 15s: add_latency "a" 500ms
                at 20s: disconnect "a" -> "b"
                at 25s: add_packet_loss "a" -> "b" 5%
                at 30s: reduce_capacity "a" 20
              }
            }
        "#;
        let ast = parse(src).unwrap();
        assert_eq!(ast.failures.len(), 6);

        // Check crash
        match &ast.failures[0].failure {
            FailureInjection::Crash { node_id } => assert_eq!(node_id, "a"),
            _ => panic!("expected crash"),
        }
        // Check disconnect
        match &ast.failures[3].failure {
            FailureInjection::Disconnect { from, to } => {
                assert_eq!(from, "a");
                assert_eq!(to, "b");
            }
            _ => panic!("expected disconnect"),
        }
        // Check add_packet_loss
        match &ast.failures[4].failure {
            FailureInjection::AddPacketLoss { from, to, rate } => {
                assert_eq!(from, "a");
                assert_eq!(to, "b");
                assert!((rate - 0.05).abs() < 0.001);
            }
            _ => panic!("expected add_packet_loss"),
        }
        // Check reduce_capacity
        match &ast.failures[5].failure {
            FailureInjection::ReduceCapacity { node_id, new_capacity } => {
                assert_eq!(node_id, "a");
                assert_eq!(*new_capacity, 20);
            }
            _ => panic!("expected reduce_capacity"),
        }
    }

    #[test]
    fn parse_comments() {
        let src = r#"
            # This is a comment
            scenario "test" {
              // Another comment
              seed: 42
            }
        "#;
        let ast = parse(src).unwrap();
        assert_eq!(ast.name, "test");
    }

    #[test]
    fn parse_error_missing_brace() {
        let src = r#"scenario "test" { seed: 42"#;
        let result = parse(src);
        assert!(result.is_err());
    }

    #[test]
    fn parse_error_unknown_node_kind() {
        let src = r#"scenario "test" { nodes { microservice "x" { capacity: 100, latency: 10ms } } }"#;
        let result = parse(src);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("node kind"));
    }

    #[test]
    fn parse_error_missing_string() {
        let src = r#"scenario 42 { }"#;
        let result = parse(src);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("string"));
    }

    #[test]
    fn ast_to_scenario_conversion() {
        let src = r#"
            scenario "test" {
              seed: 99
              nodes {
                service "svc" {
                  name: "My Service"
                  capacity: 200
                  latency: 20ms
                  error_rate: 2%
                  timeout: 1000ms
                  shed: drop
                }
              }
              edges {
                "svc" -> "svc" { latency: 5ms }
              }
              traffic {
                start: 10 rps
                target: 200 rps
                ramp: 10s
              }
            }
        "#;
        let ast = parse(src).unwrap();
        let scenario = ast.to_scenario();
        assert_eq!(scenario.name, "test");
        assert_eq!(scenario.seed, 99);
        assert_eq!(scenario.nodes.len(), 1);
        assert_eq!(scenario.nodes[0].id, "svc");
        assert_eq!(scenario.nodes[0].name, "My Service");
        assert_eq!(scenario.nodes[0].capacity, 200);
        assert_eq!(scenario.nodes[0].latency_ms, 20);
        assert!((scenario.nodes[0].error_rate - 0.02).abs() < 0.001);
        assert_eq!(scenario.connections.len(), 1);
        assert_eq!(scenario.traffic.start_rps, 10);
        assert_eq!(scenario.traffic.target_rps, 200);
        assert_eq!(scenario.traffic.ramp_seconds, 10);
    }
}
