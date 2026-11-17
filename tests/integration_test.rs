//! Integration tests for SigmaOS-archlinux-repro.

use sigmaos_archlinux_repro_lib::Component;

#[test]
fn component_initialises_and_runs() {
    let mut c = Component::new();
    c.init().expect("init should succeed");
    c.run().expect("run should succeed after init");
}
