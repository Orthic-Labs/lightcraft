import test from "node:test";
import assert from "node:assert/strict";
import { readJson } from "./lib.mjs";

test("synthetic catalog binds unique CC0 inputs & regression expectations", () => {
  const fixture = readJson("fixtures/desktop/catalog.json");
  assert.equal(fixture.kind, "synthetic-catalog");
  assert.equal(new Set(fixture.photos.map((photo) => photo.sourceRecipe)).size, fixture.photos.length);
  assert.ok(fixture.photos.some((photo) => photo.format === "ARW"));
  assert.equal(fixture.expected.noDuplicateSourceRecipes, true);
});

test("native journey covers hidden WKWebView/WebView2 control contracts", () => {
  const fixture = readJson("fixtures/desktop/native-journey.json");
  assert.equal(fixture.hidden, true);
  assert.deepEqual(fixture.webviews, ["WKWebView", "WebView2"]);
  for (const id of ["ipc", "stalePreview", "cache", "preferences", "gesture", "arwImport", "lightroomImport", "engineExport", "catalogRecovery"]) {
    assert.ok(fixture.scenarios.some((scenario) => scenario.id === id), id);
  }
});

test("installed identity requires catalog recovery & separate installer proof", () => {
  const fixture = readJson("fixtures/desktop/installed-baseline.json");
  assert.equal(fixture.baseline.required, true);
  assert.equal(fixture.catalogRecovery.required, true);
  assert.equal(fixture.installerRollback.qualified, false);
  assert.equal(fixture.installerRollback.requiredForCutover, true);
  assert.equal(fixture.qualification.runner, "rightkit-qa");
});
