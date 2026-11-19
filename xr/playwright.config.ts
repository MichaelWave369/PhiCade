import { defineConfig } from "@playwright/test";
export default defineConfig({
  testDir: "./tests/browser",
  timeout: 45_000,
  workers: 1,
  use: {
    baseURL: "http://127.0.0.1:1430",
    viewport: { width: 1536, height: 960 },
    launchOptions: {
      args: [
        "--use-gl=angle",
        "--use-angle=swiftshader",
        "--enable-unsafe-swiftshader",
      ],
      ...(process.env.XR_BROWSER_PATH
        ? { executablePath: process.env.XR_BROWSER_PATH }
        : {}),
    },
  },
  webServer: {
    command:
      "npm exec --workspace @phicade/xr -- vite --host 127.0.0.1 --port 1430",
    url: "http://127.0.0.1:1430",
    reuseExistingServer: !process.env.CI,
  },
  reporter: [["list"], ["html", { open: "never" }]],
});
