# FaultLab Scenario Language — Grammar Specification

## Overview

FaultLab's scenario language is a small, declarative DSL for defining
distributed systems topologies and simulation parameters. It is designed
to be readable by engineers, easy to parse, and directly mappable to the
internal `Scenario` JSON schema consumed by the Rust simulation engine.

## Design Goals

- **Readable** — an engineer should understand a scenario at a glance.
- **Concise** — common cases require minimal boilerplate.
- **Explicit** — no hidden defaults; every property is visible.
- **Line-oriented** — easy to parse, good error messages with line/column.
- **Round-trippable** — visual editor ↔ code editor sync without loss.

## Top-level Structure

A scenario file has four sections, three of which are optional:

```
scenario "name" {
  nodes { ... }       // required — at least one node
  edges { ... }       // optional — connections between nodes
  traffic { ... }     // optional — defaults to 10→100 rps over 30s
  failures { ... }    // optional — scheduled failure injections
  seed: 42            // optional — defaults to 42
}
```

## Comments

```
# Line comments (hash)
// Also line comments (double slash)
```

## Nodes Section

Each node is declared with a kind keyword and an identifier:

```
nodes {
  client "client" {
    name: "Web Client"
    capacity: 1000
    latency: 5ms
    error_rate: 0%
    timeout: 5000ms
  }

  service "checkout-api" {
    name: "Checkout API"
    capacity: 120
    latency: 40ms
    error_rate: 1%
    timeout: 800ms
    queue_limit: 50
    retry: exponential { base: 100ms, max: 5000ms, max_retries: 3, jitter: 0.2 }
    shed: reject
  }

  database "orders-db" {
    name: "Orders DB"
    capacity: 80
    latency: 25ms
    error_rate: 0.5%
    timeout: 2000ms
    replication: leader
    replication_lag: 0ms
  }

  database "replica-db" {
    name: "Replica DB"
    capacity: 80
    latency: 25ms
    replication: replica
    replication_lag: 300ms
  }

  cache "redis" {
    name: "Redis Cache"
    capacity: 500
    latency: 2ms
    cache_hit_rate: 80%
  }

  queue "request-queue" {
    name: "Request Queue"
    capacity: 200
    latency: 1ms
    queue_limit: 100
    shed: drop
  }

  external_api "payment-gw" {
    name: "Payment Gateway"
    capacity: 50
    latency: 100ms
    error_rate: 2%
    timeout: 3000ms
  }
}
```

### Node Kinds

| Keyword         | ComponentKind  |
|-----------------|----------------|
| `client`        | Client         |
| `service`       | Service        |
| `queue`         | Queue          |
| `cache`         | Cache          |
| `database`      | Database       |
| `external_api`  | ExternalApi    |

### Node Properties

| Property           | Type      | Required | Default       |
|--------------------|-----------|----------|---------------|
| `name`             | string    | no       | same as id    |
| `capacity`         | integer   | yes      | —             |
| `latency`          | duration  | yes      | —             |
| `error_rate`       | percent   | no       | 0%            |
| `timeout`          | duration  | no       | 5000ms        |
| `queue_limit`      | integer   | no       | none          |
| `cache_hit_rate`   | percent   | no       | none          |
| `replication`      | enum      | no       | standalone    |
| `replication_lag`  | duration  | no       | 0ms           |
| `retry`            | policy    | no       | immediate     |
| `shed`             | enum      | no       | drop          |

### Retry Policies

```
retry: immediate { max_retries: 3, jitter: 0.1 }
retry: fixed { delay: 200ms, max_retries: 5, jitter: 0.2 }
retry: exponential { base: 100ms, max: 5000ms, max_retries: 3, jitter: 0.3, budget: 100 }
```

| Field         | Type      | Required | Default |
|---------------|-----------|----------|---------|
| `max_retries` | integer   | no       | 3       |
| `jitter`      | fraction  | no       | 0.0     |
| `delay`       | duration  | fixed only | —     |
| `base`        | duration  | exp only | —       |
| `max`         | duration  | exp only | —       |
| `budget`      | integer   | no       | none    |

### Shed Policies

```
shed: drop
shed: reject
shed: backpressure
```

## Edges Section

Edges define directed connections between nodes:

```
edges {
  "client" -> "checkout-api" { latency: 10ms, packet_loss: 0%, bandwidth: 0 }
  "checkout-api" -> "orders-db" { latency: 5ms }
  "checkout-api" -> "redis" { latency: 3ms }
  "checkout-api" -> "payment-gw" { latency: 50ms, packet_loss: 0.5% }
}
```

### Edge Properties

| Property      | Type     | Required | Default |
|---------------|----------|----------|---------|
| `latency`     | duration | no       | 0ms     |
| `packet_loss` | percent  | no       | 0%      |
| `bandwidth`   | integer  | no       | 0 (∞)   |

Shorthand without a property block uses defaults:

```
edges {
  "client" -> "checkout-api"
  "checkout-api" -> "orders-db" { latency: 5ms }
}
```

## Traffic Section

```
traffic {
  start: 20 rps
  target: 500 rps
  ramp: 30s
}
```

Defaults: `start: 10 rps, target: 100 rps, ramp: 30s`.

## Failures Section

Scheduled failures fire at a specific simulation time:

```
failures {
  at 20s: crash "orders-db"
  at 25s: recover "orders-db"
  at 30s: add_latency "checkout-api" 500ms
  at 40s: disconnect "checkout-api" -> "payment-gw"
  at 45s: add_packet_loss "client" -> "checkout-api" 5%
  at 50s: reduce_capacity "orders-db" 20
}
```

### Failure Types

| Syntax                                          | FailureInjection    |
|-------------------------------------------------|---------------------|
| `crash "node"`                                  | Crash               |
| `recover "node"`                                | Recover             |
| `add_latency "node" <duration>`                 | AddLatency          |
| `disconnect "from" -> "to"`                     | Disconnect          |
| `add_packet_loss "from" -> "to" <percent>`      | AddPacketLoss       |
| `reduce_capacity "node" <integer>`              | ReduceCapacity      |

## Duration Syntax

```
5ms      # milliseconds
100ms    # milliseconds
2s       # seconds (converted to ms internally)
1m       # minutes (converted to ms internally)
```

## Percent Syntax

```
0%       # 0.0
1%       # 0.01
0.5%     # 0.005
80%      # 0.8
100%     # 1.0
```

## Fraction Syntax

```
0.0      # no jitter
0.2      # 20% jitter
0.3      # 30% jitter
```

## Complete Example

```
# Retry storm scenario — overloaded API retrying a slow database
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
```

## EBNF Grammar

```ebnf
scenario      = "scenario" string "{" section* "}" ;
section       = nodes_section | edges_section | traffic_section | failures_section | seed_decl ;
seed_decl     = "seed" ":" integer ;

nodes_section = "nodes" "{" node_decl* "}" ;
node_decl     = node_kind string "{" node_prop* "}" ;
node_kind     = "client" | "service" | "queue" | "cache" | "database" | "external_api" ;
node_prop     = prop_name ":" prop_value ;
prop_name     = "name" | "capacity" | "latency" | "error_rate" | "timeout"
              | "queue_limit" | "cache_hit_rate" | "replication" | "replication_lag"
              | "retry" | "shed" ;
prop_value    = string | integer | duration | percent | fraction
              | retry_policy | shed_policy | replication_role ;

retry_policy  = ("immediate" | "fixed" | "exponential") "{" retry_param* "}" ;
retry_param   = ("max_retries" | "jitter" | "delay" | "base" | "max" | "budget") ":" (integer | fraction | duration) ;

shed_policy   = "drop" | "reject" | "backpressure" ;
replication   = "standalone" | "leader" | "replica" ;

edges_section = "edges" "{" edge_decl* "}" ;
edge_decl     = string "->" string [ "{" edge_prop* "}" ] ;
edge_prop     = ("latency" | "packet_loss" | "bandwidth") ":" (duration | percent | integer) ;

traffic_section = "traffic" "{" traffic_prop* "}" ;
traffic_prop    = ("start" | "target" | "ramp") ":" (integer "rps" | duration) ;

failures_section = "failures" "{" failure_decl* "}" ;
failure_decl     = "at" duration ":" failure_type ;
failure_type     = "crash" string
                 | "recover" string
                 | "add_latency" string duration
                 | "disconnect" string "->" string
                 | "add_packet_loss" string "->" string percent
                 | "reduce_capacity" string integer ;

duration      = integer ("ms" | "s" | "m") ;
percent       = number "%" ;
fraction      = number ;
string        = '"' char* '"' ;
integer       = digit+ ;
number        = digit+ ["." digit+] ;
```

## Mapping to Scenario JSON

The DSL maps directly to the `Scenario` struct:

| DSL                    | JSON field                    |
|------------------------|-------------------------------|
| `scenario "name"`      | `name`                        |
| `seed: N`              | `seed`                        |
| node declarations      | `nodes[]`                     |
| edge declarations      | `connections[]`               |
| `traffic { ... }`      | `traffic`                     |
| `failures { ... }`     | (stored separately, injected at runtime) |

## Error Reporting

The parser will produce errors with:
- Line and column numbers
- Expected vs. found tokens
- Context-aware messages (e.g. "unknown node kind 'microservice', expected one of: client, service, queue, cache, database, external_api")
- Suggestions for typos (e.g. "did you mean 'database'?")
