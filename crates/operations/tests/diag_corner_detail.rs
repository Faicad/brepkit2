//! Diagnostic (C-02): dump every face's vertices for BOTH a uniform-radius
//! `fillet_variable` (broken) and `fillet_rolling_ball` (reference, watertight),
//! so we can copy the correct topology.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    deprecated,
    clippy::print_stdout
)]

use brepkit_operations::fillet::{FilletRadiusLaw, fillet_rolling_ball, fillet_variable};
use brepkit_operations::primitives::make_box;
use brepkit_topology::Topology;
use brepkit_topology::explorer::{solid_edges, solid_faces};
use brepkit_topology::face::FaceSurface;

const SIDE: f64 = 10.0;
const R: f64 = 1.0;

fn fmt(p: brepkit_math::vec::Point3) -> String {
    format!("({:.3},{:.3},{:.3})", p.x(), p.y(), p.z())
}

fn dump(topo: &Topology, solid: brepkit_topology::solid::SolidId, label: &str) {
    let faces = solid_faces(topo, solid).unwrap();
    println!("=== {label}: {} faces ===", faces.len());
    for fid in &faces {
        let face = topo.face(*fid).unwrap();
        let surf = match face.surface() {
            FaceSurface::Plane { .. } => "Plane",
            FaceSurface::Nurbs(_) => "Nurbs",
            other => {
                let d = std::mem::discriminant(other);
                let s = format!("{:?}", d);
                Box::leak(s.into_boxed_str())
            }
        };
        let mut pts = Vec::new();
        for oe in topo.wire(face.outer_wire()).unwrap().edges() {
            let e = topo.edge(oe.edge()).unwrap();
            pts.push(fmt(topo.vertex(oe.oriented_start(e)).unwrap().point()));
        }
        println!(
            "  face {:>2} [{}] n={} {}",
            fid.index(),
            surf,
            pts.len(),
            pts.join(" ")
        );
    }
    // Edge free report
    let mut counts: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for fid in &faces {
        let face = topo.face(*fid).unwrap();
        for wid in std::iter::once(face.outer_wire()).chain(face.inner_wires().iter().copied()) {
            for oe in topo.wire(wid).unwrap().edges() {
                *counts.entry(oe.edge().index()).or_insert(0) += 1;
            }
        }
    }
    let free = counts.values().filter(|&&n| n == 1).count();
    println!("  -> free edges: {free}");
}

#[test]
fn dump_rolling_ball_reference() {
    let mut topo = Topology::new();
    let solid = make_box(&mut topo, SIDE, SIDE, SIDE).unwrap();
    let edges = solid_edges(&topo, solid).unwrap();
    let res = fillet_rolling_ball(&mut topo, solid, &edges, R).unwrap();
    dump(&topo, res, "rolling_ball r=1 (reference)");

    let mut topo2 = Topology::new();
    let solid2 = make_box(&mut topo2, SIDE, SIDE, SIDE).unwrap();
    let edges2 = solid_edges(&topo2, solid2).unwrap();
    let laws: Vec<_> = edges2
        .iter()
        .copied()
        .map(|e| (e, FilletRadiusLaw::Constant(R)))
        .collect();
    let res2 = fillet_variable(&mut topo2, solid2, &laws).unwrap();
    dump(&topo2, res2, "fillet_variable r=1 (broken)");
}
