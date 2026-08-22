export interface ComputationResult {
  hash: number;
  durationMs: number;
}

export function runWasmPipeline(input: Uint8Array): ComputationResult {
  const start = Date.now();
  let hash = 0;
  for (let i = 0; i < input.length; i++) {
    hash = (hash * 31 + input[i]) >>> 0;
  }
  return {
    hash,
    durationMs: Date.now() - start
  };
}
