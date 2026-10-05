//! Chamfer builder: orchestrates the full chamfer pipeline.
//!
//! Supports symmetric, asymmetric, and distance-angle chamfer modes on
//! planar face pairs (v1). Reuses the analytic fast path and face trimming
//! infrastructure from the fillet pipeline.

use std::collections::HashSet;

use brepkit_topology::Topology;
use brepkit_topology::edge::EdgeId;
use brepkit_topology::face::FaceId;
use brepkit_topology::shell::Shell;
use brepkit_topology::solid::{Solid, SolidId};

use crate::analytic;
use crate::builder_utils::sample_nurbs_endpoints;
use crate::chamfer_corner::{self, ChamferStripe};
use crate::sew;
use crate::spine::Spine;
use crate::stripe::Stripe;
use crate::trimmer::{self, TrimKeep};
use crate::{BlendError, BlendResult};

/// Internal representation of a chamfer edge set with its distance parameters.
enum ChamferEdgeSet {
    /// Two explicit distances (d1 on face 1, d2 on face 2).
    TwoDistance {
        /// Edges to chamfer.
        edges: Vec<EdgeId>,
        /// Distance on face 1.
        d1: f64,
        /// Distance on face 2.
        d2: f64,
    },
    /// Distance on face 1 plus angle from face 1 toward face 2.
    DistanceAngle {
        /// Edges to chamfer.
        edges: Vec<EdgeId>,
        /// Distance on face 1.
        distance: f64,
        /// Angle from face 1 (radians).
        angle: f64,
    },
}

/// Builder for chamfer (bevel) operations on solid edges.
///
/// Collects edge sets with their distance parameters, then computes and
/// assembles the chamfered solid in a single `build()` call.
pub struct ChamferBuilder<'a> {
    topo: &'a mut Topology,
    solid: SolidId,
    edge_sets: Vec<ChamferEdgeSet>,
}

impl<'a> ChamferBuilder<'a> {
    /// Create a new chamfer builder for the given solid.
    #[must_use]
    pub fn new(topo: &'a mut Topology, solid: SolidId) -> Self {
        Self {
            topo,
            solid,
            edge_sets: Vec::new(),
        }
    }

    /// Add edges with symmetric chamfer distance (d1 = d2 = d).
    ///
    /// Returns `&mut Self` for method chaining.
    pub fn add_edges_symmetric(&mut self, edges: &[EdgeId], d: f64) -> &mut Self {
        self.edge_sets.push(ChamferEdgeSet::TwoDistance {
            edges: edges.to_vec(),
            d1: d,
            d2: d,
        });
        self
    }

    /// Add edges with asymmetric chamfer distances.
    ///
    /// `d1` is the distance on face 1, `d2` on face 2.
    ///
    /// Returns `&mut Self` for method chaining.
    pub fn add_edges_asymmetric(&mut self, edges: &[EdgeId], d1: f64, d2: f64) -> &mut Self {
        self.edge_sets.push(ChamferEdgeSet::TwoDistance {
            edges: edges.to_vec(),
            d1,
            d2,
        });
        self
    }

    /// Add edges with distance-angle chamfer.
    ///
    /// `distance` is measured on face 1; `angle` (radians) determines
    /// the depth on face 2 as `distance * tan(angle)`.
    ///
    /// Returns `&mut Self` for method chaining.
    pub fn add_edges_distance_angle(
        &mut self,
        edges: &[EdgeId],
        distance: f64,
        angle: f64,
    ) -> &mut Self {
        self.edge_sets.push(ChamferEdgeSet::DistanceAngle {
            edges: edges.to_vec(),
            distance,
            angle,
        });
        self
    }

    /// Compute and build the chamfered solid.
    ///
    /// # Algorithm
    ///
    /// 1. Build adjacency index for the solid.
    /// 2. For each target edge, find the two adjacent faces.
    /// 3. Build single-edge spines (no chain propagation in v1).
    /// 4. Compute stripes via analytic fast path or record failure.
    /// 5. Trim adjacent faces along contact curves.
    /// 6. Assemble new solid from trimmed faces, blend faces, and untouched
    ///    original faces.
    ///
    /// # Errors
    ///
    /// Returns [`BlendError`] if no edges were specified, or if topology
    /// lookups fail. Individual edge failures are recorded in
    /// [`BlendResult::failed`] rather than aborting the whole operation.
    #[allow(clippy::too_many_lines)]
    pub fn build(self) -> Result<BlendResult, BlendError> {
        let all_edges: Vec<(EdgeId, f64, f64)> = self
            .edge_sets
            .into_iter()
            .flat_map(|set| {
                let (edges, d1, d2) = match set {
                    ChamferEdgeSet::TwoDistance { edges, d1, d2 } => (edges, d1, d2),
                    ChamferEdgeSet::DistanceAngle {
                        edges,
                        distance,
                        angle,
                    } => {
                        let d2 = distance * angle.tan();
                        (edges, distance, d2)
                    }
                };
                edges.into_iter().map(move |eid| (eid, d1, d2))
            })
            .collect();

        if all_edges.is_empty() {
            return Err(BlendError::Topology(
                brepkit_topology::TopologyError::Empty {
                    entity: "chamfer edge set",
                },
            ));
        }

        let topo = self.topo;

        let adjacency = topo.build_adjacency(self.solid)?;

        let shell_id = topo.solid(self.solid)?.outer_shell();
        let original_faces: Vec<FaceId> = topo.shell(shell_id)?.faces().to_vec();

        let mut touched_faces: HashSet<FaceId> = HashSet::new();

        let mut succeeded: Vec<EdgeId> = Vec::new();
        let mut failed: Vec<(EdgeId, BlendError)> = Vec::new();
        let mut stripe_results: Vec<ChamferStripe> = Vec::new();

        for (edge_id, d1, d2) in &all_edges {
            let result = compute_chamfer_stripe(topo, &adjacency, *edge_id, *d1, *d2);
            match result {
                Ok(sr) => {
                    touched_faces.insert(sr.stripe.face1);
                    touched_faces.insert(sr.stripe.face2);
                    stripe_results.push(sr);
                    succeeded.push(*edge_id);
                }
                Err(e) => {
                    failed.push((*edge_id, e));
                }
            }
        }

        // If no stripes succeeded, return the original solid with all failures.
        if stripe_results.is_empty() {
            return Ok(BlendResult {
                solid: self.solid,
                succeeded: Vec::new(),
                failed,
                is_partial: false,
            });
        }

        // Where three or more chamfered edges meet, a bevel cannot run to the
        // vertex: the bevels overlap each other there and the corner is left
        // open. Each such vertex needs a flat patch through the three contact
        // points, and each bevel has to stop where that patch begins. Both come
        // out of the same computation — see `chamfer_corner`.
        //
        // All of it has to happen before the trimming loop below rewrites the
        // topology. The analytic path derives which way a contact line runs by
        // finding the spine edge in the face's own wires; the trimmer splits
        // that edge and propagates the split into every wire that uses it, so
        // afterwards the lookup misses and the contact direction falls back to
        // the bisector — measured: 10 of a cube's 12 bevels then come out
        // mirrored outside the solid.
        let patches = chamfer_corner::corner_patches(topo, &stripe_results);
        let setbacks = chamfer_corner::setbacks(topo, &stripe_results, &patches);
        let mut bevels: Vec<Stripe> = Vec::with_capacity(stripe_results.len());
        for (i, entry) in stripe_results.iter().enumerate() {
            let (start, end) = setbacks.get(i).copied().unwrap_or((0.0, 0.0));
            bevels.push(windowed_stripe(topo, entry, start, end)?);
        }

        let mut face_replacements: std::collections::HashMap<FaceId, FaceId> =
            std::collections::HashMap::new();

        let mut stripe_contact_edges: Vec<(
            Option<brepkit_topology::edge::EdgeId>,
            Option<brepkit_topology::edge::EdgeId>,
        )> = Vec::new();
        for sr in &stripe_results {
            let stripe = &sr.stripe;
            stripe_contact_edges.push((None, None));

            let contact1_pts = sample_nurbs_endpoints(&stripe.contact1);
            let contact2_pts = sample_nurbs_endpoints(&stripe.contact2);

            // Keep the side of each contact line AWAY from the chamfered edge:
            // the strip between the contact line and the old edge is what the
            // bevel replaces.
            //
            // The side must be resolved inside the trimmer. Its Left/Right
            // frame follows each face's own wire traversal, and the two faces
            // of one edge traverse that edge in *opposite* directions, so any
            // single side picked out here is correct for one face and
            // backwards for the other — which trims that face down to the
            // sliver that should have been removed and drops the bulk of it.
            // A plane-side test cannot tell them apart either: the bevel lies
            // on the inward side of both face normals, so it yields the same
            // constant for both.
            let spine_pt = stripe.spine.evaluate(topo, 0.0)?;
            let keep = TrimKeep::AwayFrom(spine_pt);

            let current_face1 = face_replacements
                .get(&stripe.face1)
                .copied()
                .unwrap_or(stripe.face1);
            let trim1 = trimmer::trim_face(
                topo,
                current_face1,
                &contact1_pts,
                &[(0.0, 0.0), (1.0, 0.0)],
                keep,
            );

            match trim1 {
                Ok(tr) if tr.trimmed_face != current_face1 => {
                    if let Some(slot) = stripe_contact_edges.last_mut() {
                        slot.0 = tr.contact_edge;
                    }
                    face_replacements.insert(stripe.face1, tr.trimmed_face);
                }
                Ok(_) => {}
                Err(e) => {
                    log::warn!("chamfer trimming failed on face {:?}: {e}", stripe.face1);
                }
            }

            let current_face2 = face_replacements
                .get(&stripe.face2)
                .copied()
                .unwrap_or(stripe.face2);
            let trim2 = trimmer::trim_face(
                topo,
                current_face2,
                &contact2_pts,
                &[(0.0, 0.0), (1.0, 0.0)],
                keep,
            );

            match trim2 {
                Ok(tr) if tr.trimmed_face != current_face2 => {
                    if let Some(slot) = stripe_contact_edges.last_mut() {
                        slot.1 = tr.contact_edge;
                    }
                    face_replacements.insert(stripe.face2, tr.trimmed_face);
                }
                Ok(_) => {}
                Err(e) => {
                    log::warn!("chamfer trimming failed on face {:?}: {e}", stripe.face2);
                }
            }
        }

        let mut blend_face_ids: Vec<FaceId> = Vec::new();

        for si in 0..stripe_results.len() {
            // Reuse the trimmed neighbours' contact edges (mirrors the fillet
            // builder): a freshly minted duplicate leaves both copies use-1
            // and opens the shell along the chamfer flanks.
            let (c1, c2) = stripe_contact_edges
                .get(si)
                .copied()
                .unwrap_or((None, None));
            let blend_face_id =
                crate::builder_utils::create_blend_face_with_contacts(topo, &bevels[si], c1, c2)?
                    .face;
            blend_face_ids.push(blend_face_id);
        }

        let mut result_faces: Vec<FaceId> = Vec::new();

        for &fid in &original_faces {
            if !touched_faces.contains(&fid) {
                result_faces.push(fid);
            }
        }

        for &fid in &touched_faces {
            let replacement = face_replacements.get(&fid).copied();
            result_faces.push(replacement.unwrap_or(fid));
        }

        result_faces.extend(&blend_face_ids);
        result_faces.extend(chamfer_corner::build_patches(topo, &patches)?);

        // Every producer above mints the edges it needs: the trimmer on the
        // trimmed neighbour, the bevel for its own flanks, the corner patch for
        // its own boundary. The curves they share are the same geometry but
        // separate entities, so the shell only becomes a manifold once they are
        // welded. This is the same pass `fillet_builder` runs.
        let result_faces = sew::weld_faces(topo, &result_faces)?;

        let new_shell = Shell::new(result_faces)?;
        let new_shell_id = topo.add_shell(new_shell);
        let new_solid = Solid::new(new_shell_id, Vec::new());
        let new_solid_id = topo.add_solid(new_solid);

        let is_partial = !failed.is_empty();
        Ok(BlendResult {
            solid: new_solid_id,
            succeeded,
            failed,
            is_partial,
        })
    }
}

/// A chamfer's end cross-section is a straight chord, not an arc.
///
/// `create_blend_face_with_contacts` gives a stripe's two cross edges the arc
/// of its end section when one is present. The chamfer path fills
/// `CircSection` with a chord half-length — `analytic.rs` notes the struct "is
/// shaped for fillets" — so an arc there bulges a boundary that is really a
/// straight line. Dropping the sections leaves those edges as lines: correct,
/// and the same curve the corner patch's own edges carry, so welding keeps the
/// straight one rather than a half-chord bulge.
fn straight_ended(mut stripe: Stripe) -> Stripe {
    stripe.sections.clear();
    stripe
}

/// The stripe over `[start, length - end]` of its spine.
///
/// Rebuilt through the analytic path the full stripe came from, so the contact
/// lines, surface and pcurves stay consistent with each other. The full stripe
/// is returned unchanged when there is nothing to hold back, or when the
/// analytic path cannot serve the shortened spine — a bevel that runs slightly
/// too far degrades more gracefully than no bevel at all.
fn windowed_stripe(
    topo: &Topology,
    entry: &ChamferStripe,
    start: f64,
    end: f64,
) -> Result<Stripe, BlendError> {
    let full = &entry.stripe;
    if start <= 0.0 && end <= 0.0 {
        return Ok(straight_ended(full.clone()));
    }

    let spine = full.spine.window(start, full.spine.length() - end);
    let shortened = analytic::try_analytic_chamfer(
        &entry.surf1,
        &entry.surf2,
        &spine,
        topo,
        entry.d1,
        entry.d2,
        full.face1,
        full.face2,
    )?;

    Ok(straight_ended(
        shortened.map_or_else(|| full.clone(), |sr| sr.stripe),
    ))
}

/// Compute a chamfer stripe for a single edge using the adjacency index.
///
/// # Errors
///
/// Returns [`BlendError`] if the edge is non-manifold, if topology lookups
/// fail, or if the analytic path cannot produce a result.
fn compute_chamfer_stripe(
    topo: &Topology,
    adjacency: &brepkit_topology::adjacency::AdjacencyIndex,
    edge_id: EdgeId,
    d1: f64,
    d2: f64,
) -> Result<ChamferStripe, BlendError> {
    let adj_faces = adjacency.faces_for_edge(edge_id);
    if adj_faces.len() != 2 {
        log::warn!(
            "edge {edge_id:?} has {} adjacent faces (expected 2) — cannot chamfer non-manifold or boundary edges",
            adj_faces.len()
        );
        return Err(BlendError::StartSolutionFailure {
            edge: edge_id,
            t: 0.0,
        });
    }
    let face1 = adj_faces[0];
    let face2 = adj_faces[1];

    let surf1 = topo.face(face1)?.surface().clone();
    let surf2 = topo.face(face2)?.surface().clone();

    let spine = Spine::from_single_edge(topo, edge_id)?;

    if let Some(result) =
        analytic::try_analytic_chamfer(&surf1, &surf2, &spine, topo, d1, d2, face1, face2)?
    {
        return Ok(ChamferStripe {
            edge: edge_id,
            stripe: result.stripe,
            surf1,
            surf2,
            d1,
            d2,
        });
    }

    log::debug!(
        target: "brepkit_approx",
        "chamfer: analytic path unavailable for {}+{} — v1 has no walker fallback, returning UnsupportedSurface",
        surf1.type_tag(),
        surf2.type_tag()
    );
    // v1: no walker fallback for non-analytic surface pairs.
    Err(BlendError::UnsupportedSurface {
        face: face1,
        surface_tag: format!(
            "{}+{} (walker not yet integrated)",
            surf1.type_tag(),
            surf2.type_tag()
        ),
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;
    use brepkit_topology::adjacency::AdjacencyIndex;
    use brepkit_topology::face::FaceSurface;
    use brepkit_topology::test_utils::make_unit_cube_manifold;

    /// Find the first manifold edge of the solid (shared by exactly 2 faces).
    fn find_manifold_edge(topo: &Topology, solid: SolidId) -> EdgeId {
        let adjacency = AdjacencyIndex::build(topo, solid).unwrap();
        let shell_id = topo.solid(solid).unwrap().outer_shell();
        let faces = topo.shell(shell_id).unwrap().faces().to_vec();

        for &fid in &faces {
            let face = topo.face(fid).unwrap();
            let wire = topo.wire(face.outer_wire()).unwrap();
            for oe in wire.edges() {
                let adj = adjacency.faces_for_edge(oe.edge());
                if adj.len() == 2 {
                    return oe.edge();
                }
            }
        }
        panic!("cube should have manifold edges");
    }

    #[test]
    fn chamfer_builder_symmetric() {
        let mut topo = Topology::new();
        let solid = make_unit_cube_manifold(&mut topo);
        let target_edge = find_manifold_edge(&topo, solid);

        let shell_id = topo.solid(solid).unwrap().outer_shell();
        let original_face_count = topo.shell(shell_id).unwrap().faces().len();

        let mut builder = ChamferBuilder::new(&mut topo, solid);
        builder.add_edges_symmetric(&[target_edge], 0.1);
        let result = builder.build().expect("chamfer build should succeed");

        let result_solid = topo.solid(result.solid).unwrap();
        let result_shell = topo.shell(result_solid.outer_shell()).unwrap();

        assert!(
            result_shell.faces().len() > original_face_count,
            "expected more faces after chamfer: got {}, original {}",
            result_shell.faces().len(),
            original_face_count,
        );

        assert!(result.succeeded.contains(&target_edge));
        assert!(result.failed.is_empty());
        assert!(!result.is_partial);

        let mut found_chamfer_plane = false;
        for &fid in result_shell.faces() {
            let face = topo.face(fid).unwrap();
            if matches!(face.surface(), FaceSurface::Plane { .. }) {
                found_chamfer_plane = true;
            }
        }
        assert!(
            found_chamfer_plane,
            "chamfer should produce a planar blend surface"
        );
    }

    #[test]
    fn chamfer_builder_distance_angle() {
        let mut topo = Topology::new();
        let solid = make_unit_cube_manifold(&mut topo);
        let target_edge = find_manifold_edge(&topo, solid);

        let shell_id = topo.solid(solid).unwrap().outer_shell();
        let original_face_count = topo.shell(shell_id).unwrap().faces().len();

        // 45-degree angle means d2 = distance * tan(45deg) = distance.
        let distance = 0.15;
        let angle = std::f64::consts::FRAC_PI_4;

        let mut builder = ChamferBuilder::new(&mut topo, solid);
        builder.add_edges_distance_angle(&[target_edge], distance, angle);
        let result = builder.build().expect("chamfer build should succeed");

        let result_solid = topo.solid(result.solid).unwrap();
        let result_shell = topo.shell(result_solid.outer_shell()).unwrap();

        assert!(
            result_shell.faces().len() > original_face_count,
            "expected more faces after distance-angle chamfer"
        );
        assert!(result.succeeded.contains(&target_edge));
        assert!(result.failed.is_empty());
    }

    #[test]
    fn chamfer_builder_empty_edges_error() {
        let mut topo = Topology::new();
        let solid = make_unit_cube_manifold(&mut topo);

        let builder = ChamferBuilder::new(&mut topo, solid);
        let result = builder.build();
        assert!(result.is_err(), "empty edge set should produce an error");
    }
}
