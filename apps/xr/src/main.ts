import * as THREE from "three";
import {
  PhiLightGun,
  mapScreenRay,
  mapVideoContent,
  calibrate,
  IDENTITY_CALIBRATION,
  type Aim,
  type Calibration,
} from "../../../xr/lightgun/PhiLightGun";
import { questGamepad, neutralInput } from "../../../xr/interaction/gamepad";
import { PhotonRange } from "../../../xr/lightgun/range";
import {
  HOLLER_MAP,
  HOLLER_PROFILE,
  HollerCircuit,
  reconstruct,
} from "../../../xr/worldwalk/scene";
import type { Presence, Pose, XrAction } from "../../../xr/protocol";
import { WiiMotionAdapter } from "../../../xr/motion/WiiMotionAdapter";
import "./styles.css";

const el = <T extends HTMLElement>(id: string) =>
  document.getElementById(id) as T;
const status = (text: string) => {
  el("status").textContent = text;
};
const renderer = new THREE.WebGLRenderer({
  antialias: true,
  alpha: true,
  powerPreference: "high-performance",
});
renderer.setPixelRatio(Math.min(devicePixelRatio, 1.5));
renderer.setSize(innerWidth, innerHeight);
renderer.xr.enabled = true;
renderer.outputColorSpace = THREE.SRGBColorSpace;
el("scene").append(renderer.domElement);
renderer.domElement.tabIndex = 0;
const scene = new THREE.Scene();
scene.background = new THREE.Color(0x080f1c);
scene.fog = new THREE.FogExp2(0x080f1c, 0.028);
const camera = new THREE.PerspectiveCamera(
  58,
  innerWidth / innerHeight,
  0.05,
  90,
);
const rig = new THREE.Group();
scene.add(rig);
rig.add(camera);
camera.position.set(0, 1.65, 6.8);
camera.lookAt(0, 1.4, -3);
const environment = new THREE.Group();
scene.add(environment);
scene.add(new THREE.HemisphereLight(0x93a9c2, 0x0b1922, 2.1));
const light = new THREE.DirectionalLight(0xe1e8d6, 3);
light.position.set(2, 8, 4);
scene.add(light);
const mat = (color: number, emissive = 0, intensity = 0.3) =>
  new THREE.MeshStandardMaterial({
    color,
    emissive,
    emissiveIntensity: intensity,
    roughness: 0.6,
    metalness: 0.25,
  });
const box = (
  parent: THREE.Object3D,
  size: number[],
  pos: number[],
  material: THREE.Material,
) => {
  const m = new THREE.Mesh(
    new THREE.BoxGeometry(...(size as [number, number, number])),
    material,
  );
  m.position.set(...(pos as [number, number, number]));
  parent.add(m);
  return m;
};
box(environment, [32, 0.1, 32], [0, -0.06, -4], mat(0x101e2a));
const grid = new THREE.GridHelper(32, 32, 0x3d6461, 0x1c333f);
grid.position.y = 0.002;
environment.add(grid);
box(environment, [32, 9, 0.3], [0, 4, -10], mat(0x101925));
box(environment, [0.3, 9, 30], [-11, 4, -1], mat(0x15202a));
box(environment, [0.3, 9, 30], [11, 4, -1], mat(0x15202a));
for (let z = -9; z < 9; z += 3) {
  box(environment, [0.05, 0.025, 2], [-5, 0.02, z], mat(0x71d7c2, 0x32c9ad, 3));
  box(environment, [0.05, 0.025, 2], [5, 0.02, z], mat(0x71d7c2, 0x32c9ad, 3));
}
for (let x = -9; x <= 9; x += 3) {
  box(environment, [0.04, 5, 0.05], [x, 2.5, -9.8], mat(0x32676e, 0x1e848b, 2));
}
function textTexture(
  text: string,
  sub: string,
  color: string,
  width = 1024,
  height = 256,
) {
  const c = document.createElement("canvas");
  c.width = width;
  c.height = height;
  const ctx = c.getContext("2d")!;
  ctx.fillStyle = "#0c171e";
  ctx.fillRect(0, 0, width, height);
  ctx.strokeStyle = color;
  ctx.lineWidth = 6;
  ctx.strokeRect(10, 10, width - 20, height - 20);
  ctx.textAlign = "center";
  ctx.fillStyle = color;
  ctx.font = `bold ${height * 0.25}px sans-serif`;
  ctx.fillText(text, width / 2, height * 0.47);
  ctx.fillStyle = "#a5c1b9";
  ctx.font = `${height * 0.1}px monospace`;
  ctx.fillText(sub, width / 2, height * 0.74);
  const texture = new THREE.CanvasTexture(c);
  texture.colorSpace = THREE.SRGBColorSpace;
  return texture;
}
const sign = new THREE.Mesh(
  new THREE.PlaneGeometry(8, 2),
  new THREE.MeshBasicMaterial({
    map: textTexture(
      "ΦCADE // NEW DIMENSIONS",
      "PHI369 LABS · ALL PLAYERS WELCOME",
      "#c6f572",
    ),
  }),
);
sign.position.set(0, 5, -9.8);
environment.add(sign);
const range = new PhotonRange(),
  world = new HollerCircuit();
const canvases = [0, 1, 2].map(() => {
  const c = document.createElement("canvas");
  c.width = 640;
  c.height = 480;
  return c;
});
const textures = canvases.map((c) => {
  const t = new THREE.CanvasTexture(c);
  t.colorSpace = THREE.SRGBColorSpace;
  return t;
});
const cabinets: THREE.Group[] = [];
const screens: THREE.Mesh[] = [];
const interactables: THREE.Object3D[] = [];
const labels = ["PHOTON RANGE", "HOLLER CIRCUIT", "YOUR PHICADE"],
  colors = [0xc6f572, 0x70cbd8, 0xf7b56c];
for (let i = 0; i < 3; i++) {
  const c = new THREE.Group();
  c.position.set((i - 1) * 3, 0, -3.6);
  c.rotation.y = (1 - i) * 0.16;
  scene.add(c);
  cabinets.push(c);
  box(c, [2.1, 2.8, 1.2], [0, 1.4, -0.3], mat(0x14222b));
  box(c, [1.94, 0.9, 0.08], [0, 0.54, 0.35], mat(0x0b161f));
  box(c, [2.16, 0.12, 1.3], [0, 0.06, -0.3], mat(colors[i], colors[i], 0.5));
  for (const x of [-1.02, 1.02])
    box(c, [0.045, 2.7, 0.07], [x, 1.4, 0.32], mat(colors[i], colors[i], 1));
  box(c, [2.1, 0.24, 0.85], [0, 1.19, 0.52], mat(0x21343e));
  const screen = new THREE.Mesh(
    new THREE.PlaneGeometry(1.7, 1.275),
    new THREE.MeshBasicMaterial({ map: textures[i] }),
  );
  screen.position.set(0, 2.05, 0.315);
  screen.userData.game = i;
  screens.push(screen);
  interactables.push(screen);
  c.add(screen);
  const marquee = new THREE.Mesh(
    new THREE.PlaneGeometry(1.94, 0.42),
    new THREE.MeshBasicMaterial({
      map: textTexture(
        labels[i],
        "ΦCADE XR / ORIGINAL PLAY",
        `#${colors[i].toString(16)}`,
      ),
    }),
  );
  marquee.position.set(0, 2.94, 0.34);
  c.add(marquee);
  const joy = new THREE.Mesh(
    new THREE.CylinderGeometry(0.035, 0.035, 0.26, 10),
    mat(0x879d9e),
  );
  joy.position.set(-0.48, 1.46, 0.67);
  c.add(joy);
  const ball = new THREE.Mesh(
    new THREE.SphereGeometry(0.1, 12, 8),
    mat(colors[i], colors[i], 0.2),
  );
  ball.position.set(-0.48, 1.6, 0.67);
  ball.userData.game = i;
  ball.userData.control = "START";
  c.add(ball);
  interactables.push(ball);
  for (let b = 0; b < 2; b++) {
    const button = new THREE.Mesh(
      new THREE.CylinderGeometry(0.09, 0.09, 0.055, 16),
      mat(b ? 0xf1788b : colors[i]),
    );
    button.position.set(0.33 + b * 0.3, 1.36, 0.68);
    button.userData.game = i;
    button.userData.control = b ? "B" : "A";
    c.add(button);
    interactables.push(button);
  }
}
const theater = new THREE.Mesh(
  new THREE.PlaneGeometry(6.4, 4.8),
  new THREE.MeshBasicMaterial({ map: textures[0] }),
);
theater.position.set(0, 2.8, -3);
theater.visible = false;
scene.add(theater);
interactables.push(theater);
theater.userData.game = 0;
const worldGroup = new THREE.Group();
worldGroup.position.set(0, 0.1, -3);
worldGroup.scale.setScalar(0.28);
scene.add(worldGroup);
worldGroup.visible = false;
const worldCoins = new Map<string, THREE.Object3D>();
for (const tile of reconstruct(HOLLER_PROFILE, HOLLER_MAP)) {
  const x = tile.x - 5,
    z = tile.z - 4;
  if (tile.kind === "wall")
    box(worldGroup, [0.94, 1, 0.94], [x, 0.5, z], mat(0x2f6672, 0x173943, 0.3));
  else if (tile.kind === "coin") {
    const m = new THREE.Mesh(
      new THREE.OctahedronGeometry(0.22),
      mat(0xc6f572, 0xa6e95d, 1),
    );
    m.position.set(x, 0.35, z);
    worldCoins.set(`${tile.x},${tile.z}`, m);
    worldGroup.add(m);
  } else
    box(worldGroup, [0.8, 0.08, 0.8], [x, 0.04, z], mat(0xc97ce9, 0x8949ac, 1));
}
box(worldGroup, [11, 0.05, 9], [0, 0, 0], mat(0x142e36));
const player = new THREE.Mesh(
  new THREE.SphereGeometry(0.24, 12, 8),
  mat(0xf7b56c, 0xd58435, 0.8),
);
worldGroup.add(player);
const reticle = new THREE.Mesh(
  new THREE.RingGeometry(0.018, 0.024, 24),
  new THREE.MeshBasicMaterial({
    color: 0xc6f572,
    side: THREE.DoubleSide,
    depthTest: false,
  }),
);
reticle.visible = false;
reticle.renderOrder = 10;
scene.add(reticle);
const teleport = new THREE.Mesh(
  new THREE.RingGeometry(0.15, 0.19, 24),
  new THREE.MeshBasicMaterial({ color: 0x70f3c5, side: THREE.DoubleSide }),
);
teleport.rotation.x = -Math.PI / 2;
teleport.visible = false;
scene.add(teleport);
const guns = [
  new PhiLightGun(),
  new PhiLightGun({
    id: "photon-range-v1",
    hand: "left",
    seat: 2,
    offscreenReload: true,
    reloadButton: true,
    calibration: IDENTITY_CALIBRATION,
  }),
];
const motion = [new WiiMotionAdapter(), new WiiMotionAdapter()];
let localOwnsDemo = true;
export function setLocalCabinetControl(owns: boolean) {
  localOwnsDemo = owns;
}
let active = 0,
  theaterOn = false,
  worldMode = "screen",
  xrMode = "desktop",
  headlock = false,
  crt = true,
  aimGuide = true;
let calibrationPoints: [number, number][] = [];
let calibrating = false;
const calibrationTargets: [number, number][] = [
  [0.2, 0.2],
  [0.8, 0.2],
  [0.5, 0.8],
];
let remoteVideo: HTMLVideoElement | null = null;
let sendInput: (actions: XrAction[]) => void = () => {};
let sendPresence: (p: Presence) => void = () => {};
let remoteControls = false,
  remoteLightgun = false;
export function setRemoteBridge(
  video: HTMLVideoElement | null,
  input: typeof sendInput,
  presence: typeof sendPresence,
  owns: boolean,
  device: "gamepad" | "lightgun" = "gamepad",
) {
  remoteVideo = video;
  sendInput = input;
  sendPresence = presence;
  remoteControls = owns;
  remoteLightgun = device === "lightgun";
}
const raycaster = new THREE.Raycaster(),
  pointer = new THREE.Vector2();
const inverse = new THREE.Matrix4();
const origin = new THREE.Vector3(),
  direction = new THREE.Vector3();
function aimAt(
  screen: THREE.Mesh,
  ray: THREE.Ray,
  calibration: Calibration = IDENTITY_CALIBRATION,
): Aim {
  screen.updateWorldMatrix(true, false);
  inverse.copy(screen.matrixWorld).invert();
  const local = ray.clone().applyMatrix4(inverse);
  return mapScreenRay(
    local.origin,
    local.direction,
    screen === theater ? 6.4 : 1.7,
    screen === theater ? 4.8 : 1.275,
    calibration,
  );
}
function gameScreen() {
  return theaterOn ? theater : screens[active];
}
function choose(index: number) {
  active = index;
  theater.userData.game = index;
  (theater.material as THREE.MeshBasicMaterial).map = textures[index];
  document
    .querySelectorAll("[data-game]")
    .forEach((b, i) => b.classList.toggle("selected", i === index));
  el("game-title").textContent = labels[index];
  updatePresentation();
  status(
    index === 0
      ? "PHOTON RANGE — aim and fire. Point offscreen + trigger, or press R, to reload."
      : index === 1
        ? "HOLLER CIRCUIT — collect 3 lights, then reach the violet exit. Arrow keys / left stick to move."
        : "YOUR PHICADE — pair with XR Host on your Windows PC to play a live session.",
  );
}
function updatePresentation() {
  worldGroup.visible = active === 1 && worldMode !== "screen";
  theater.visible = theaterOn && !worldGroup.visible;
  cabinets.forEach((c) => (c.visible = !theaterOn && xrMode !== "ar"));
  if (xrMode === "ar") {
    theater.visible = !worldGroup.visible;
    theaterOn = true;
  }
  if (worldMode === "first" && active === 1) {
    worldGroup.scale.setScalar(1);
    worldGroup.position.set(0, 0, 0);
    rig.position.set(world.x - 5, 0, world.z - 4);
    camera.position.set(0, 1.5, 0);
  } else {
    worldGroup.scale.setScalar(xrMode === "ar" ? 0.16 : 0.28);
    worldGroup.position.set(0, xrMode === "ar" ? 0.75 : 0.3, -2);
  }
}
function fire(ray: THREE.Ray, trigger: boolean, reload = false, gunIndex = 0) {
  const gun = guns[gunIndex];
  const raw = aimAt(gameScreen(), ray);
  let aim = aimAt(gameScreen(), ray, gun.profile.calibration);
  if (active === 2 && remoteVideo)
    aim = mapVideoContent(aim, remoteVideo.videoWidth, remoteVideo.videoHeight);
  const result = gun.sample(aim, trigger, reload);
  if (calibrating && result.fired) {
    calibrationPoints.push([raw.u, raw.v]);
    if (calibrationPoints.length === 3) {
      try {
        gun.profile.calibration = calibrate(
          calibrationPoints,
          calibrationTargets,
        );
        localStorage.setItem(
          "phicade-xr-calibration",
          JSON.stringify(gun.profile.calibration),
        );
        status(
          "Aim calibrated for this screen. Reset when changing screen position.",
        );
      } catch (e) {
        status(String(e));
      }
      calibrating = false;
    } else
      status(
        `Calibration ${calibrationPoints.length + 1}/3: shoot the highlighted cross.`,
      );
    return result;
  }
  if (active === 0) {
    if (!localOwnsDemo) return result;
    if (result.reloaded) range.reload(gunIndex === 0 ? 1 : 2);
    if (result.fired) range.fire(aim.u, aim.v, gunIndex === 0 ? 1 : 2);
  } else if (active === 2 && remoteControls && remoteLightgun && gunIndex === 0)
    sendInput(result.actions);
  if (result.fired && gunIndex === 0) {
    reticle.position.copy(
      ray.at(
        Math.max(
          0.1,
          ray.origin.distanceTo(
            gameScreen().getWorldPosition(new THREE.Vector3()),
          ),
        ),
        new THREE.Vector3(),
      ),
    );
  }
  return result;
}
try {
  const saved = JSON.parse(
    localStorage.getItem("phicade-xr-calibration") ?? "null",
  );
  if (
    Array.isArray(saved) &&
    saved.length === 6 &&
    saved.every((x) => Number.isFinite(x) && Math.abs(x) <= 10)
  )
    guns[0].profile.calibration = saved as unknown as Calibration;
} catch {
  /* calibration is optional */
}
function select(ray: THREE.Ray) {
  raycaster.ray.copy(ray);
  const hits = raycaster.intersectObjects(
    interactables.filter((m) => m.visible && m.parent?.visible),
  );
  const hit = hits[0];
  if (!hit) return;
  if (hit.object.userData.menu !== undefined) {
    menuActions[Number(hit.object.userData.menu)]();
    return;
  }
  const index = Number(hit.object.userData.game);
  if (active !== index) {
    choose(index);
    return;
  }
  if (hit.object.userData.control) {
    if (active === 0) range.reload();
    else if (active === 1)
      world.move(
        hit.object.userData.control === "A" ? 1 : 0,
        hit.object.userData.control === "B" ? 1 : 0,
      );
    else if (remoteControls) {
      const button = String(hit.object.userData.control);
      sendInput([{ kind: "button", button, pressed: true }]);
      setTimeout(() => {
        if (remoteControls)
          sendInput([{ kind: "button", button, pressed: false }]);
      }, 70);
    }
  }
}
const keys = new Set<string>();
let drag = false,
  moved = false,
  lastMouse = { x: 0, y: 0 },
  mouseHeld = false;
renderer.domElement.addEventListener("pointerdown", (e) => {
  drag = true;
  moved = false;
  lastMouse = { x: e.clientX, y: e.clientY };
  renderer.domElement.focus();
});
renderer.domElement.addEventListener("pointermove", (e) => {
  pointer.set(
    (e.clientX / innerWidth) * 2 - 1,
    (-e.clientY / innerHeight) * 2 + 1,
  );
  if (
    drag &&
    (Math.abs(e.clientX - lastMouse.x) > 2 ||
      Math.abs(e.clientY - lastMouse.y) > 2)
  ) {
    moved = true;
    const euler = new THREE.Euler().setFromQuaternion(camera.quaternion, "YXZ");
    euler.y -= (e.clientX - lastMouse.x) * 0.004;
    euler.x = THREE.MathUtils.clamp(
      euler.x - (e.clientY - lastMouse.y) * 0.004,
      -1.2,
      1.2,
    );
    camera.quaternion.setFromEuler(euler);
  }
  lastMouse = { x: e.clientX, y: e.clientY };
});
renderer.domElement.addEventListener("pointerup", (e) => {
  drag = false;
  if (moved) return;
  pointer.set(
    (e.clientX / innerWidth) * 2 - 1,
    (-e.clientY / innerHeight) * 2 + 1,
  );
  raycaster.setFromCamera(pointer, camera);
  select(raycaster.ray);
  fire(raycaster.ray, true);
  mouseHeld = true;
  setTimeout(() => {
    fire(raycaster.ray, false);
    mouseHeld = false;
  }, 80);
});
const keyMap: Record<string, string> = {
  ArrowUp: "UP",
  ArrowDown: "DOWN",
  ArrowLeft: "LEFT",
  ArrowRight: "RIGHT",
  z: "A",
  x: "B",
  Enter: "START",
  " ": "SELECT",
};
function keyEvent(e: KeyboardEvent, pressed: boolean) {
  if ((e.target as HTMLElement)?.matches("input,textarea,select,button"))
    return;
  if (keyMap[e.key]) {
    e.preventDefault();
    if (active === 1 && localOwnsDemo && pressed && !e.repeat) {
      const moves: Record<string, [number, number]> = {
        ArrowUp: [0, -1],
        ArrowDown: [0, 1],
        ArrowLeft: [-1, 0],
        ArrowRight: [1, 0],
      };
      if (moves[e.key]) world.move(...moves[e.key]);
    }
    if (active === 2 && remoteControls)
      sendInput([{ kind: "button", button: keyMap[e.key], pressed }]);
  }
  if (pressed) keys.add(e.key.toLowerCase());
  else keys.delete(e.key.toLowerCase());
  if (pressed && e.key.toLowerCase() === "r") range.reload();
}
window.addEventListener("keydown", (e) => keyEvent(e, true));
window.addEventListener("keyup", (e) => keyEvent(e, false));
window.addEventListener("blur", () => {
  keys.clear();
  mouseHeld = false;
  if (remoteControls) sendInput(neutralInput(remoteLightgun));
});
document
  .querySelectorAll<HTMLButtonElement>("[data-game]")
  .forEach((b, i) => (b.onclick = () => choose(i)));
el("reload").onclick = () => {
  range.reload();
  if (remoteControls && remoteLightgun)
    sendInput([
      { kind: "button", button: "GUN_RELOAD", pressed: true },
      { kind: "button", button: "GUN_RELOAD", pressed: false },
    ]);
  status("Reloaded.");
};
el("theater").onclick = () => {
  theaterOn = !theaterOn;
  updatePresentation();
};
el<HTMLSelectElement>("world-mode").onchange = (e) => {
  worldMode = (e.target as HTMLSelectElement).value;
  choose(1);
};
el<HTMLInputElement>("crt").onchange = (e) =>
  (crt = (e.target as HTMLInputElement).checked);
el<HTMLInputElement>("aim").onchange = (e) =>
  (aimGuide = (e.target as HTMLInputElement).checked);
el<HTMLInputElement>("headlock").onchange = (e) => {
  headlock = (e.target as HTMLInputElement).checked;
  if (headlock) {
    theaterOn = true;
    updatePresentation();
  }
};
el<HTMLInputElement>("scale").oninput = (e) => {
  const v = Number((e.target as HTMLInputElement).value);
  theater.scale.setScalar(v);
};
el<HTMLSelectElement>("hand").onchange = (e) => {
  const hand = (e.target as HTMLSelectElement).value;
  guns[0].profile.hand = hand === "left" ? "left" : "right";
  status(
    hand === "dual"
      ? "Dual guns: right is player 1, left is player 2. Native core topology remains explicit."
      : `${hand} hand selected.`,
  );
};
el<HTMLSelectElement>("quality").onchange = (e) => {
  if (renderer.xr.isPresenting) {
    status("Exit XR before changing render scale.");
    return;
  }
  const q = Number((e.target as HTMLSelectElement).value);
  renderer.xr.setFramebufferScaleFactor(q);
  renderer.setPixelRatio(Math.min(devicePixelRatio, 1.5) * q);
};
el("calibrate").onclick = () => {
  calibrationPoints = [];
  calibrating = true;
  guns[0].profile.calibration = IDENTITY_CALIBRATION;
  choose(active === 2 ? 2 : 0);
  status("Calibration 1/3: shoot the highlighted cross on the screen.");
};
function recenter() {
  rig.position.set(0, 0, 0);
  rig.rotation.set(0, 0, 0);
  camera.position.set(
    0,
    renderer.xr.isPresenting ? 0 : 1.65,
    renderer.xr.isPresenting ? 4 : 6.8,
  );
  camera.lookAt(0, 1.4, -3);
  theater.position.set(0, 2.4, -3);
  theater.rotation.set(0, 0, 0);
  worldMode = "screen";
  el<HTMLSelectElement>("world-mode").value = "screen";
  updatePresentation();
}
el("reset").onclick = recenter;
el("hide-ui").onclick = () => {
  el("ui").classList.toggle("minimal");
  el("hide-ui").textContent = el("ui").classList.contains("minimal")
    ? "Show controls"
    : "Hide controls";
};
el("place").onclick = () => {
  headlock = false;
  el<HTMLInputElement>("headlock").checked = false;
  const view = renderer.xr.isPresenting ? renderer.xr.getCamera() : camera;
  view.getWorldPosition(origin);
  view.getWorldDirection(direction);
  theater.position.copy(origin).addScaledVector(direction, 2);
  theater.quaternion.copy(view.getWorldQuaternion(new THREE.Quaternion()));
  theaterOn = true;
  updatePresentation();
  status("Screen placed. Stay aware of your room and headset boundary.");
};
let hitSource: XRHitTestSource | null = null,
  hitPose: THREE.Matrix4 | null = null;
async function enterXR(mode: "immersive-vr" | "immersive-ar") {
  if (!navigator.xr || !isSecureContext) {
    status(
      "WebXR needs HTTPS (or localhost) and a supported headset browser. Desktop preview is available.",
    );
    return;
  }
  if (renderer.xr.isPresenting) {
    await renderer.xr.getSession()?.end();
    return;
  }
  try {
    let actual = mode;
    if (!(await navigator.xr.isSessionSupported(mode))) {
      if (
        mode === "immersive-ar" &&
        (await navigator.xr.isSessionSupported("immersive-vr"))
      ) {
        actual = "immersive-vr";
        status("Passthrough unavailable; entering VR.");
      } else
        throw new Error(
          "Requested XR mode is unavailable; use desktop preview.",
        );
    }
    const session = await navigator.xr.requestSession(actual, {
      optionalFeatures: [
        "local-floor",
        "bounded-floor",
        ...(actual === "immersive-ar" ? ["hit-test", "dom-overlay"] : []),
      ],
      ...(actual === "immersive-ar" ? { domOverlay: { root: el("ui") } } : {}),
    });
    renderer.xr.setReferenceSpaceType("local-floor");
    await renderer.xr.setSession(session);
    xrMode = actual === "immersive-ar" ? "ar" : "vr";
    environment.visible = xrMode !== "ar";
    scene.background = xrMode === "ar" ? null : new THREE.Color(0x080f1c);
    scene.fog = xrMode === "ar" ? null : new THREE.FogExp2(0x080f1c, 0.028);
    el("ui").classList.add("xr-active");
    rig.position.set(0, 0, 4);
    camera.position.set(0, 0, 0);
    updatePresentation();
    if (xrMode === "ar" && session.requestHitTestSource) {
      try {
        hitSource =
          (await session.requestHitTestSource({
            space: await session.requestReferenceSpace("viewer"),
          })) ?? null;
      } catch {
        status(
          "Surface hit testing unavailable. Use Place screen for world placement.",
        );
      }
    }
    session.addEventListener("end", () => {
      hitSource?.cancel();
      hitSource = null;
      hitPose = null;
      xrMode = "desktop";
      environment.visible = true;
      scene.background = new THREE.Color(0x080f1c);
      scene.fog = new THREE.FogExp2(0x080f1c, 0.028);
      el("ui").classList.remove("xr-active");
      if (remoteControls) sendInput(neutralInput(remoteLightgun));
      recenter();
    });
    session.addEventListener("visibilitychange", () => {
      if (session.visibilityState !== "visible" && remoteControls)
        sendInput(neutralInput(remoteLightgun));
    });
  } catch (e) {
    status(String(e));
  }
}
el("vr").onclick = () => void enterXR("immersive-vr");
el("ar").onclick = () => void enterXR("immersive-ar");
if (navigator.xr && isSecureContext)
  Promise.all([
    navigator.xr.isSessionSupported("immersive-vr"),
    navigator.xr.isSessionSupported("immersive-ar"),
  ])
    .then(
      ([vr, ar]) =>
        (el("capability").textContent = ar
          ? "VR + PASSTHROUGH READY"
          : vr
            ? "VR READY"
            : "DESKTOP PREVIEW"),
    )
    .catch(() =>
      status("XR capabilities unavailable. Desktop preview is ready."),
    );
const controllers = [
  renderer.xr.getController(0),
  renderer.xr.getController(1),
];
const squeezing = new Set<number>();
controllers.forEach((controller, i) => {
  rig.add(controller);
  const line = new THREE.Line(
    new THREE.BufferGeometry().setFromPoints([
      new THREE.Vector3(),
      new THREE.Vector3(0, 0, -6),
    ]),
    new THREE.LineBasicMaterial({
      color: colors[i],
      transparent: true,
      opacity: 0.65,
    }),
  );
  controller.add(line);
  controller.addEventListener("connected", (e) => {
    controller.userData.input = e.data;
  });
  controller.addEventListener("disconnected", () => {
    controller.userData.input = null;
    motion[i].reset();
    if (remoteControls) sendInput(neutralInput(remoteLightgun));
  });
  controller.addEventListener("selectstart", () => {
    controller.userData.trigger = true;
    controller.getWorldPosition(origin);
    direction
      .set(0, 0, -1)
      .applyQuaternion(controller.getWorldQuaternion(new THREE.Quaternion()));
    select(new THREE.Ray(origin.clone(), direction.clone()));
  });
  controller.addEventListener(
    "selectend",
    () => (controller.userData.trigger = false),
  );
  controller.addEventListener("squeezestart", () => squeezing.add(i));
  controller.addEventListener("squeezeend", () => {
    squeezing.delete(i);
    if (xrMode === "ar" && hitPose) {
      theater.position.setFromMatrixPosition(hitPose);
      theater.position.y += 0.6;
      theater.lookAt(
        renderer.xr.getCamera().getWorldPosition(new THREE.Vector3()),
      );
      status("Screen placed on the detected surface.");
    } else if (teleport.visible) {
      const head = renderer.xr
        .getCamera()
        .getWorldPosition(new THREE.Vector3());
      rig.position.x += teleport.position.x - head.x;
      rig.position.z += teleport.position.z - head.z;
    }
    teleport.visible = false;
  });
});
// Ray-operated controls remain usable inside VR even when DOM overlays are absent.
const menu = new THREE.Group();
scene.add(menu);
menu.position.set(0, 3.8, -3.1);
const menuActions = [
  () => choose(0),
  () => choose(1),
  () => choose(2),
  () => {
    theaterOn = !theaterOn;
    updatePresentation();
  },
  () => {
    range.reload();
    world.reset();
  },
  () => {
    worldMode =
      worldMode === "screen"
        ? "diorama"
        : worldMode === "diorama"
          ? "first"
          : "screen";
    choose(1);
  },
  () => void renderer.xr.getSession()?.end(),
];
["RANGE", "WORLD", "PC", "THEATER", "RELOAD", "3D MODE", "EXIT XR"].forEach(
  (label, i) => {
    const m = new THREE.Mesh(
      new THREE.PlaneGeometry(0.75, 0.28),
      new THREE.MeshBasicMaterial({
        map: textTexture(label, "", "#c6f572", 512, 160),
      }),
    );
    m.position.set((i - 3) * 0.83, 0, 0);
    m.userData.menu = i;
    menu.add(m);
    interactables.push(m);
  },
);
// Handle the small in-world menu before the cabinet click path.
const remoteAvatar = new THREE.Group();
scene.add(remoteAvatar);
remoteAvatar.visible = false;
const remoteHead = new THREE.Mesh(
  new THREE.BoxGeometry(0.23, 0.24, 0.2),
  mat(0xc97ce9, 0x643e81, 0.7),
);
remoteAvatar.add(remoteHead);
const remoteHands = [0, 1].map(() => {
  const m = new THREE.Mesh(new THREE.SphereGeometry(0.07, 8, 8), mat(0x70cbd8));
  remoteAvatar.add(m);
  return m;
});
export function showRemotePresence(p: Presence | null) {
  remoteAvatar.visible = !!p;
  if (!p) return;
  [p.head, p.left, p.right].forEach((pose, i) => {
    const m = i === 0 ? remoteHead : remoteHands[i - 1];
    m.visible = !!pose;
    if (pose) {
      m.position.copy(pose.position);
      m.quaternion.copy(pose.orientation);
    }
  });
}
function poseOf(o: THREE.Object3D): Pose {
  const p = o.getWorldPosition(new THREE.Vector3()),
    q = o.getWorldQuaternion(new THREE.Quaternion());
  return {
    position: { x: p.x, y: p.y, z: p.z },
    orientation: { x: q.x, y: q.y, z: q.z, w: q.w },
  };
}
function drawScreens(time: number) {
  const c = canvases[0].getContext("2d")!;
  c.fillStyle = "#081b27";
  c.fillRect(0, 0, 640, 480);
  c.strokeStyle = "#193747";
  c.lineWidth = 1;
  for (let i = 0; i < 640; i += 40) {
    c.beginPath();
    c.moveTo(i, 0);
    c.lineTo(i, 480);
    c.stroke();
  }
  for (let i = 0; i < 480; i += 40) {
    c.beginPath();
    c.moveTo(0, i);
    c.lineTo(640, i);
    c.stroke();
  }
  const tx = range.target.u * 640,
    ty = range.target.v * 480;
  c.strokeStyle = "#c6f572";
  c.lineWidth = 6;
  c.beginPath();
  c.arc(tx, ty, 40, 0, Math.PI * 2);
  c.stroke();
  c.fillStyle = "#f7b56c";
  c.beginPath();
  c.arc(tx, ty, 12, 0, Math.PI * 2);
  c.fill();
  c.fillStyle = "#cde9df";
  c.font = "bold 22px monospace";
  c.fillText(
    `PHOTON RANGE   ${range.score.toString().padStart(5, "0")}`,
    20,
    32,
  );
  c.font = "16px monospace";
  c.fillText(
    `P1 ${range.ammo[0]}/6   P2 ${range.ammo[1]}/6     OFFSCREEN = RELOAD`,
    20,
    457,
  );
  const w = canvases[1].getContext("2d")!;
  w.fillStyle = "#081b27";
  w.fillRect(0, 0, 640, 480);
  w.fillStyle = "#70cbd8";
  w.font = "bold 20px monospace";
  w.fillText("HOLLER CIRCUIT / COLLECT THE LIGHT", 20, 30);
  const unit = 42,
    ox = 89,
    oy = 50;
  for (let z = 0; z < HOLLER_MAP.length; z++)
    for (let x = 0; x < HOLLER_MAP[z].length; x++) {
      const t = HOLLER_MAP[z][x];
      w.fillStyle = t === "#" ? "#2f6672" : "#102b35";
      w.fillRect(ox + x * unit, oy + z * unit, unit - 2, unit - 2);
      if (t === "o" && !world.collected.has(`${x},${z}`)) {
        w.fillStyle = "#c6f572";
        w.beginPath();
        w.arc(ox + x * unit + 20, oy + z * unit + 20, 7, 0, Math.PI * 2);
        w.fill();
      }
      if (t === "E") {
        w.fillStyle = "#c97ce9";
        w.fillRect(ox + x * unit + 10, oy + z * unit + 10, 20, 20);
      }
    }
  w.fillStyle = "#f7b56c";
  w.fillRect(ox + world.x * unit + 8, oy + world.z * unit + 8, 24, 24);
  w.fillStyle = "#cde9df";
  w.font = "16px monospace";
  w.fillText(
    world.won
      ? "ALL LIGHTS HOME. YOU DID IT!"
      : `${world.collected.size}/3 LIGHTS · ${world.steps} STEPS · ARROWS / STICK`,
    24,
    457,
  );
  const r = canvases[2].getContext("2d")!;
  r.fillStyle = "#101b27";
  r.fillRect(0, 0, 640, 480);
  if (remoteVideo && remoteVideo.readyState >= 2) {
    const aspect = remoteVideo.videoWidth / remoteVideo.videoHeight;
    const h = Math.min(480, 640 / aspect),
      width = h * aspect;
    r.drawImage(remoteVideo, (640 - width) / 2, (480 - h) / 2, width, h);
  } else {
    r.fillStyle = "#f7b56c";
    r.textAlign = "center";
    r.font = "bold 32px sans-serif";
    r.fillText("YOUR GAMES. THIS SPACE.", 320, 200);
    r.fillStyle = "#8fa9a0";
    r.font = "16px monospace";
    r.fillText("PAIR WITH PHICADE XR HOST ON YOUR PC", 320, 245);
    r.textAlign = "left";
  }
  if (calibrating) {
    const t = calibrationTargets[calibrationPoints.length],
      ctx = canvases[active].getContext("2d")!;
    ctx.strokeStyle = "#ff6db3";
    ctx.lineWidth = 5;
    ctx.beginPath();
    ctx.moveTo(t[0] * 640 - 20, t[1] * 480);
    ctx.lineTo(t[0] * 640 + 20, t[1] * 480);
    ctx.moveTo(t[0] * 640, t[1] * 480 - 20);
    ctx.lineTo(t[0] * 640, t[1] * 480 + 20);
    ctx.stroke();
  }
  if (crt)
    for (const canvas of canvases) {
      const ctx = canvas.getContext("2d")!;
      ctx.fillStyle = "#00000030";
      for (let y = 0; y < 480; y += 4) ctx.fillRect(0, y, 640, 1);
    }
  textures.forEach((t) => (t.needsUpdate = true));
  player.position.set(world.x - 5, 0.3, world.z - 4);
  worldCoins.forEach((m, id) => {
    m.visible = !world.collected.has(id);
    m.rotation.y = time * 0.001;
  });
}
let lastTime = 0,
  drawAt = 0,
  presenceAt = 0,
  gameMoveAt = 0,
  snapAt = 0,
  frames = 0,
  fpsAt = 0,
  remotePadAt = 0;
renderer.setAnimationLoop((time, frame) => {
  const dt = Math.min((time - lastTime) / 1000, 0.05) || 0.016;
  lastTime = time;
  frames++;
  if (time - fpsAt > 1000) {
    el("stats").textContent =
      `${Math.round((frames * 1000) / (time - fpsAt))} FPS`;
    frames = 0;
    fpsAt = time;
    el("score").textContent =
      active === 0
        ? `${range.score} POINTS · ${range.ammo[0]} / 6 AMMO`
        : active === 1
          ? `${world.collected.size}/3 LIGHTS · ${world.steps} STEPS`
          : remoteControls
            ? "PAIRED · CONTROLLER"
            : "PAIR WITH YOUR PC";
  }
  if (!renderer.xr.isPresenting) {
    const step = 2.4 * dt;
    direction.set(0, 0, 0);
    if (keys.has("w")) direction.z -= step;
    if (keys.has("s")) direction.z += step;
    if (keys.has("a")) direction.x -= step;
    if (keys.has("d")) direction.x += step;
    direction.applyQuaternion(camera.quaternion);
    direction.y = 0;
    camera.position.add(direction);
    camera.position.x = THREE.MathUtils.clamp(camera.position.x, -9, 9);
    camera.position.z = THREE.MathUtils.clamp(camera.position.z, -8, 12);
    raycaster.setFromCamera(pointer, camera);
    const a = aimAt(gameScreen(), raycaster.ray);
    reticle.visible = aimGuide && !a.offscreen && !worldGroup.visible;
    if (reticle.visible) {
      const hit = raycaster.intersectObject(gameScreen())[0];
      if (hit) {
        reticle.position.copy(hit.point);
        reticle.quaternion.copy(
          gameScreen().getWorldQuaternion(new THREE.Quaternion()),
        );
        reticle.position.addScaledVector(raycaster.ray.direction, -0.01);
      }
    }
  }
  if (frame && hitSource) {
    const hit = frame.getHitTestResults(hitSource)[0];
    const pose = hit?.getPose(renderer.xr.getReferenceSpace()!);
    hitPose = pose
      ? new THREE.Matrix4().fromArray(pose.transform.matrix)
      : null;
  }
  const remotePadActions: XrAction[] = [];
  for (let i = 0; i < controllers.length; i++) {
    const c = controllers[i],
      input = c.userData.input as XRInputSource | undefined;
    if (!input) continue;
    c.getWorldPosition(origin);
    direction
      .set(0, 0, -1)
      .applyQuaternion(c.getWorldQuaternion(new THREE.Quaternion()));
    const ray = new THREE.Ray(origin.clone(), direction.clone());
    const hand = el<HTMLSelectElement>("hand").value;
    const gunIndex =
      hand === "dual" ? (input.handedness === "left" ? 1 : 0) : 0;
    const motionSample = motion[i].sample(
      poseOf(c),
      time,
      input.gamepad?.axes.slice(-2),
    );
    const reloadGesture = el<HTMLSelectElement>("reload-gesture").value;
    if (hand === "dual" || input.handedness === guns[0].profile.hand) {
      const result = fire(
        ray,
        !!c.userData.trigger,
        !!input.gamepad?.buttons[4]?.pressed ||
          (reloadGesture === "shake" && motionSample.shake) ||
          (reloadGesture === "swing" && motionSample.swing),
        gunIndex,
      );
      if (result.fired)
        void input.gamepad?.hapticActuators?.[0]?.pulse(0.4, 35);
      const a = aimAt(gameScreen(), ray);
      reticle.visible = aimGuide && !a.offscreen && !worldGroup.visible;
      const hit = new THREE.Raycaster(
        ray.origin,
        ray.direction,
      ).intersectObject(gameScreen())[0];
      if (hit) {
        reticle.position.copy(hit.point).addScaledVector(ray.direction, -0.01);
        reticle.quaternion.copy(
          gameScreen().getWorldQuaternion(new THREE.Quaternion()),
        );
      }
    }
    if (squeezing.has(i) && direction.y < -0.1 && xrMode !== "ar") {
      const t = -origin.y / direction.y;
      const p = ray.at(t, new THREE.Vector3());
      teleport.visible =
        t > 0 && t < 12 && Math.abs(p.x) < 9 && p.z > -8 && p.z < 10;
      if (teleport.visible) teleport.position.set(p.x, 0.015, p.z);
    }
    const axes = input.gamepad?.axes ?? [];
    const x = axes[axes.length - 2] ?? 0,
      z = axes[axes.length - 1] ?? 0;
    if (input.handedness === "left") {
      if (
        active === 1 &&
        localOwnsDemo &&
        time - gameMoveAt > 170 &&
        (Math.abs(x) > 0.6 || Math.abs(z) > 0.6)
      ) {
        world.move(
          Math.abs(x) > Math.abs(z) ? Math.sign(x) : 0,
          Math.abs(z) >= Math.abs(x) ? Math.sign(z) : 0,
        );
        gameMoveAt = time;
      } else if (
        el<HTMLSelectElement>("locomotion").value === "smooth" &&
        active !== 1 &&
        (active !== 2 || !remoteControls)
      ) {
        const move = new THREE.Vector3(x, 0, z)
          .multiplyScalar(dt * 1.7)
          .applyQuaternion(rig.quaternion);
        rig.position.add(move);
        rig.position.x = THREE.MathUtils.clamp(rig.position.x, -8, 8);
        rig.position.z = THREE.MathUtils.clamp(rig.position.z, -7, 10);
      }
    } else if (
      (active !== 2 || !remoteControls) &&
      Math.abs(x) > 0.75 &&
      time - snapAt > 400
    ) {
      rig.rotation.y -= (Math.sign(x) * Math.PI) / 6;
      snapAt = time;
    }
    if (
      active === 2 &&
      remoteControls &&
      input.gamepad &&
      time - remotePadAt > 33 &&
      input.handedness !== "none"
    ) {
      remotePadActions.push(...questGamepad(input.gamepad, input.handedness));
    }
  }
  if (remotePadActions.length) {
    sendInput(remotePadActions);
    remotePadAt = time;
  }
  if (active === 1 && worldMode === "first") {
    rig.position.set(world.x - 5, 0, world.z - 4);
    if (!Number.isFinite(rig.position.x)) {
      worldMode = "screen";
      recenter();
      status("WorldWalk camera unavailable; returned to spatial screen.");
    }
  }
  if (headlock) {
    const view = renderer.xr.isPresenting ? renderer.xr.getCamera() : camera;
    view.getWorldPosition(origin);
    view.getWorldDirection(direction);
    theater.position.copy(origin).addScaledVector(direction, 2);
    theater.quaternion.copy(view.getWorldQuaternion(new THREE.Quaternion()));
  }
  if (time - presenceAt > 100) {
    sendPresence({
      head: poseOf(renderer.xr.isPresenting ? renderer.xr.getCamera() : camera),
      left: controllers.find((c) => c.userData.input?.handedness === "left")
        ? poseOf(
            controllers.find((c) => c.userData.input?.handedness === "left")!,
          )
        : null,
      right: controllers.find((c) => c.userData.input?.handedness === "right")
        ? poseOf(
            controllers.find((c) => c.userData.input?.handedness === "right")!,
          )
        : null,
    });
    presenceAt = time;
  }
  if (time - drawAt > 33) {
    drawScreens(time);
    drawAt = time;
  }
  renderer.render(scene, camera);
});
window.addEventListener("resize", () => {
  camera.aspect = innerWidth / innerHeight;
  camera.updateProjectionMatrix();
  renderer.setSize(innerWidth, innerHeight);
});
// Dynamic bridge module keeps transport independent of the arcade renderer.
void import("./pairing").then((m) =>
  m.installPairing({
    status,
    choose,
    setRemoteBridge,
    showRemotePresence,
    canvases,
    range,
    world,
    setLocalCabinetControl,
  }),
);
drawScreens(0);
