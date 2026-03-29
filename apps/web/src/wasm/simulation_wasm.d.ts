declare module './wasm/simulation_wasm.js' {
  export class Simulation {
    loadScenario(json: string): void
    start(): void
    pause(): void
    reset(): void
    step(): boolean
    run(maxSteps: number): number
    isRunning(): boolean
    currentTime(): number
    getMetrics(): string
    getState(): string
    getRecentEvents(): string
    pendingEvents(): number
    injectFailure(json: string): void
  }
}
