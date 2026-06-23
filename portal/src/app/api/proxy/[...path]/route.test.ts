import assert from "node:assert/strict";
import test from "node:test";

import { resolveLegacyFallbackPath } from "./legacy-path-fallback.ts";

test("maps badge generate endpoint to legacy singular path", () => {
  assert.equal(resolveLegacyFallbackPath("api/badges/generate"), "api/badge/generate");
});

test("normalizes leading slashes for fallback lookup", () => {
  assert.equal(resolveLegacyFallbackPath("/api/badges/generate"), "api/badge/generate");
});

test("returns null when no legacy fallback is defined", () => {
  assert.equal(resolveLegacyFallbackPath("api/analyze"), null);
});
