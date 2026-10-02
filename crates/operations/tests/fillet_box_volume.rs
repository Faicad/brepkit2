//! C-01, engine half: the volume a walking-engine box fillet must produce.
//!
//! The expected value here is a closed form, not a recorded number. A box with
//! all 12 edges filleted at radius `r` is the Minkowski sum of the eroded core
//! with a ball of radius `r`, so Steiner's formula gives its volume exactly:
//!
//! ```text
//!   x = lx - 2r, y = ly - 2r, z = lz - 2r          (eroded core)
//!   V = x*y*z                                       core itself
//!     + 2r * (xy + yz + xz)                         6 face slabs
//!     + pi * r^2 * (x + y + z)                      12 quarter-cylinder bands
//!     + (4/3) * pi * r^3                            8 sphere octants
//! ```
//!
//! Two independent checks pin this formula down before it is trusted as an
//! oracle:
//!
//! * Limit: `r -> 0` returns the un-filleted volume exactly. Expanding the
//!   closed form gives `V = lx*ly*lz - 2.5752*(lx+ly+lz)*r^2 + O(r^3)`, i.e.
//!   the linear term cancels — so any implementation drifting *linearly* in `r`
//!   is wrong by construction, not just off by a constant.
//! * Cross-engine: `fillet_rolling_ball` is a separate solver (analytic contact
//!   points, no walking), and it reproduces this closed form to measurement
//!   tolerance. That is the control case: it proves the measurement path and
//!   the expected value agree, so a failure in the `fillet_v2` case can only
//!   come from that engine.
//!
//! Filleting removes material, so every result must also stay **below** the
//! un-filleted volume. That bound alone catches the regression without any
//! closed form at all.

#![allow(clippy::unwrap_used, clippy::expect_used, deprecated)]

use brepkit_operations::blend_ops::fillet_v2;
use brepkit_operations::fillet::fillet_rolling_ball;
use brepkit_operations::measure::solid_volume;
use brepkit_operations::primitives::make_box;
use brepkit_topology::Topology;
use brepkit_topology::explorer::solid_edges;

/// Chord-height tolerance handed to the volume integral (tessellation based).
const MEASURE_TOL: f64 = 0.05;

/// Agreement band between a measured volume and the closed form.
///
/// Wider than the tessellation error because the walking engine approximates
/// the blend surface with NURBS; it is still two orders of magnitude tighter
/// than the observed regression (+135 at `r = 1`).
const VOLUME_TOL: f64 = 2.0;

/// Fillet radii swept by the calibration tests. `10.0` is the cube side, so
/// every radius here leaves a non-degenerate eroded core (`side - 2r > 0`).
const RADII: [f64; 4] = [0.1, 0.5, 1.0, 2.0];

/// Cube side used throughout.
const SIDE: f64 = 10.0;

/// Steiner closed form for a box with all 12 edges filleted at `radius`.
fn analytic_box_fillet_volume(side: f64, radius: f64) -> f64 {
    let x = side - 2.0 * radius;
    let core = x * x * x;
    let slabs = 2.0 * radius * 3.0 * x * x;
    let bands = std::f64::consts::PI * radius * radius * 3.0 * x;
    let caps = (4.0 / 3.0) * std::f64::consts::PI * radius * radius * radius;
    core + slabs + bands + caps
}

/// A binding for an engine that rounds every edge of the fresh cube at a
/// caller-supplied radius.
type Engine = fn(
    &mut Topology,
    brepkit_topology::solid::SolidId,
    f64,
) -> Result<brepkit_topology::solid::SolidId, String>;

/// Run one engine over the whole edge set of a fresh cube.
fn measure(engine: Engine, radius: f64) -> f64 {
    let mut topo = Topology::new();
    let solid = make_box(&mut topo, SIDE, SIDE, SIDE).unwrap();
    let result_solid =
        engine(&mut topo, solid, radius).expect("engine must succeed on a plain cube");
    solid_volume(&topo, result_solid, MEASURE_TOL).unwrap()
}

/// A single engine's drift across the radius sweep, formatted for assertions.
fn drift(engine: Engine, label: &str) -> Vec<String> {
    let mut out = Vec::new();
    for &r in &RADII {
        let expected = analytic_box_fillet_volume(SIDE, r);
        let measured = measure(engine, r);
        let delta = measured - expected;
        if delta.abs() >= VOLUME_TOL {
            out.push(format!(
                "{label} r={r}: expected {expected:.4}, measured {measured:.4} (delta {delta:+.4})"
            ));
        }
    }
    out
}

fn rolling_ball(
    topo: &mut Topology,
    solid: brepkit_topology::solid::SolidId,
    radius: f64,
) -> Result<brepkit_topology::solid::SolidId, String> {
    let edges = solid_edges(topo, solid).unwrap();
    fillet_rolling_ball(topo, solid, &edges, radius).map_err(|e| e.to_string())
}

fn walking(
    topo: &mut Topology,
    solid: brepkit_topology::solid::SolidId,
    radius: f64,
) -> Result<brepkit_topology::solid::SolidId, String> {
    let edges = solid_edges(topo, solid).unwrap();
    fillet_v2(topo, solid, &edges, radius)
        .map(|res| res.solid)
        .map_err(|e| e.to_string())
}

#[test]
fn closed_form_converges_to_the_unfilleted_box() {
    for &r in &[1e-6, 1e-4, 1e-3] {
        let v = analytic_box_fillet_volume(SIDE, r);
        assert!(
            (v - SIDE * SIDE * SIDE).abs() < 1e-3,
            "the closed form must converge to the un-filleted volume as r -> 0; \
             at r={r} it gives {v:.9}"
        );
    }
}

#[test]
fn rolling_ball_box_fillet_matches_the_closed_form() {
    let failures = drift(rolling_ball, "rolling_ball");
    assert!(
        failures.is_empty(),
        "the rolling-ball engine must reproduce the closed form (control case: if this \
         fails, the oracle or the measurement path is wrong, not the walking engine):\n{}",
        failures.join("\n")
    );
}

/// Extent of every planar face along the two in-plane axes.
///
/// Returns one `(extent_a, extent_b)` per plane face, sorted so callers need
/// not care about the face's orientation.
fn plane_face_extents(topo: &Topology, solid: brepkit_topology::solid::SolidId) -> Vec<(f64, f64)> {
    let shell = topo.solid(solid).unwrap().outer_shell();
    let faces = brepkit_topology::explorer::solid_faces(topo, solid).unwrap();
    let mut out = Vec::new();
    for fid in faces {
        if !matches!(
            topo.face(fid).unwrap().surface(),
            brepkit_topology::face::FaceSurface::Plane { .. }
        ) {
            continue;
        }
        let mut lo = [f64::INFINITY; 3];
        let mut hi = [f64::NEG_INFINITY; 3];
        let wire = topo.wire(topo.face(fid).unwrap().outer_wire()).unwrap();
        for oe in wire.edges() {
            let e = topo.edge(oe.edge()).unwrap();
            for vid in [e.start(), e.end()] {
                let p = topo.vertex(vid).unwrap().point();
                let coords = [p.x(), p.y(), p.z()];
                for k in 0..3 {
                    lo[k] = lo[k].min(coords[k]);
                    hi[k] = hi[k].max(coords[k]);
                }
            }
        }
        let mut extents: Vec<f64> = (0..3).map(|k| hi[k] - lo[k]).collect();
        // Drop the thickness axis (the one along the face normal).
        extents.sort_by(|a, b| a.partial_cmp(b).unwrap());
        out.push((extents[1], extents[2]));
    }
    let _ = shell;
    out
}

/// Axial extent of every cylindrical blend band, in spine-length units.
///
/// A box fillet turns each of its 12 edges into a quarter-cylinder running
/// along that edge. Projecting the band's wire points onto the cylinder axis
/// recovers how much of the edge the band covers: the whole edge, or only the
/// part between the two corner patches.
fn band_axial_extents(topo: &Topology, solid: brepkit_topology::solid::SolidId) -> Vec<f64> {
    let mut out = Vec::new();
    for fid in brepkit_topology::explorer::solid_faces(topo, solid).unwrap() {
        let face = topo.face(fid).unwrap();
        let brepkit_topology::face::FaceSurface::Cylinder(cyl) = face.surface() else {
            continue;
        };
        let axis = cyl.axis();
        let mut lo = f64::INFINITY;
        let mut hi = f64::NEG_INFINITY;
        for wid in std::iter::once(face.outer_wire()).chain(face.inner_wires().iter().copied()) {
            let wire = topo.wire(wid).unwrap();
            for oe in wire.edges() {
                let e = topo.edge(oe.edge()).unwrap();
                for vid in [e.start(), e.end()] {
                    let s = (topo.vertex(vid).unwrap().point() - cyl.origin()).dot(axis);
                    lo = lo.min(s);
                    hi = hi.max(s);
                }
            }
        }
        out.push(hi - lo);
    }
    out
}

#[test]
fn every_blend_band_stops_one_radius_short_of_each_end() {
    let mut offenders = Vec::new();
    for &r in &RADII {
        let mut topo = Topology::new();
        let solid = make_box(&mut topo, SIDE, SIDE, SIDE).unwrap();
        let result = walking(&mut topo, solid, r).unwrap();

        let extents = band_axial_extents(&topo, result);
        assert_eq!(extents.len(), 12, "a box fillet emits one band per edge");

        // A band must give the spherical corner patch the whole radius at each
        // end: running the full edge makes it overlap both its neighbours and
        // the corner patch, adding material a fillet can only remove.
        let expected = SIDE - 2.0 * r;
        for (i, measured) in extents.iter().enumerate() {
            if (measured - expected).abs() > 1e-6 {
                offenders.push(format!(
                    "fillet_v2 r={r}: band {i} covers {measured:.4} of the {SIDE} edge, \
                     expected {expected:.4}"
                ));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "every blend band must be set back by one radius at each corner:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn every_plane_face_shrinks_by_one_fillet_radius_per_side() {
    let mut offenders = Vec::new();
    for &r in &RADII {
        let mut topo = Topology::new();
        let solid = make_box(&mut topo, SIDE, SIDE, SIDE).unwrap();
        let result = walking(&mut topo, solid, r).unwrap();
        let expected_extent = SIDE - 2.0 * r;
        let extents = plane_face_extents(&topo, result);
        assert_eq!(extents.len(), 6, "a filleted box keeps its 6 plane faces");
        for span in extents {
            for e in [span.0, span.1] {
                if (e - expected_extent).abs() > 1e-6 {
                    offenders.push(format!(
                        "fillet_v2 r={r}: plane face spans {:.4} and {:.4}, expected \
                         {expected_extent:.4} on both — the face was left untrimmed",
                        span.0, span.1
                    ));
                }
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "every plane face must be trimmed back to the contact lines:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn walking_engine_box_fillet_matches_the_closed_form() {
    let failures = drift(walking, "fillet_v2");
    assert!(
        failures.is_empty(),
        "fillet_v2 must reproduce the closed form on a box fillet:\n{}",
        failures.join("\n")
    );
}

/// The cheapest statement of the same contract, with no closed form involved:
/// filleting a convex solid removes material, so the result can never be
/// larger than its input.
#[test]
fn box_fillet_never_adds_material() {
    let mut offenders = Vec::new();
    for &r in &RADII {
        let measured = measure(walking, r);
        let unfilleted = SIDE * SIDE * SIDE;
        if measured > unfilleted + VOLUME_TOL {
            offenders.push(format!(
                "fillet_v2 r={r}: {measured:.4} exceeds the un-filleted {unfilleted:.4} — \
                 a convex fillet can only remove material"
            ));
        }
    }
    assert!(
        offenders.is_empty(),
        "every box fillet must stay below the input volume:\n{}",
        offenders.join("\n")
    );
}
