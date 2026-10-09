import assert from "node:assert/strict";
import test from "node:test";
import { SliderCommandQueue } from "../../apps/lightcraft-desktop/web/src/inspector/sliderCommandQueue.ts";

const deferred = () => {
  let resolve;
  const promise = new Promise(done => { resolve = done; });
  return { promise, resolve };
};
const tick = () => new Promise(resolve => setImmediate(resolve));

test("collapses burst updates to final value before end", async () => {
  const calls = [];
  const queue = new SliderCommandQueue(async (id, params) => { calls.push([id, params]); });
  await queue.begin("Exposure");
  queue.set("exposure", 1);
  queue.set("exposure", 2);
  queue.set("exposure", 3);
  await queue.end();
  assert.deepEqual(calls, [
    ["develop.beginInteraction", { label: "Exposure" }],
    ["develop.set", { control: "exposure", value: 3 }],
    ["develop.endInteraction", {}],
  ]);
});

test("keeps end after delayed in-flight set & isolates next gesture", async () => {
  const calls = [];
  const gate = deferred();
  let blocked = false;
  const queue = new SliderCommandQueue(async (id, params) => {
    calls.push([id, params]);
    if (id === "develop.set" && !blocked) { blocked = true; await gate.promise; }
  });
  await queue.begin("Exposure");
  queue.set("exposure", 1);
  await tick();
  const secondPending = queue.set("exposure", 2);
  queue.set("exposure", 3);
  const firstEnd = queue.end();
  const secondBegin = queue.begin("Contrast");
  const secondSet = queue.set("contrast", 4);
  const secondEnd = queue.end();
  assert.deepEqual(calls.map(([id]) => id), ["develop.beginInteraction", "develop.set"]);
  gate.resolve();
  await Promise.all([firstEnd, secondBegin, secondPending, secondSet, secondEnd]);
  assert.deepEqual(calls.map(([id]) => id), [
    "develop.beginInteraction", "develop.set", "develop.set", "develop.endInteraction",
    "develop.beginInteraction", "develop.set", "develop.endInteraction",
  ]);
  assert.deepEqual(calls[2], ["develop.set", { control: "exposure", value: 3 }]);
  assert.deepEqual(calls[4], ["develop.beginInteraction", { label: "Contrast" }]);
  assert.deepEqual(calls[5], ["develop.set", { control: "contrast", value: 4 }]);
});

test("cancel drops unsent value & waits for in-flight set", async () => {
  const calls = [];
  const gate = deferred();
  const queue = new SliderCommandQueue(async (id, params) => {
    calls.push([id, params]);
    if (id === "develop.set") await gate.promise;
  });
  await queue.begin("Exposure");
  queue.set("exposure", 1);
  await tick();
  const unsent = queue.set("exposure", 2);
  const cancel = queue.cancel();
  const nextBegin = queue.begin("Contrast");
  const nextSet = queue.set("contrast", 4);
  const nextEnd = queue.end();
  assert.deepEqual(calls.map(([id]) => id), ["develop.beginInteraction", "develop.set"]);
  gate.resolve();
  await Promise.all([unsent, cancel, nextBegin, nextSet, nextEnd]);
  assert.deepEqual(calls.map(([id]) => id), [
    "develop.beginInteraction", "develop.set", "develop.cancelInteraction",
    "develop.beginInteraction", "develop.set", "develop.endInteraction",
  ]);
});

test("rejected set settles caller & queue remains usable", async () => {
  const calls = [];
  let rejectSet = true;
  const queue = new SliderCommandQueue(async (id, params) => {
    calls.push([id, params]);
    if (id === "develop.set" && rejectSet) { rejectSet = false; throw new Error("set failed"); }
  });
  await queue.begin("Exposure");
  const failure = queue.set("exposure", 1).catch(reason => reason);
  const end = queue.end();
  assert.equal((await failure).message, "set failed");
  await end;
  await queue.begin("Contrast");
  await queue.set("contrast", 4);
  await queue.cancel();
  assert.deepEqual(calls.map(([id]) => id), [
    "develop.beginInteraction", "develop.set", "develop.endInteraction",
    "develop.beginInteraction", "develop.set", "develop.cancelInteraction",
  ]);
});

test("rejected begin drops generation & permits next gesture", async () => {
  const calls = [];
  let rejectBegin = true;
  const queue = new SliderCommandQueue(async (id, params) => {
    calls.push([id, params]);
    if (id === "develop.beginInteraction" && rejectBegin) { rejectBegin = false; throw new Error("begin failed"); }
  });
  const beginFailure = queue.begin("Exposure").catch(reason => reason);
  const setFailure = queue.set("exposure", 1).catch(reason => reason);
  assert.equal((await beginFailure).message, "begin failed");
  assert.equal((await setFailure).message, "begin failed");
  await queue.begin("Contrast");
  await queue.set("contrast", 4);
  await queue.end();
  assert.deepEqual(calls.map(([id]) => id), [
    "develop.beginInteraction", "develop.beginInteraction", "develop.set", "develop.endInteraction",
  ]);
});
