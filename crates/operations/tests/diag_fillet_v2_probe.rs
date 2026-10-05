//! Probe: what does blend's uniform fillet look like per-face?
#![allow(clippy::unwrap_used, clippy::print_stdout)]

use brepkit_blend::fillet_builder::FilletBuilder;
use brepkit_blend::radius_law::RadiusLaw;
use brepkit_operations::primitives::make_box;
use brepkit_topology::Topology;
use brepkit_topology::explorer::solid_faces;
use brepkit_topology::validation::validate_shell_closed;

#[test]
fn probe_blend_uniform_topology() {
    let mut topo = Topology::new();
    let solid = make_box(&mut topo, 10.0, 10.0, 10.0).unwrap();
    let all: Vec<_> = brepkit_topology::explorer::solid_edges(&topo, solid).unwrap();
    let mut b = FilletBuilder::new(&mut topo, solid);
    b.add_edges_with_law(&all, RadiusLaw::Constant(1.0));
    let res = b.build().unwrap();
    let faces = solid_faces(&topo, res.solid).unwrap();
    let sd = topo.solid(res.solid).unwrap();
    let sh = topo.shell(sd.outer_shell()).unwrap();
    for f in &faces {
        let face = topo.face(*f).unwrap();
        let w = topo.wire(face.outer_wire()).unwrap();
        print!("F{}:{}v ", f.index(), w.edges().len());
    }
    println!();
    println!("closed={}", validate_shell_closed(sh, &topo).is_ok());
}
