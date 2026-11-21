//! Integration tests for sigmaos-archlinux-repro.

use archlinux_repro::{compare_trees, CommandRunner, CopyRunner, ReproJob, BuildSpec, BuildRunner};

#[test]
fn test_end_to_end_repro_check() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("pkgsrc");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(src.join("PKGBUILD"), "pkgname=x\n").unwrap();
    std::fs::write(src.join("built.bin"), "artifact-bytes").unwrap();

    let spec = BuildSpec {
        pkgbase: "x".into(), version: "1.0-1".into(), arch: "x86_64".into(),
        src_dir: src, makepkg_args: vec![],
    };
    let job = ReproJob::new(spec);
    let work = dir.path().join("work");
    std::fs::create_dir_all(&work).unwrap();
    let res = job.run(&CopyRunner, &work).unwrap();
    assert!(res.reproducible);
    assert!(res.build1_dir.join("built.bin").exists());
}

#[test]
fn test_command_runner_failure() {
    let runner = CommandRunner::new("exit 1");
    let spec = BuildSpec {
        pkgbase: "x".into(), version: "1".into(), arch: "any".into(),
        src_dir: std::path::PathBuf::from("/tmp"), makepkg_args: vec![],
    };
    let dir = tempfile::tempdir().unwrap();
    assert!(runner.build(&spec, &dir.path().join("out")).is_err());
}

#[test]
fn test_compare_nested_trees() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a");
    let b = dir.path().join("b");
    std::fs::create_dir_all(a.join("sub/deep")).unwrap();
    std::fs::create_dir_all(b.join("sub/deep")).unwrap();
    std::fs::write(a.join("sub/deep/f"), "same").unwrap();
    std::fs::write(b.join("sub/deep/f"), "same").unwrap();
    let r = compare_trees(&a, &b).unwrap();
    assert!(r.is_reproducible());
}
