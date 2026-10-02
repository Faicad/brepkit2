//! Corner setbacks: how much of each stripe end the corner patch takes over.
//!
//! A stripe built over a whole edge reaches the vertex itself. Where three or
//! more filleted edges meet, that is too far: the ball rolling along the edge
//! pokes out through every face it is not tangent to, so the stripe overlaps
//! both its neighbours and the spherical patch that covers the vertex — adding
//! material back that the fillet was supposed to remove.
//!
//! The stripe has to stop where the ball finally becomes tangent to those faces
//! as well, because at that point the rolling ball stops sweeping a cylinder
//! and starts sweeping the corner sphere. See the derivation of `setback_at`
//! for the closed form this module uses.

use std::collections::HashMap;

use brepkit_math::vec::Vec3;
use brepkit_topology::Topology;
use brepkit_topology::adjacency::AdjacencyIndex;
use brepkit_topology::edge::EdgeId;
use brepkit_topology::face::{FaceId, FaceSurface};
use brepkit_topology::vertex::VertexId;

/// Smallest `1 + n1·n2` we trust. Approaching zero means the two faces of the
/// fillet edge have become coplanar, so there is no unique ball centre.
const COPLANAR_TOL: f64 = 1e-6;

/// Below this the stripe approaches the third face too slowly for the
/// setback to be meaningful (it is also the sign test for "the ball is leaving
/// that face behind rather than running into it").
const APPROACH_TOL: f64 = 1e-9;

/// A setback never takes more than half of the stripe. Anything beyond that
/// means the corner swallowed the edge, and leaving the stripe whole is safer.
const MAX_SETBACK_FRACTION: f64 = 0.5;

/// Per-end setback of every requested stripe, in the spine's arc-length units.
pub type Setbacks = HashMap<EdgeId, (f64, f64)>;

/// The faces touched by a set of edges, indexed by which vertex they meet at.
fn gather_corner_input(topo: &Topology, edges: &[EdgeId]) -> HashMap<VertexId, Vec<EdgeId>> {
    let mut at_vertex: HashMap<VertexId, Vec<EdgeId>> = HashMap::new();
    for &edge in edges {
        let Ok(e) = topo.edge(edge) else {
            continue;
        };
        at_vertex.entry(e.start()).or_default().push(edge);
        at_vertex.entry(e.end()).or_default().push(edge);
    }
    at_vertex
}

/// Inward unit normal of a planar face: pointing into the material.
///
/// A face records whether the surface normal was flipped for its own traversal,
/// so the geometric normal alone is the *outward* one unless reversed.
fn inward_normal(topo: &Topology, face: FaceId) -> Option<Vec3> {
    let f = topo.face(face).ok()?;
    let FaceSurface::Plane { normal, .. } = f.surface() else {
        return None;
    };
    let inward = if f.is_reversed() { *normal } else { -*normal };
    inward.normalize().ok()
}

/// Unit tangent of `edge` at one of its ends, pointing away from that end.
fn outbound_tangent(topo: &Topology, edge: EdgeId, at: VertexId) -> Option<Vec3> {
    let e = topo.edge(edge).ok()?;
    let a = topo.vertex(e.start()).ok()?.point();
    let b = topo.vertex(e.end()).ok()?.point();
    let dir = if e.start() == at { b - a } else { a - b };
    dir.normalize().ok()
}

/// How far `edge` must be held back from `vertex`, or `None` to leave it whole.
///
/// For an edge whose two faces have unit inward normals `n1`, `n2` meeting it
/// along the unit tangent `u`, the ball of radius `r` tangent to both faces
/// keeps its centre at `w = r (n1 + n2) / (1 + n1·n2)` off every point of the
/// edge: that solves `w·n1 = w·n2 = r` within `span(n1, n2)`, and `w` is
/// automatically perpendicular to `u` because `u` is perpendicular to both
/// normals.
///
/// A third face `n3` at the vertex is then at signed distance
/// `d(s) = (w·n3) + s (u·n3)` from the ball centre `s` along the edge. At
/// `s = 0` the ball is tangent to it, in front of it, or — for the corner we
/// care about — through it; the setback is where `d(s) = r`, i.e.
/// `(r - w·n3) / (u·n3)`. When several faces qualify, the nearest one wins.
fn setback_at(
    topo: &Topology,
    adjacency: &AdjacencyIndex,
    at_vertex: &HashMap<VertexId, Vec<EdgeId>>,
    vertex: VertexId,
    edge: EdgeId,
    radius: f64,
) -> Option<f64> {
    let incident = at_vertex.get(&vertex)?;
    // Below three filleted edges there is no corner patch to hand anything to:
    // two stripes meeting simply butt against each other's end cross-section.
    if incident.len() < 3 {
        return None;
    }

    // Every face the filleted edges touch at this vertex. Restricting to the
    // filleted edges is what makes this agree with `corner::classify_corner`,
    // which counts stripes, not the solid's topology.
    let mut faces: Vec<FaceId> = Vec::new();
    for &e in incident {
        for &f in adjacency.faces_for_edge(e) {
            if !faces.contains(&f) {
                faces.push(f);
            }
        }
    }

    let own = adjacency.faces_for_edge(edge);
    if own.len() != 2 || faces.len() < 3 {
        return None;
    }
    let n1 = inward_normal(topo, own[0])?;
    let n2 = inward_normal(topo, own[1])?;
    let u = outbound_tangent(topo, edge, vertex)?;

    let cross_term = n1.dot(n2);
    if (1.0 + cross_term) < COPLANAR_TOL {
        return None;
    }
    let w = (n1 + n2) * (radius / (1.0 + cross_term));

    let mut best: Option<f64> = None;
    for face in faces {
        if face == own[0] || face == own[1] {
            continue;
        }
        let n3 = inward_normal(topo, face)?;
        let approach = u.dot(n3);
        // Moving away from the vertex has to increase the clearance; otherwise
        // this face is behind the stripe and imposes nothing.
        if approach <= APPROACH_TOL {
            continue;
        }
        let clearance = w.dot(n3);
        // Already at arm's length at the vertex => no corner to resolve here.
        if clearance >= radius {
            continue;
        }
        let s = (radius - clearance) / approach;
        if s > 0.0 {
            best = Some(best.map_or(s, |b: f64| b.min(s)));
        }
    }
    best
}

/// Setbacks for every requested stripe.
///
/// `targets` carries `(edge, radius at the start end, radius at the end end)`;
/// variable-radius laws are read at the end they apply to. Returns one entry
/// per target edge, `(start, end)`, either being `0.0` when that end needs no
/// setback.
#[must_use]
pub fn compute_setbacks(
    topo: &Topology,
    adjacency: &AdjacencyIndex,
    targets: &[(EdgeId, f64, f64)],
) -> Setbacks {
    let edges: Vec<EdgeId> = targets.iter().map(|&(e, _, _)| e).collect();
    let at_vertex = gather_corner_input(topo, &edges);

    let mut out = Setbacks::new();
    for &(edge, r_start, r_end) in targets {
        let Ok(e) = topo.edge(edge) else {
            continue;
        };
        let (va, vb) = (e.start(), e.end());
        let length = {
            let a = topo
                .vertex(va)
                .ok()
                .map(brepkit_topology::vertex::Vertex::point);
            let b = topo
                .vertex(vb)
                .ok()
                .map(brepkit_topology::vertex::Vertex::point);
            match (a, b) {
                (Some(a), Some(b)) => (b - a).length(),
                _ => 0.0,
            }
        };
        let budget = MAX_SETBACK_FRACTION * length;

        let start = setback_at(topo, adjacency, &at_vertex, va, edge, r_start)
            .filter(|s| *s > 0.0 && *s < budget)
            .unwrap_or(0.0);
        let end = setback_at(topo, adjacency, &at_vertex, vb, edge, r_end)
            .filter(|s| *s > 0.0 && *s < budget)
            .unwrap_or(0.0);

        out.insert(edge, (start, end));
    }
    out
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use brepkit_topology::explorer::solid_edges;
    use brepkit_topology::test_utils::make_unit_cube_manifold;

    /// Three mutually orthogonal faces give each stripe away exactly one
    /// radius. The unit fixture keeps `radius` well inside the setback budget,
    /// which is a separate guard.
    #[test]
    fn cube_edges_are_set_back_by_one_radius() {
        let mut topo = Topology::new();
        let solid = make_unit_cube_manifold(&mut topo);
        let edges = solid_edges(&topo, solid).unwrap();
        let adjacency = brepkit_topology::adjacency::AdjacencyIndex::build(&topo, solid).unwrap();

        let radius = 0.2;
        let targets: Vec<_> = edges.iter().map(|&e| (e, radius, radius)).collect();

        let setbacks = compute_setbacks(&topo, &adjacency, &targets);
        assert_eq!(setbacks.len(), 12);
        for (edge, (start, end)) in setbacks {
            let e = topo.edge(edge).unwrap();
            let a = topo.vertex(e.start()).unwrap().point();
            let b = topo.vertex(e.end()).unwrap().point();
            assert!(
                ((b - a).length() - 1.0).abs() < 1e-9,
                "the fixture is a unit cube"
            );
            assert!(
                (start - radius).abs() < 1e-9,
                "edge {edge:?}: start setback {start}, expected one radius"
            );
            assert!(
                (end - radius).abs() < 1e-9,
                "edge {edge:?}: end setback {end}, expected one radius"
            );
        }
    }

    /// Two stripes meeting at a vertex butt against each other's end
    /// cross-section — nothing hands over to a corner patch, so nothing gives
    /// up length.
    #[test]
    fn two_stripes_meeting_directly_need_no_setback() {
        let mut topo = Topology::new();
        let solid = make_unit_cube_manifold(&mut topo);
        let edges = solid_edges(&topo, solid).unwrap();
        let adjacency = brepkit_topology::adjacency::AdjacencyIndex::build(&topo, solid).unwrap();

        // Two adjacent edges of one face share a vertex.
        let first = edges[0];
        let shared = topo.edge(first).unwrap().end();
        let partner = *edges
            .iter()
            .skip(1)
            .find(|&&e| {
                topo.edge(e)
                    .is_ok_and(|e| e.start() == shared || e.end() == shared)
            })
            .expect("a cube edge shares each end with two others");

        let targets = vec![(first, 0.2, 0.2), (partner, 0.2, 0.2)];
        let setbacks = compute_setbacks(&topo, &adjacency, &targets);
        assert_eq!(setbacks.len(), 2);
        for (&edge, &(start, end)) in &setbacks {
            assert_eq!(
                (start, end),
                (0.0, 0.0),
                "edge {edge:?} should keep its full length"
            );
        }
    }
}
