export interface RemainingBudgetSummary {
  hostId: string;
  ecoBand: "Low" | "Medium" | "High";
  ecoFlopsLimit: number;
  turnsUsedToday: number;
  maxDailyTurns: number;
  turnsRemaining: number;
  evolutionEnabled: boolean;
}

/**
 * Host-local oracle call (WASM or native FFI) – must be implemented
 * by linking to the Rust inner-ledger, not re-implementing logic.
 */
export function queryRemainingEcoBudget(hostId: string): RemainingBudgetSummary {
  // placeholder type signature – implementation lives on host node side.
  throw new Error("host-local oracle only");
}
