#!/usr/bin/env bash
set -euo pipefail
mkdir -p artifacts
cc -std=c99 -Wall -Wextra -Werror -shared -fPIC xr/fixtures/photon-range-core.c -o artifacts/photon_range_libretro.so
cargo run -p phicade-libretro --bin xr_lightgun_qualify -- artifacts/photon_range_libretro.so > artifacts/xr-lightgun-qualification.json
cat artifacts/xr-lightgun-qualification.json
