import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { normalizeCatalog, filterCatalog, safeOutboundLink } from "../discovery/catalog.mjs";

const fixture = JSON.parse(await readFile(new URL("../discovery/seed.json", import.meta.url), "utf8"));

test("source-backed seed produces bounded, explicit metadata cards", () => {
  const s = normalizeCatalog(fixture);
  assert.equal(s.sampledCount, 5);
  assert.equal(s.upstreamCount, 5);
  assert.equal(s.games.find(x => x.console === "Xbox").title, "Halo: Combat Evolved");
  assert(s.games.every(x => safeOutboundLink(x.sourceUrl)));
  assert(s.games.every(x => x.completeness === null));
});

test("title, console, and target-platform filters combine without guessing", () => {
  const { games } = normalizeCatalog(fixture);
  assert.equal(filterCatalog(games, { query: "super mario" }).length, 1);
  assert.equal(filterCatalog(games, { console: "Nintendo 64", platform: "Linux" }).length, 2);
  assert.equal(filterCatalog(games, { console: "Xbox", query: "halo" }).length, 1);
  assert.equal(filterCatalog(games, { console: "SNES", query: "halo" }).length, 0);
});

test("invalid schema, oversized catalogs, and unsafe URLs fail safely", () => {
  assert.throws(() => normalizeCatalog({ schema: 3, games: [] }), /Unsupported catalog/);
  assert.throws(() => normalizeCatalog({ ...fixture, games: Array(12001).fill({}) }), /exceeds review limit/);
  assert.equal(safeOutboundLink("javascript:alert(1)"), false);
  assert.equal(safeOutboundLink("https://github.com.evil.test/repo"), false);
  assert.equal(safeOutboundLink("https://decompbuddy.com/games/super-mario-64-nintendo-64-sm64pc"), true);
});

test("normalization strips unsupported links and ignores executable instructions", () => {
  const malicious = {
    schema: 1, count: 1,
    games: [{ title: "<img onerror=alert(1)>", slug: "fixture-test", console: "PC", project: "demo",
      repoUrl: "https://evil.example/install.exe", platforms: ["Windows"], facts: { completeness: "unknown" },
      installCommand: "curl bad | bash" }],
  };
  const result = normalizeCatalog(malicious);
  assert.equal(result.games[0].repoUrl, null);
  assert.equal(result.games[0].title, "<img onerror=alert(1)>");
  assert(!JSON.stringify(result).includes("installCommand"));
});

test("sampling spans a large list and is consistently capped", () => {
  const games = Array.from({ length: 4038 }, (_, i) => ({ title: "Project " + i, slug: "project-" + i, console: "PC", platforms: ["Windows"] }));
  const result = normalizeCatalog({ schema: 1, generatedAt: "2026-11-21", count: 4038, games });
  assert.equal(result.games.length, 600);
  assert.equal(result.upstreamCount, 4038);
  assert.equal(result.games[0].title, "Project 0");
  assert.notEqual(result.games.at(-1).title, "Project 599");
});
