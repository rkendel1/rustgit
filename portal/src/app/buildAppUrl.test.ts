import assert from "node:assert/strict";
import test from "node:test";

import { buildAppUrl } from "./buildAppUrl.ts";

test("builds a same-origin proxy URL, never a raw host:port", () => {
  assert.equal(buildAppUrl("ws-1", "/dashboard"), "/api/app-proxy/ws-1/dashboard");
  assert.equal(buildAppUrl("ws-1", undefined), "/api/app-proxy/ws-1/");
});

test("returns null without a workspace id, never falls back to a raw port", () => {
  assert.equal(buildAppUrl(null, "/"), null);
});
