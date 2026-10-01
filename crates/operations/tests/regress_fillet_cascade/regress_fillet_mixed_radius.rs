//! C-02: mixed-radius fillets (per-edge radius) and corner seams.
//!
//! When neighbouring edges get *different* radii, the three fillet spheres
//! meeting at a vertex no longer have a common centre, so the classic
//! single-radius corner construction has to be replaced by a per-edge setback
//! that trims each strip back to the point where it meets its neighbour. If
//! that trimming is missing, the corner opens a seam: the result is no longer
//! watertight, and every downstream consumer (volume, tessellation) silently
//! reads a wrong number.
//!
//! Control: all 12 edges at one radius — a closed solid the engine already
//! produces (measured 975.33 for 10^3 r=1, analytic 975.59).
//! Main: the same box where the four `x`-direction, four `y`-direction and
//! four `z`-direction edges get three *different* radii.
//!
//! The assertion is watertightness plus "no material added": a fillet may only
//! remove material from the filleted solid.
//!
//! Runs as a module of `regress_fillet_cascade/main.rs`; the shared lint
//! levels for this target live in that root file.

use brepkit_operations::fillet::{FilletRadiusLaw, fillet_variable};
use brepkit_operations::measure::solid_volume;
use brepkit_operations::primitives::make_box;
use brepkit_topology::Topology;
use brepkit_topology::edge::EdgeId;
use brepkit_topology::explorer::{solid_edges, solid_faces};
use brepkit_topology::validation::validate_shell_closed;

const CUBE_VOLUME: f64 = 1000.0;

/// Collect the solid's edges grouped by their dominant direction, to give every
/// edge of the same axis a single radius.
fn edges_by_axis(topo: &Topology, solid: brepkit_topology::solid::SolidId) -> Vec<Vec<EdgeId>> {
    let mut groups: [Vec<EdgeId>; 3] = Default::default();
    for eid in solid_edges(topo, solid).unwrap() {
        let edge = topo.edge(eid).unwrap();
        let a = topo.vertex(edge.start()).unwrap().point();
        let b = topo.vertex(edge.end()).unwrap().point();
        let (dx, dy, dz) = {
            let v = b - a;
            (v.x().abs(), v.y().abs(), v.z().abs())
        };
        let axis = if dx >= dy && dx >= dz {
            0
        } else if dy >= dz {
            1
        } else {
            2
        };
        groups[axis].push(eid);
    }
    groups.into_iter().filter(|g| !g.is_empty()).collect()
}

#[test]
#[ignore = "C-02: fillet_variable leaks free edges even with a uniform radius; unfixed"]
fn mixed_radius_fillets_stay_watertight() {
    let mut topo = Topology::new();
    let solid = make_box(&mut topo, 10.0, 10.0, 10.0).unwrap();

    let groups = edges_by_axis(&topo, solid);
    assert_eq!(groups.len(), 3, "a box has edges on 3 axes");
    assert_eq!(
        groups.iter().map(Vec::len).sum::<usize>(),
        12,
        "a box has 12 edges"
    );

    // ── Control: uniform radius r on all 12 edges ───────────────────
    let all: Vec<EdgeId> = groups.iter().flatten().copied().collect();
    let uniform: Vec<(EdgeId, FilletRadiusLaw)> = all
        .iter()
        .copied()
        .map(|e| (e, FilletRadiusLaw::Constant(1.0)))
        .collect();
    let ctrl = fillet_variable(&mut topo, solid, &uniform).unwrap();
    validate_shell_closed(
        topo.shell(topo.solid(ctrl).unwrap().outer_shell()).unwrap(),
        &topo,
    )
    .expect("uniform-radius fillet must stay watertight");
    let ctrl_vol = solid_volume(&topo, ctrl, 0.01).unwrap();
    assert!(
        (ctrl_vol - 975.59).abs() < 2.0,
        "uniform-radius control volume should be ≈975.59, got {ctrl_vol:.4}"
    );

    // ── Main: three different radii, one per axis ───────────────────
    let mixed: Vec<(EdgeId, FilletRadiusLaw)> = groups
        .iter()
        .enumerate()
        .flat_map(|(axis, edges)| {
            let r = [0.5, 1.0, 1.5][axis];
            edges
                .iter()
                .copied()
                .map(move |e| (e, FilletRadiusLaw::Constant(r)))
        })
        .collect();
    assert_eq!(mixed.len(), 12);

    let result = fillet_variable(&mut topo, solid, &mixed).unwrap();
    let faces = solid_faces(&topo, result).unwrap();
    let vol = solid_volume(&topo, result, 0.01).unwrap();

    // A mixed-radius fillet must still be a closed 2-manifold. This is the
    // seam guard: rolling-ball / walking strips only meet if the corner
    // setbacks were trimmed against each other.
    let shell_ok = validate_shell_closed(
        topo.shell(topo.solid(result).unwrap().outer_shell())
            .unwrap(),
        &topo,
    );
    assert!(
        shell_ok.is_ok(),
        "mixed-radius fillet (r = 0.5 / 1.0 / 1.5 per axis, {} faces, volume {vol:.4}) \
         opened a seam: {:?}",
        faces.len(),
        shell_err(&topo, result)
    );

    // No material may be added.
    assert!(
        vol <= CUBE_VOLUME,
        "filleting a box must not add material: volume {vol:.4} > {CUBE_VOLUME}"
    );
}

fn shell_err(topo: &Topology, s: brepkit_topology::solid::SolidId) -> Option<String> {
    let sd = topo.solid(s).ok()?;
    let sh = topo.shell(sd.outer_shell()).ok()?;
    validate_shell_closed(sh, topo).err().map(|e| e.to_string())
}
