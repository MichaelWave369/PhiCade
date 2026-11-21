/**
 * Creates a bounded, source-credited metadata snapshot for the static Pages site.
 * No copyrighted games, binaries, firmware, source archives or artwork fetched.
 */
import { readFile, writeFile, mkdir } from "node:fs/promises";
import { dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { normalizeCatalog, PUBLIC_CATALOG_URL } from "./catalog.mjs";

const seed = fileURLToPath(new URL("./seed.json", import.meta.url));

async function fetchBoundedJson(url) {
  const response = await fetch(url, { signal: AbortSignal.timeout(18_000), headers: { Accept: "application/json" } });
  if (!response.ok || !response.body) throw new Error("Catalog unavailable: HTTP " + response.status);
  const announced = Number(response.headers.get("content-length"));
  const limit = 20 * 1024 * 1024;
  if (announced > limit) throw new Error("Catalog too large");
  let size = 0;
  const chunks = [];
  for await (const chunk of response.body) {
    size += chunk.length;
    if (size > limit) throw new Error("Catalog exceeded 20 MiB limit");
    chunks.push(chunk);
  }
  return JSON.parse(Buffer.concat(chunks).toString("utf8"));
}

export async function build({ output, fixture } = {}) {
  if (!output) throw new Error("usage: node build-catalog.mjs --output <file> [--fixture <test.json>]");
  let data, sourceStatus;
  try {
    if (fixture) {
      data = JSON.parse(await readFile(fixture, "utf8"));
      sourceStatus = "test-fixture";
    } else {
      data = await fetchBoundedJson(PUBLIC_CATALOG_URL);
      sourceStatus = "upstream-snapshot";
    }
    const result = normalizeCatalog(data);
    await mkdir(dirname(output), { recursive: true });
    await writeFile(output, JSON.stringify({ ...result, sourceStatus }) + "\n", "utf8");
    console.log("Discovery: " + result.sampledCount + " of " + result.upstreamCount + " upstream listings; " + sourceStatus);
    return result;
  } catch (error) {
    if (fixture) throw error;
    const fallback = normalizeCatalog(JSON.parse(await readFile(seed, "utf8")));
    await mkdir(dirname(output), { recursive: true });
    await writeFile(output, JSON.stringify({
      ...fallback, sourceStatus: "curated-fallback", upstreamCount: null,
      upstreamGeneratedAt: null, error: "Upstream catalog unavailable; sample only"
    }) + "\n", "utf8");
    console.warn("Discovery: upstream unavailable (" + error.message + "); publishing a labeled starter index.");
    return fallback;
  }
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  const outAt = process.argv.indexOf("--output");
  const fixtureAt = process.argv.indexOf("--fixture");
  build({ output: outAt > -1 ? process.argv[outAt + 1] : "", fixture: fixtureAt > -1 ? process.argv[fixtureAt + 1] : undefined })
    .catch(e => { console.error("Discovery build: " + e.message); process.exitCode = 1; });
}
