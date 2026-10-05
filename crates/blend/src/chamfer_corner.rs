//! Corner patches for chamfers — the flat triangles that close a chamfered vertex.
//!
//! A chamfered vertex where three or more chamfered edges meet is bounded by
//! nothing. Each bevel is built over the whole of its edge, so at the vertex
//! the bevels run into one another instead of terminating, and the corner is
//! left open: a cube chamfered on all twelve edges comes back as 6 side faces
//! and 12 bevels — 18 quads, no triangles — instead of the 26 faces a closed
//! result needs.
//!
//! Two things are missing and they have to land together:
//!
//! 1. **The corner patch itself.** Three chamfered edges meeting at a vertex
//!    cut it off along a plane. The patch is the triangle through the three
//!    contact points, one per face at the vertex: the point where the two
//!    contact lines that two of the edges leave on that face cross. Three
//!    faces give three such points, and they are the triangle's corners.
//! 2. **A setback on each bevel.** The patch only closes the shell if the
//!    bevels stop where the patch starts. Running a bevel to the vertex makes
//!    it overlap the patch instead of meeting it.
//!
//! The setback falls out of (1): the patch's corners sit at a definite
//! distance along each edge from the vertex, and that distance is how far the
//! edge's bevel has to be held back. So this module computes the corners first
//! and reads the setbacks off them, rather than deriving the two separately.
//!
//! Everything here is exact for the case `chamfer` supports — planar faces and
//! straight contact lines — and conservative otherwise: a vertex whose corners
//! cannot be resolved to exactly three distinct points gets no patch, and an
//! edge whose setback cannot be resolved keeps its full length. That leaves the
//! result as it is today rather than guessing at a shape.

use std::collections::HashMap;

use brepkit_math::vec::{Point3, Vec3};
use brepkit_topology::Topology;
use brepkit_topology::edge::{Edge, EdgeCurve, EdgeId};
use brepkit_topology::face::{Face, FaceId, FaceSurface};
use brepkit_topology::vertex::{Vertex, VertexId};
use brepkit_topology::wire::{OrientedEdge, Wire};

use crate::BlendError;
use crate::stripe::Stripe;

/// Two points this close are the same vertex.
///
/// Matches the tolerance `sew::weld_faces` merges on, so a patch corner
/// produced here welds against the bevel end that should share it.
const MERGE_TOL: f64 = 1e-6;

/// A setback never takes more than half of a stripe: beyond that the corner
/// has swallowed the edge, and leaving the bevel whole is safer.
const MAX_SETBACK_FRACTION: f64 = 0.5;

/// One chamfered edge, with everything needed to rebuild its stripe later.
pub struct ChamferStripe {
    /// The edge being chamfered.
    pub edge: EdgeId,
    /// The stripe computed over the full length of that edge.
    pub stripe: Stripe,
    /// Surface of [`Stripe::face1`].
    pub surf1: FaceSurface,
    /// Surface of [`Stripe::face2`].
    pub surf2: FaceSurface,
    /// Chamfer distance measured on `face1`.
    pub d1: f64,
    /// Chamfer distance measured on `face2`.
    pub d2: f64,
}

/// One chamfered vertex and the triangle that cuts it off.
#[derive(Clone, Debug)]
pub struct CornerPatch {
    /// The vertex being cut off.
    pub vertex: VertexId,
    /// The contact points the patch passes through, unordered.
    pub points: Vec<Point3>,
    /// Every face a chamfered edge touches at this vertex.
    pub faces: Vec<FaceId>,
}

/// Endpoints of the two contact lines of a stripe.
fn contacts(stripe: &Stripe) -> ((Point3, Point3), (Point3, Point3)) {
    let (u0, u1) = stripe.contact1.domain();
    let (v0, v1) = stripe.contact2.domain();
    (
        (stripe.contact1.evaluate(u0), stripe.contact1.evaluate(u1)),
        (stripe.contact2.evaluate(v0), stripe.contact2.evaluate(v1)),
    )
}

/// The contact line `entry` leaves on `face`, if it touches that face.
fn contact_on(entry: &ChamferStripe, face: FaceId) -> Option<(Point3, Point3)> {
    let (a, b) = contacts(&entry.stripe);
    if entry.stripe.face1 == face {
        Some(a)
    } else if entry.stripe.face2 == face {
        Some(b)
    } else {
        None
    }
}

/// Where two segments cross, or `None` when they are parallel or the crossing
/// lies outside either segment.
///
/// Solves `a0 + u·s == b0 + v·t` for the two parameters by projecting onto `u`
/// and `v`; a crossing is returned only when both land in `[0, 1]`.
fn segment_crossing(a: (Point3, Point3), b: (Point3, Point3)) -> Option<Point3> {
    let u = a.1 - a.0;
    let v = b.1 - b.0;
    let w = a.0 - b.0;

    let uu = u.dot(u);
    let vv = v.dot(v);
    let uv = u.dot(v);
    let denom = uu * vv - uv * uv;
    // Parallel, or one of the segments has no length.
    if denom <= f64::EPSILON * uu.max(vv).max(1.0) {
        return None;
    }

    let uw = u.dot(w);
    let vw = v.dot(w);
    let s = (uv * vw - vv * uw) / denom;
    let t = (uu * vw - uv * uw) / denom;

    if !(0.0..=1.0).contains(&s) || !(0.0..=1.0).contains(&t) {
        return None;
    }
    Some(a.0 + u * s)
}

/// Push `p` onto `list` unless a point already there is the same one.
fn merge_point(list: &mut Vec<Point3>, p: Point3) {
    if list.iter().any(|q| (*q - p).length() <= MERGE_TOL) {
        return;
    }
    list.push(p);
}

/// The patch at every vertex where three or more chamfered edges meet.
///
/// A vertex with fewer than three gets no patch: two bevels meeting there butt
/// against each other's end cross-section and nothing has to hand anything
/// over.
#[must_use]
pub fn corner_patches(topo: &Topology, entries: &[ChamferStripe]) -> Vec<CornerPatch> {
    let mut at_vertex: HashMap<VertexId, Vec<usize>> = HashMap::new();
    for (i, entry) in entries.iter().enumerate() {
        let Ok(edge) = topo.edge(entry.edge) else {
            continue;
        };
        at_vertex.entry(edge.start()).or_default().push(i);
        at_vertex.entry(edge.end()).or_default().push(i);
    }

    let mut out = Vec::new();

    for (vertex, ids) in at_vertex {
        if ids.len() < 3 {
            continue;
        }

        // Every face a chamfered edge touches here. Restricting to chamfered
        // edges — not the solid's topology — is what makes the patch agree with
        // the bevels that have to meet it.
        let mut faces: Vec<FaceId> = Vec::new();
        for &i in &ids {
            for face in [entries[i].stripe.face1, entries[i].stripe.face2] {
                if !faces.contains(&face) {
                    faces.push(face);
                }
            }
        }

        // On each face, the contact lines two chamfered edges leave there cross
        // at one corner of the patch.
        let mut points: Vec<Point3> = Vec::new();
        for &face in &faces {
            let mut lines: Vec<(Point3, Point3)> = Vec::new();
            for &i in &ids {
                if let Some(line) = contact_on(&entries[i], face) {
                    lines.push(line);
                }
            }
            for a in 0..lines.len() {
                for b in (a + 1)..lines.len() {
                    if let Some(p) = segment_crossing(lines[a], lines[b]) {
                        merge_point(&mut points, p);
                    }
                }
            }
        }

        if !points.is_empty() {
            out.push(CornerPatch {
                vertex,
                points,
                faces,
            });
        }
    }

    out
}

/// How far each stripe must be held back from each of its ends, in the spine's
/// arc-length units.
///
/// The patch corner that lies furthest along an edge is where that edge's bevel
/// has to stop. A vertex with no resolved patch, or a setback that would eat
/// more than half the edge, leaves the bevel whole.
#[must_use]
pub fn setbacks(
    topo: &Topology,
    entries: &[ChamferStripe],
    patches: &[CornerPatch],
) -> Vec<(f64, f64)> {
    let by_vertex: HashMap<VertexId, &CornerPatch> =
        patches.iter().map(|p| (p.vertex, p)).collect();

    entries
        .iter()
        .map(|entry| {
            let Some(edge) = topo.edge(entry.edge).ok() else {
                return (0.0, 0.0);
            };
            let (start, end) = (edge.start(), edge.end());
            let (Some(pa), Some(pb)) = (
                topo.vertex(start)
                    .ok()
                    .map(brepkit_topology::vertex::Vertex::point),
                topo.vertex(end)
                    .ok()
                    .map(brepkit_topology::vertex::Vertex::point),
            ) else {
                return (0.0, 0.0);
            };

            let Some(axis) = (pb - pa).normalize().ok() else {
                return (0.0, 0.0);
            };
            let budget = MAX_SETBACK_FRACTION * entry.stripe.spine.length();

            // Furthest patch corner measured along the edge, going away from
            // `from`. `dir` is the outbound direction at that end, so the same
            // expression serves both ends.
            let along = |vertex: VertexId, from: Point3, dir: Vec3| -> f64 {
                let reach = by_vertex.get(&vertex).map_or(0.0, |patch| {
                    patch
                        .points
                        .iter()
                        .map(|&p| (p - from).dot(dir))
                        .fold(0.0, f64::max)
                });
                if reach > 0.0 && reach < budget {
                    reach
                } else {
                    0.0
                }
            };

            (along(start, pa, axis), along(end, pb, -axis))
        })
        .collect()
}

/// Inward unit normal of a planar face: pointing into the material.
///
/// A face records whether the surface normal was flipped for its own traversal,
/// so the geometric normal is the *outward* one unless reversed.
fn inward_normal(topo: &Topology, face: FaceId) -> Option<Vec3> {
    let f = topo.face(face).ok()?;
    let FaceSurface::Plane { normal, .. } = f.surface() else {
        return None;
    };
    let inward = if f.is_reversed() { *normal } else { -*normal };
    inward.normalize().ok()
}

/// Build the corner patches as faces.
///
/// A patch whose corners are not exactly three distinct points, or whose
/// outward direction cannot be resolved from the faces it meets, is skipped:
/// an open corner degrades more gracefully than a wrongly placed triangle.
///
/// # Errors
///
/// Returns [`BlendError`] if wire or face construction fails.
pub fn build_patches(
    topo: &mut Topology,
    patches: &[CornerPatch],
) -> Result<Vec<FaceId>, BlendError> {
    let mut out = Vec::new();

    for patch in patches {
        if patch.points.len() != 3 {
            continue;
        }

        // Outward is away from the material, which at a corner is away from the
        // sum of the faces' inward normals.
        let mut inward = Vec3::new(0.0, 0.0, 0.0);
        let mut resolved = true;
        for &face in &patch.faces {
            match inward_normal(topo, face) {
                Some(n) => inward += n,
                None => resolved = false,
            }
        }
        if !resolved {
            continue;
        }
        let Some(outward) = inward.normalize().ok().map(|n| -n) else {
            continue;
        };

        let mut tri = [patch.points[0], patch.points[1], patch.points[2]];
        let Some(mut normal) = (tri[1] - tri[0]).cross(tri[2] - tri[0]).normalize().ok() else {
            continue;
        };
        if normal.dot(outward) < 0.0 {
            tri.swap(1, 2);
            let Some(flipped) = (tri[1] - tri[0]).cross(tri[2] - tri[0]).normalize().ok() else {
                continue;
            };
            normal = flipped;
        }

        let origin = Vec3::new(tri[0].x(), tri[0].y(), tri[0].z());
        let surface = FaceSurface::Plane {
            normal,
            d: normal.dot(origin),
        };

        let verts: Vec<VertexId> = tri
            .iter()
            .map(|&p| topo.add_vertex(Vertex::new(p, MERGE_TOL)))
            .collect();

        let mut oes = Vec::with_capacity(3);
        for i in 0..3 {
            let a = verts[i];
            let b = verts[(i + 1) % 3];
            oes.push(OrientedEdge::new(
                topo.add_edge(Edge::new(a, b, EdgeCurve::Line)),
                true,
            ));
        }

        let wire = topo.add_wire(Wire::new(oes, true)?);
        out.push(topo.add_face(Face::new(wire, Vec::new(), surface)));
    }

    Ok(out)
}
