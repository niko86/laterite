/* The unit lane's stand-in for the BUILT wasm module (#591): the lane runs
 * with "no Rust/wasm toolchain" (the decision at the top of vitest.config.ts),
 * so `src/wasm/` does not exist on the unit runner and any test that makes
 * the module id RESOLVE — engine.wasm.test.ts's vi.mock does — failed there
 * while passing locally, where a developer's build output happens to sit on
 * disk. The alias in vitest.config.ts points the id here instead. Nothing in
 * this file ever executes: the mock factory replaces the module wholesale;
 * this exists purely so resolution succeeds identically on both machines. */

const never = (): never => {
  throw new Error(
    "the wasm stub executed — a unit test reached the real module surface " +
      "without mocking it; wasm belongs to the e2e and wasm lanes",
  );
};

export default never;
export const validate = never;
export const compute_fixes = never;
export const apply_fixes = never;
export type Fix = never;
