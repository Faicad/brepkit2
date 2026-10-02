//! C-01, remaining half: a fillet must hand back a *closed 2-manifold*.
//!
//! Volume agreement (pinned in `fillet_box_volume.rs`) is necessary but not
//! sufficient. A shell can integrate to exactly the right volume and still be
//! topologically unusable: if two adjacent faces each own a private copy of
//! their shared boundary edge, the shell has free edges, `validate_shell_closed`
//! rejects it, and every downstream consumer that needs adjacency (offsetting,
//! shelling, the wasm `is_valid` gate) has nothing to walk.
//!
//! The invariant asserted here is purely topological and needs no closed form:
//!
//! * every edge of the result shell is referenced by **exactly two** oriented
//!   edges (equivalently, `validate_shell_closed` accepts the shell);
//! * the result is a single connected component.
//!
//! Both are checked against a control case that runs the same measurement on
//! `fillet_rolling_ball` — a different solver. If the control fails, the
//! measurement or the expectation is wrong, not the engine under test.
//!
//! Edge count is reported but not asserted: it depends on how an engine
//! chooses to discretise the blend surface. What is asserted is the *usage
//! count* of each edge, which is discretisation-independent.

#![allow(clippy::unwrap_used, clippy::expect_used, deprecated)]

use std::collections::HashMap;

use brepkit_operations::blend_ops::fillet_v2;
use brepkit_operations::fillet::fillet_rolling_ball;
use brepkit_operations::primitives::make_box;
use brepkit_topology::Topology;
use brepkit_topology::explorer::solid_edges;
use brepkit_topology::solid::SolidId;
use brepkit_topology::validation::validate_shell_closed;

/// Cube side used throughout.
const SIDE: f64 = 10.0;

/// Radii swept. Each leaves a non-degenerate eroded core.
const RADII: [f64; 3] = [0.5, 1.0, 2.0];

/// A binding for an engine that rounds every edge of a fresh cube.
type Engine = fn(&mut Topology, SolidId, f64) -> Result<SolidId, String>;

fn rolling_ball(topo: &mut Topology, solid: SolidId, radius: f64) -> Result<SolidId, String> {
    let edges = solid_edges(topo, solid).unwrap();
    fillet_rolling_ball(topo, solid, &edges, radius).map_err(|e| e.to_string())
}

fn walking(topo: &mut Topology, solid: SolidId, radius: f64) -> Result<SolidId, String> {
    let edges = solid_edges(topo, solid).unwrap();
    fillet_v2(topo, solid, &edges, radius)
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

    // Map each edge to the faces that use it.
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

/// Full report of one engine run at one radius, formatted for assertions.
fn report(engine: Engine, radius: f64) -> Result<String, String> {
    let mut topo = Topology::new();
    let solid = make_box(&mut topo, SIDE, SIDE, SIDE).unwrap();
    let result = engine(&mut topo, solid, radius)?;

    let usage = edge_usage(&topo, result);
    let free = usage.iter().filter(|(_, n)| *n == 1).count();
    let over = usage.iter().filter(|(_, n)| *n > 2).count();
    let components = face_components(&topo, result);

    let shell = topo.solid(result).unwrap().outer_shell();
    let validator = validate_shell_closed(topo.shell(shell).unwrap(), &topo);

    let mut out = format!(
        "r={radius}: edges={}, free={free}, over-shared={over}, components={components}, \
         validate_shell_closed={}",
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

/// Control: the rolling-ball solver — a different engine, analytic contact
/// points — already produces a closed manifold. This pins the measurement:
/// if this ever fails, the harness or the expectation is at fault, not
/// `fillet_v2`.
#[test]
fn control_rolling_ball_box_fillet_is_a_closed_manifold() {
    let mut offenders = Vec::new();
    for &r in &RADII {
        // Uncomment to see the control's own numbers:
        // eprintln!("{}", report(rolling_ball, r).unwrap());
        let mut topo = Topology::new();
        let solid = make_box(&mut topo, SIDE, SIDE, SIDE).unwrap();
        let result = rolling_ball(&mut topo, solid, r).unwrap();

        let shell = topo.solid(result).unwrap().outer_shell();
        if let Err(e) = validate_shell_closed(topo.shell(shell).unwrap(), &topo) {
            offenders.push(format!("rolling_ball r={r}: {e}"));
        }
        if face_components(&topo, result) != 1 {
            offenders.push(format!("rolling_ball r={r}: shell is not connected"));
        }
    }
    assert!(
        offenders.is_empty(),
        "the rolling-ball engine must produce a closed, connected manifold (control case — if \
         this fails the measurement is wrong, not fillet_v2):\n{}",
        offenders.join("\n")
    );
}

/// The defect: `fillet_v2` leaves the filleted cube with free edges, so the
/// shell is not a closed 2-manifold and is rejected by `validate_shell_closed`.
#[test]
fn walking_engine_box_fillet_is_a_closed_manifold() {
    let mut offenders = Vec::new();
    for &r in &RADII {
        let mut topo = Topology::new();
        let solid = make_box(&mut topo, SIDE, SIDE, SIDE).unwrap();
        let result = walking(&mut topo, solid, r).unwrap();

        let shell = topo.solid(result).unwrap().outer_shell();
        if let Err(e) = validate_shell_closed(topo.shell(shell).unwrap(), &topo) {
            offenders.push(format!("fillet_v2 r={r}: {e}"));
        }
        if face_components(&topo, result) != 1 {
            offenders.push(format!(
                "fillet_v2 r={r}: shell splits into multiple components"
            ));
        }
    }
    assert!(
        offenders.is_empty(),
        "every fillet_v2 result must be a closed, connected 2-manifold:\n{}\n{}",
        offenders.join("\n"),
        RADII
            .iter()
            .filter_map(|&r| report(walking, r).ok())
            .collect::<Vec<_>>()
            .join("\n")
    );
}
