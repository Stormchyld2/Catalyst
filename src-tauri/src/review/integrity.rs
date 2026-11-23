use std::collections::HashSet;

use sha2::{Digest, Sha256};

use super::model::ReviewSourceIntegrity;

/// Calculates the SHA-256 digest of the exact bytes supplied and returns
/// the digest as lowercase hexadecimal.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);

    for byte in digest {
        use std::fmt::Write;
        write!(&mut hex, "{byte:02x}").expect("writing to String cannot fail");
    }

    hex
}

/// Creates the integrity record for one completed manuscript review file.
///
/// `exported_bytes` must be the exact bytes that will be written to the
/// review package after YAML frontmatter, recipient-specific metadata,
/// the visible review-copy notice, and manuscript Markdown are assembled.
pub fn source_integrity(
    source_id: impl Into<String>,
    relative_path: impl AsRef<str>,
    exported_bytes: &[u8],
) -> Result<ReviewSourceIntegrity, String> {
    let source_id = source_id.into();

    if source_id.trim().is_empty() {
        return Err("Review source ID must not be empty.".into());
    }

    let path = normalize_relative_path(relative_path.as_ref())?;

    Ok(ReviewSourceIntegrity {
        source_id,
        path,
        sha256: sha256_hex(exported_bytes),
    })
}

/// Calculates the deterministic SHA-256 identity for the complete manuscript
/// source inventory.
///
/// The result is independent of filesystem enumeration order and operating
/// system path separators. Records are ordered by normalized relative path
/// and then stable source ID before the canonical representation is hashed.
pub fn manuscript_sha256(files: &[ReviewSourceIntegrity]) -> Result<String, String> {
    if files.is_empty() {
        return Err("Cannot calculate manuscript integrity without manuscript sources.".into());
    }

    let mut records = Vec::with_capacity(files.len());

    for file in files {
        if file.source_id.trim().is_empty() {
            return Err("Review source ID must not be empty.".into());
        }

        let path = normalize_relative_path(&file.path)?;
        validate_sha256(&file.sha256)?;

        records.push((
            path,
            file.source_id.clone(),
            file.sha256.to_ascii_lowercase(),
        ));
    }

    records.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| left.1.cmp(&right.1))
    });

    let mut paths = HashSet::new();
    let mut source_ids = HashSet::new();

    for (path, source_id, _) in &records {
        if !paths.insert(path.as_str()) {
            return Err(format!("Duplicate manuscript review path '{path}'."));
        }

        if !source_ids.insert(source_id.as_str()) {
            return Err(format!(
                "Duplicate manuscript source ID '{source_id}'."
            ));
        }
    }

    let mut canonical = Vec::new();

    for (path, source_id, hash) in records {
        append_canonical_field(&mut canonical, path.as_bytes());
        append_canonical_field(&mut canonical, source_id.as_bytes());
        append_canonical_field(&mut canonical, hash.as_bytes());
    }

    Ok(sha256_hex(&canonical))
}

/// Normalizes a review-package-relative path into the canonical form used by
/// the integrity manifest.
///
/// Canonical review paths:
/// - are relative to the package root;
/// - use `/` separators on every operating system;
/// - cannot contain parent traversal;
/// - cannot contain a Windows drive prefix.
pub fn normalize_relative_path(path: &str) -> Result<String, String> {
    let trimmed = path.trim();

    if trimmed.is_empty() {
        return Err("Review package path must not be empty.".into());
    }

    let normalized = trimmed.replace('\\', "/");

    if normalized.starts_with('/') {
        return Err("Review package path must be relative.".into());
    }

    if normalized.len() >= 2 && normalized.as_bytes()[1] == b':' {
        return Err("Review package path must not contain a drive prefix.".into());
    }

    let mut parts = Vec::new();

    for part in normalized.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                return Err("Review package path must not contain parent traversal.".into());
            }
            value => parts.push(value),
        }
    }

    if parts.is_empty() {
        return Err("Review package path must identify a file.".into());
    }

    Ok(parts.join("/"))
}

fn validate_sha256(value: &str) -> Result<(), String> {
    let value = value.trim();

    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(
            "Review source SHA-256 must be a 64-character hexadecimal value.".into(),
        );
    }

    Ok(())
}

/// Appends one field to the canonical whole-manuscript representation.
///
/// Length-prefixing prevents ambiguous field boundaries while preserving a
/// deterministic byte representation.
fn append_canonical_field(buffer: &mut Vec<u8>, field: &[u8]) {
    buffer.extend_from_slice(field.len().to_string().as_bytes());
    buffer.push(b':');
    buffer.extend_from_slice(field);
    buffer.push(b'\n');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_exact_bytes_with_sha256() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn source_integrity_hashes_completed_export_bytes() {
        let source = source_integrity(
            "scn-1",
            r"Manuscript\Chapter 01\Scene 01.md",
            b"complete exported markdown",
        )
        .unwrap();

        assert_eq!(source.source_id, "scn-1");
        assert_eq!(source.path, "Manuscript/Chapter 01/Scene 01.md");
        assert_eq!(
            source.sha256,
            sha256_hex(b"complete exported markdown")
        );
    }

    #[test]
    fn normalizes_package_paths() {
        assert_eq!(
            normalize_relative_path(r"Manuscript\Chapter 01\Scene 01.md").unwrap(),
            "Manuscript/Chapter 01/Scene 01.md"
        );
    }

    #[test]
    fn rejects_parent_traversal() {
        assert!(normalize_relative_path("../outside.md").is_err());
        assert!(normalize_relative_path("Manuscript/../../outside.md").is_err());
    }

    #[test]
    fn rejects_absolute_paths() {
        assert!(normalize_relative_path("/outside.md").is_err());
        assert!(normalize_relative_path(r"C:\outside.md").is_err());
    }

    #[test]
    fn manuscript_hash_is_independent_of_input_order() {
        let first = ReviewSourceIntegrity {
            source_id: "scn-1".into(),
            path: "Manuscript/Chapter 01/Scene 01.md".into(),
            sha256: sha256_hex(b"one"),
        };

        let second = ReviewSourceIntegrity {
            source_id: "scn-2".into(),
            path: "Manuscript/Chapter 01/Scene 02.md".into(),
            sha256: sha256_hex(b"two"),
        };

        let forward =
            manuscript_sha256(&[first.clone(), second.clone()]).unwrap();

        let reverse =
            manuscript_sha256(&[second, first]).unwrap();

        assert_eq!(forward, reverse);
    }

    #[test]
    fn manuscript_hash_changes_when_exported_bytes_change() {
        let original = ReviewSourceIntegrity {
            source_id: "scn-1".into(),
            path: "Manuscript/Chapter 01/Scene 01.md".into(),
            sha256: sha256_hex(b"original"),
        };

        let changed = ReviewSourceIntegrity {
            source_id: "scn-1".into(),
            path: "Manuscript/Chapter 01/Scene 01.md".into(),
            sha256: sha256_hex(b"changed"),
        };

        assert_ne!(
            manuscript_sha256(&[original]).unwrap(),
            manuscript_sha256(&[changed]).unwrap()
        );
    }

    #[test]
    fn rejects_duplicate_paths() {
        let first = ReviewSourceIntegrity {
            source_id: "scn-1".into(),
            path: "Manuscript/Scene.md".into(),
            sha256: sha256_hex(b"one"),
        };

        let second = ReviewSourceIntegrity {
            source_id: "scn-2".into(),
            path: "Manuscript/Scene.md".into(),
            sha256: sha256_hex(b"two"),
        };

        assert!(manuscript_sha256(&[first, second]).is_err());
    }

    #[test]
    fn rejects_duplicate_source_ids_with_different_paths() {
        let first = ReviewSourceIntegrity {
            source_id: "scn-1".into(),
            path: "Manuscript/Chapter 01/Scene 01.md".into(),
            sha256: sha256_hex(b"one"),
        };

        let second = ReviewSourceIntegrity {
            source_id: "scn-1".into(),
            path: "Manuscript/Chapter 02/Scene 01.md".into(),
            sha256: sha256_hex(b"two"),
        };

        assert!(manuscript_sha256(&[first, second]).is_err());
    }
}