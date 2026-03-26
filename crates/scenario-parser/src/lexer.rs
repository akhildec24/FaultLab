//! Lexer — tokenizes FaultLab DSL source code into tokens with
//! line/column tracking for error reporting.

use std::fmt;

/// A token kind produced by the lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Scenario,
    Nodes,
    Edges,
    Traffic,
    Failures,
    Seed,
    Client,
    Service,
    Queue,
    Cache,
    Database,
    ExternalApi,
    Name,
    Capacity,
    Latency,
    ErrorRate,
    Timeout,
    QueueLimit,
    CacheHitRate,
    Replication,
    ReplicationLag,
    Retry,
    Shed,
    Immediate,
    Fixed,
    Exponential,
    Drop,
    Reject,
    Backpressure,
    Standalone,
    Leader,
    Replica,
    Start,
    Target,
    Ramp,
    At,
    Crash,
    Recover,
    AddLatency,
    Disconnect,
    AddPacketLoss,
    ReduceCapacity,
    Rps,

    // Literals
    String(String),
    Integer(u64),
    Float(f64),
    Duration(u64), // in milliseconds
    Percent(f64),  // 0.0–1.0

    // Symbols
    LBrace,
    RBrace,
    Colon,
    Arrow,       // ->
    Comma,
    Hash,        // # (line comment start — consumed by lexer)

    // Special
    Eof,
}

/// A token with its position in the source.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Scenario => write!(f, "scenario"),
            TokenKind::Nodes => write!(f, "nodes"),
            TokenKind::Edges => write!(f, "edges"),
            TokenKind::Traffic => write!(f, "traffic"),
            TokenKind::Failures => write!(f, "failures"),
            TokenKind::Seed => write!(f, "seed"),
            TokenKind::Client => write!(f, "client"),
            TokenKind::Service => write!(f, "service"),
            TokenKind::Queue => write!(f, "queue"),
            TokenKind::Cache => write!(f, "cache"),
            TokenKind::Database => write!(f, "database"),
            TokenKind::ExternalApi => write!(f, "external_api"),
            TokenKind::Name => write!(f, "name"),
            TokenKind::Capacity => write!(f, "capacity"),
            TokenKind::Latency => write!(f, "latency"),
            TokenKind::ErrorRate => write!(f, "error_rate"),
            TokenKind::Timeout => write!(f, "timeout"),
            TokenKind::QueueLimit => write!(f, "queue_limit"),
            TokenKind::CacheHitRate => write!(f, "cache_hit_rate"),
            TokenKind::Replication => write!(f, "replication"),
            TokenKind::ReplicationLag => write!(f, "replication_lag"),
            TokenKind::Retry => write!(f, "retry"),
            TokenKind::Shed => write!(f, "shed"),
            TokenKind::Immediate => write!(f, "immediate"),
            TokenKind::Fixed => write!(f, "fixed"),
            TokenKind::Exponential => write!(f, "exponential"),
            TokenKind::Drop => write!(f, "drop"),
            TokenKind::Reject => write!(f, "reject"),
            TokenKind::Backpressure => write!(f, "backpressure"),
            TokenKind::Standalone => write!(f, "standalone"),
            TokenKind::Leader => write!(f, "leader"),
            TokenKind::Replica => write!(f, "replica"),
            TokenKind::Start => write!(f, "start"),
            TokenKind::Target => write!(f, "target"),
            TokenKind::Ramp => write!(f, "ramp"),
            TokenKind::At => write!(f, "at"),
            TokenKind::Crash => write!(f, "crash"),
            TokenKind::Recover => write!(f, "recover"),
            TokenKind::AddLatency => write!(f, "add_latency"),
            TokenKind::Disconnect => write!(f, "disconnect"),
            TokenKind::AddPacketLoss => write!(f, "add_packet_loss"),
            TokenKind::ReduceCapacity => write!(f, "reduce_capacity"),
            TokenKind::Rps => write!(f, "rps"),
            TokenKind::String(s) => write!(f, "\"{}\"", s),
            TokenKind::Integer(i) => write!(f, "{}", i),
            TokenKind::Float(fl) => write!(f, "{}", fl),
            TokenKind::Duration(d) => write!(f, "{}ms", d),
            TokenKind::Percent(p) => write!(f, "{}%", p * 100.0),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Hash => write!(f, "#"),
            TokenKind::Eof => write!(f, "end of input"),
        }
    }
}

/// A lexical error with position information.
#[derive(Debug, Clone)]
pub struct LexError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Lex error at {}:{}: {}", self.line, self.col, self.message)
    }
}

/// The lexer — converts source text into a vector of tokens.
pub struct Lexer<'a> {
    src: &'a [u8],
    pos: usize,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src: src.as_bytes(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    /// Tokenize the entire source, returning tokens or the first error.
    pub fn tokenize(mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        loop {
            self.skip_whitespace_and_comments();
            if self.pos >= self.src.len() {
                tokens.push(Token {
                    kind: TokenKind::Eof,
                    line: self.line,
                    col: self.col,
                });
                break;
            }
            let token = self.next_token()?;
            tokens.push(token);
        }
        Ok(tokens)
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            // Whitespace
            while self.pos < self.src.len() && self.src[self.pos].is_ascii_whitespace() {
                self.advance();
            }
            // Line comments: # or //
            if self.pos < self.src.len() {
                let b = self.src[self.pos];
                if b == b'#' {
                    while self.pos < self.src.len() && self.src[self.pos] != b'\n' {
                        self.advance();
                    }
                    continue;
                }
                if b == b'/' && self.pos + 1 < self.src.len() && self.src[self.pos + 1] == b'/' {
                    while self.pos < self.src.len() && self.src[self.pos] != b'\n' {
                        self.advance();
                    }
                    continue;
                }
            }
            break;
        }
    }

    fn advance(&mut self) -> u8 {
        let b = self.src[self.pos];
        self.pos += 1;
        if b == b'\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        b
    }

    fn peek(&self) -> Option<u8> {
        if self.pos < self.src.len() {
            Some(self.src[self.pos])
        } else {
            None
        }
    }

    fn peek_at(&self, offset: usize) -> Option<u8> {
        if self.pos + offset < self.src.len() {
            Some(self.src[self.pos + offset])
        } else {
            None
        }
    }

    fn next_token(&mut self) -> Result<Token, LexError> {
        let line = self.line;
        let col = self.col;
        let b = self.src[self.pos];

        // String literal
        if b == b'"' {
            return self.lex_string(line, col);
        }

        // Number (integer, float, duration, percent)
        if b.is_ascii_digit() {
            return self.lex_number(line, col);
        }

        // Symbols
        if b == b'{' {
            self.advance();
            return Ok(Token { kind: TokenKind::LBrace, line, col });
        }
        if b == b'}' {
            self.advance();
            return Ok(Token { kind: TokenKind::RBrace, line, col });
        }
        if b == b':' {
            self.advance();
            return Ok(Token { kind: TokenKind::Colon, line, col });
        }
        if b == b',' {
            self.advance();
            return Ok(Token { kind: TokenKind::Comma, line, col });
        }
        if b == b'-' && self.peek_at(1) == Some(b'>') {
            self.advance();
            self.advance();
            return Ok(Token { kind: TokenKind::Arrow, line, col });
        }

        // Identifier or keyword
        if b.is_ascii_alphabetic() || b == b'_' {
            return self.lex_identifier(line, col);
        }

        Err(LexError {
            message: format!("unexpected character '{}'", b as char),
            line,
            col,
        })
    }

    fn lex_string(&mut self, line: usize, col: usize) -> Result<Token, LexError> {
        self.advance(); // opening quote
        let mut s = String::new();
        while self.pos < self.src.len() {
            let b = self.src[self.pos];
            if b == b'"' {
                self.advance(); // closing quote
                return Ok(Token {
                    kind: TokenKind::String(s),
                    line,
                    col,
                });
            }
            if b == b'\\' {
                self.advance();
                if self.pos >= self.src.len() {
                    return Err(LexError {
                        message: "unterminated string escape".into(),
                        line,
                        col,
                    });
                }
                let esc = self.advance();
                match esc {
                    b'n' => s.push('\n'),
                    b't' => s.push('\t'),
                    b'\\' => s.push('\\'),
                    b'"' => s.push('"'),
                    _ => {
                        return Err(LexError {
                            message: format!("invalid escape sequence '\\{}'", esc as char),
                            line,
                            col,
                        })
                    }
                }
            } else {
                s.push(b as char);
                self.advance();
            }
        }
        Err(LexError {
            message: "unterminated string".into(),
            line,
            col,
        })
    }

    fn lex_number(&mut self, line: usize, col: usize) -> Result<Token, LexError> {
        let start = self.pos;
        while self.pos < self.src.len() && self.src[self.pos].is_ascii_digit() {
            self.advance();
        }
        let mut is_float = false;
        // Decimal part
        if self.pos < self.src.len() && self.src[self.pos] == b'.' {
            is_float = true;
            self.advance();
            while self.pos < self.src.len() && self.src[self.pos].is_ascii_digit() {
                self.advance();
            }
        }
        let num_str = std::str::from_utf8(&self.src[start..self.pos]).unwrap();

        // Check for duration suffix (ms, s, m)
        if self.pos < self.src.len() {
            let remaining = std::str::from_utf8(&self.src[self.pos..]).unwrap_or("");
            if remaining.starts_with("ms") {
                self.advance();
                self.advance();
                let n: u64 = num_str.parse().map_err(|_| LexError {
                    message: format!("invalid duration: {}", num_str),
                    line,
                    col,
                })?;
                return Ok(Token {
                    kind: TokenKind::Duration(n),
                    line,
                    col,
                });
            }
            if remaining.starts_with('s') && !remaining.starts_with("seed") {
                self.advance();
                let n: u64 = num_str.parse().map_err(|_| LexError {
                    message: format!("invalid duration: {}", num_str),
                    line,
                    col,
                })?;
                return Ok(Token {
                    kind: TokenKind::Duration(n * 1000),
                    line,
                    col,
                });
            }
            if remaining.starts_with('m') && !remaining.starts_with("ms") {
                // Check it's not part of a longer word
                let after_m = self.peek_at(1);
                if after_m.is_none() || !after_m.unwrap().is_ascii_alphabetic() {
                    self.advance();
                    let n: u64 = num_str.parse().map_err(|_| LexError {
                        message: format!("invalid duration: {}", num_str),
                        line,
                        col,
                    })?;
                    return Ok(Token {
                        kind: TokenKind::Duration(n * 60_000),
                        line,
                        col,
                    });
                }
            }
        }

        // Check for percent
        if self.peek() == Some(b'%') {
            self.advance();
            let n: f64 = num_str.parse().map_err(|_| LexError {
                message: format!("invalid percent: {}", num_str),
                line,
                col,
            })?;
            return Ok(Token {
                kind: TokenKind::Percent(n / 100.0),
                line,
                col,
            });
        }

        // Plain number
        if is_float {
            let n: f64 = num_str.parse().map_err(|_| LexError {
                message: format!("invalid number: {}", num_str),
                line,
                col,
            })?;
            Ok(Token {
                kind: TokenKind::Float(n),
                line,
                col,
            })
        } else {
            let n: u64 = num_str.parse().map_err(|_| LexError {
                message: format!("invalid integer: {}", num_str),
                line,
                col,
            })?;
            Ok(Token {
                kind: TokenKind::Integer(n),
                line,
                col,
            })
        }
    }

    fn lex_identifier(&mut self, line: usize, col: usize) -> Result<Token, LexError> {
        let start = self.pos;
        while self.pos < self.src.len()
            && (self.src[self.pos].is_ascii_alphanumeric() || self.src[self.pos] == b'_')
        {
            self.advance();
        }
        let word = std::str::from_utf8(&self.src[start..self.pos]).unwrap();

        let kind = match word {
            "scenario" => TokenKind::Scenario,
            "nodes" => TokenKind::Nodes,
            "edges" => TokenKind::Edges,
            "traffic" => TokenKind::Traffic,
            "failures" => TokenKind::Failures,
            "seed" => TokenKind::Seed,
            "client" => TokenKind::Client,
            "service" => TokenKind::Service,
            "queue" => TokenKind::Queue,
            "cache" => TokenKind::Cache,
            "database" => TokenKind::Database,
            "external_api" => TokenKind::ExternalApi,
            "name" => TokenKind::Name,
            "capacity" => TokenKind::Capacity,
            "latency" => TokenKind::Latency,
            "error_rate" => TokenKind::ErrorRate,
            "timeout" => TokenKind::Timeout,
            "queue_limit" => TokenKind::QueueLimit,
            "cache_hit_rate" => TokenKind::CacheHitRate,
            "replication" => TokenKind::Replication,
            "replication_lag" => TokenKind::ReplicationLag,
            "retry" => TokenKind::Retry,
            "shed" => TokenKind::Shed,
            "immediate" => TokenKind::Immediate,
            "fixed" => TokenKind::Fixed,
            "exponential" => TokenKind::Exponential,
            "drop" => TokenKind::Drop,
            "reject" => TokenKind::Reject,
            "backpressure" => TokenKind::Backpressure,
            "standalone" => TokenKind::Standalone,
            "leader" => TokenKind::Leader,
            "replica" => TokenKind::Replica,
            "start" => TokenKind::Start,
            "target" => TokenKind::Target,
            "ramp" => TokenKind::Ramp,
            "at" => TokenKind::At,
            "crash" => TokenKind::Crash,
            "recover" => TokenKind::Recover,
            "add_latency" => TokenKind::AddLatency,
            "disconnect" => TokenKind::Disconnect,
            "add_packet_loss" => TokenKind::AddPacketLoss,
            "reduce_capacity" => TokenKind::ReduceCapacity,
            "rps" => TokenKind::Rps,
            _ => TokenKind::String(word.to_string()),
        };

        Ok(Token { kind, line, col })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_simple_scenario() {
        let src = r#"scenario "test" { seed: 42 }"#;
        let tokens = Lexer::new(src).tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Scenario);
        assert_eq!(tokens[1].kind, TokenKind::String("test".into()));
        assert_eq!(tokens[2].kind, TokenKind::LBrace);
        assert_eq!(tokens[3].kind, TokenKind::Seed);
        assert_eq!(tokens[4].kind, TokenKind::Colon);
        assert_eq!(tokens[5].kind, TokenKind::Integer(42));
        assert_eq!(tokens[6].kind, TokenKind::RBrace);
        assert_eq!(tokens[7].kind, TokenKind::Eof);
    }

    #[test]
    fn lex_durations() {
        let tokens = Lexer::new("5ms 2s 1m").tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Duration(5));
        assert_eq!(tokens[1].kind, TokenKind::Duration(2000));
        assert_eq!(tokens[2].kind, TokenKind::Duration(60000));
    }

    #[test]
    fn lex_percents() {
        let tokens = Lexer::new("0% 1% 0.5% 80%").tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Percent(0.0));
        assert_eq!(tokens[1].kind, TokenKind::Percent(0.01));
        assert!(matches!(tokens[2].kind, TokenKind::Percent(p) if (p - 0.005).abs() < 0.001));
        assert_eq!(tokens[3].kind, TokenKind::Percent(0.8));
    }

    #[test]
    fn lex_comments() {
        let src = "# comment\nservice \"x\" {} // also comment\n";
        let tokens = Lexer::new(src).tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Service);
        assert_eq!(tokens[1].kind, TokenKind::String("x".into()));
    }

    #[test]
    fn lex_arrow() {
        let tokens = Lexer::new("a -> b").tokenize().unwrap();
        assert_eq!(tokens[1].kind, TokenKind::Arrow);
    }

    #[test]
    fn lex_error_unexpected_char() {
        let result = Lexer::new("@").tokenize();
        assert!(result.is_err());
    }
}
