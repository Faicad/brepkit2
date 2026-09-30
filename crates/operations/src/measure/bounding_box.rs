//! Bounding box computation for B-rep solids.

use std::collections::HashSet;

use brepkit_math::aabb::Aabb3;
use brepkit_math::vec::Point3;
use brepkit_topology::Topology;
use brepkit_topology::face::{FaceId, FaceSurface};
use brepkit_topology::solid::SolidId;

use super::helpers::collect_solid_vertex_points;

/// Compute the axis-aligned bounding box of a solid.
///
/// Uses vertex positions as the base AABB, then expands for non-planar
/// surfaces by sampling edge midpoints on the surface. This captures
/// curvature without over-expanding (unlike projecting the surface's
/// full theoretical extent).
///
/// # Errors
///
/// Returns an error if the solid has no vertices or a topology lookup fails.
pub fn solid_bounding_box(
    topo: &Topology,
    solid: SolidId,
) -> Result<Aabb3, crate::OperationsError> {
    let points = collect_solid_vertex_points(topo, solid)?;
    let mut aabb = Aabb3::try_from_points(points.iter().copied()).ok_or_else(|| {
        crate::OperationsError::InvalidInput {
            reason: "solid has no vertices".into(),
        }
    })?;

    // Expand AABB for non-planar faces by sampling edge midpoints on the
    // actual surface. This captures curvature (e.g., the arc midpoint of a
    // fillet cylinder) without over-expanding to the surface's full extent.
    let solid_data = topo.solid(solid)?;
    let shell = topo.shell(solid_data.outer_shell())?;
    for &fid in shell.faces() {
        if let Ok(face) = topo.face(fid) {
            expand_aabb_for_face(topo, &mut aabb, fid, face.surface());
        }
    }

    Ok(aabb)
}

/// Compute a conservative axis-aligned bounding box over an arbitrary set of
/// faces (e.g. one connected component of a multi-region solid).
///
/// Like [`solid_bounding_box`], the box starts from the faces' vertex
/// positions and is then expanded for surface curvature, so the returned box
/// is a conservative *outer* bound of every face in the set. Used by the
/// disjoint-fuse fast path to test whether two operands' components are
/// spatially separated.
///
/// # Errors
///
/// Returns an error if the face set is empty (no vertices) or a topology
/// lookup fails.
pub fn face_set_bounding_box(
    topo: &Topology,
    faces: &[FaceId],
) -> Result<Aabb3, crate::OperationsError> {
    let mut vertex_ids = HashSet::new();
    for &fid in faces {
        let face = topo.face(fid)?;
        for wire_id in std::iter::once(face.outer_wire()).chain(face.inner_wires().iter().copied())
        {
            let wire = topo.wire(wire_id)?;
            for oe in wire.edges() {
                let edge = topo.edge(oe.edge())?;
                vertex_ids.insert(edge.start());
                vertex_ids.insert(edge.end());
            }
        }
    }

    let mut points = Vec::with_capacity(vertex_ids.len());
    for vid in vertex_ids {
        points.push(topo.vertex(vid)?.point());
    }
    let mut aabb = Aabb3::try_from_points(points.iter().copied()).ok_or_else(|| {
        crate::OperationsError::InvalidInput {
            reason: "face set has no vertices".into(),
        }
    })?;

    for &fid in faces {
        if let Ok(face) = topo.face(fid) {
            expand_aabb_for_face(topo, &mut aabb, fid, face.surface());
        }
    }

    Ok(aabb)
}

/// Expand an AABB to include a point.
fn aabb_include(aabb: &mut Aabb3, p: Point3) {
    *aabb = aabb.union(Aabb3 { min: p, max: p });
}

/// Expand an AABB for a face, accounting for surface curvature.
///
/// Uses different strategies based on surface type:
/// - **Sphere/Torus**: analytic expansion (full surface extent)
/// - **Cylinder/Cone**: wire-bounded expansion (sample edge midpoints
///   to avoid over-expanding for partial arcs like fillets)
/// - **NURBS**: sparse interior grid sampling
/// - **Plane**: no expansion needed
#[allow(clippy::too_many_lines)]
fn expand_aabb_for_face(
    topo: &Topology,
    aabb: &mut Aabb3,
    face_id: brepkit_topology::face::FaceId,
    surface: &FaceSurface,
) {
    // Always sample wire midpoints — captures curvature of curved boundary
    // edges (Circle, Ellipse, NurbsCurve) regardless of surface type.
    // Critical for: cone base discs (Plane face with circle edge), partial
    // arcs whose extremes lie between vertices, and any curved edge on a
    // planar face.
    sample_face_wire_midpoints(topo, aabb, face_id);

    match surface {
        FaceSurface::Plane { .. } => {}

        // Sphere and torus: use analytic expansion (these are typically full
        // or near-full surfaces where the extremes can be far from vertices).
        FaceSurface::Sphere(s) => {
            let c = s.center();
            let r = s.radius();
            aabb_include(aabb, Point3::new(c.x() - r, c.y() - r, c.z() - r));
            aabb_include(aabb, Point3::new(c.x() + r, c.y() + r, c.z() + r));
        }
        FaceSurface::Torus(t) => {
            // Per-dim half-extent of a torus = R * sqrt(1 - axis.d²) + r.
            // The R*sqrt(1-axis.d²) term is the major-circle's extent in
            // world dim d (zero for the dim aligned with the torus axis);
            // the r term is the minor radius offset, which can extend
            // freely in any direction. Replaces the previous formula that
            // applied (R+r) to all dimensions and over-estimated the
            // axis-aligned dim by `R`.
            let c = t.center();
            let r_major = t.major_radius();
            let r_minor = t.minor_radius();
            let axis = t.z_axis();
            let hx = r_major * (1.0 - axis.x() * axis.x()).max(0.0).sqrt() + r_minor;
            let hy = r_major * (1.0 - axis.y() * axis.y()).max(0.0).sqrt() + r_minor;
            let hz = r_major * (1.0 - axis.z() * axis.z()).max(0.0).sqrt() + r_minor;
            aabb_include(aabb, Point3::new(c.x() - hx, c.y() - hy, c.z() - hz));
            aabb_include(aabb, Point3::new(c.x() + hx, c.y() + hy, c.z() + hz));
        }

        // Cylinder: expand radially at each face vertex's axis projection.
        // Unlike the old approach that used AABB corners (which over-expands
        // for fillet cylinders), this uses the face's own vertices to
        // constrain the expansion to the actual face extent.
        FaceSurface::Cylinder(c) => {
            let c = c.clone();
            expand_trimmed_revolution(
                topo,
                aabb,
                face_id,
                |p| c.project_point(p),
                |u, v| c.evaluate(u, v),
            );
        }

        // Cone: same treatment — the trimming edges bound both the angular
        // sweep and the axial (radius-varying) extent.
        FaceSurface::Cone(c) => {
            let c = c.clone();
            expand_trimmed_revolution(
                topo,
                aabb,
                face_id,
                |p| c.project_point(p),
                |u, v| c.evaluate(u, v),
            );
        }

        // NURBS: sample the surface at a sparse interior grid.
        FaceSurface::Nurbs(nurbs) => {
            let (u_min, u_max) = nurbs.domain_u();
            let (v_min, v_max) = nurbs.domain_v();
            let n_samples = 4;
            #[allow(clippy::cast_precision_loss)]
            for iu in 1..n_samples {
                let u = u_min + (u_max - u_min) * (iu as f64) / (n_samples as f64);
                for iv in 1..n_samples {
                    let v = v_min + (v_max - v_min) * (iv as f64) / (n_samples as f64);
                    aabb_include(aabb, nurbs.evaluate(u, v));
                }
            }
        }
    }
}

/// Sample edge midpoints along a face's outer wire to expand the AABB.
///
/// Returns `true` if any curved (non-Line) edges were found. For curved
/// edges (Circle, Ellipse, NurbsCurve), sampling at 0.25, 0.5, 0.75
/// captures the curvature.
fn sample_face_wire_midpoints(
    topo: &Topology,
    aabb: &mut Aabb3,
    face_id: brepkit_topology::face::FaceId,
) -> bool {
    let Ok(face) = topo.face(face_id) else {
        return false;
    };
    let Ok(wire) = topo.wire(face.outer_wire()) else {
        return false;
    };
    let mut has_curved = false;
    for oe in wire.edges() {
        let Ok(edge) = topo.edge(oe.edge()) else {
            continue;
        };
        if !matches!(edge.curve(), brepkit_topology::edge::EdgeCurve::Line) {
            has_curved = true;
        }
        let Ok(sv) = topo.vertex(edge.start()) else {
            continue;
        };
        let Ok(ev) = topo.vertex(edge.end()) else {
            continue;
        };
        let p_start = sv.point();
        let p_end = ev.point();
        let (t0, t1) = edge.curve().domain_with_endpoints(p_start, p_end);
        for &frac in &[0.25, 0.5, 0.75] {
            let t = t0 + (t1 - t0) * frac;
            let pt = edge.curve().evaluate_with_endpoints(t, p_start, p_end);
            aabb_include(aabb, pt);
        }
    }
    has_curved
}

/// Angular samples used when a revolution patch's trimming carries no usable
/// angular span (a full sweep, or a degenerate boundary).
const FULL_RING_SAMPLES: usize = 32;

/// Shortest distance between two angles on a circle of the given period.
fn wrap_dist(a: f64, b: f64, period: f64) -> f64 {
    let d = (a - b).rem_euclid(period);
    if d > period * 0.5 { period - d } else { d }
}

/// The contiguous angular span covered by `angles`, as `(centre, half-width)`,
/// measured in the same units as the angles.
///
/// Returns `None` when the angles cancel out — the samples are spread over
/// the whole period, so no narrower span can be inferred.
fn circular_span(angles: &[f64], period: f64) -> Option<(f64, f64)> {
    let tau = std::f64::consts::TAU;
    let (sx, sy) = angles.iter().fold((0.0, 0.0), |(x, y), &a| {
        let t = tau * a / period;
        (x + t.cos(), y + t.sin())
    });
    if sx.hypot(sy) < 1e-12 {
        return None;
    }
    let mu = sy.atan2(sx).rem_euclid(tau) * period / tau;
    let mut half = 0.0_f64;
    for &a in angles {
        half = half.max(wrap_dist(a, mu, period));
    }
    Some((mu, half))
}

/// Every sampled point of a face's trimming boundary: the endpoints of each
/// edge plus interior samples of the curved ones.
fn face_boundary_samples(topo: &Topology, face_id: FaceId) -> Vec<Point3> {
    use brepkit_topology::edge::EdgeCurve;

    let mut pts = Vec::new();
    let Ok(face) = topo.face(face_id) else {
        return pts;
    };
    for wid in std::iter::once(face.outer_wire()).chain(face.inner_wires().iter().copied()) {
        let Ok(wire) = topo.wire(wid) else {
            continue;
        };
        for oe in wire.edges() {
            let Ok(edge) = topo.edge(oe.edge()) else {
                continue;
            };
            let (Ok(sv), Ok(ev)) = (topo.vertex(edge.start()), topo.vertex(edge.end())) else {
                continue;
            };
            let p0 = sv.point();
            let p1 = ev.point();
            pts.push(p0);
            pts.push(p1);
            if !matches!(edge.curve(), EdgeCurve::Line) {
                let (t0, t1) = edge.curve().domain_with_endpoints(p0, p1);
                for &frac in &[0.25, 0.5, 0.75] {
                    pts.push(
                        edge.curve()
                            .evaluate_with_endpoints(t0 + (t1 - t0) * frac, p0, p1),
                    );
                }
            }
        }
    }
    pts
}

/// Expand the AABB for a trimmed surface of revolution (cylinder or cone).
///
/// The patch's extremes along each axis are attained either on the trimming
/// boundary (already sampled) or at an interior critical angle — one of the
/// four cardinal directions, provided that direction lies inside the patch's
/// angular sweep. Adding a full ring at every boundary vertex instead (the
/// previous behaviour) claims the whole untrimmed surface, so a quarter-turn
/// fillet patch reports the complete cylinder's box.
fn expand_trimmed_revolution<F, G>(
    topo: &Topology,
    aabb: &mut Aabb3,
    face_id: FaceId,
    project: F,
    evaluate: G,
) where
    F: Fn(Point3) -> (f64, f64),
    G: Fn(f64, f64) -> Point3,
{
    let samples = face_boundary_samples(topo, face_id);
    if samples.len() < 2 {
        return;
    }

    let period = std::f64::consts::TAU;
    let mut us = Vec::with_capacity(samples.len());
    let mut vs = Vec::with_capacity(samples.len());
    for &p in &samples {
        let (u, v) = project(p);
        us.push(u);
        vs.push(v);
    }
    let v_min = vs.iter().copied().fold(f64::INFINITY, f64::min);
    let v_max = vs.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    let mut u_candidates: Vec<f64> = Vec::new();
    match circular_span(&us, period) {
        Some((mu, half)) if half < period * 0.5 - 1e-9 => {
            u_candidates.push(mu - half);
            u_candidates.push(mu + half);
            // Interior extrema: a cardinal direction the sweep actually
            // passes through.
            for k in 0..4 {
                let cardinal = period * (k as f64) / 4.0;
                if wrap_dist(cardinal, mu, period) <= half + 1e-9 {
                    u_candidates.push(cardinal);
                }
            }
        }
        // Full sweep (or not enough information to narrow it): keep the box
        // conservative by sampling the whole ring.
        _ =>
        {
            #[allow(clippy::cast_precision_loss)]
            for i in 0..FULL_RING_SAMPLES {
                u_candidates.push(period * (i as f64) / (FULL_RING_SAMPLES as f64));
            }
        }
    }

    for u in u_candidates {
        aabb_include(aabb, evaluate(u, v_min));
        aabb_include(aabb, evaluate(u, v_max));
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use brepkit_math::curves::Circle3D;
    use brepkit_math::surfaces::CylindricalSurface;
    use brepkit_math::vec::Vec3;
    use brepkit_topology::edge::{Edge, EdgeCurve};
    use brepkit_topology::face::Face;
    use brepkit_topology::vertex::Vertex;
    use brepkit_topology::wire::{OrientedEdge, Wire};

    use super::*;

    /// A cylinder patch of radius 1 about the z-axis, trimmed to the quarter
    /// sweep `θ ∈ [0°, 90°]` over `z ∈ [0, 1]`.
    ///
    /// Its true AABB is `[0,1] × [0,1] × [0,1]` — the patch never reaches
    /// negative x or y, even though the *untrimmed* cylinder does.
    fn quarter_cylinder_face(topo: &mut Topology) -> FaceId {
        let cyl =
            CylindricalSurface::new(Point3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0), 1.0)
                .expect("cylinder");
        let circle = Circle3D::new(Point3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0), 1.0)
            .expect("circle");

        let v = |topo: &mut Topology, x: f64, y: f64, z: f64| {
            topo.add_vertex(Vertex::new(Point3::new(x, y, z), 1e-7))
        };
        let a = v(topo, 1.0, 0.0, 0.0); // θ = 0°, z = 0
        let b = v(topo, 0.0, 1.0, 0.0); // θ = 90°, z = 0
        let c = v(topo, 0.0, 1.0, 1.0); // θ = 90°, z = 1
        let d = v(topo, 1.0, 0.0, 1.0); // θ = 0°, z = 1

        // Loop A→B→C→D→A. The top rim is stored as D→C and traversed
        // reversed, so both arcs are the *short* 90° sweep (storing it as
        // C→D would make the parameterisation run the long way round).
        let bottom_arc = topo.add_edge(Edge::new(a, b, EdgeCurve::Circle(circle.clone())));
        let right = topo.add_edge(Edge::new(b, c, EdgeCurve::Line));
        let top_arc = topo.add_edge(Edge::new(d, c, EdgeCurve::Circle(circle)));
        let left = topo.add_edge(Edge::new(d, a, EdgeCurve::Line));

        let wire = topo.add_wire(
            Wire::new(
                vec![
                    OrientedEdge::new(bottom_arc, true),
                    OrientedEdge::new(right, true),
                    OrientedEdge::new(top_arc, false),
                    OrientedEdge::new(left, true),
                ],
                true,
            )
            .expect("wire"),
        );
        topo.add_face(Face::new(wire, vec![], FaceSurface::Cylinder(cyl)))
    }

    /// E-02: a trimmed cylinder's bounding box follows the trimming edges,
    /// not the full analytic surface.
    #[test]
    fn trimmed_cylinder_bbox_respects_arc_bounds() {
        let mut topo = Topology::new();
        let face = quarter_cylinder_face(&mut topo);

        let aabb = face_set_bounding_box(&topo, &[face]).expect("bbox");
        let slack = 1e-6;

        assert!(
            aabb.min.x() >= -slack,
            "min.x reached into the unswept quadrant: {}",
            aabb.min.x()
        );
        assert!(
            aabb.min.y() >= -slack,
            "min.y reached into the unswept quadrant: {}",
            aabb.min.y()
        );
        // Conservative in the other direction: the patch really does span
        // x,y ∈ [0,1] and z ∈ [0,1].
        assert!(
            aabb.max.x() >= 1.0 - slack,
            "max.x too tight: {}",
            aabb.max.x()
        );
        assert!(
            aabb.max.y() >= 1.0 - slack,
            "max.y too tight: {}",
            aabb.max.y()
        );
        assert!(
            aabb.max.z() >= 1.0 - slack,
            "max.z too tight: {}",
            aabb.max.z()
        );
        assert!(aabb.min.z() <= slack, "min.z too tight: {}", aabb.min.z());
    }
}
