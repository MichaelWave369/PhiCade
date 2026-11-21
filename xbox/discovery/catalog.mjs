/** PhiCade Decomp Discovery: metadata-only public catalog adapter.
 * Independent implementation. No GPL app code, download or execution support.
 */
export const PUBLIC_CATALOG_URL = "https://tanookistudios.com/decomp-buddy/catalog.json";
export const MAX_SOURCE_RECORDS = 12000;
export const MAX_PUBLIC_RECORDS = 600;

function cleanText(value, max = 180) {
  return typeof value === "string" ? value.replace(/[\u0000-\u001f\u007f]/g, " ").trim().slice(0, max) : "";
}

function safeProjectUrl(raw, repo) {
  let candidate = typeof raw === "string" ? raw : "";
  if (!candidate && typeof repo === "string" && /^[a-z\d_.-]+\/[a-z\d_.-]+$/i.test(repo)) {
    candidate = "https://github.com/" + repo;
  }
  try {
    const url = new URL(candidate);
    if (url.protocol !== "https:" || url.username || url.password || url.port) return null;
    if (!["github.com", "gitlab.com", "codeberg.org"].includes(url.hostname.toLowerCase())) return null;
    return url.href;
  } catch {
    return null;
  }
}

export function normalizeCatalog(doc, limit = MAX_PUBLIC_RECORDS) {
  if (!doc || doc.schema !== 1 || !Array.isArray(doc.games)) {
    throw new Error("Unsupported catalog format (expected schema 1 with games[])");
  }
  if (doc.games.length > MAX_SOURCE_RECORDS) throw new Error("Catalog exceeds review limit");
  if (!Number.isInteger(limit) || limit < 1 || limit > MAX_PUBLIC_RECORDS) throw new Error("Invalid public record limit");
  const stride = Math.max(1, doc.games.length / limit);
  const out = [];
  const seen = new Set();
  for (let i = 0; i < Math.min(doc.games.length, limit); i++) {
    const raw = doc.games[Math.floor(i * stride)];
    if (!raw || typeof raw !== "object") continue;
    const title = cleanText(raw.title);
    const slug = cleanText(raw.slug, 150);
    if (!title || !/^[a-z0-9][a-z0-9-]{0,149}$/.test(slug) || seen.has(slug)) continue;
    seen.add(slug);
    const platforms = Array.isArray(raw.platforms)
      ? [...new Set(raw.platforms.map(x => cleanText(x, 35)).filter(Boolean))].slice(0, 8)
      : [];
    const completeness = cleanText(raw.facts?.completeness, 70);
    out.push({
      id: slug, title,
      console: cleanText(raw.console, 65) || "Unspecified",
      project: cleanText(raw.project, 80) || "Community project",
      platforms,
      status: cleanText(raw.status, 60) || "unknown",
      completeness: completeness || null,
      version: cleanText(raw.version || raw.latestVersion, 48) || null,
      repoUrl: safeProjectUrl(raw.repoUrl, raw.repo),
      sourceUrl: "https://decompbuddy.com/games/" + slug
    });
  }
  return {
    schema: 1,
    type: "phicade.discovery.sample.v1",
    source: "Decomp Buddy public catalog",
    sourceUrl: PUBLIC_CATALOG_URL,
    upstreamGeneratedAt: typeof doc.generatedAt === "string" ? doc.generatedAt.slice(0, 35) : null,
    upstreamCount: Number.isSafeInteger(doc.count) && doc.count >= doc.games.length ? doc.count : doc.games.length,
    sampledCount: out.length,
    scope: "Metadata subset for source discovery; no verified playability or safety claims",
    games: out
  };
}

export function filterCatalog(games, { query = "", console = "all", platform = "all" } = {}) {
  if (!Array.isArray(games)) return [];
  const q = String(query).trim().toLowerCase().slice(0, 120);
  return games.filter(game => {
    if (console !== "all" && game.console !== console) return false;
    if (platform !== "all" && !game.platforms.includes(platform)) return false;
    const hay = [game.title, game.project, game.console, game.completeness, game.status, ...game.platforms].join(" ").toLowerCase();
    return !q || hay.includes(q);
  });
}

export function safeOutboundLink(url) {
  try {
    const u = new URL(url);
    return u.protocol === "https:" && !u.username && !u.password && !u.port &&
      ["github.com", "gitlab.com", "codeberg.org", "decompbuddy.com"].includes(u.hostname.toLowerCase());
  } catch {
    return false;
  }
}
