//! Diagnostic (C-02): measure what `fillet_variable` actually hands back.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    deprecated,
    clippy::print_stdout
)]

use std::collections::HashMap;

use brepkit_operations::fillet::{FilletRadiusLaw, fillet_rolling_ball, fillet_variable};
use brepkit_operations::measure::solid_volume;
use brepkit_operations::primitives::make_box;
use brepkit_topology::Topology;
use brepkit_topology::explorer::{solid_edges, solid_faces};
use brepkit_topology::solid::SolidId;
use brepkit_topology::validation::validate_shell_closed;

const SIDE: f64 = 10.0;

fn edge_usage(topo: &Topology, solid: SolidId) -> Vec<(usize, usize)> {
    let shell = topo.solid(solid).unwrap().outer_shell();
    let mut counts: HashMap<usize, usize> = HashMap::new();
    for fid in topo.shell(shell).unwrap().faces() {
        let face = topo.face(*fid).unwrap();
        for wid in std::iter::once(face.outer_wire()).chain(face.inner_wires().iter().copied()) {
            for oe in topo.wire(wid).unwrap().edges() {
                *counts.entry(oe.edge().index()).or_insert(0) += 1;
            }
        }
    }
    let mut out: Vec<(usize, usize)> = counts.into_iter().collect();
    out.sort_unstable();
    out
}

fn face_components(topo: &Topology, solid: SolidId) -> usize {
    let shell = topo.solid(solid).unwrap().outer_shell();
    let faces = topo.shell(shell).unwrap().faces().to_vec();

    let mut users: HashMap<usize, Vec<usize>> = HashMap::new();
    for (i, fid) in faces.iter().enumerate() {
        let face = topo.face(*fid).unwrap();
        for wid in std::iter::once(face.outer_wire()).chain(face.inner_wires().iter().copied()) {
            for oe in topo.wire(wid).unwrap().edges() {
                users.entry(oe.edge().index()).or_default().push(i);
            }
        }
    }

    let mut seen = vec![false; faces.len()];
    let mut components = 0usize;
    for start in 0..faces.len() {
        if seen[start] {
            continue;
        }
        components += 1;
        let mut stack = vec![start];
        seen[start] = true;
        while let Some(i) = stack.pop() {
            let face = topo.face(faces[i]).unwrap();
            for wid in std::iter::once(face.outer_wire()).chain(face.inner_wires().iter().copied())
            {
                for oe in topo.wire(wid).unwrap().edges() {
                    for &j in users.get(&oe.edge().index()).unwrap_or(&Vec::new()) {
                        if !seen[j] {
                            seen[j] = true;
                            stack.push(j);
                        }
                    }
                }
            }
        }
    }
    components
}

fn dump(label: &str, topo: &Topology, solid: SolidId) {
    let usage = edge_usage(topo, solid);
    let free = usage.iter().filter(|(_, n)| *n == 1).count();
    let over = usage.iter().filter(|(_, n)| *n > 2).count();
    let comps = face_components(topo, solid);
    let faces = solid_faces(topo, solid).unwrap();
    let shell = topo.solid(solid).unwrap().outer_shell();
    let closed = validate_shell_closed(topo.shell(shell).unwrap(), topo);
    let vol = solid_volume(topo, solid, 0.01).unwrap_or(f64::NAN);

    println!(
        "{label}: V/E/F={}/{}/{}, free={free}, over={over}, components={comps}, vol={vol:.4}, closed={:?}",
        topo.num_vertices(),
        usage.len(),
        faces.len(),
        match &closed {
            Ok(()) => "Ok".to_string(),
            Err(e) => format!("Err({e})"),
        }
    );
    let free_idx: Vec<usize> = usage
        .iter()
        .filter(|(_, n)| *n == 1)
        .map(|(i, _)| *i)
        .collect();
    println!("  free edges: {free_idx:?}");

    if std::env::var("DUMP_FACES").is_ok() {
        for fid in &faces {
            let face = topo.face(*fid).unwrap();
            let mut pts = Vec::new();
            for oe in topo.wire(face.outer_wire()).unwrap().edges() {
                let v = topo
                    .vertex(oe.oriented_start(topo.edge(oe.edge()).unwrap()))
                    .unwrap();
                pts.push(format!(
                    "({:.3},{:.3},{:.3})",
                    v.point().x(),
                    v.point().y(),
                    v.point().z()
                ));
            }
            println!(
                "  face {}: surf={:?} n={} {:?}",
                fid.index(),
                std::mem::discriminant(face.surface()),
                pts.len(),
                pts
            );
        }
    }
}

#[test]
fn diag_uniform_and_mixed() {
    let radii: Vec<f64> = vec![1.0];

    // ── fillet_variable, uniform radius ──────────────────────────────
    let mut topo = Topology::new();
    let solid = make_box(&mut topo, SIDE, SIDE, SIDE).unwrap();
    let edges = solid_edges(&topo, solid).unwrap();
    let laws: Vec<_> = edges
        .iter()
        .copied()
        .map(|e| (e, FilletRadiusLaw::Constant(radii[0])))
        .collect();
    let res = fillet_variable(&mut topo, solid, &laws).unwrap();
    dump("fillet_variable r=1", &topo, res);

    // ── control: fillet_rolling_ball, same radius ────────────────────
    let mut topo2 = Topology::new();
    let solid2 = make_box(&mut topo2, SIDE, SIDE, SIDE).unwrap();
    let edges2 = solid_edges(&topo2, solid2).unwrap();
    let ctrl = fillet_rolling_ball(&mut topo2, solid2, &edges2, 1.0).unwrap();
    dump("rolling_ball   r=1", &topo2, ctrl);

    // ── fillet_variable, mixed radii per axis ────────────────────────
    let mut topo3 = Topology::new();
    let solid3 = make_box(&mut topo3, SIDE, SIDE, SIDE).unwrap();
    let mut groups: [Vec<_>; 3] = Default::default();
    for eid in solid_edges(&topo3, solid3).unwrap() {
        let e = topo3.edge(eid).unwrap();
        let a = topo3.vertex(e.start()).unwrap().point();
        let b = topo3.vertex(e.end()).unwrap().point();
        let v = b - a;
        let (dx, dy, dz) = (v.x().abs(), v.y().abs(), v.z().abs());
        let axis = if dx >= dy && dx >= dz {
            0
        } else if dy >= dz {
            1
        } else {
            2
        };
        groups[axis].push(eid);
    }
    let mixed: Vec<_> = groups
        .iter()
        .enumerate()
        .flat_map(|(axis, g)| {
            let r = [0.5, 1.0, 1.5][axis];
            g.iter()
                .copied()
                .map(move |e| (e, FilletRadiusLaw::Constant(r)))
        })
        .collect();
    let res3 = fillet_variable(&mut topo3, solid3, &mixed).unwrap();
    dump("fillet_variable mixed", &topo3, res3);
}
