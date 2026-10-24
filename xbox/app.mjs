import { moveSelection, readNavigationInput } from "./navigation.mjs";

const grid = document.getElementById("launch-grid");
const cards = [...grid.querySelectorAll("a.launch-card")];
const helpPanel = document.getElementById("help-panel");
const helpToggle = document.getElementById("help-toggle");
const helpClose = document.getElementById("help-close");
const controllerState = document.getElementById("controller-state");

let selected = 0;
let previous = { direction: null, accept: false, back: false, help: false };
let nextRepeatAt = 0;
let controllerText = "";

function columnCount() {
  const resolved = window.getComputedStyle(grid).gridTemplateColumns;
  return Math.max(1, resolved.split(/\s+/).filter(Boolean).length);
}

function setSelection(index, shouldFocus = true) {
  selected = Math.max(0, Math.min(cards.length - 1, index));
  cards.forEach((card, i) => card.classList.toggle("is-selected", i === selected));
  if (shouldFocus) {
    cards[selected]?.focus({ preventScroll: true });
    cards[selected]?.scrollIntoView({ block: "nearest", inline: "nearest" });
  }
}

function move(direction) {
  const next = moveSelection(selected, direction, columnCount(), cards.length);
  if (next !== selected) setSelection(next);
}

function setHelp(open) {
  helpPanel.hidden = !open;
  helpToggle.setAttribute("aria-expanded", String(open));
  if (open) helpClose.focus();
  else setSelection(selected);
}

function launchSelection() {
  if (!helpPanel.hidden) return;
  const url = cards[selected]?.href;
  // All selectable destinations are hardcoded HTTPS links in index.html.
  if (url && new URL(url).protocol === "https:") window.location.assign(url);
}

helpToggle.addEventListener("click", () => setHelp(helpPanel.hidden));
helpClose.addEventListener("click", () => setHelp(false));

cards.forEach((card, index) => {
  card.addEventListener("focus", () => setSelection(index, false));
  card.addEventListener("pointerenter", () => setSelection(index, false));
});

document.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && !helpPanel.hidden) {
    event.preventDefault();
    setHelp(false);
    return;
  }
  if (!helpPanel.hidden) return;
  const direction = {
    ArrowUp: "up", ArrowDown: "down", ArrowLeft: "left", ArrowRight: "right",
  }[event.key];
  if (direction) {
    event.preventDefault();
    move(direction);
  }
  // Enter uses native anchor activation for keyboard and accessibility.
});

function firstGamepad() {
  try {
    const pads = navigator.getGamepads?.();
    return pads ? Array.from(pads).find((pad) => pad?.connected) ?? null : null;
  } catch {
    return null;
  }
}

function gamepadTick(now) {
  const pad = firstGamepad();
  const text = pad ? "GAMEPAD API DETECTED" : "KEYBOARD / POINTER";
  if (controllerText !== text) {
    controllerState.textContent = text;
    controllerText = text;
  }
  const input = readNavigationInput(pad);
  if (input.direction) {
    if (input.direction !== previous.direction) {
      if (helpPanel.hidden) move(input.direction);
      nextRepeatAt = now + 340;
    } else if (now >= nextRepeatAt) {
      if (helpPanel.hidden) move(input.direction);
      nextRepeatAt = now + 170;
    }
  }
  if (input.help && !previous.help) setHelp(helpPanel.hidden);
  if (input.back && !previous.back && !helpPanel.hidden) setHelp(false);
  if (input.accept && !previous.accept && helpPanel.hidden) launchSelection();
  previous = input;
  window.requestAnimationFrame(gamepadTick);
}

setSelection(0, false);
window.requestAnimationFrame(gamepadTick);
