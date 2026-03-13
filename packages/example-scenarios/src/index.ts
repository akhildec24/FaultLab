import type { Scenario } from '@faultlab/scenario-schema'

/// The retry storm scenario from the product spec.
/// Traffic ramps from 20 to 500 rps over 30 seconds.
/// At 20s the database slows. The service retries immediately,
/// causing a retry storm that overwhelms the database.
export const retryStorm: Scenario = {
  name: 'Retry Storm',
  nodes: [
    {
      id: 'client',
      kind: 'client',
      name: 'Customer',
      state: 'healthy',
      capacity: 1000,
      latencyMs: 5,
      errorRate: 0,
      timeoutMs: 5000,
    },
    {
      id: 'checkout-api',
      kind: 'service',
      name: 'Checkout API',
      state: 'healthy',
      capacity: 120,
      latencyMs: 40,
      errorRate: 0.01,
      timeoutMs: 800,
    },
    {
      id: 'orders-db',
      kind: 'database',
      name: 'Orders DB',
      state: 'healthy',
      capacity: 80,
      latencyMs: 25,
      errorRate: 0.005,
      timeoutMs: 2000,
    },
  ],
  connections: [
    { from: 'client', to: 'checkout-api', latencyMs: 10, packetLoss: 0 },
    { from: 'checkout-api', to: 'orders-db', latencyMs: 10, packetLoss: 0 },
  ],
  trafficStartRps: 20,
  trafficTargetRps: 500,
  trafficRampSeconds: 30,
  seed: 42,
}

export const scenarios: Record<string, Scenario> = {
  'retry-storm': retryStorm,
}
