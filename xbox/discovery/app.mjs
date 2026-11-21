/** Read-only catalog view. Remote data is rendered via textContent, never HTML. */
import { normalizeCatalog, filterCatalog, safeOutboundLink } from "./catalog.mjs";

const $ = id => document.getElementById(id);
const cards = $("cards");
let state = { records: [], sourceStatus: "unavailable", upstreamCount: null, sampledCount: 0, date: null };

function external(url, text) {
  if (!safeOutboundLink(url)) return document.createTextNode("Source unavailable");
  const a = document.createElement("a");
  a.href = url;
  a.target = "_blank";
  a.rel = "noopener noreferrer";
  a.textContent = text + " ↗";
  return a;
}

function option(select, value) {
  const o = document.createElement("option");
  o.value = value;
  o.textContent = value;
  select.append(o);
}

function article(game) {
  const div = document.createElement("article");
  div.className = "card";
  const heading = document.createElement("div");
  heading.className = "card-top";
  const consoleName = document.createElement("span");
  consoleName.className = "console";
  consoleName.textContent = game.console;
  const project = document.createElement("span");
  project.className = "project";
  project.textContent = game.project;
  heading.append(consoleName, project);
  const title = document.createElement("h3");
  title.textContent = game.title;
  const chips = document.createElement("div");
  chips.className = "chips";
  if (game.platforms.length) for (const p of game.platforms) {
    const chip = document.createElement("span");
    chip.className = "chip";
    chip.textContent = p;
    chips.append(chip);
  } else {
    const chip = document.createElement("span");
    chip.className = "meta";
    chip.textContent = "Target platforms not specified";
    chips.append(chip);
  }
  const evidence = document.createElement("p");
  evidence.className = "claimed";
  evidence.textContent = game.completeness
    ? "Completeness listed by upstream: " + game.completeness
    : "Completeness not independently qualified";
  const footer = document.createElement("div");
  footer.className = "card-actions";
  footer.append(external(game.sourceUrl, "Source details"));
  if (game.repoUrl) footer.append(external(game.repoUrl, "Project repo"));
  div.append(heading, title, chips, evidence, footer);
  return div;
}

function render() {
  const matches = filterCatalog(state.records, {
    query: $("search").value,
    console: $("console").value,
    platform: $("platform").value,
  });
  cards.replaceChildren(...matches.map(article));
  $("empty").hidden = matches.length > 0;
  $("results-count").textContent = "Showing " + matches.length + " of " + state.records.length + " metadata entries in this sample.";
}

async function readLocal(url) {
  const response = await fetch(url, { cache: "no-cache" });
  if (!response.ok) throw new Error("HTTP " + response.status);
  return response.json();
}

async function init() {
  let payload;
  try {
    payload = await readLocal("./catalog.json");
    if (payload.type !== "phicade.discovery.sample.v1" || !Array.isArray(payload.games)) throw new Error("Malformed build snapshot");
    const sourceLike = {
      schema: 1, generatedAt: payload.upstreamGeneratedAt, count: payload.upstreamCount ?? payload.games.length,
      games: payload.games.map(g => ({ ...g, slug: g.id, facts: { completeness: g.completeness } })),
    };
    const validated = normalizeCatalog(sourceLike);
    state.records = validated.games;
    state.sourceStatus = payload.sourceStatus === "upstream-snapshot" ? "upstream-snapshot" : "curated-fallback";
    state.upstreamCount = state.sourceStatus === "upstream-snapshot" ? payload.upstreamCount : null;
    state.date = state.sourceStatus === "upstream-snapshot" ? payload.upstreamGeneratedAt : null;
  } catch {
    try {
      const fallback = await readLocal("./seed.json");
      state.records = normalizeCatalog(fallback).games;
    } catch {
      state.records = [];
    }
    state.sourceStatus = "curated-fallback";
  }
  state.sampledCount = state.records.length;
  if (state.sourceStatus === "upstream-snapshot") {
    $("catalog-status").textContent = "Credited upstream metadata snapshot · no downloads";
    $("coverage").textContent = state.sampledCount + " sampled listings from " + state.upstreamCount +
      " upstream entries · source published " + (state.date ? state.date.slice(0,10) : "unknown");
  } else {
    $("catalog-status").textContent = "Curated starter index · live catalog not loaded";
    $("coverage").textContent = state.sampledCount + " starter listings · full catalog at Decomp Buddy";
  }
  for (const c of [...new Set(state.records.map(x => x.console))].sort()) option($("console"), c);
  for (const p of [...new Set(state.records.flatMap(x => x.platforms))].sort()) option($("platform"), p);
  render();
}

$("search").addEventListener("input", render);
$("console").addEventListener("change", render);
$("platform").addEventListener("change", render);
$("search-form").addEventListener("reset", () => setTimeout(render, 0));
await init();
