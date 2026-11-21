//! Deterministic tree comparison — the heart of `repro`'s verdict.
//!
//! Compares two build output trees file-by-file: presence, size, and
//! content hash. Produces a [`DiffReport`] mirroring what diffoscope
//! summarizes (added/removed/modified files).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use sha2::Digest;

/// A single file difference.
#[derive(Debug, Clone)]
pub struct FileDiff {
    /// Path relative to the compared root.
    pub path: PathBuf,
    /// Kind of difference.
    pub kind: DiffKind,
}

/// Difference kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffKind {
    /// Only in first tree.
    OnlyFirst,
    /// Only in second tree.
    OnlySecond,
    /// In both but content differs.
    Modified,
}

/// Comparison report between two build trees.
#[derive(Debug, Default)]
pub struct DiffReport {
    /// All differences found.
    pub diffs: Vec<FileDiff>,
    /// Files identical in both trees.
    pub identical: usize,
}

impl DiffReport {
    /// Whether the trees are bit-identical.
    pub fn is_reproducible(&self) -> bool {
        self.diffs.is_empty()
    }

    /// Count by kind.
    pub fn count(&self, kind: DiffKind) -> usize {
        self.diffs.iter().filter(|d| d.kind == kind).count()
    }
}

/// Compare two directories recursively.
pub fn compare_trees(dir_a: &Path, dir_b: &Path) -> Result<DiffReport, crate::ReproError> {
    let a = hash_tree(dir_a)?;
    let b = hash_tree(dir_b)?;
    let mut report = DiffReport::default();
    for (path, ha) in &a {
        match b.get(path) {
            None => report.diffs.push(FileDiff { path: path.clone(), kind: DiffKind::OnlyFirst }),
            Some(hb) if hb == ha => report.identical += 1,
            Some(_) => report.diffs.push(FileDiff { path: path.clone(), kind: DiffKind::Modified }),
        }
    }
    for path in b.keys() {
        if !a.contains_key(path) {
            report.diffs.push(FileDiff { path: path.clone(), kind: DiffKind::OnlySecond });
        }
    }
    report.diffs.sort_by(|x, y| x.path.cmp(&y.path));
    Ok(report)
}

/// SHA-256 of a file's contents (streamed).
fn hash_file(path: &Path) -> Result<[u8; 32], crate::ReproError> {
    let data = std::fs::read(path)?;
    let mut h = sha2::Sha256::new();
    h.update(&data);
    Ok(h.finalize().into())
}

/// Recursively hash all files under `root` → rel_path → digest.
fn hash_tree(root: &Path) -> Result<BTreeMap<PathBuf, [u8; 32]>, crate::ReproError> {
    let mut out = BTreeMap::new();
    if !root.is_dir() {
        return Err(crate::ReproError::InvalidOutput(format!(
            "{} is not a directory",
            root.display()
        )));
    }
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let p = entry.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.is_file() {
                let rel = p.strip_prefix(root).unwrap_or(&p).to_path_buf();
                out.insert(rel, hash_file(&p)?);
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_trees() {
        let d = tempfile::tempdir().unwrap();
        let a = d.path().join("a");
        let b = d.path().join("b");
        std::fs::create_dir_all(&a).unwrap();
        std::fs::create_dir_all(&b).unwrap();
        std::fs::write(a.join("x"), "same").unwrap();
        std::fs::write(b.join("x"), "same").unwrap();
        let r = compare_trees(&a, &b).unwrap();
        assert!(r.is_reproducible());
        assert_eq!(r.identical, 1);
    }

    #[test]
    fn test_modified_file() {
        let d = tempfile::tempdir().unwrap();
        let a = d.path().join("a");
        let b = d.path().join("b");
        std::fs::create_dir_all(&a).unwrap();
        std::fs::create_dir_all(&b).unwrap();
        std::fs::write(a.join("x"), "v1").unwrap();
        std::fs::write(b.join("x"), "v2").unwrap();
        let r = compare_trees(&a, &b).unwrap();
        assert!(!r.is_reproducible());
        assert_eq!(r.count(DiffKind::Modified), 1);
    }

    #[test]
    fn test_added_removed() {
        let d = tempfile::tempdir().unwrap();
        let a = d.path().join("a");
        let b = d.path().join("b");
        std::fs::create_dir_all(&a).unwrap();
        std::fs::create_dir_all(&b).unwrap();
        std::fs::write(a.join("only_a"), "x").unwrap();
        std::fs::write(b.join("only_b"), "x").unwrap();
        let r = compare_trees(&a, &b).unwrap();
        assert_eq!(r.count(DiffKind::OnlyFirst), 1);
        assert_eq!(r.count(DiffKind::OnlySecond), 1);
    }
}
