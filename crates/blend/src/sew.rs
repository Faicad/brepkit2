//! Weld duplicate topological entities into one closed shell.
//!
//! A blend result is assembled from parts that were each built in isolation:
//! the trimmer mints a contact edge on the trimmed neighbour, the stripe
//! builder mints its own copy for the blend flank, the corner solver mints a
//! third for the patch boundary. Geometrically they are the *same* curve — the
//! same endpoints, the same arc — but topologically they are three different
//! [`EdgeId`]s. Each is then referenced by exactly one face, so the shell has
//! free edges everywhere: `validate_shell_closed` rejects it, and every
//! consumer that walks adjacency (offsetting, shelling, the wasm `is_valid`
//! gate) has nothing to traverse.
//!
//! This module fixes that after assembly, in one place, without any of the
//! producers having to know about each other:
//!
//! 1. **Vertices** that describe the same point collapse to one
//!    [`VertexId`] — a spatial hash keeps this linear in the vertex count
//!    instead of quadratic.
//! 2. **Edges** whose welded endpoint pair matches collapse to one
//!    [`EdgeId`]. The surviving edge keeps the curve of whichever copy was
//!    seen first, with its endpoints in that curve's own direction, so the
//!    parameterisation still runs start-to-end along the geometry.
//! 3. **Wires and faces** are rebuilt over the welded edges, each oriented
//!    edge flipped as needed to preserve the traversal direction it had.
//!
//! Step 2 keys on the endpoint pair alone. That is sound here because no two
//! distinct boundary curves of a blend result share both endpoints: the three
//! arcs of a spherical corner patch run between three *different* pairs of
//! tangent points, and a stripe's four sides are four distinct pairs.

use std::collections::HashMap;

use brepkit_math::vec::Point3;
use brepkit_topology::Topology;
use brepkit_topology::edge::{Edge, EdgeId};
use brepkit_topology::face::{Face, FaceId};
use brepkit_topology::vertex::VertexId;
use brepkit_topology::wire::{OrientedEdge, Wire, WireId};

use crate::BlendError;

/// Two points closer than this are the same vertex.
///
/// The blend builders place a tangent point by solving for it independently in
/// each producer, so the copies agree to solver tolerance but not bit-for-bit.
const WELD_TOL: f64 = 1e-6;

/// A face wire needs at least this many sides to enclose anything.
const MIN_WIRE_SIDES: usize = 3;

/// Collapses vertices that describe the same point.
///
/// Buckets by a grid of side [`WELD_TOL`] and probes the 27 neighbouring
/// cells, so two points that straddle a cell boundary still meet.
struct VertexWelder {
    tol: f64,
    cells: HashMap<(i64, i64, i64), Vec<VertexId>>,
}

impl VertexWelder {
    fn new(tol: f64) -> Self {
        Self {
            tol,
            cells: HashMap::new(),
        }
    }

    fn cell(p: Point3, tol: f64) -> (i64, i64, i64) {
        #[allow(clippy::cast_precision_loss)]
        let q = |v: f64| (v / tol).round() as i64;
        (q(p.x()), q(p.y()), q(p.z()))
    }

    /// Return the representative of `v`, registering `v` itself if it is new.
    fn weld(&mut self, topo: &Topology, v: VertexId) -> Result<VertexId, BlendError> {
        let p = topo.vertex(v)?.point();
        let (i, j, k) = Self::cell(p, self.tol);

        let mut candidates: Vec<VertexId> = Vec::new();
        for di in -1..=1_i64 {
            for dj in -1..=1_i64 {
                for dk in -1..=1_i64 {
                    if let Some(list) = self.cells.get(&(i + di, j + dj, k + dk)) {
                        candidates.extend_from_slice(list);
                    }
                }
            }
        }
        for cand in candidates {
            let q = topo.vertex(cand)?.point();
            if (q - p).length() <= self.tol {
                return Ok(cand);
            }
        }

        self.cells.entry((i, j, k)).or_default().push(v);
        Ok(v)
    }
}

/// Collapses edges that span the same pair of welded vertices.
///
/// Maps an unordered endpoint pair to the surviving edge and that edge's own
/// `(start, end)`, so a caller can tell whether its traversal runs with or
/// against the edge.
#[derive(Default)]
struct EdgeWelder {
    by_ends: HashMap<(usize, usize), (EdgeId, VertexId, VertexId)>,
}

impl EdgeWelder {
    /// Weld one oriented-edge occurrence onto the shell's shared edge set.
    ///
    /// `vrep` maps every original vertex to its welded representative. Returns
    /// the surviving edge and whether this occurrence traverses it forward, or
    /// `None` when the occurrence collapsed to a point and must be dropped.
    fn weld(
        &mut self,
        topo: &mut Topology,
        oe: &OrientedEdge,
        vrep: &HashMap<usize, VertexId>,
    ) -> Result<Option<(EdgeId, bool)>, BlendError> {
        let eid = oe.edge();
        let edge = topo.edge(eid)?;

        // Where this occurrence runs, in welded vertices.
        let from = vrep[&oe.oriented_start(edge).index()];
        let to = vrep[&oe.oriented_end(edge).index()];
        if from == to {
            // Both endpoints welded together: a zero-length side, which a wire
            // cannot hold.
            return Ok(Option::None);
        }

        // Where the edge itself runs, in welded vertices. The curve is
        // parameterised from `start` to `end`, so the representative has to
        // keep that order.
        let own_start = vrep[&edge.start().index()];
        let own_end = vrep[&edge.end().index()];
        if own_start == own_end {
            return Ok(Option::None);
        }

        let key = if own_start.index() <= own_end.index() {
            (own_start.index(), own_end.index())
        } else {
            (own_end.index(), own_start.index())
        };

        if let Some(&(rep, rep_start, rep_end)) = self.by_ends.get(&key) {
            if from == rep_start && to == rep_end {
                return Ok(Some((rep, true)));
            }
            if from == rep_end && to == rep_start {
                return Ok(Some((rep, false)));
            }
            // Same endpoints, but this traversal lines up with neither
            // direction of the surviving edge. Drop it rather than emit a wire
            // that cannot close.
            return Ok(Option::None);
        }

        // First copy of this curve: make it the representative, keeping its
        // curve (which still runs `own_start` -> `own_end`).
        let rep = if own_start == edge.start() && own_end == edge.end() {
            eid
        } else {
            let curve = edge.curve().clone();
            topo.add_edge(Edge::new(own_start, own_end, curve))
        };
        self.by_ends.insert(key, (rep, own_start, own_end));

        let forward = from == own_start && to == own_end;
        Ok(Some((rep, forward)))
    }
}

/// All wires of a face, outer first.
fn face_wires(topo: &Topology, face: FaceId) -> Result<Vec<WireId>, BlendError> {
    let f = topo.face(face)?;
    Ok(std::iter::once(f.outer_wire())
        .chain(f.inner_wires().iter().copied())
        .collect())
}

/// Whether the oriented edges chain end-to-end into a closed loop.
///
/// `Wire::new` only rejects an empty list, so closure is checked here: a wire
/// that no longer links up means welding broke a face, and that face is passed
/// through untouched instead.
fn is_closed(topo: &Topology, oes: &[OrientedEdge]) -> Result<bool, BlendError> {
    if oes.len() < MIN_WIRE_SIDES {
        return Ok(false);
    }
    for pair in oes.windows(2) {
        let a = topo.edge(pair[0].edge())?;
        let b = topo.edge(pair[1].edge())?;
        if pair[0].oriented_end(a) != pair[1].oriented_start(b) {
            return Ok(false);
        }
    }
    let last = topo.edge(oes[oes.len() - 1].edge())?;
    let first = topo.edge(oes[0].edge())?;
    Ok(oes[oes.len() - 1].oriented_end(last) == oes[0].oriented_start(first))
}

/// Weld `faces` so that every edge is one entity shared by its neighbours.
///
/// Returns the replacement faces, in the same order as `faces`. A face whose
/// wire cannot be rebuilt — fewer than three surviving sides, or a loop that
/// no longer closes — is passed through unchanged rather than dropped: a
/// slightly open shell degrades more gracefully than one with a missing face.
///
/// # Errors
///
/// Returns [`BlendError`] if topology lookups fail.
pub fn weld_faces(topo: &mut Topology, faces: &[FaceId]) -> Result<Vec<FaceId>, BlendError> {
    // Pass 1 — collapse duplicate vertices.
    let mut vrep: HashMap<usize, VertexId> = HashMap::new();
    let mut vw = VertexWelder::new(WELD_TOL);
    for &fid in faces {
        for wid in face_wires(topo, fid)? {
            for oe in topo.wire(wid)?.edges() {
                let e = topo.edge(oe.edge())?;
                for v in [e.start(), e.end()] {
                    let idx = v.index();
                    if let std::collections::hash_map::Entry::Vacant(slot) = vrep.entry(idx) {
                        let rep = vw.weld(topo, v)?;
                        slot.insert(rep);
                    }
                }
            }
        }
    }

    // Pass 2 — collapse duplicate edges and rebuild every face over them.
    let mut ew = EdgeWelder::default();
    let mut out = Vec::with_capacity(faces.len());

    for &fid in faces {
        let (surface, reversed) = {
            let f = topo.face(fid)?;
            (f.surface().clone(), f.is_reversed())
        };

        let mut rebuilt: Vec<WireId> = Vec::new();
        let mut usable = true;

        for wid in face_wires(topo, fid)? {
            let oes = topo.wire(wid)?.edges().to_vec();
            let mut new_oes: Vec<OrientedEdge> = Vec::with_capacity(oes.len());

            for oe in &oes {
                let Some((rep, forward)) = ew.weld(topo, oe, &vrep)? else {
                    continue;
                };
                // Welding can turn two consecutive sides into the same edge —
                // the two halves of a split curve that turned out to be one
                // curve. Collapse the pair; a wire that repeats a side cannot
                // close cleanly.
                if new_oes.last().is_some_and(|last| last.edge() == rep) {
                    continue;
                }
                new_oes.push(OrientedEdge::new(rep, forward));
            }

            if !is_closed(topo, &new_oes)? {
                usable = false;
                break;
            }
            // `is_closed` guarantees at least MIN_WIRE_SIDES sides, and
            // `Wire::new` only rejects an empty list.
            rebuilt.push(topo.add_wire(Wire::new(new_oes, true)?));
        }

        if !usable || rebuilt.is_empty() {
            out.push(fid);
            continue;
        }

        let mut wires = rebuilt.into_iter();
        // `rebuilt` is non-empty here, and `is_closed` guarantees each of its
        // wires has at least MIN_WIRE_SIDES sides.
        let Some(outer) = wires.next() else {
            out.push(fid);
            continue;
        };
        let inners: Vec<WireId> = wires.collect();
        let new_face = if reversed {
            Face::new_reversed(outer, inners, surface)
        } else {
            Face::new(outer, inners, surface)
        };
        out.push(topo.add_face(new_face));
    }

    Ok(out)
}
