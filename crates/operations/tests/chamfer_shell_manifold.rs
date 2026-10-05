//! A chamfer must hand back a *closed 2-manifold*, not just a plausible solid.
//!
//! `chamfer_v2` used to assemble its result the way `fillet_v2` did before the
//! C-01 welding fix: each producer minted the edges it needed, so the curves
//! two neighbours share existed as separate `EdgeId`s each referenced by a
//! single face, and `validate_shell_closed` rejected the shell. Every consumer
//! that walks adjacency — offsetting, shelling, the wasm `is_valid` gate — had
//! nothing to traverse.
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
//! # What was missing, and why welding alone could not supply it
//!
//! Two independent defects stood between `chamfer_v2` and a closed shell, and
//! they had to land together:
//!
//! 1. *Trim keep-side* — fixed earlier (see `chamfer_trim_extent.rs`): the
//!    trimmer kept the sliver next to the bevel and dropped the bulk of one of
//!    the two faces sharing the edge.
//! 2. *Missing corner patches* — fixed here. `chamfer_builder` built each bevel
//!    over the whole of its edge, so at a vertex where three chamfered edges
//!    meet the bevels ran into one another and the corner was bounded by
//!    nothing: 18 quads, 18 disconnected components, and no corner patches at
//!    all.
//!
//! The fix is in `blend::chamfer_corner`. Each such vertex gets a flat
//! triangle through the three contact points — the plane that cuts the corner
//! off — and each bevel is set back along its spine so that it stops where
//! that triangle starts instead of overlapping it. The setback is read off the
//! triangle's own corners, so the two agree by construction rather than by two
//! independent derivations happening to match.
//!
//! Welding is still needed, but for the opposite reason from the fillet case:
//! now that the corner patches exist there *is* a second copy of every shared
//! curve to collapse, and `sew::weld_faces` merges them once the shell is
//! assembled. Measured before the patches existed, that collapse count was
//! zero — which is what made welding a no-op and proved the faces had to be
//! built rather than deduplicated.
//!
//! # What was ruled out, so nobody re-runs it
//!
//! Calling `corner::compute_corners`, the way `fillet_builder` does. It does
//! produce 8 corner faces and the right face count, but they come out as NURBS
//! spherical patches rather than flat triangles and the shell still does not
//! close — `V/E/F = 100/96/26`, `free = 96`, 26 components. `corner.rs` routes
//! 3+ stripe vertices to `spherical_triangle`, documented as "rolling-ball
//! sphere" and "great-circle arcs"; a chamfer corner is a flat triangle and a
//! spherical patch's boundary arcs do not land on the straight contact lines it
//! has to meet. See `the_spherical_corner_path_is_wrong_for_chamfer` in
//! `chamfer_corner_patches.rs` for the standing record.
//!
//! # One caveat about reading Euler here
//!
//! `validate_shell_closed` accepting a shell means *every edge is referenced
//! exactly twice*, not that `V - E + F == 2`. The control engine passes this
//! test while carrying surplus vertices — measured `V = 32` against the 24 a
//! chamfered cube needs, giving `V - E + F = 10`. It is watertight but not
//! Euler-clean, so a `V - E + F == 2` assertion would fail on the very engine
//! this file uses as its reference. Neither engine is expected to satisfy it.

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

/// `chamfer_v2`'s result must be a closed, connected 2-manifold too.
///
/// This was red until `chamfer_corner` landed. The old measurement, kept here
/// as the "before": `V/E/F = 76/72/18`, `free = 72`, `over-shared = 0`, 18
/// components — every edge orphaned on both sides and 18 separate pieces,
/// because the eight corner triangles did not exist and the twelve bevels each
/// ran the full length of their edge.
///
/// The control engine on the same inputs gives `F = 26` with `free = 0`, and
/// that is now what the walking engine produces as well.
#[test]
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
