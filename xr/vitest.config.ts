import { defineConfig } from "vitest/config";
export default defineConfig({test:{include:["xr/tests/**/*.test.ts"],environment:"node"}});
