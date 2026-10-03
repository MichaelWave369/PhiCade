export interface ControllerSnapshot {
  index: number;
  id: string;
  mapping: string;
  connected: boolean;
  buttonCount: number;
  axisCount: number;
}

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
