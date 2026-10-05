//! Diagnostic: does blend's fillet ADD material on a single edge of a box?
//! (regression check for delegating fillet_variable to the blend engine)
#![allow(clippy::unwrap_used, clippy::print_stdout)]

use brepkit_blend::fillet_builder::FilletBuilder;
use brepkit_operations::blend_ops::fillet_v2;
use brepkit_operations::measure::solid_volume;
use brepkit_operations::primitives::make_box;
use brepkit_topology::Topology;
use brepkit_topology::explorer::solid_edges;

#[test]
fn probe_single_edge_blend() {
    // fillet_v2 (blend public entry), single edge r=1
    let mut topo = Topology::new();
    let solid = make_box(&mut topo, 10.0, 10.0, 10.0).unwrap();
    let edges = solid_edges(&topo, solid).unwrap();
    let res = fillet_v2(&mut topo, solid, &[edges[0]], 1.0).unwrap();
    let v = solid_volume(&topo, res.solid, 0.05).unwrap();
    println!("fillet_v2 single edge r=1 volume={v:.4} (box=1000)");

    // FilletBuilder per-edge add_edges_with_law (how fillet_variable delegates)
    let mut topo = Topology::new();
    let solid = make_box(&mut topo, 10.0, 10.0, 10.0).unwrap();
    let edges = solid_edges(&topo, solid).unwrap();
    let mut b = FilletBuilder::new(&mut topo, solid);
    b.add_edges_with_law(
        &[edges[0]],
        brepkit_blend::radius_law::RadiusLaw::Constant(1.0),
    );
    let res = b.build().unwrap();
    let v = solid_volume(&topo, res.solid, 0.05).unwrap();
    println!(
        "FilletBuilder per-edge single edge r=1 volume={v:.4} succeeded={}",
        res.succeeded.len()
    );

    // Full cube uniform r=1 for reference
    let mut topo = Topology::new();
    let solid = make_box(&mut topo, 10.0, 10.0, 10.0).unwrap();
    let edges = solid_edges(&topo, solid).unwrap();
    let res = fillet_v2(&mut topo, solid, &edges, 1.0).unwrap();
    let v = solid_volume(&topo, res.solid, 0.05).unwrap();
    println!("fillet_v2 full cube r=1 volume={v:.4}");
}
