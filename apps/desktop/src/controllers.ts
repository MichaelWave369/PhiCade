import type { GameAction } from "./actionBus";

export interface ControllerSnapshot {
  index: number;
  id: string;
  mapping: string;
  connected: boolean;
  buttonCount: number;
  axisCount: number;
}

export type GameBoyButtonState = Record<
  "A" | "B" | "SELECT" | "START" | "UP" | "DOWN" | "LEFT" | "RIGHT",
  boolean
>;

const EMPTY_STATE: GameBoyButtonState = {
  A: false, B: false, SELECT: false, START: false,
  UP: false, DOWN: false, LEFT: false, RIGHT: false,
};

export function listConnectedControllers(): ControllerSnapshot[] {
  if (typeof navigator.getGamepads !== "function") return [];
  return Array.from(navigator.getGamepads())
    .filter((pad): pad is Gamepad => pad !== null && pad.connected)
    .map((pad) => ({
      index: pad.index,
      id: pad.id,
      mapping: pad.mapping,
      connected: pad.connected,
      buttonCount: pad.buttons.length,
      axisCount: pad.axes.length,
    }));
}

export function readGameBoyButtons(index = 0, deadzone = 0.18): GameBoyButtonState {
  if (typeof navigator.getGamepads !== "function") return { ...EMPTY_STATE };
  const pad = navigator.getGamepads()[index];
  if (!pad?.connected) return { ...EMPTY_STATE };

  const pressed = (button: number) => Boolean(pad.buttons[button]?.pressed);
  const x = pad.axes[0] ?? 0;
  const y = pad.axes[1] ?? 0;

  return {
    A: pressed(0),
    B: pressed(1),
    SELECT: pressed(8),
    START: pressed(9),
    UP: pressed(12) || y < -deadzone,
    DOWN: pressed(13) || y > deadzone,
    LEFT: pressed(14) || x < -deadzone,
    RIGHT: pressed(15) || x > deadzone,
  };
}

export function diffGameBoyButtons(
  previous: GameBoyButtonState,
  next: GameBoyButtonState,
): GameAction[] {
  const actions: GameAction[] = [];
  for (const button of Object.keys(next) as Array<keyof GameBoyButtonState>) {
    if (previous[button] !== next[button]) {
      actions.push({ kind: "button", button, pressed: next[button] });
    }
  }
  return actions;
}

export function emptyGameBoyButtons(): GameBoyButtonState {
  return { ...EMPTY_STATE };
}
