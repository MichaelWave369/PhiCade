import type { XrAction } from "../protocol";
import { BUTTONS, AXES } from "../protocol";
export function neutralInput(lightgun: boolean): XrAction[] {
  return [
    ...BUTTONS.filter((b) => lightgun || !b.startsWith("GUN_")).map(
      (button) => ({
        kind: "button" as const,
        button,
        pressed: button === "GUN_OFFSCREEN",
      }),
    ),
    ...AXES.filter((a) => lightgun || !a.startsWith("GUN_")).map((axis) => ({
      kind: "axis" as const,
      axis,
      value: axis.startsWith("GUN_") ? -32768 : 0,
    })),
  ];
}
/** WebXR standard gamepad order: trigger, squeeze, touchpad, stick, primary, secondary. */
export function questGamepad(
  gamepad: Pick<Gamepad, "buttons" | "axes">,
  hand: "left" | "right",
): XrAction[] {
  const names =
    hand === "left"
      ? ["L2", "L", null, "SELECT", "X", "Y"]
      : ["R2", "R", null, "START", "A", "B"];
  const actions: XrAction[] = names.flatMap((button, i) =>
    button
      ? [
          {
            kind: "button" as const,
            button,
            pressed: !!gamepad.buttons[i]?.pressed,
          },
        ]
      : [],
  );
  const axes = gamepad.axes;
  const x = axes[axes.length - 2] ?? 0,
    y = axes[axes.length - 1] ?? 0;
  if (hand === "left")
    for (const [button, pressed] of [
      ["UP", y < -0.25],
      ["DOWN", y > 0.25],
      ["LEFT", x < -0.25],
      ["RIGHT", x > 0.25],
    ] as const)
      actions.push({ kind: "button", button, pressed });
  for (const [axis, value] of [
    [hand === "left" ? "LEFT_X" : "RIGHT_X", x],
    [hand === "left" ? "LEFT_Y" : "RIGHT_Y", y],
  ] as const) {
    actions.push({
      kind: "axis",
      axis,
      value: Math.round(
        Math.max(-1, Math.min(1, Number.isFinite(value) ? value : 0)) * 32767,
      ),
    });
  }
  return actions;
}
