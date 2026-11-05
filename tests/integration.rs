use sigmaos_archlinux_repro::*;

#[test]
fn component_is_constructible_and_runnable() {
    // Arrange
    let c = sigmaos_archlinux_repro::ArchlinuxRepro::new("sigmaos-archlinux-repro").expect("valid name");
    // Act
    let out = c.run().expect("run succeeds");
    // Assert
    assert!(out.contains("sigmaos-archlinux-repro"));
}
