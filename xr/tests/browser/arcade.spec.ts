import { test,expect } from "@playwright/test";
test("desktop arcade renders, shoots a target and reconstructs the canonical tile scene",async({page})=>{
  const errors:string[]=[];page.on("pageerror",e=>errors.push(e.message));
  await page.goto("/");await expect(page.locator("#scene canvas")).toBeVisible();
  await expect(page.locator("#stats")).toContainText("FPS");
  await page.getByRole("button",{name:"Theater view"}).click();
  // Project the known target center through the same initial perspective camera.
  await page.locator("#scene canvas").click({position:{x:768,y:356}});
  await expect(page.locator("#score")).toContainText("100 POINTS");
  await page.getByRole("button",{name:"Reload [R]"}).click();await expect(page.locator("#score")).toContainText("6 / 6");
  await page.locator('[data-game="world"]').click();await page.locator("#scene canvas").focus();await page.keyboard.press("ArrowRight");await expect(page.locator("#score")).toContainText("1 STEPS");
  await page.locator("#world-mode").selectOption("diorama");await expect(page.locator("#game-title")).toHaveText("HOLLER CIRCUIT");
  await page.locator("#world-mode").selectOption("first");await page.getByRole("button",{name:"Recenter"}).click();
  await page.getByRole("button",{name:"Mixed reality"}).click();await expect(page.locator("#status")).toContainText(/unavailable|HTTPS|preview/i);
  await page.screenshot({path:"artifacts/xr-arcade-desktop.png"});expect(errors).toEqual([]);
});
test("malformed host offers cannot grant input or establish a connection",async({page})=>{
  await page.goto("/");await page.locator("#pairing").evaluate(e=>(e as HTMLDetailsElement).open=true);await page.locator("#offer").fill("bad-pairing-code");await page.locator("#connect").click();await expect(page.locator("#status")).toContainText(/Error|Invalid/i);await expect(page.locator("#answer")).toHaveValue("");
});
test("two browsers exchange WebRTC video, presence and bounded input",async({browser})=>{
  const host=await browser.newPage(),guest=await browser.newPage();await host.goto("/");await guest.goto("/");
  for(const page of [host,guest])await page.locator("#pairing").evaluate(e=>(e as HTMLDetailsElement).open=true);
  await host.locator("#host-room").click();await expect(host.locator("#room-offer")).not.toHaveValue("");
  await guest.locator("#offer").fill(await host.locator("#room-offer").inputValue());await guest.locator("#connect").click();await expect(guest.locator("#answer")).not.toHaveValue("");
  await host.locator("#room-answer").fill(await guest.locator("#answer").inputValue());await host.locator("#accept-room").click();
  await expect(guest.locator("#capability")).toContainText("PAIRED",{timeout:20_000});await expect(guest.locator("#game-title")).toHaveText("YOUR PHICADE");await expect(guest.locator("#score")).toContainText("CONTROLLER");
  const stats=await guest.locator("video").evaluate(v=>({width:(v as HTMLVideoElement).videoWidth,height:(v as HTMLVideoElement).videoHeight}));expect(stats.width).toBeGreaterThan(0);expect(stats.height).toBeGreaterThan(0);
  await guest.locator("#disconnect").click();await expect(guest.locator("#status")).toContainText("Disconnected");await host.close();await guest.close();
});
