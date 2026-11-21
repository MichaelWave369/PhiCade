//! Scarlett Lab Rung 1: bounded, read-only inspection of generic x86-64 PE32+ files.
//!
//! This is *not* an Xbox emulator, Xbox executable detector, loader or runner.
//! No OS services, host execution, file writes or privileged features exist here.
#![forbid(unsafe_code)]

use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fmt;

pub const SCHEMA: &str = "phicade.scarlett-lab.inspection.v1";
pub const MAX_IMAGE_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_SECTIONS: usize = 96;
const IMAGE_FILE_MACHINE_AMD64: u16 = 0x8664;
const IMAGE_NT_OPTIONAL_HDR64_MAGIC: u16 = 0x020b;
const SECTION_HEADER_SIZE: usize = 40;
const MIN_OPTIONAL64_SIZE: usize = 112;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InspectError {
    TooLarge,
    Truncated(&'static str),
    Unsupported(&'static str),
    Invalid(&'static str),
}

impl fmt::Display for InspectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge => write!(f, "input exceeds Scarlett Lab size limit"),
            Self::Truncated(field) => write!(f, "truncated PE field: {field}"),
            Self::Unsupported(reason) => write!(f, "unsupported PE input: {reason}"),
            Self::Invalid(reason) => write!(f, "invalid PE input: {reason}"),
        }
    }
}

impl std::error::Error for InspectError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LabCapabilities {
    pub inspection_only: bool,
    pub launches_executables: bool,
    pub executes_guest_code: bool,
    pub supports_xbox_games: bool,
    pub has_graphics_backend: bool,
    pub integrates_with_arcade_sessions: bool,
}

pub fn capabilities() -> LabCapabilities {
    LabCapabilities {
        inspection_only: true,
        launches_executables: false,
        executes_guest_code: false,
        supports_xbox_games: false,
        has_graphics_backend: false,
        integrates_with_arcade_sessions: false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Section {
    pub name: String,
    pub virtual_address: u32,
    pub virtual_size: u32,
    pub raw_offset: u32,
    pub raw_size: u32,
    pub characteristics: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Inspection {
    pub schema: &'static str,
    pub format: &'static str,
    pub architecture: &'static str,
    pub bytes: usize,
    pub sha256: String,
    pub entry_point_rva: u32,
    pub image_base: u64,
    pub size_of_image: u32,
    pub size_of_headers: u32,
    pub sections: Vec<Section>,
    pub scope: &'static str,
    pub capabilities: LabCapabilities,
}

fn span<'a>(
    input: &'a [u8],
    offset: usize,
    length: usize,
    field: &'static str,
) -> Result<&'a [u8], InspectError> {
    let end = offset
        .checked_add(length)
        .ok_or(InspectError::Invalid("offset arithmetic overflow"))?;
    input.get(offset..end).ok_or(InspectError::Truncated(field))
}

fn u16le(input: &[u8], offset: usize, field: &'static str) -> Result<u16, InspectError> {
    let value = span(input, offset, 2, field)?;
    Ok(u16::from_le_bytes([value[0], value[1]]))
}

fn u32le(input: &[u8], offset: usize, field: &'static str) -> Result<u32, InspectError> {
    let value = span(input, offset, 4, field)?;
    Ok(u32::from_le_bytes([value[0], value[1], value[2], value[3]]))
}

fn u64le(input: &[u8], offset: usize, field: &'static str) -> Result<u64, InspectError> {
    let value = span(input, offset, 8, field)?;
    Ok(u64::from_le_bytes([
        value[0], value[1], value[2], value[3],
        value[4], value[5], value[6], value[7],
    ]))
}

/// Parse metadata only. Never loads libraries, maps executable pages, resolves
/// imports, decodes executable instructions, or starts a process.
pub fn inspect(input: &[u8]) -> Result<Inspection, InspectError> {
    if input.len() > MAX_IMAGE_BYTES {
        return Err(InspectError::TooLarge);
    }
    let dos = span(input, 0, 64, "DOS header")?;
    if &dos[..2] != b"MZ" {
        return Err(InspectError::Unsupported("missing MZ signature"));
    }

    let nt = u32le(input, 0x3c, "e_lfanew")? as usize;
    if span(input, nt, 4, "PE signature")? != b"PE\0\0" {
        return Err(InspectError::Unsupported("missing PE signature"));
    }
    let coff = nt
        .checked_add(4)
        .ok_or(InspectError::Invalid("COFF offset overflow"))?;
    let _header = span(input, coff, 20, "COFF header")?;
    if u16le(input, coff, "machine")? != IMAGE_FILE_MACHINE_AMD64 {
        return Err(InspectError::Unsupported("only AMD64 is inspected"));
    }
    let count = usize::from(u16le(input, coff + 2, "section count")?);
    if count == 0 || count > MAX_SECTIONS {
        return Err(InspectError::Invalid("section count out of range"));
    }
    let optional_size = usize::from(u16le(input, coff + 16, "optional header size")?);
    if optional_size < MIN_OPTIONAL64_SIZE {
        return Err(InspectError::Invalid("PE32+ optional header too small"));
    }
    let optional = coff
        .checked_add(20)
        .ok_or(InspectError::Invalid("optional header offset overflow"))?;
    span(input, optional, optional_size, "optional header")?;
    if u16le(input, optional, "optional header magic")?
        != IMAGE_NT_OPTIONAL_HDR64_MAGIC
    {
        return Err(InspectError::Unsupported("only PE32+ is inspected"));
    }

    let entry = u32le(input, optional + 16, "entry-point RVA")?;
    let base = u64le(input, optional + 24, "image base")?;
    let size_of_image = u32le(input, optional + 56, "size of image")?;
    let size_of_headers = u32le(input, optional + 60, "size of headers")?;
    if size_of_image == 0 || size_of_headers == 0 {
        return Err(InspectError::Invalid("empty image or header size"));
    }
    if usize::try_from(size_of_headers).ok().filter(|&n| n <= input.len()).is_none() {
        return Err(InspectError::Truncated("declared PE headers"));
    }
    if entry >= size_of_image && entry != 0 {
        return Err(InspectError::Invalid("entry point exceeds image"));
    }

    let table = optional
        .checked_add(optional_size)
        .ok_or(InspectError::Invalid("section table offset overflow"))?;
    let table_len = count
        .checked_mul(SECTION_HEADER_SIZE)
        .ok_or(InspectError::Invalid("section table size overflow"))?;
    span(input, table, table_len, "section table")?;
    if table.checked_add(table_len).ok_or(InspectError::Invalid("header overflow"))?
        > size_of_headers as usize
    {
        return Err(InspectError::Invalid("section table exceeds declared headers"));
    }

    let mut sections = Vec::with_capacity(count);
    let mut raw_ranges: Vec<(usize, usize)> = Vec::with_capacity(count);
    for i in 0..count {
        let start = table + i * SECTION_HEADER_SIZE;
        let section = span(input, start, SECTION_HEADER_SIZE, "section header")?;
        let name_bytes = &section[0..8];
        let name_end = name_bytes.iter().position(|&b| b == 0).unwrap_or(8);
        let name = String::from_utf8_lossy(&name_bytes[..name_end]).into_owned();
        let virtual_size = u32le(input, start + 8, "virtual size")?;
        let virtual_address = u32le(input, start + 12, "virtual address")?;
        let raw_size = u32le(input, start + 16, "raw size")?;
        let raw_offset = u32le(input, start + 20, "raw offset")?;
        let characteristics = u32le(input, start + 36, "characteristics")?;

        let virtual_end = virtual_address
            .checked_add(virtual_size.max(raw_size))
            .ok_or(InspectError::Invalid("virtual section range overflow"))?;
        if virtual_end > size_of_image {
            return Err(InspectError::Invalid("section exceeds declared image"));
        }
        if raw_size != 0 {
            let from = raw_offset as usize;
            let to = from
                .checked_add(raw_size as usize)
                .ok_or(InspectError::Invalid("raw section offset overflow"))?;
            if from < size_of_headers as usize {
                return Err(InspectError::Invalid("raw section overlaps PE headers"));
            }
            span(input, from, raw_size as usize, "raw section data")?;
            if raw_ranges.iter().any(|&(a, b)| from < b && a < to) {
                return Err(InspectError::Invalid("overlapping raw section data"));
            }
            raw_ranges.push((from, to));
        }
        sections.push(Section {
            name,
            virtual_address,
            virtual_size,
            raw_offset,
            raw_size,
            characteristics,
        });
    }

    Ok(Inspection {
        schema: SCHEMA,
        format: "PE32+",
        architecture: "AMD64",
        bytes: input.len(),
        sha256: format!("{:x}", Sha256::digest(input)),
        entry_point_rva: entry,
        image_base: base,
        size_of_image,
        size_of_headers,
        sections,
        scope: "generic-pe32+-metadata-only; not Xbox detection or compatibility evidence",
        capabilities: capabilities(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put16(v: &mut [u8], at: usize, n: u16) {
        v[at..at + 2].copy_from_slice(&n.to_le_bytes());
    }
    fn put32(v: &mut [u8], at: usize, n: u32) {
        v[at..at + 4].copy_from_slice(&n.to_le_bytes());
    }
    fn put64(v: &mut [u8], at: usize, n: u64) {
        v[at..at + 8].copy_from_slice(&n.to_le_bytes());
    }

    /// An original synthetic header fixture, not copied from a Microsoft
    /// executable. The buffer is metadata test data and cannot run as a game.
    fn fixture() -> Vec<u8> {
        let mut v = vec![0u8; 0x400];
        v[0..2].copy_from_slice(b"MZ");
        put32(&mut v, 0x3c, 0x80);
        v[0x80..0x84].copy_from_slice(b"PE\0\0");
        let coff = 0x84;
        put16(&mut v, coff, 0x8664);
        put16(&mut v, coff + 2, 1);
        put16(&mut v, coff + 16, 0xf0);
        let opt = coff + 20;
        put16(&mut v, opt, 0x20b);
        put32(&mut v, opt + 16, 0x1000);
        put64(&mut v, opt + 24, 0x140000000);
        put32(&mut v, opt + 32, 0x1000);
        put32(&mut v, opt + 36, 0x200);
        put32(&mut v, opt + 56, 0x2000);
        put32(&mut v, opt + 60, 0x200);
        put32(&mut v, opt + 108, 16);
        let sec = opt + 0xf0;
        v[sec..sec + 5].copy_from_slice(b".text");
        put32(&mut v, sec + 8, 0x180);
        put32(&mut v, sec + 12, 0x1000);
        put32(&mut v, sec + 16, 0x200);
        put32(&mut v, sec + 20, 0x200);
        put32(&mut v, sec + 36, 0x60000020);
        v
    }

    #[test]
    fn inspects_source_built_pe64_header_without_execution() {
        let report = inspect(&fixture()).unwrap();
        assert_eq!(report.format, "PE32+");
        assert_eq!(report.architecture, "AMD64");
        assert_eq!(report.sections[0].name, ".text");
        assert_eq!(report.sections[0].raw_offset, 0x200);
        assert_eq!(report.entry_point_rva, 0x1000);
        assert_eq!(report.sha256.len(), 64);
        assert!(!report.capabilities.launches_executables);
        assert!(!report.capabilities.supports_xbox_games);
    }

    #[test]
    fn json_is_stable_and_does_not_claim_console_compatibility() {
        let json = serde_json::to_value(inspect(&fixture()).unwrap()).unwrap();
        assert_eq!(json["schema"], SCHEMA);
        assert_eq!(json["capabilities"]["executesGuestCode"], false);
        assert_eq!(json["capabilities"]["integratesWithArcadeSessions"], false);
        assert_eq!(json["sections"][0]["virtualAddress"], 0x1000);
    }

    #[test]
    fn rejects_truncated_and_non_pe() {
        assert!(matches!(inspect(&[]), Err(InspectError::Truncated(_))));
        let mut v = fixture();
        v[0] = b'X';
        assert!(matches!(inspect(&v), Err(InspectError::Unsupported(_))));
        v = fixture();
        v[0x80] = b'Q';
        assert!(matches!(inspect(&v), Err(InspectError::Unsupported(_))));
    }

    #[test]
    fn rejects_unbounded_nt_pointer() {
        let mut v = fixture();
        put32(&mut v, 0x3c, u32::MAX);
        assert!(matches!(inspect(&v), Err(InspectError::Truncated(_))));
    }

    #[test]
    fn rejects_other_cpu_and_pe32_header() {
        let mut v = fixture();
        put16(&mut v, 0x84, 0x14c);
        assert!(matches!(inspect(&v), Err(InspectError::Unsupported(_))));
        v = fixture();
        put16(&mut v, 0x98, 0x10b);
        assert!(matches!(inspect(&v), Err(InspectError::Unsupported(_))));
    }

    #[test]
    fn rejects_bad_section_count_and_short_optional_header() {
        let mut v = fixture();
        put16(&mut v, 0x86, (MAX_SECTIONS + 1) as u16);
        assert!(matches!(inspect(&v), Err(InspectError::Invalid(_))));
        v = fixture();
        put16(&mut v, 0x84 + 16, 90);
        assert!(matches!(inspect(&v), Err(InspectError::Invalid(_))));
    }

    #[test]
    fn rejects_raw_section_out_of_bounds() {
        let mut v = fixture();
        let sec = 0x98 + 0xf0;
        put32(&mut v, sec + 20, 0x3ff);
        assert!(matches!(inspect(&v), Err(InspectError::Truncated(_))));
    }

    #[test]
    fn rejects_raw_section_overlapping_headers() {
        let mut v = fixture();
        put32(&mut v, 0x188 + 20, 0x100);
        assert!(matches!(inspect(&v), Err(InspectError::Invalid(_))));
    }

    #[test]
    fn rejects_virtual_section_outside_image() {
        let mut v = fixture();
        put32(&mut v, 0x188 + 12, 0x1f00);
        assert!(matches!(inspect(&v), Err(InspectError::Invalid(_))));
    }

    #[test]
    fn rejects_out_of_range_entry_point() {
        let mut v = fixture();
        put32(&mut v, 0x98 + 16, 0x2000);
        assert!(matches!(inspect(&v), Err(InspectError::Invalid(_))));
    }

    #[test]
    fn rejects_oversized_images_without_parsing() {
        let v = vec![0u8; MAX_IMAGE_BYTES + 1];
        assert_eq!(inspect(&v), Err(InspectError::TooLarge));
    }

    #[test]
    fn fingerprints_are_deterministic_and_content_bound() {
        let a = fixture();
        let mut b = a.clone();
        b[0x200] = 1;
        let first = inspect(&a).unwrap();
        assert_eq!(first.sha256, inspect(&a).unwrap().sha256);
        assert_ne!(first.sha256, inspect(&b).unwrap().sha256);
    }

    #[test]
    fn section_table_cannot_exceed_headers() {
        let mut v = fixture();
        put16(&mut v, 0x86, 10);
        assert!(matches!(inspect(&v), Err(InspectError::Invalid(_))));
    }
}
