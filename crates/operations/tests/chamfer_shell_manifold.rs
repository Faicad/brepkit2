//! A chamfer must hand back a *closed 2-manifold*, not just a plausible solid.
//!
//! `chamfer_v2` assembles its result the same way `fillet_v2` did before the
//! C-01 welding fix: each producer mints the edges it needs, so the curves two
//! neighbours share exist as two separate `EdgeId`s, each referenced by a
//! single face. `validate_shell_closed` then rejects the shell, and every
//! consumer that walks adjacency (offsetting, shelling, the wasm `is_valid`
//! gate) has nothing to traverse.
//!
//! The invariant asserted here is purely topological and needs no closed form:
//!
//! * every edge of the result shell is referenced by **exactly two** oriented
//!   edges (equivalently, `validate_shell_closed` accepts the shell);
//! * the result is a single connected component.
//!
//! Both are checked against a control that runs the same measurement on
//! `operations::chamfer::chamfer` — a different solver that rebuilds face
//! polygons and assembles through the shared spatial-hash dedup. If the
//! control fails, the measurement or the expectation is wrong, not the engine
//! under test.
//!
//! **The main case is still red and is marked `#[ignore]`.** Measured at
//! `d = 0.5 / 1 / 2`, identically at all three: `F = 18`, `E = 72`, `free = 72`,
//! `over-shared = 0`, 18 connected components. Two independent defects stand
//! between `chamfer_v2` and a closed shell:
//!
//! 1. *Trim keep-side* — fixed in the same round (see
//!    `chamfer_trim_extent.rs`): the trimmer kept the sliver next to the bevel
//!    and dropped the bulk of one of the two faces sharing the edge. Each of
//!    the six side faces is now a full `(S-2d)` square, so this half landed.
//! 2. *Missing corner patches* — still open, and it is the whole of the
//!    remaining gap. `chamfer_builder` never calls `corner::compute_corners`
//!    (the fillet builder does, at `fillet_builder.rs:183`); grep the module
//!    for "corner" and there are zero hits. So the eight corners where three
//!    chamfered edges meet are bounded by nothing at all.
//!
//! The face count settles it. A chamfered cube has 6 side faces + 12 bevels +
//! **8 corner triangles** = 26 faces, and the control engine
//!    (`operations::chamfer::chamfer`, a different solver) produces exactly
//!    that: `F = 26`, side-count histogram `[(3, 8), (4, 18)]`, `free = 0`.
//! `chamfer_v2` produces `F = 18` with histogram `[(4, 18)]` — the eight
//! triangles are simply absent. Each of the 18 faces is individually a closed
//! quad, which is why the shell reads as 18 disconnected components rather
//! than as one shell with holes.
//!
//! So this is **not** the duplicate-entity problem the fillet side had
//! (`blend/src/sew.rs`), and the reason is stronger than "the endpoints
//! differ": there is no second copy of any curve to weld *in the first place*.
//! Measured over the 72 edge occurrences, the number of distinct
//! `(start, end)` position pairs quantised to 1e-6 is also 72 — collapsible
//! count 0. `weld_faces` keys on exactly that pair, so it has nothing to
//! collapse and is a genuine no-op here. Hooking it in would not help; the
//! missing faces have to be built.
//!
//! Every claim on this page is pinned by an assertion in
//! `chamfer_corner_patches.rs` — the face budget, the side-count histogram,
//! the weld-collapse count, and the fact that each face is individually closed
//! while sharing no edge with any neighbour. That file also carries the
//! control case that keeps the expectations honest, and documents what to do
//! with the numbers when the corner patches land.

#![allow(clippy::unwrap_used, clippy::expect_used, deprecated)]

use std::collections::HashMap;

use brepkit_operations::blend_ops::chamfer_v2;
use brepkit_operations::primitives::make_box;
use brepkit_topology::Topology;
use brepkit_topology::explorer::solid_edges;
use brepkit_topology::solid::SolidId;
use brepkit_topology::validation::validate_shell_closed;

/// Cube side used throughout.
const SIDE: f64 = 10.0;

/// Chamfer distances swept. Each leaves a non-degenerate eroded core.
const DISTANCES: [f64; 3] = [0.5, 1.0, 2.0];

/// A binding for an engine that chamfers every edge of a fresh cube.
type Engine = fn(&mut Topology, SolidId, f64) -> Result<SolidId, String>;

fn rebuild(topo: &mut Topology, solid: SolidId, d: f64) -> Result<SolidId, String> {
    let edges = solid_edges(topo, solid).unwrap();
    brepkit_operations::chamfer::chamfer(topo, solid, &edges, d).map_err(|e| e.to_string())
}

fn walking(topo: &mut Topology, solid: SolidId, d: f64) -> Result<SolidId, String> {
    let edges = solid_edges(topo, solid).unwrap();
    chamfer_v2(topo, solid, &edges, d, d)
        .map(|res| res.solid)
        .map_err(|e| e.to_string())
}

/// How many faces reference each edge of the solid's outer shell.
///
/// Returns one entry per distinct edge: `(edge index, usage count)`.
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

/// Number of connected components in the shell's face adjacency graph.
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

/// Full report of one engine run at one distance, formatted for assertions.
fn report(engine: Engine, d: f64) -> Result<String, String> {
    let mut topo = Topology::new();
    let solid = make_box(&mut topo, SIDE, SIDE, SIDE).unwrap();
    let result = engine(&mut topo, solid, d)?;

    let usage = edge_usage(&topo, result);
    let free = usage.iter().filter(|(_, n)| *n == 1).count();
    let over = usage.iter().filter(|(_, n)| *n > 2).count();
    let components = face_components(&topo, result);
    let n_faces = topo
        .shell(topo.solid(result).unwrap().outer_shell())
        .unwrap()
        .faces()
        .len();

    let shell = topo.solid(result).unwrap().outer_shell();
    let validator = validate_shell_closed(topo.shell(shell).unwrap(), &topo);

    let mut out = format!(
        "d={d}: V/E/F={}/{}/{n_faces}, free={free}, over-shared={over}, \
         components={components}, validate_shell_closed={}",
        topo.num_vertices(),
        usage.len(),
        match &validator {
            Ok(()) => "Ok".to_string(),
            Err(e) => format!("Err({e})"),
        }
    );
    if free > 0 {
        use std::fmt::Write;
        let _ = write!(
            out,
            "\n  free edges: {:?}",
            usage
                .iter()
                .filter(|(_, n)| *n == 1)
                .map(|(e, _)| *e)
                .collect::<Vec<_>>()
        );
    }
    Ok(out)
}

/// Control: the polygon-rebuild solver — a different engine, a shared
/// spatial-hash assembly — already produces a closed manifold. This pins the
/// measurement: if this ever fails, the harness or the expectation is at
/// fault, not `chamfer_v2`.
#[test]
fn control_rebuild_box_chamfer_is_a_closed_manifold() {
    let mut offenders = Vec::new();
    for &d in &DISTANCES {
        let mut topo = Topology::new();
        let solid = make_box(&mut topo, SIDE, SIDE, SIDE).unwrap();
        let result = match rebuild(&mut topo, solid, d) {
            Ok(r) => r,
            Err(e) => {
                offenders.push(format!("chamfer (control) d={d}: {e}"));
                continue;
            }
        };

        let shell = topo.solid(result).unwrap().outer_shell();
        if let Err(e) = validate_shell_closed(topo.shell(shell).unwrap(), &topo) {
            offenders.push(format!("chamfer (control) d={d}: {e}"));
        }
        if face_components(&topo, result) != 1 {
            offenders.push(format!("chamfer (control) d={d}: shell is not connected"));
        }
    }
    assert!(
        offenders.is_empty(),
        "the polygon-rebuild engine must produce a closed, connected manifold (control case — if \
         this fails the measurement is wrong, not chamfer_v2):\n{}",
        offenders.join("\n")
    );
}

/// The defect: `chamfer_v2` leaves the chamfered cube with free edges, so the
/// shell is not a closed 2-manifold and is rejected by `validate_shell_closed`.
///
/// Measured at `d = 0.5 / 1 / 2` (identical at all three): `V / E / F = 76 / 72
/// / 18` with `free = 72`, `over-shared = 0` and 18 components — *every* edge
/// is orphaned on both sides, and the 18 faces form 18 separate pieces. The
/// control engine on the same inputs gives `F = 26` with `free = 0`; the eight
/// corner triangles it builds and this engine does not are the whole gap (see
/// the module docs).
#[test]
#[ignore = "chamfer_v2 builds no corner patches, so the 8 corners are unbounded and every edge is free"]
fn walking_engine_box_chamfer_is_a_closed_manifold() {
    let mut offenders = Vec::new();
    for &d in &DISTANCES {
        let mut topo = Topology::new();
        let solid = make_box(&mut topo, SIDE, SIDE, SIDE).unwrap();
        let result = match walking(&mut topo, solid, d) {
            Ok(r) => r,
            Err(e) => {
                offenders.push(format!("chamfer_v2 d={d}: {e}"));
                continue;
            }
        };

        let shell = topo.solid(result).unwrap().outer_shell();
        if let Err(e) = validate_shell_closed(topo.shell(shell).unwrap(), &topo) {
            offenders.push(format!("chamfer_v2 d={d}: {e}"));
        }
        if face_components(&topo, result) != 1 {
            offenders.push(format!(
                "chamfer_v2 d={d}: shell splits into multiple components"
            ));
        }
    }
    assert!(
        offenders.is_empty(),
        "every chamfer_v2 result must be a closed, connected 2-manifold:\n{}\n{}",
        offenders.join("\n"),
        DISTANCES
            .iter()
            .filter_map(|&d| report(walking, d).ok())
            .collect::<Vec<_>>()
            .join("\n")
    );
}
