//! SigmaOS archlinux-repro component.
//!
//! Rust implementation of `archlinux/archlinux-repro`: the `repro` tool
//! that verifies package reproducibility by building twice in clean
//! chroots and comparing the outputs (with optional diffoscope).
//!
//! Upstream `repro` is a bash script wrapping `systemd-nspawn` +
//! `makepkg` + `diffoscope`. This port implements the portable core:
//! build-twice orchestration and deterministic tree comparison — the
//! chroot execution is abstracted behind a [`BuildRunner`] trait.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod compare;
pub mod error;
pub mod job;
pub mod runner;

pub use compare::{DiffReport, compare_trees};
pub use error::ReproError;
pub use job::{ReproJob, ReproResult};
pub use runner::{BuildRunner, BuildSpec, CommandRunner, CopyRunner};
