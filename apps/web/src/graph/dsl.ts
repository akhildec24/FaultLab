/**
 * DSL generator — converts the visual graph to FaultLab DSL source.
 *
 * This is the reverse of the Rust parser: it takes GraphNodes and
 * GraphEdges and produces a .fault file that can be edited in the
 * code editor and re-parsed.
 */

import type { GraphNode, GraphEdge } from './types'

const KIND_TO_KEYWORD: Record<string, string> = {
  client: 'client',
  service: 'service',
  queue: 'queue',
  cache: 'cache',
  database: 'database',
  external_api: 'external_api',
}

function fmtDuration(ms: number): string {
  if (ms === 0) return '0ms'
  if (ms < 1000) return `${ms}ms`
  if (ms % 1000 === 0) return `${ms / 1000}s`
  return `${ms}ms`
}

function fmtPercent(p: number): string {
  if (p === 0) return '0%'
  const pct = p * 100
  if (pct % 1 === 0) return `${pct}%`
  return `${pct.toFixed(1)}%`
}

function retryStrategyStr(strategy: string): string {
  if (strategy === 'immediate') return 'immediate'
  if (strategy === 'fixed') return 'fixed'
  if (strategy === 'exponential') return 'exponential'
  return 'immediate'
}

function shedPolicyStr(policy: string): string {
  if (policy === 'drop') return 'drop'
  if (policy === 'reject') return 'reject'
  if (policy === 'backpressure') return 'backpressure'
  return 'drop'
}

function replicationRoleStr(role: string): string {
  if (role === 'leader') return 'leader'
  if (role === 'replica') return 'replica'
  return 'standalone'
}

/**
 * Generate FaultLab DSL source from the visual graph.
 */
export function graphToDsl(
  nodes: GraphNode[],
  edges: GraphEdge[],
  scenarioName = 'untitled',
  seed = 42,
): string {
  const lines: string[] = []
  lines.push(`scenario "${scenarioName}" {`)
  lines.push(`  seed: ${seed}`)
  lines.push('')

  // Nodes
  if (nodes.length > 0) {
    lines.push('  nodes {')
    for (const node of nodes) {
      const keyword = KIND_TO_KEYWORD[node.kind] || 'service'
      lines.push(`    ${keyword} "${node.id}" {`)
      lines.push(`      name: "${node.label}"`)
      lines.push(`      capacity: ${node.capacity}`)
      lines.push(`      latency: ${fmtDuration(node.latency_ms)}`)
      if (node.error_rate > 0) lines.push(`      error_rate: ${fmtPercent(node.error_rate)}`)
      lines.push(`      timeout: ${fmtDuration(node.timeout_ms)}`)
      if (node.queue_limit !== null && node.queue_limit !== undefined) {
        lines.push(`      queue_limit: ${node.queue_limit}`)
      }
      if (node.replication_role && node.replication_role !== 'standalone') {
        lines.push(`      replication: ${replicationRoleStr(node.replication_role)}`)
      }
      if (node.replication_lag_ms && node.replication_lag_ms > 0) {
        lines.push(`      replication_lag: ${fmtDuration(node.replication_lag_ms)}`)
      }

      // Retry policy
      const rp = node.retry_policy
      if (rp) {
        const strat = retryStrategyStr(rp.strategy)
        const params: string[] = [`max_retries: ${rp.max_retries}`, `jitter: ${rp.jitter}`]
        if (rp.budget !== null && rp.budget !== undefined) {
          params.push(`budget: ${rp.budget}`)
        }
        lines.push(`      retry: ${strat} { ${params.join(', ')} }`)
      }

      // Shed policy
      if (node.shed_policy && node.shed_policy !== 'drop') {
        lines.push(`      shed: ${shedPolicyStr(node.shed_policy)}`)
      }

      lines.push('    }')
    }
    lines.push('  }')
  }

  // Edges
  if (edges.length > 0) {
    lines.push('')
    lines.push('  edges {')
    for (const edge of edges) {
      const props: string[] = []
      if (edge.latency_ms > 0) props.push(`latency: ${fmtDuration(edge.latency_ms)}`)
      if (edge.packet_loss > 0) props.push(`packet_loss: ${fmtPercent(edge.packet_loss)}`)
      if (edge.bandwidth_rps && edge.bandwidth_rps > 0) {
        props.push(`bandwidth: ${edge.bandwidth_rps}`)
      }
      if (props.length > 0) {
        lines.push(`    "${edge.from}" -> "${edge.to}" { ${props.join(', ')} }`)
      } else {
        lines.push(`    "${edge.from}" -> "${edge.to}"`)
      }
    }
    lines.push('  }')
  }

  // Traffic (default)
  lines.push('')
  lines.push('  traffic {')
  lines.push('    start: 10 rps')
  lines.push('    target: 100 rps')
  lines.push('    ramp: 30s')
  lines.push('  }')

  lines.push('}')
  return lines.join('\n')
}

/**
 * Validate DSL source by checking basic syntax rules on the TS side.
 * Full validation is done by the Rust parser when available via WASM.
 */
export interface DslError {
  line: number
  col: number
  message: string
}

export function validateDsl(source: string): DslError[] {
  const errors: DslError[] = []
  const lines = source.split('\n')

  // Basic checks — full parsing is done in Rust
  let foundScenario = false
  let braceDepth = 0

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim()
    if (line.startsWith('#') || line.startsWith('//')) continue

    if (line.startsWith('scenario ')) foundScenario = true

    for (const ch of line) {
      if (ch === '{') braceDepth++
      if (ch === '}') braceDepth--
    }

    // Check for unclosed strings
    const quotes = (line.match(/"/g) || []).length
    if (quotes % 2 !== 0) {
      errors.push({
        line: i + 1,
        col: line.length,
        message: 'unclosed string literal',
      })
    }
  }

  if (!foundScenario) {
    errors.push({
      line: 1,
      col: 1,
      message: 'missing scenario declaration',
    })
  }

  if (braceDepth !== 0) {
    errors.push({
      line: lines.length,
      col: 1,
      message: `unbalanced braces (${braceDepth > 0 ? 'missing }' : 'extra }'})`,
    })
  }

  return errors
}
