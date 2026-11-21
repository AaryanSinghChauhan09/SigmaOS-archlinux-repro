//! Build execution abstraction.
//!
//! Upstream `repro` runs builds inside `systemd-nspawn` containers. This
//! port abstracts that behind a trait so the orchestration is testable
//! without chroots; a `CommandRunner` executes real shell commands.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::ReproError;

/// A package to build reproducibly.
#[derive(Debug, Clone)]
pub struct BuildSpec {
    /// Package base name.
    pub pkgbase: String,
    /// Full version `epoch:pkgver-pkgrel`.
    pub version: String,
    /// Architecture.
    pub arch: String,
    /// Source directory (PKGBUILD location or VCS checkout).
    pub src_dir: PathBuf,
    /// Extra makepkg args.
    pub makepkg_args: Vec<String>,
}

/// Runs a build and produces output in `out_dir`.
pub trait BuildRunner {
    /// Build `spec` into `out_dir`; returns path to produced artifacts dir.
    fn build(&self, spec: &BuildSpec, out_dir: &Path) -> Result<PathBuf, ReproError>;
}

/// Runs builds via external commands (makepkg/devtools under nspawn).
///
/// `build_cmd` is a template with `{src}` and `{out}` placeholders, e.g.
/// `systemd-nspawn -D /var/lib/repro/root -- makepkg -C -f -p {src}/PKGBUILD`.
#[derive(Debug, Clone)]
pub struct CommandRunner {
    /// Command template.
    pub build_cmd: String,
}

impl CommandRunner {
    /// New runner with a command template.
    pub fn new(build_cmd: impl Into<String>) -> Self {
        Self { build_cmd: build_cmd.into() }
    }
}

impl BuildRunner for CommandRunner {
    fn build(&self, spec: &BuildSpec, out_dir: &Path) -> Result<PathBuf, ReproError> {
        std::fs::create_dir_all(out_dir)?;
        let cmd = self
            .build_cmd
            .replace("{src}", &spec.src_dir.display().to_string())
            .replace("{out}", &out_dir.display().to_string())
            .replace("{pkgbase}", &spec.pkgbase)
            .replace("{arch}", &spec.arch);
        let status = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .status()
            .map_err(|e| ReproError::BuildFailed(format!("spawn: {e}")))?;
        if !status.success() {
            return Err(ReproError::BuildFailed(format!(
                "command failed ({status}): {cmd}"
            )));
        }
        Ok(out_dir.to_path_buf())
    }
}

/// Test runner that copies `src_dir` content into `out_dir` — simulates a
/// deterministic "build" for unit tests.
#[derive(Debug, Default)]
pub struct CopyRunner;

impl BuildRunner for CopyRunner {
    fn build(&self, spec: &BuildSpec, out_dir: &Path) -> Result<PathBuf, ReproError> {
        copy_dir(&spec.src_dir, out_dir)?;
        Ok(out_dir.to_path_buf())
    }
}

fn copy_dir(src: &Path, dst: &Path) -> Result<(), ReproError> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let p = entry.path();
        let target = dst.join(entry.file_name());
        if p.is_dir() {
            copy_dir(&p, &target)?;
        } else {
            std::fs::copy(&p, &target)?;
        }
    }
    Ok(())
}
