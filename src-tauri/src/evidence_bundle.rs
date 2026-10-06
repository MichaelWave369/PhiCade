//! Deterministic, dependency-free ZIP32 packaging for verified PhiCade evidence.
//!
//! ZIP entries use STORED compression, fixed 1980-01-01 DOS timestamps,
//! stable sorted filenames and no host-specific file attributes.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub const BUNDLE_SCHEMA: &str = "phicade.portable-evidence-bundle.v1";
const MAX_BUNDLE_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleFile {
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleManifest {
    pub schema: String,
    pub record_status: String,
    pub suite_id: String,
    pub report_id: u64,
    pub suite_report_sha256: String,
    pub model_digest: String,
    pub files: Vec<BundleFile>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleArtifact {
    pub bundle_path: String,
    pub bundle_sha256: String,
    pub manifest: BundleManifest,
}

fn hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn valid_path(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('/')
        && !name.contains('\\')
        && !name.split('/').any(|segment| segment.is_empty() || segment == "." || segment == "..")
        && name.bytes().all(|b| b.is_ascii_graphic())
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn u16le(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn u32le(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn zip_stored(files: &BTreeMap<String, Vec<u8>>) -> Result<Vec<u8>, String> {
    let count = u16::try_from(files.len()).map_err(|_| "too many ZIP entries")?;
    let mut out = Vec::new();
    let mut central = Vec::new();

    for (name, data) in files {
        if !valid_path(name) {
            return Err(format!("unsafe ZIP entry path: {name}"));
        }
        let filename = name.as_bytes();
        let filename_len = u16::try_from(filename.len()).map_err(|_| "ZIP filename too long")?;
        let data_len = u32::try_from(data.len()).map_err(|_| "ZIP entry exceeds ZIP32")?;
        let offset = u32::try_from(out.len()).map_err(|_| "ZIP offset exceeds ZIP32")?;
        let crc = crc32(data);

        u32le(&mut out, 0x0403_4b50);
        u16le(&mut out, 20);
        u16le(&mut out, 0);
        u16le(&mut out, 0);
        u16le(&mut out, 0);
        u16le(&mut out, 33);
        u32le(&mut out, crc);
        u32le(&mut out, data_len);
        u32le(&mut out, data_len);
        u16le(&mut out, filename_len);
        u16le(&mut out, 0);
        out.extend_from_slice(filename);
        out.extend_from_slice(data);

        u32le(&mut central, 0x0201_4b50);
        u16le(&mut central, 20);
        u16le(&mut central, 20);
        u16le(&mut central, 0);
        u16le(&mut central, 0);
        u16le(&mut central, 0);
        u16le(&mut central, 33);
        u32le(&mut central, crc);
        u32le(&mut central, data_len);
        u32le(&mut central, data_len);
        u16le(&mut central, filename_len);
        u16le(&mut central, 0);
        u16le(&mut central, 0);
        u16le(&mut central, 0);
        u16le(&mut central, 0);
        u32le(&mut central, 0);
        u32le(&mut central, offset);
        central.extend_from_slice(filename);
    }

    let directory_start = u32::try_from(out.len()).map_err(|_| "ZIP directory offset exceeds ZIP32")?;
    let directory_size = u32::try_from(central.len()).map_err(|_| "ZIP directory exceeds ZIP32")?;
    out.extend_from_slice(&central);
    u32le(&mut out, 0x0605_4b50);
    u16le(&mut out, 0);
    u16le(&mut out, 0);
    u16le(&mut out, count);
    u16le(&mut out, count);
    u32le(&mut out, directory_size);
    u32le(&mut out, directory_start);
    u16le(&mut out, 0);
    Ok(out)
}

pub fn assemble_bundle(
    suite_id: &str,
    report_id: u64,
    report_hash: &str,
    model_digest: &str,
    mut files: BTreeMap<String, Vec<u8>>,
) -> Result<(Vec<u8>, BundleManifest), String> {
    if files.contains_key("manifest.json") {
        return Err("manifest.json is reserved".into());
    }
    if !files.contains_key("suite-report.json") {
        return Err("portable evidence bundle requires suite-report.json".into());
    }
    if files.get("suite-report.json").map(|b| hash(b)) != Some(report_hash.to_owned()) {
        return Err("bundle suite report bytes do not match pinned SHA-256".into());
    }
    let sum: usize = files.values().map(Vec::len).sum();
    if sum > MAX_BUNDLE_BYTES {
        return Err("portable evidence bundle exceeds 64 MiB safety limit".into());
    }

    let entries = files.iter().map(|(path, bytes)| BundleFile {
        path: path.clone(),
        sha256: hash(bytes),
        size_bytes: bytes.len() as u64,
    }).collect();
    let manifest = BundleManifest {
        schema: BUNDLE_SCHEMA.into(),
        record_status: "VERIFIED_EXPORT".into(),
        suite_id: suite_id.into(),
        report_id,
        suite_report_sha256: report_hash.into(),
        model_digest: model_digest.into(),
        files: entries,
    };
    let json = serde_json::to_vec_pretty(&manifest)
        .map_err(|e| format!("serialize bundle manifest: {e}"))?;
    files.insert("manifest.json".into(), json);
    let zip = zip_stored(&files)?;
    if zip.len() > MAX_BUNDLE_BYTES + 4 * 1024 * 1024 {
        return Err("portable evidence ZIP exceeds size safety limit".into());
    }
    Ok((zip, manifest))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc32_matches_standard_reference() {
        assert_eq!(crc32(b"123456789"), 0xcbf4_3926);
    }

    #[test]
    fn bundle_is_deterministic_and_manifest_hashes_content() {
        let bytes = b"{\"a\":1}".to_vec();
        let mut files = BTreeMap::new();
        files.insert("suite-report.json".into(), bytes.clone());
        files.insert("trials/fixture.json".into(), b"test".to_vec());
        let (first, manifest) = assemble_bundle(
            "suite-v1", 1, &hash(&bytes), "digest-a", files.clone()
        ).expect("bundle");
        let (second, _) = assemble_bundle(
            "suite-v1", 1, &hash(&bytes), "digest-a", files
        ).expect("bundle repeated");
        assert_eq!(first, second);
        assert_eq!(&first[..4], &[0x50, 0x4b, 0x03, 0x04]);
        assert_eq!(manifest.files.len(), 2);
        assert_eq!(manifest.files[0].sha256, hash(&bytes));
    }

    #[test]
    fn reject_path_traversal_and_forged_report_hash() {
        let bytes = b"report".to_vec();
        let mut files = BTreeMap::new();
        files.insert("suite-report.json".into(), bytes.clone());
        files.insert("../outside".into(), b"nope".to_vec());
        assert!(assemble_bundle("suite-v1", 1, &hash(&bytes), "digest", files).is_err());
        let mut files = BTreeMap::new();
        files.insert("suite-report.json".into(), bytes);
        assert!(assemble_bundle("suite-v1", 1, "bad-hash", "digest", files).is_err());
    }
}
