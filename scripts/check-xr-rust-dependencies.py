"""Check the license closure of the two XR Rust packages from the locked graph."""
import json
import subprocess

metadata = json.loads(subprocess.check_output([
    "cargo", "metadata", "--format-version", "1", "--locked",
    "--filter-platform", "x86_64-unknown-linux-gnu",
], text=True))
packages = {p["id"]: p for p in metadata["packages"]}
nodes = {n["id"]: n for n in metadata["resolve"]["nodes"]}
pending = [p["id"] for p in packages.values() if p["name"] in ("phicade-xr-protocol", "phicade-xr-bridge")]
visited = set()
allowed = {"MIT", "Apache-2.0", "MIT OR Apache-2.0", "Apache-2.0 OR MIT", "(Apache-2.0 OR MIT)", "Unicode-3.0", "BSD-3-Clause", "ISC", "MIT/Apache-2.0", "Unlicense OR MIT", "(MIT OR Apache-2.0) AND Unicode-3.0"}
while pending:
    ident = pending.pop()
    if ident in visited:
        continue
    visited.add(ident)
    package = packages[ident]
    if package.get("license") not in allowed:
        raise SystemExit(f"Unreviewed license: {package['name']} {package.get('license')}")
    pending.extend(dep["pkg"] for dep in nodes[ident]["deps"])
print(f"PASS: {len(visited)} locked Rust packages in the XR dependency closure have reviewed licenses.")
