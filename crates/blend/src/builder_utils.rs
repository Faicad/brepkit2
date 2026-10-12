//! Shared utilities for fillet and chamfer builders.
//!
//! Functions used by both [`FilletBuilder`](crate::fillet_builder::FilletBuilder)
//! and [`ChamferBuilder`](crate::chamfer_builder::ChamferBuilder) for creating
//! blend faces and sampling contact curves.

use brepkit_math::nurbs::curve::NurbsCurve;
use brepkit_math::traits::ParametricSurface;
use brepkit_math::vec::{Point3, Vec3};
use brepkit_topology::Topology;
use brepkit_topology::edge::{Edge, EdgeCurve};
use brepkit_topology::face::{Face, FaceId, FaceSurface};
use brepkit_topology::vertex::{Vertex, VertexId};
use brepkit_topology::wire::{OrientedEdge, Wire};

use crate::BlendError;
use crate::stripe::Stripe;

/// Sample the start and end points of a NURBS curve.
#[must_use]
pub fn sample_nurbs_endpoints(curve: &NurbsCurve) -> Vec<Point3> {
    let (t0, t1) = curve.domain();
    vec![curve.evaluate(t0), curve.evaluate(t1)]
}

/// Create a blend face from a stripe's surface and contact curves.
///
/// Builds a minimal quadrilateral wire from the four contact-curve endpoints
/// and associates the blend surface with it.
///
/// # Errors
///
/// Returns [`BlendError`] if wire or face construction fails.
/// [`create_blend_face`] that REUSES the trimmers' contact edges when they
/// span the same contacts. Minting fresh edges for curves the trimmed
/// neighbours already carry leaves two edge entities per contact — each used
/// by one face — opening the shell along every blend flank. A trimmer edge
/// is adopted (with its vertices) when its endpoints match the stripe's
/// contact endpoints within the weld band, in either orientation; otherwise
/// that side falls back to a fresh edge.
/// First and last traversal vertices of an adopted contact chain.
fn chain_ends(topo: &Topology, oes: &[OrientedEdge]) -> Option<(VertexId, VertexId)> {
    let first = oes.first()?;
    let last = oes.last()?;
    let e_first = topo.edge(first.edge()).ok()?;
    let e_last = topo.edge(last.edge()).ok()?;
    Some((first.oriented_start(e_first), last.oriented_end(e_last)))
}

/// Adopt the trimmed neighbour's contact boundary — see the caller.
pub fn create_blend_face_with_contacts(
    topo: &mut Topology,
    stripe: &Stripe,
    contact1_edge: Option<brepkit_topology::edge::EdgeId>,
    contact1_face: Option<FaceId>,
    contact2_edge: Option<brepkit_topology::edge::EdgeId>,
    contact2_face: Option<FaceId>,
) -> Result<BlendFaceInfo, BlendError> {
    const WELD: f64 = 1e-5;
    let (t0_1, t1_1) = stripe.contact1.domain();
    let (t0_2, t1_2) = stripe.contact2.domain();

    let p1_start = stripe.contact1.evaluate(t0_1);
    let p1_end = stripe.contact1.evaluate(t1_1);
    let p2_start = stripe.contact2.evaluate(t0_2);
    let p2_end = stripe.contact2.evaluate(t1_2);

    // Adopt the trimmed neighbour's contact boundary for the span
    // `(want_s, want_e)`, returning the oriented-edge chain in traversal
    // order — a single edge when the neighbour still holds the full span,
    // or the run of sub-segments a LATER stripe's re-trim left behind.
    //
    // The recorded contact edge can go stale: a later stripe's re-trim of
    // the same face splits the earlier full-span contact edge, and only
    // shorter sub-segments survive in the trimmed face's wire. Direct
    // adoption checks the recorded edge first; when it misses, walk the
    // side's face wire from the vertex matching `want_s` to the one
    // matching `want_e`, requiring every traversed edge to lie along the
    // straight contact segment, and adopt that chain.
    let adopt = |topo: &Topology,
                 eid: Option<brepkit_topology::edge::EdgeId>,
                 face: Option<FaceId>,
                 want_s: Point3,
                 want_e: Point3|
     -> Option<Vec<OrientedEdge>> {
        if let Some(eid) = eid {
            let e = topo.edge(eid).ok()?;
            let (sv, ev) = (e.start(), e.end());
            let sp = topo.vertex(sv).ok()?.point();
            let ep = topo.vertex(ev).ok()?.point();
            if (sp - want_s).length() <= WELD && (ep - want_e).length() <= WELD {
                return Some(vec![OrientedEdge::new(eid, true)]);
            }
            if (sp - want_e).length() <= WELD && (ep - want_s).length() <= WELD {
                return Some(vec![OrientedEdge::new(eid, false)]);
            }
        }
        // Chain walk along the trimmed face's current boundary.
        let face = face?;
        let wire = topo.face(face).ok()?.outer_wire();
        let oes = topo.wire(wire).ok()?.edges().to_vec();
        let n = oes.len();
        if n == 0 {
            return None;
        }
        let vertex_pos =
            |vid: VertexId| -> Option<Point3> { topo.vertex(vid).ok().map(Vertex::point) };
        let on_segment = |q: Point3| -> bool {
            let d = want_e - want_s;
            let len2 = d.length_squared();
            if len2 <= 1e-18 {
                return false;
            }
            let t = (q - want_s).dot(d) / len2;
            if !(-1e-9..=1.0 + 1e-9).contains(&t) {
                return false;
            }
            (want_s + d * t - q).length() <= WELD
        };
        // At a T junction several wire edges can start at `want_s` (the
        // contact segment and a later stripe's contact line meeting it),
        // so a start candidate must also run ALONG the segment: require
        // its far endpoint to lie on it before walking.
        let start_idx = (0..n).find(|&i| {
            topo.edge(oes[i].edge()).ok().and_then(|e| {
                let sp = vertex_pos(oes[i].oriented_start(e))?;
                let tp = vertex_pos(oes[i].oriented_end(e))?;
                Some((sp - want_s).length() <= WELD && on_segment(tp))
            }) == Some(true)
        });
        let end_idx = (0..n).find(|&i| {
            topo.edge(oes[i].edge()).ok().and_then(|e| {
                let sp = vertex_pos(oes[i].oriented_start(e))?;
                let tp = vertex_pos(oes[i].oriented_end(e))?;
                Some((tp - want_s).length() <= WELD && on_segment(sp))
            }) == Some(true)
        });
        // The wire may traverse the contact span in either direction.
        // Forward: an occurrence starts at `want_s`; walk forward to the
        // one ending at `want_e`. Backward: an occurrence ENDS at
        // `want_s`; walk backward to the one starting at `want_e`, then
        // reverse the collected run (flipping each orientation) so the
        // chain still reads want_s → want_e.
        match (start_idx, end_idx) {
            (Some(start_idx), _) => {
                let mut chain: Vec<OrientedEdge> = Vec::new();
                let mut cur = start_idx;
                loop {
                    let oe = oes[cur];
                    let e = topo.edge(oe.edge()).ok()?;
                    let sp = vertex_pos(oe.oriented_start(e))?;
                    let tp = vertex_pos(oe.oriented_end(e))?;
                    if !on_segment(sp) || !on_segment(tp) {
                        return None;
                    }
                    chain.push(oe);
                    if (tp - want_e).length() <= WELD {
                        return Some(chain);
                    }
                    cur = (cur + 1) % n;
                    if chain.len() >= n {
                        return None;
                    }
                }
            }
            (None, Some(end_idx)) => {
                let mut chain: Vec<OrientedEdge> = Vec::new();
                let mut cur = end_idx;
                loop {
                    let oe = oes[cur];
                    let e = topo.edge(oe.edge()).ok()?;
                    let sp = vertex_pos(oe.oriented_start(e))?;
                    let tp = vertex_pos(oe.oriented_end(e))?;
                    if !on_segment(sp) || !on_segment(tp) {
                        return None;
                    }
                    chain.push(OrientedEdge::new(oe.edge(), !oe.is_forward()));
                    if (sp - want_e).length() <= WELD {
                        chain.reverse();
                        return Some(chain);
                    }
                    cur = (cur + n - 1) % n;
                    if chain.len() >= n {
                        return None;
                    }
                }
            }
            (None, None) => None,
        }
    };
    let chain1 = adopt(topo, contact1_edge, contact1_face, p1_start, p1_end);
    // Contact 2 traverses end -> start in the quad below.
    let chain2 = adopt(topo, contact2_edge, contact2_face, p2_end, p2_start);

    // Create/reuse vertices (snapshot then allocate). An adopted chain's
    // endpoints are the trimmer's own vertices — reusing them is what lets
    // the end-cap notch (which keys on `VertexId`) match later.
    let (v1s, v1e) = chain1
        .as_ref()
        .and_then(|oes| chain_ends(topo, oes))
        .unwrap_or_else(|| {
            (
                topo.add_vertex(Vertex::new(p1_start, 1e-7)),
                topo.add_vertex(Vertex::new(p1_end, 1e-7)),
            )
        });
    let (v2e, v2s) = chain2
        .as_ref()
        .and_then(|oes| chain_ends(topo, oes))
        .unwrap_or_else(|| {
            (
                topo.add_vertex(Vertex::new(p2_end, 1e-7)),
                topo.add_vertex(Vertex::new(p2_start, 1e-7)),
            )
        });

    // Build the blend loop: contact1 side (p1_start → p1_end), cross end,
    // contact2 side (p2_end → p2_start), cross start. An adopted chain's
    // occurrences are used verbatim so the blend face shares the neighbour's
    // sub-segment edge entities; a side without adoption falls back to a
    // fresh longitudinal edge along the stripe's own contact curve.
    // Cross edges carry the true end cross-section arcs when the stripe has
    // sections: the fillet's end profile is a circular arc, and a straight
    // chord both misrepresents the surface boundary and can never be shared
    // with a notched end cap. The arc's plane normal comes from the two
    // contact endpoints and the section centre.
    let arc_curve =
        |sec: &crate::section::CircSection, a: Point3, b: Point3| -> Option<EdgeCurve> {
            let u = a - sec.center;
            let v = b - sec.center;
            let n = u.cross(v);
            let n = n.normalize().ok()?;
            let circle = brepkit_math::curves::Circle3D::new(sec.center, n, sec.radius).ok()?;
            Some(EdgeCurve::Circle(circle))
        };
    let end_curve = stripe
        .sections
        .last()
        .and_then(|sec| arc_curve(sec, p1_end, p2_end))
        .unwrap_or(EdgeCurve::Line);
    let start_curve = stripe
        .sections
        .first()
        .and_then(|sec| arc_curve(sec, p2_start, p1_start))
        .unwrap_or(EdgeCurve::Line);
    let e1 = topo.add_edge(Edge::new(v1e, v2e, end_curve));
    let e3 = topo.add_edge(Edge::new(v2s, v1s, start_curve));

    let mut loop_edges: Vec<OrientedEdge> = if let Some(oes) = &chain1 {
        oes.clone()
    } else {
        let e0 = topo.add_edge(Edge::new(
            v1s,
            v1e,
            EdgeCurve::NurbsCurve(stripe.contact1.clone()),
        ));
        vec![OrientedEdge::new(e0, true)]
    };
    loop_edges.push(OrientedEdge::new(e1, true));
    if let Some(oes) = &chain2 {
        loop_edges.extend(oes.iter().copied());
    } else {
        let e2 = topo.add_edge(Edge::new(
            v2e,
            v2s,
            EdgeCurve::NurbsCurve(stripe.contact2.clone()),
        ));
        loop_edges.push(OrientedEdge::new(e2, true));
    }
    loop_edges.push(OrientedEdge::new(e3, true));

    let wire = Wire::new(loop_edges, true)?;
    let wire_id = topo.add_wire(wire);

    let face = Face::new(wire_id, Vec::new(), stripe.surface.clone());
    let face_id = topo.add_face(face);

    Ok(BlendFaceInfo {
        face: face_id,
        cross_end: (e1, v1e, v2e),
        cross_start: (e3, v2s, v1s),
    })
}

/// A created blend face plus its two cross edges (the end cross-section
/// arcs), each with its (from, to) vertices in the blend wire's traversal
/// direction — the handles the end-cap notch surgery needs to SHARE those
/// arcs instead of leaving both sides use-1.
pub struct BlendFaceInfo {
    /// The blend face.
    pub face: FaceId,
    /// Cross edge at the spine end: `(edge, from, to)`.
    pub cross_end: (brepkit_topology::edge::EdgeId, VertexId, VertexId),
    /// Cross edge at the spine start: `(edge, from, to)`.
    pub cross_start: (brepkit_topology::edge::EdgeId, VertexId, VertexId),
}

/// Replace a face's two-edge corner path `from -> corner -> to` with the
/// single cross-section arc `edge`, notching the fillet's end profile out of
/// an end cap so the cap and the blend share one edge entity. Both replaced
/// edges must be straight (the box corner sides); returns whether a
/// replacement happened.
pub fn notch_face_corner_with_arc(
    topo: &mut Topology,
    face_id: FaceId,
    arc: (brepkit_topology::edge::EdgeId, VertexId, VertexId),
) -> Result<Option<FaceId>, BlendError> {
    let (arc_eid, va, vb) = arc;
    let wire_id = topo.face(face_id)?.outer_wire();
    let oes = topo.wire(wire_id)?.edges().to_vec();
    let n = oes.len();
    if n < 3 {
        return Ok(None);
    }
    let ends = |oe: &OrientedEdge| -> Result<(VertexId, VertexId), BlendError> {
        let e = topo.edge(oe.edge())?;
        Ok((oe.oriented_start(e), oe.oriented_end(e)))
    };
    for i in 0..n {
        let j = (i + 1) % n;
        let (s0, e0) = ends(&oes[i])?;
        let (s1, e1) = ends(&oes[j])?;
        if e0 != s1 || e0 == va || e0 == vb {
            continue;
        }
        let fwd = s0 == va && e1 == vb;
        let rev = s0 == vb && e1 == va;
        if !(fwd || rev) {
            continue;
        }
        let both_straight = [oes[i].edge(), oes[j].edge()].iter().all(|&eid| {
            topo.edge(eid)
                .is_ok_and(|e| matches!(e.curve(), EdgeCurve::Line))
        });
        if !both_straight {
            continue;
        }
        let mut new_oes: Vec<OrientedEdge> = Vec::with_capacity(n - 1);
        for (k, oe) in oes.iter().enumerate() {
            if k == i {
                new_oes.push(OrientedEdge::new(arc_eid, fwd));
            } else if k != j {
                new_oes.push(*oe);
            }
        }
        let new_wire = topo.add_wire(Wire::new(new_oes, true)?);
        let (surface, reversed, inners) = {
            let f = topo.face(face_id)?;
            (
                f.surface().clone(),
                f.is_reversed(),
                f.inner_wires().to_vec(),
            )
        };
        let new_face = if reversed {
            Face::new_reversed(new_wire, inners, surface)
        } else {
            Face::new(new_wire, inners, surface)
        };
        let nf = topo.add_face(new_face);
        return Ok(Some(nf));
    }
    Ok(None)
}

/// Adapter that provides [`ParametricSurface`] for a `FaceSurface::Plane`.
///
/// Planes store only a normal and signed distance `d`, with no parametric
/// frame.  This adapter builds an orthonormal UV frame from the normal so
/// that the walking engine can evaluate, project, and differentiate the
/// plane surface uniformly.
pub struct PlaneAdapter {
    /// Origin point on the plane (the point closest to the world origin).
    pub origin: Point3,
    /// U-direction tangent (unit vector in the plane).
    pub u_dir: Vec3,
    /// V-direction tangent (unit vector in the plane, orthogonal to `u_dir`).
    pub v_dir: Vec3,
    /// Outward-facing unit normal.
    pub norm: Vec3,
}

impl PlaneAdapter {
    /// Build a `PlaneAdapter` from a plane normal and signed distance.
    ///
    /// The UV frame is constructed by choosing a non-parallel reference vector
    /// and computing the cross products.
    #[must_use]
    pub fn from_normal_and_d(normal: Vec3, d: f64) -> Self {
        let origin = Point3::new(normal.x() * d, normal.y() * d, normal.z() * d);

        // Pick a reference vector that is not parallel to the normal.
        let ref_vec = if normal.x().abs() < 0.9 {
            Vec3::new(1.0, 0.0, 0.0)
        } else {
            Vec3::new(0.0, 1.0, 0.0)
        };

        let u_dir = normal
            .cross(ref_vec)
            .normalize()
            .unwrap_or(Vec3::new(1.0, 0.0, 0.0));
        let v_dir = normal
            .cross(u_dir)
            .normalize()
            .unwrap_or(Vec3::new(0.0, 1.0, 0.0));

        Self {
            origin,
            u_dir,
            v_dir,
            norm: normal,
        }
    }
}

impl ParametricSurface for PlaneAdapter {
    fn evaluate(&self, u: f64, v: f64) -> Point3 {
        self.origin + self.u_dir * u + self.v_dir * v
    }

    fn normal(&self, _u: f64, _v: f64) -> Vec3 {
        self.norm
    }

    fn project_point(&self, point: Point3) -> (f64, f64) {
        let d = point - self.origin;
        (d.dot(self.u_dir), d.dot(self.v_dir))
    }

    fn partial_u(&self, _u: f64, _v: f64) -> Vec3 {
        self.u_dir
    }

    fn partial_v(&self, _u: f64, _v: f64) -> Vec3 {
        self.v_dir
    }
}

/// A [`ParametricSurface`] view that negates the wrapped surface's normal.
///
/// The walking engine's blend constraint places the rolling-ball centre on the
/// `+normal` side of each surface (`centre = p + r·normal`), so the surfaces
/// must present their **inward** (toward-material) normals. `PlaneAdapter`
/// flips a plane via its stored normal, but analytic/NURBS surfaces have an
/// intrinsic outward normal that can't be re-oriented in place — wrapping one
/// here flips it so a fillet against a curved neighbour solves the internal
/// (material-side) branch instead of the external common-tangent one.
pub struct FlippedNormalSurface<'a> {
    inner: &'a dyn ParametricSurface,
}

impl<'a> FlippedNormalSurface<'a> {
    /// Wrap a surface so its normal is negated.
    #[must_use]
    pub const fn new(inner: &'a dyn ParametricSurface) -> Self {
        Self { inner }
    }
}

impl ParametricSurface for FlippedNormalSurface<'_> {
    fn evaluate(&self, u: f64, v: f64) -> Point3 {
        self.inner.evaluate(u, v)
    }

    fn normal(&self, u: f64, v: f64) -> Vec3 {
        -self.inner.normal(u, v)
    }

    fn project_point(&self, point: Point3) -> (f64, f64) {
        self.inner.project_point(point)
    }

    fn partial_u(&self, u: f64, v: f64) -> Vec3 {
        self.inner.partial_u(u, v)
    }

    fn partial_v(&self, u: f64, v: f64) -> Vec3 {
        self.inner.partial_v(u, v)
    }
}

/// Extract a `&dyn ParametricSurface` from a `FaceSurface`, or build a
/// `PlaneAdapter` for plane faces.
///
/// Returns `Ok(adapter)` for planes and `Err(face_id)` for unsupported types.
/// For analytic and NURBS surfaces that already implement `ParametricSurface`,
/// the reference is extracted directly and the adapter is unused.
///
/// # Usage pattern
///
/// ```ignore
/// let mut adapter = None;
/// let surf: &dyn ParametricSurface = surface_ref_or_adapter(&face_surface, &mut adapter);
/// ```
#[must_use]
pub fn surface_ref_or_adapter<'a>(
    surface: &'a FaceSurface,
    adapter_slot: &'a mut Option<PlaneAdapter>,
) -> &'a dyn ParametricSurface {
    // For Plane faces, we need to populate the adapter_slot first,
    // then return a reference to it. For all other variants, we can
    // return a reference directly to the surface inside FaceSurface.
    if let FaceSurface::Plane { normal, d } = surface {
        let adapter = adapter_slot.insert(PlaneAdapter::from_normal_and_d(*normal, *d));
        return adapter as &dyn ParametricSurface;
    }
    match surface {
        FaceSurface::Plane { .. } => {
            // Already handled above; this arm is unreachable.
            adapter_slot.insert(PlaneAdapter::from_normal_and_d(
                Vec3::new(0.0, 0.0, 1.0),
                0.0,
            )) as &dyn ParametricSurface
        }
        FaceSurface::Cylinder(c) => c as &dyn ParametricSurface,
        FaceSurface::Cone(c) => c as &dyn ParametricSurface,
        FaceSurface::Sphere(s) => s as &dyn ParametricSurface,
        FaceSurface::Torus(t) => t as &dyn ParametricSurface,
        FaceSurface::Nurbs(n) => n as &dyn ParametricSurface,
    }
}
