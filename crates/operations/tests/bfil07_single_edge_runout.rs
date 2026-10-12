//! Regression test for B-FIL-07: fillet a SINGLE edge of a 10×10×10 box —
//! the stripe terminates at sharp corners (single-stripe runout at each
//! end). The result must be a closed manifold with no twin/odd-usage edges.
//!
//! Provenance (2026-10-12): the registry claimed twin edges at the runout
//! leave the shell non-manifold. It does not reproduce: `validate_shell_closed`
//! accepts the result and every edge is used exactly twice (7 faces — two
//! trimmed planes, the blend cylinder, and split/notched caps — an
//! implementation detail, not asserted). The twin-edge claim remains valid
//! only for the SHARED-corner variant, which is the open B-FIL-03 miter
//! work pinned by `bfil03_sharp_corner_miter.rs`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, deprecated)]

use std::collections::HashMap;

use brepkit_operations::blend_ops::fillet_v2;
use brepkit_operations::primitives::make_box;
use brepkit_topology::Topology;
use brepkit_topology::explorer::solid_edges;
use brepkit_topology::solid::SolidId;
use brepkit_topology::validation::validate_shell_closed;

const SIDE: f64 = 10.0;
const R: f64 = 2.0;

/// Edge-use census: how many oriented-edge occurrences reference each edge
/// of the result shell.
fn edge_usage(topo: &Topology, solid: SolidId) -> Vec<(usize, usize)> {
    let shell = topo.solid(solid).unwrap().outer_shell();
    let mut counts: HashMap<usize, usize> = HashMap::new();
    for fid in topo.shell(shell).unwrap().faces() {
        let f = topo.face(*fid).unwrap();
        for oe in topo.wire(f.outer_wire()).unwrap().edges() {
            *counts.entry(oe.edge().index()).or_default() += 1;
        }
    }
    let mut out: Vec<(usize, usize)> = counts.into_iter().collect();
    out.sort_unstable();
    out
}

#[test]
fn bfil07_single_edge_fillet_runout_is_manifold() {
    let mut topo = Topology::new();
    let solid = make_box(&mut topo, SIDE, SIDE, SIDE).unwrap();

    let edges = solid_edges(&topo, solid).unwrap();
    let result = fillet_v2(&mut topo, solid, &[edges[0]], R).unwrap().solid;

    let shell = topo.solid(result).unwrap().outer_shell();
    let validity = validate_shell_closed(topo.shell(shell).unwrap(), &topo);
    assert!(
        validity.is_ok(),
        "B-FIL-07: single-edge fillet runout must close, got {validity:?}"
    );

    let usage = edge_usage(&topo, result);
    let free: Vec<usize> = usage
        .iter()
        .filter(|(_, n)| *n != 2)
        .map(|(e, _)| *e)
        .collect();
    assert!(
        free.is_empty(),
        "B-FIL-07: no twin/odd-usage edges allowed at the runout, got {free:?}"
    );
}
