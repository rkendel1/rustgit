import assert from "node:assert/strict";
import test from "node:test";

import { rewriteLocationHeader } from "./location-rewrite.ts";

test("rewrites an absolute raw-port redirect onto the proxy origin", () => {
  const result = rewriteLocationHeader(
    "http://127.0.0.1:51694/dashboard",
    "http://127.0.0.1:51694",
    "http://localhost:3000",
    "ws-123",
  );
  assert.equal(
    result,
    "http://localhost:3000/api/app-proxy/ws-123/dashboard",
  );
});

test("rewrites relative redirects and preserves query and hash", () => {
  const result = rewriteLocationHeader(
    "/dashboard?x=1#section-2",
    "http://127.0.0.1:51694",
    "http://localhost:3000",
    "ws-123",
  );
  assert.equal(
    result,
    "http://localhost:3000/api/app-proxy/ws-123/dashboard?x=1#section-2",
  );
});

test("throws on malformed location", () => {
  assert.throws(
    () =>
      rewriteLocationHeader(
        "http://[invalid",
        "http://127.0.0.1:51694",
        "http://localhost:3000",
        "ws-123",
      ),
    /Invalid URL/u,
  );
});
