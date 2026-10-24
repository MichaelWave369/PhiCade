import test from "node:test";
import assert from "node:assert/strict";
import { moveSelection, readNavigationInput } from "../navigation.mjs";

test("horizontal navigation stays within the current row", () => {
  assert.equal(moveSelection(0, "left", 3, 6), 0);
  assert.equal(moveSelection(2, "right", 3, 6), 2);
  assert.equal(moveSelection(3, "right", 3, 6), 4);
});

test("vertical navigation respects the final incomplete row", () => {
  assert.equal(moveSelection(1, "down", 3, 5), 4);
  assert.equal(moveSelection(2, "down", 3, 5), 2);
  assert.equal(moveSelection(3, "up", 3, 5), 0);
});

test("one-column layouts and invalid inputs stay bounded", () => {
  assert.equal(moveSelection(0, "down", 1, 6), 1);
  assert.equal(moveSelection(5, "down", 1, 6), 5);
  assert.equal(moveSelection(4, "left", 1, 6), 4);
  assert.equal(moveSelection(500, "left", 0, 6), 5);
  assert.equal(moveSelection(3, "down", 3, 0), 0);
});

test("no exposed gamepad does not grant any input", () => {
  assert.deepEqual(readNavigationInput(null), { direction: null, accept: false, back: false, help: false });
});

test("mapped D-pad, buttons and axes are interpreted", () => {
  const buttons = Array.from({ length: 16 }, () => ({ pressed: false }));
  buttons[15] = { pressed: true };
  buttons[0] = { pressed: true };
  buttons[2] = { pressed: true };
  assert.deepEqual(readNavigationInput({ buttons, axes: [0, 0] }), {
    direction: "right", accept: true, back: false, help: true,
  });
  buttons[15] = { pressed: false };
  assert.equal(readNavigationInput({ buttons, axes: [-0.7, 0] }).direction, "left");
  assert.equal(readNavigationInput({ buttons, axes: [0.2, 0.1] }).direction, null);
});
