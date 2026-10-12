//! Regression test + ready-repro for B-FIL-08.
//!
//! 1. **Closed rim** (green): filleting the full circular rim of a cylinder
//!    hands back a closed shell — the annular closed-rim assembly path
//!    works; the registry's free-edge claim does not reproduce here.
//! 2. **Mixed radius** (ignored, open): two ADJACENT box edges with
//!    different radii leave the shared sharp corner non-manifold. This is
//!    the same junction gap as B-FIL-03 — unequal stripes meeting at a
//!    sharp corner need the cylinder-cylinder miter still to be
//!    implemented — so the ready-repro here shares that acceptance target.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, deprecated)]

use std::collections::HashMap;

use brepkit_blend::fillet_builder::FilletBuilder;
use brepkit_blend::radius_law::RadiusLaw;
use brepkit_operations::blend_ops::fillet_v2;
use brepkit_operations::primitives::{make_box, make_cylinder};
use brepkit_topology::Topology;
use brepkit_topology::explorer::solid_edges;
use brepkit_topology::solid::SolidId;
use brepkit_topology::validation::validate_shell_closed;

fn odd_usage_edges(topo: &Topology, solid: SolidId) -> Vec<usize> {
    let shell = topo.solid(solid).unwrap().outer_shell();
    let mut counts: HashMap<usize, usize> = HashMap::new();
    for fid in topo.shell(shell).unwrap().faces() {
        let f = topo.face(*fid).unwrap();
        for oe in topo.wire(f.outer_wire()).unwrap().edges() {
            *counts.entry(oe.edge().index()).or_default() += 1;
        }
    }
    let mut out: Vec<usize> = counts
        .into_iter()
        .filter(|(_, n)| *n != 2)
        .map(|(e, _)| e)
        .collect();
    out.sort_unstable();
    out
}

/// Closed rim: fillet the bottom circular rim of a cylinder.
#[test]
fn bfil08_closed_rim_fillet_is_manifold() {
    let mut topo = Topology::new();
    let solid = make_cylinder(&mut topo, 5.0, 10.0).unwrap();

    // The bottom rim: the circular edge at z = 0.
    let rim = solid_edges(&topo, solid)
        .unwrap()
        .into_iter()
        .find(|eid| {
            let e = topo.edge(*eid).unwrap();
            let a = topo.vertex(e.start()).unwrap().point();
            a.z().abs() < 1e-9
        })
        .expect("a cylinder has a bottom rim circle");

    let result = fillet_v2(&mut topo, solid, &[rim], 1.0).unwrap().solid;

    let shell = topo.solid(result).unwrap().outer_shell();
    let validity = validate_shell_closed(topo.shell(shell).unwrap(), &topo);
    assert!(
        validity.is_ok(),
        "closed-rim fillet must close: {validity:?}"
    );
    let odd = odd_usage_edges(&topo, result);
    assert!(
        odd.is_empty(),
        "closed-rim fillet has odd-usage edges: {odd:?}"
    );
}

/// Mixed radius: two adjacent box edges (sharing the origin vertex) with
/// different radii.
#[test]
#[ignore = "open: B-FIL-08 / B-FIL-03 family — unequal adjacent stripes at a shared sharp corner leave the junction non-manifold (cylinder-cylinder miter not implemented)"]
fn bfil08_mixed_radius_adjacent_fillets_are_manifold() {
    let mut topo = Topology::new();
    let solid = make_box(&mut topo, 10.0, 10.0, 10.0).unwrap();

    // Two adjacent edges sharing the origin vertex.
    let origin = brepkit_math::vec::Point3::new(0.0, 0.0, 0.0);
    let pair: Vec<_> = solid_edges(&topo, solid)
        .unwrap()
        .into_iter()
        .filter(|eid| {
            let e = topo.edge(*eid).unwrap();
            let a = topo.vertex(e.start()).unwrap().point();
            let b = topo.vertex(e.end()).unwrap().point();
            let at_origin = (a - origin).length() < 1e-9 || (b - origin).length() < 1e-9;
            at_origin && a.z().abs() < 1e-9 && b.z().abs() < 1e-9
        })
        .collect();
    assert_eq!(pair.len(), 2, "two adjacent box edges share the origin");

    let mut builder = FilletBuilder::new(&mut topo, solid);
    builder.add_edges_with_law(&[pair[0]], RadiusLaw::Constant(1.0));
    builder.add_edges_with_law(&[pair[1]], RadiusLaw::Constant(2.0));
    let result = builder.build().unwrap().solid;

    let shell = topo.solid(result).unwrap().outer_shell();
    let validity = validate_shell_closed(topo.shell(shell).unwrap(), &topo);
    assert!(
        validity.is_ok(),
        "mixed-radius fillet must close: {validity:?}"
    );
    let odd = odd_usage_edges(&topo, result);
    assert!(
        odd.is_empty(),
        "mixed-radius fillet has odd-usage edges: {odd:?}"
    );
}
