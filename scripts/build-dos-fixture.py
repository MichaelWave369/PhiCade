#!/usr/bin/env python3
"""Package only an original NASM-assembled PHITEST.COM as deterministic DOSZ."""
import argparse
import hashlib
import json
from pathlib import Path
from zipfile import ZipFile, ZipInfo, ZIP_STORED

def package(com: Path, output: Path) -> dict:
    contents = com.read_bytes()
    if not (40 <= len(contents) <= 4096):
        raise ValueError("original COM fixture outside expected bounds")
    if contents.startswith((b"MZ", b"PK\x03\x04")):
        raise ValueError("expected flat COM code, not an EXE or ZIP")
    output.parent.mkdir(parents=True, exist_ok=True)
    info = ZipInfo("PHITEST.COM", date_time=(1980, 1, 1, 0, 0, 0))
    info.compress_type = ZIP_STORED
    info.external_attr = 0o100644 << 16
    with ZipFile(output, "w") as archive:
        archive.writestr(info, contents)
    with ZipFile(output) as archive:
        if archive.namelist() != ["PHITEST.COM"] or archive.read("PHITEST.COM") != contents:
            raise AssertionError("invalid archive round trip")
    return {
        "schema": "phicade.original-dos-fixture.v1",
        "format": "dosz",
        "assembledFilename": "PHITEST.COM",
        "assembledBytes": len(contents),
        "assembledSha256": hashlib.sha256(contents).hexdigest(),
        "archiveSha256": hashlib.sha256(output.read_bytes()).hexdigest(),
        "provenance": "original NASM source compiled in GitHub Actions",
        "commercialGameDataIncluded": False,
    }

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--com", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(package(args.com, args.output), indent=2))
