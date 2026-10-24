/**
 * Small, dependency-free selection and input helpers.
 * This is UI navigation only; it never emits PhiCade Action Bus events.
 */
export function moveSelection(index, direction, columns, count) {
  if (!Number.isInteger(count) || count < 1) return 0;
  const current = Number.isInteger(index) ? Math.max(0, Math.min(count - 1, index)) : 0;
  const width = Number.isInteger(columns) && columns > 0 ? columns : 1;
  switch (direction) {
    case "left":
      return current % width > 0 ? current - 1 : current;
    case "right":
      return current % width < width - 1 && current + 1 < count ? current + 1 : current;
    case "up":
      return current - width >= 0 ? current - width : current;
    case "down":
      return current + width < count ? current + width : current;
    default:
      return current;
  }
}

function pressed(pad, index) {
  return Boolean(pad?.buttons?.[index]?.pressed);
}

/** Read the standard gamepad mapping without assuming that Xbox Edge exposes it. */
export function readNavigationInput(pad, deadzone = 0.55) {
  if (!pad) return { direction: null, accept: false, back: false, help: false };
  const horizontal = Number.isFinite(pad.axes?.[0]) ? pad.axes[0] : 0;
  const vertical = Number.isFinite(pad.axes?.[1]) ? pad.axes[1] : 0;
  let direction = null;
  if (pressed(pad, 12) || vertical < -deadzone) direction = "up";
  else if (pressed(pad, 13) || vertical > deadzone) direction = "down";
  else if (pressed(pad, 14) || horizontal < -deadzone) direction = "left";
  else if (pressed(pad, 15) || horizontal > deadzone) direction = "right";

  return {
    direction,
    accept: pressed(pad, 0), // A
    back: pressed(pad, 1),   // B
    help: pressed(pad, 2),   // X
  };
}
