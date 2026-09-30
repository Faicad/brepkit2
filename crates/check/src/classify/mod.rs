//! Point-in-solid classification (ray casting + winding numbers).
//!
//! The primary entry point is [`classify_point`], which uses analytic ray
//! casting with UV boundary containment to determine whether a 3D point
//! lies inside, outside, or on the boundary of a B-Rep solid.

pub(crate) mod boundary;
pub(crate) mod ray_surface;
pub(crate) mod winding;

use brepkit_math::vec::{Point3, Vec3};
use brepkit_topology::Topology;
use brepkit_topology::face::{FaceId, FaceSurface};
use brepkit_topology::solid::SolidId;

use crate::CheckError;

/// Result of classifying a point relative to a solid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointClassification {
    /// The point is inside the solid.
    Inside,
    /// The point is outside the solid.
    Outside,
    /// The point is on the boundary (within tolerance).
    OnBoundary,
}

/// Options controlling the classification algorithm.
#[derive(Debug, Clone)]
pub struct ClassifyOptions {
    /// Distance threshold for "on boundary" detection.
    pub tolerance: f64,
    /// Maximum recovery attempts when ray hits face boundary.
    pub max_recovery_attempts: usize,
}

impl Default for ClassifyOptions {
    fn default() -> Self {
        Self {
            tolerance: 1e-6,
            max_recovery_attempts: 10,
        }
    }
}

/// Classify a point relative to a solid using analytic ray casting.
///
/// Uses three irrational ray directions for majority-vote consensus.
/// If the first two agree, the third is skipped. If all three disagree
/// (very rare — indicates grazing rays), perturbed recovery directions
/// are tried.
///
/// # Errors
///
/// Returns an error if the solid or its faces contain invalid topology references.
#[allow(clippy::cast_precision_loss, clippy::too_many_lines)]
pub fn classify_point(
    topo: &Topology,
    solid: SolidId,
    point: Point3,
    options: &ClassifyOptions,
) -> Result<PointClassification, CheckError> {
    let solid_data = topo.solid(solid)?;
    // Cavity (inner) shells bound material too: a solid's voids are part of
    // its boundary. Collecting only the outer shell makes every point in a
    // cavity classify as Inside.
    let mut face_set: Vec<FaceId> = Vec::new();
    for shell_id in
        std::iter::once(solid_data.outer_shell()).chain(solid_data.inner_shells().iter().copied())
    {
        face_set.extend_from_slice(topo.shell(shell_id)?.faces());
    }

    if is_on_boundary(topo, &face_set, point, options.tolerance)? {
        return Ok(PointClassification::OnBoundary);
    }

    // Three irrational ray directions for majority-vote consensus.
    // If the first two agree, the third breaks no tie and we exit early.
    let base_dirs = [
        Vec3::new(
            0.573_576_436_351_046,
            0.740_535_693_464_567_5,
            0.350_889_803_483_932_2,
        ),
        Vec3::new(
            -0.350_889_803_483_932_2,
            0.573_576_436_351_046,
            0.740_535_693_464_567_5,
        ),
        Vec3::new(
            0.740_535_693_464_567_5,
            -0.350_889_803_483_932_2,
            0.573_576_436_351_046,
        ),
    ];

    let mut inside_votes = 0u32;
    let mut outside_votes = 0u32;

    for &dir in &base_dirs {
        let crossings = count_ray_crossings(topo, &face_set, point, dir)?;
        if crossings % 2 == 1 {
            inside_votes += 1;
        } else {
            outside_votes += 1;
        }
        // Early exit: if 2 rays agree, that's the answer.
        if inside_votes >= 2 {
            return Ok(PointClassification::Inside);
        }
        if outside_votes >= 2 {
            return Ok(PointClassification::Outside);
        }
    }

    // All three disagreed (very rare). Try perturbed directions as recovery.
    for attempt in 0..options.max_recovery_attempts {
        // Generate a pseudo-random direction from attempt index using golden ratio.
        let seed = (attempt as f64 + 1.0) * 0.618_033_988_749_895;
        let theta = seed * std::f64::consts::TAU;
        let phi = (seed * std::f64::consts::E).fract() * std::f64::consts::PI;
        let dir = Vec3::new(phi.sin() * theta.cos(), phi.sin() * theta.sin(), phi.cos());

        let crossings = count_ray_crossings(topo, &face_set, point, dir)?;
        if crossings % 2 == 1 {
            inside_votes += 1;
        } else {
            outside_votes += 1;
        }
        let remaining = options.max_recovery_attempts as u32 - attempt as u32;
        if inside_votes > outside_votes + remaining {
            return Ok(PointClassification::Inside);
        }
        if outside_votes > inside_votes + remaining {
            return Ok(PointClassification::Outside);
        }
    }

    // Majority vote from all attempts.
    if inside_votes > outside_votes {
        Ok(PointClassification::Inside)
    } else {
        Ok(PointClassification::Outside)
    }
}

/// Checks if a point is within `tolerance` of any face boundary.
///
/// Uses analytic point-to-surface distance for all surface types, then
/// verifies the projection falls within the face polygon.
fn is_on_boundary(
    topo: &Topology,
    faces: &[FaceId],
    point: Point3,
    tolerance: f64,
) -> Result<bool, CheckError> {
    for &fid in faces {
        let face = topo.face(fid)?;
        let dist = match face.surface() {
            FaceSurface::Plane { normal, d } => {
                let pv = Vec3::new(point.x(), point.y(), point.z());
                (normal.dot(pv) - d).abs()
            }
            FaceSurface::Cylinder(cyl) => {
                let (u, v) = cyl.project_point(point);
                let on_surface = cyl.evaluate(u, v);
                (point - on_surface).length()
            }
            FaceSurface::Cone(cone) => {
                let (u, v) = cone.project_point(point);
                let on_surface = cone.evaluate(u, v);
                (point - on_surface).length()
            }
            FaceSurface::Sphere(sph) => {
                let (u, v) = sph.project_point(point);
                let on_surface = sph.evaluate(u, v);
                (point - on_surface).length()
            }
            FaceSurface::Torus(tor) => {
                let (u, v) = tor.project_point(point);
                let on_surface = tor.evaluate(u, v);
                (point - on_surface).length()
            }
            FaceSurface::Nurbs(nurbs) => {
                match brepkit_math::nurbs::projection::project_point_to_surface(
                    nurbs, point, tolerance,
                ) {
                    Ok(proj) => proj.distance,
                    Err(_) => f64::INFINITY,
                }
            }
        };
        if dist < tolerance && boundary::point_in_face_boundary(topo, fid, point)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Classify a point relative to a solid using generalized winding numbers.
///
/// More robust than ray casting for imperfect geometry (small gaps,
/// T-junctions). Sums the signed solid angles of triangulated faces and
/// classifies based on the resulting winding number.
///
/// # Errors
///
/// Returns an error if the solid or its faces contain invalid topology references.
pub fn classify_point_winding(
    topo: &Topology,
    solid: SolidId,
    point: Point3,
    options: &ClassifyOptions,
) -> Result<PointClassification, CheckError> {
    let solid_data = topo.solid(solid)?;
    let mut face_set: Vec<FaceId> = Vec::new();
    for shell_id in
        std::iter::once(solid_data.outer_shell()).chain(solid_data.inner_shells().iter().copied())
    {
        face_set.extend_from_slice(topo.shell(shell_id)?.faces());
    }
    if is_on_boundary(topo, &face_set, point, options.tolerance)? {
        return Ok(PointClassification::OnBoundary);
    }

    let w = winding::winding_number(topo, solid, point)?;
    if w > 0.5 {
        Ok(PointClassification::Inside)
    } else {
        Ok(PointClassification::Outside)
    }
}

/// Robust classification combining winding numbers and ray casting.
///
/// Uses winding numbers first, falling back to ray casting when the
/// winding number is ambiguous (between 0.4 and 0.6). This provides the
/// best accuracy for both clean and imperfect geometry.
///
/// # Errors
///
/// Returns an error if the solid or its faces contain invalid topology references.
pub fn classify_point_robust(
    topo: &Topology,
    solid: SolidId,
    point: Point3,
    options: &ClassifyOptions,
) -> Result<PointClassification, CheckError> {
    let solid_data = topo.solid(solid)?;
    let mut face_set: Vec<FaceId> = Vec::new();
    for shell_id in
        std::iter::once(solid_data.outer_shell()).chain(solid_data.inner_shells().iter().copied())
    {
        face_set.extend_from_slice(topo.shell(shell_id)?.faces());
    }
    if is_on_boundary(topo, &face_set, point, options.tolerance)? {
        return Ok(PointClassification::OnBoundary);
    }

    let w = winding::winding_number(topo, solid, point)?;
    if w > 0.6 {
        return Ok(PointClassification::Inside);
    }
    if w < 0.4 {
        return Ok(PointClassification::Outside);
    }
    classify_point(topo, solid, point, options)
}

/// Count total ray crossings across all faces of a shell.
///
/// Builds a BVH over face AABBs to skip faces whose bounding box
/// the ray does not intersect.
fn count_ray_crossings(
    topo: &Topology,
    faces: &[FaceId],
    origin: Point3,
    direction: Vec3,
) -> Result<u32, CheckError> {
    use brepkit_math::bvh::Bvh;

    let face_aabbs: Vec<(usize, brepkit_math::aabb::Aabb3)> = faces
        .iter()
        .enumerate()
        .filter_map(|(i, &fid)| crate::util::face_aabb(topo, fid).ok().map(|aabb| (i, aabb)))
        .collect();
    let bvh = Bvh::build(&face_aabbs);

    // query_ray returns the primitive IDs (the `i` values), which are
    // indices into the original `faces` slice.
    let candidates = bvh.query_ray(origin, direction);

    let mut crossings = 0u32;
    for face_idx in candidates {
        crossings += boundary::count_face_ray_crossings(topo, faces[face_idx], origin, direction)?;
    }
    Ok(crossings)
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::winding;
    use super::*;
    use brepkit_math::curves::Circle3D;
    use brepkit_topology::edge::{Edge, EdgeCurve};
    use brepkit_topology::face::Face;
    use brepkit_topology::shell::Shell;
    use brepkit_topology::solid::Solid;
    use brepkit_topology::test_utils::make_unit_cube_manifold;
    use brepkit_topology::vertex::Vertex;
    use brepkit_topology::wire::{OrientedEdge, Wire};

    /// A planar annulus in the `z = 0` plane: square outer wire spanning
    /// `[0,4]²`, circular hole of `hole_r` centred at `(2, 2)`.
    fn make_annulus_face(topo: &mut Topology, hole_r: f64) -> FaceId {
        let corners = [
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(4.0, 0.0, 0.0),
            Point3::new(4.0, 4.0, 0.0),
            Point3::new(0.0, 4.0, 0.0),
        ];
        let verts: Vec<_> = corners
            .iter()
            .map(|&p| topo.add_vertex(Vertex::new(p, 1e-7)))
            .collect();
        let edges: Vec<_> = (0..4)
            .map(|i| topo.add_edge(Edge::new(verts[i], verts[(i + 1) % 4], EdgeCurve::Line)))
            .collect();
        let outer = topo.add_wire(
            Wire::new(
                edges.iter().map(|&e| OrientedEdge::new(e, true)).collect(),
                true,
            )
            .unwrap(),
        );

        let circle =
            Circle3D::new(Point3::new(2.0, 2.0, 0.0), Vec3::new(0.0, 0.0, 1.0), hole_r).unwrap();
        let seam = topo.add_vertex(Vertex::new(circle.evaluate(0.0), 1e-7));
        let circle_edge = topo.add_edge(Edge::new(seam, seam, EdgeCurve::Circle(circle)));
        let inner =
            topo.add_wire(Wire::new(vec![OrientedEdge::new(circle_edge, true)], true).unwrap());

        topo.add_face(Face::new(
            outer,
            vec![inner],
            FaceSurface::Plane {
                normal: Vec3::new(0.0, 0.0, 1.0),
                d: 0.0,
            },
        ))
    }

    /// An axis-aligned cube shell of edge length `size` at `(ox, oy, oz)`.
    ///
    /// `reversed` builds the shell with inward-pointing orientation — the
    /// correct sense for a cavity (inner) shell.
    fn make_cube_shell(
        topo: &mut Topology,
        ox: f64,
        oy: f64,
        oz: f64,
        size: f64,
        reversed: bool,
    ) -> brepkit_topology::shell::ShellId {
        /// One cube face: its four wire edges (index, forward), plane normal
        /// and plane offset.
        type CubeFaceSpec = ([(usize, bool); 4], Vec3, f64);

        let c =
            |bx: f64, by: f64, bz: f64| Point3::new(ox + bx * size, oy + by * size, oz + bz * size);
        let v: Vec<_> = [
            c(0.0, 0.0, 0.0),
            c(1.0, 0.0, 0.0),
            c(1.0, 1.0, 0.0),
            c(0.0, 1.0, 0.0),
            c(0.0, 0.0, 1.0),
            c(1.0, 0.0, 1.0),
            c(1.0, 1.0, 1.0),
            c(0.0, 1.0, 1.0),
        ]
        .iter()
        .map(|&p| topo.add_vertex(Vertex::new(p, 1e-7)))
        .collect();

        // bottom ring, top ring, then the four verticals
        let e: Vec<_> = [
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 0),
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 4),
            (0, 4),
            (1, 5),
            (2, 6),
            (3, 7),
        ]
        .iter()
        .map(|&(a, b)| topo.add_edge(Edge::new(v[a], v[b], EdgeCurve::Line)))
        .collect();

        // (edges as (index, forward), plane normal, plane offset d)
        let specs: [CubeFaceSpec; 6] = [
            (
                [(0, false), (3, false), (2, false), (1, false)],
                Vec3::new(0.0, 0.0, -1.0),
                -oz,
            ),
            (
                [(4, true), (5, true), (6, true), (7, true)],
                Vec3::new(0.0, 0.0, 1.0),
                oz + size,
            ),
            (
                [(0, true), (9, true), (4, false), (8, false)],
                Vec3::new(0.0, -1.0, 0.0),
                -oy,
            ),
            (
                [(2, true), (11, true), (6, false), (10, false)],
                Vec3::new(0.0, 1.0, 0.0),
                oy + size,
            ),
            (
                [(3, true), (8, true), (7, false), (11, false)],
                Vec3::new(-1.0, 0.0, 0.0),
                -ox,
            ),
            (
                [(1, true), (10, true), (5, false), (9, false)],
                Vec3::new(1.0, 0.0, 0.0),
                ox + size,
            ),
        ];

        let mut faces = Vec::new();
        for (edges, normal, d) in specs {
            let wire = topo.add_wire(
                Wire::new(
                    edges
                        .iter()
                        .map(|&(i, fwd)| OrientedEdge::new(e[i], fwd))
                        .collect(),
                    true,
                )
                .unwrap(),
            );
            let surface = FaceSurface::Plane { normal, d };
            faces.push(if reversed {
                topo.add_face(Face::new_reversed(wire, vec![], surface))
            } else {
                topo.add_face(Face::new(wire, vec![], surface))
            });
        }
        topo.add_shell(Shell::new(faces).unwrap())
    }

    // ── E-03: a ray landing in a face's hole is not a crossing ──────────

    /// Ray straight up the hole's axis: the hit point lies inside the hole,
    /// so the annulus does not block the ray.
    #[test]
    fn ray_through_face_hole_is_not_a_crossing() {
        let mut topo = Topology::new();
        let face = make_annulus_face(&mut topo, 1.0);

        let n = boundary::count_face_ray_crossings(
            &topo,
            face,
            Point3::new(2.0, 2.0, -1.0),
            Vec3::new(0.0, 0.0, 1.0),
        )
        .unwrap();
        assert_eq!(n, 0, "a hit inside the hole must not count as a crossing");
    }

    /// Control case: the same face away from the hole does block the ray.
    #[test]
    fn ray_through_face_material_is_a_crossing() {
        let mut topo = Topology::new();
        let face = make_annulus_face(&mut topo, 1.0);

        let n = boundary::count_face_ray_crossings(
            &topo,
            face,
            Point3::new(0.5, 0.5, -1.0),
            Vec3::new(0.0, 0.0, 1.0),
        )
        .unwrap();
        assert_eq!(
            n, 1,
            "a hit on the material part of the annulus counts once"
        );
    }

    // ── E-05: cavity (inner) shells participate in classification ───────

    /// A point in the cavity of a hollow cube is OUTSIDE the material.
    ///
    /// Outer cube `[0,4]³`, cavity cube `[1,3]³`. A ray leaving the centre
    /// crosses the cavity wall and then the outer wall — two crossings, i.e.
    /// outside. Counting only the outer shell yields one crossing and
    /// mis-reports the cavity as inside.
    #[test]
    fn point_inside_cavity_is_outside() {
        let mut topo = Topology::new();
        let outer = make_cube_shell(&mut topo, 0.0, 0.0, 0.0, 4.0, false);
        let cavity = make_cube_shell(&mut topo, 1.0, 1.0, 1.0, 2.0, true);
        let solid = topo.add_solid(Solid::new(outer, vec![cavity]));

        let result = classify_point(
            &topo,
            solid,
            Point3::new(2.0, 2.0, 2.0),
            &ClassifyOptions::default(),
        )
        .unwrap();
        assert_eq!(
            result,
            PointClassification::Outside,
            "the cavity is void, not material"
        );
    }

    /// Same cavity geometry through the winding-number classifier: a cavity
    /// shell contributes negative winding, cancelling the outer shell's.
    #[test]
    fn point_inside_cavity_is_outside_winding() {
        let mut topo = Topology::new();
        let outer = make_cube_shell(&mut topo, 0.0, 0.0, 0.0, 4.0, false);
        let cavity = make_cube_shell(&mut topo, 1.0, 1.0, 1.0, 2.0, true);
        let solid = topo.add_solid(Solid::new(outer, vec![cavity]));

        let w = winding::winding_number(&topo, solid, Point3::new(2.0, 2.0, 2.0)).unwrap();
        assert!(
            w.abs() < 0.2,
            "winding number in a cavity must be ~0 (void), got {w}"
        );
    }

    /// Control case: a point in the material between cavity and outer wall.
    #[test]
    fn point_in_shell_wall_is_inside() {
        let mut topo = Topology::new();
        let outer = make_cube_shell(&mut topo, 0.0, 0.0, 0.0, 4.0, false);
        let cavity = make_cube_shell(&mut topo, 1.0, 1.0, 1.0, 2.0, true);
        let solid = topo.add_solid(Solid::new(outer, vec![cavity]));

        // (0.5, 0.5, 0.5) sits in the 1-unit-thick wall.
        let result = classify_point(
            &topo,
            solid,
            Point3::new(0.5, 0.5, 0.5),
            &ClassifyOptions::default(),
        )
        .unwrap();
        assert_eq!(result, PointClassification::Inside);
    }

    #[test]
    fn point_inside_box() {
        let mut topo = Topology::new();
        let solid = make_unit_cube_manifold(&mut topo);
        let center = Point3::new(0.5, 0.5, 0.5);
        let opts = ClassifyOptions::default();

        let result = classify_point(&topo, solid, center, &opts).unwrap();
        assert_eq!(result, PointClassification::Inside);
    }

    #[test]
    fn point_outside_box() {
        let mut topo = Topology::new();
        let solid = make_unit_cube_manifold(&mut topo);
        let far = Point3::new(5.0, 5.0, 5.0);
        let opts = ClassifyOptions::default();

        let result = classify_point(&topo, solid, far, &opts).unwrap();
        assert_eq!(result, PointClassification::Outside);
    }

    #[test]
    fn point_on_boundary_box() {
        let mut topo = Topology::new();
        let solid = make_unit_cube_manifold(&mut topo);
        // Center of the top face (z=1).
        let on_face = Point3::new(0.5, 0.5, 1.0);
        let opts = ClassifyOptions::default();

        let result = classify_point(&topo, solid, on_face, &opts).unwrap();
        assert_eq!(result, PointClassification::OnBoundary);
    }

    #[test]
    fn point_near_edge_outside() {
        let mut topo = Topology::new();
        let solid = make_unit_cube_manifold(&mut topo);
        // Just outside the box along the x-axis.
        let outside = Point3::new(1.001, 0.5, 0.5);
        let opts = ClassifyOptions::default();

        let result = classify_point(&topo, solid, outside, &opts).unwrap();
        assert_eq!(result, PointClassification::Outside);
    }

    #[test]
    fn point_at_corner_boundary() {
        let mut topo = Topology::new();
        let solid = make_unit_cube_manifold(&mut topo);
        // Very close to a vertex of the box.
        let near_corner = Point3::new(0.0, 0.0, 0.0);
        let opts = ClassifyOptions::default();

        let result = classify_point(&topo, solid, near_corner, &opts).unwrap();
        assert_eq!(result, PointClassification::OnBoundary);
    }

    #[test]
    fn winding_inside_box() {
        let mut topo = Topology::new();
        let solid = make_unit_cube_manifold(&mut topo);
        let center = Point3::new(0.5, 0.5, 0.5);

        let w = winding::winding_number(&topo, solid, center).unwrap();
        assert!(
            w > 0.5,
            "winding number for interior point should be > 0.5, got {w}"
        );
    }

    #[test]
    fn winding_outside_box() {
        let mut topo = Topology::new();
        let solid = make_unit_cube_manifold(&mut topo);
        let far = Point3::new(5.0, 5.0, 5.0);

        let w = winding::winding_number(&topo, solid, far).unwrap();
        assert!(
            w < 0.5,
            "winding number for exterior point should be < 0.5, got {w}"
        );
    }

    #[test]
    fn classify_winding_matches_ray() {
        let mut topo = Topology::new();
        let solid = make_unit_cube_manifold(&mut topo);
        let center = Point3::new(0.5, 0.5, 0.5);
        let opts = ClassifyOptions::default();

        let ray_result = classify_point(&topo, solid, center, &opts).unwrap();
        let winding_result = classify_point_winding(&topo, solid, center, &opts).unwrap();
        assert_eq!(ray_result, winding_result);
    }

    #[test]
    fn point_negative_quadrant_outside() {
        let mut topo = Topology::new();
        let solid = make_unit_cube_manifold(&mut topo);
        let neg = Point3::new(-1.0, -1.0, -1.0);
        let opts = ClassifyOptions::default();

        let result = classify_point(&topo, solid, neg, &opts).unwrap();
        assert_eq!(result, PointClassification::Outside);
    }

    /// Build the 3-face solid a partial-turn circle revolve produces: one
    /// trimmed torus band (u in `[0, angle]`, full tube wrap; wire = two
    /// closed rim circles + a doubled seam arc, only 2 distinct vertices)
    /// plus two planar disc caps each bounded by a single closed circle.
    fn make_partial_torus_band(topo: &mut Topology, big_r: f64, rho: f64, angle: f64) -> SolidId {
        use brepkit_math::curves::Circle3D;
        use brepkit_math::surfaces::ToroidalSurface;
        use brepkit_topology::edge::{Edge, EdgeCurve};
        use brepkit_topology::face::{Face, FaceSurface};
        use brepkit_topology::shell::Shell;
        use brepkit_topology::solid::Solid;
        use brepkit_topology::vertex::Vertex;
        use brepkit_topology::wire::{OrientedEdge, Wire};

        let (sin_a, cos_a) = angle.sin_cos();
        let v1 = topo.add_vertex(Vertex::new(Point3::new(big_r, 0.0, -rho), 1e-7));
        let v2 = topo.add_vertex(Vertex::new(
            Point3::new(big_r * cos_a, big_r * sin_a, -rho),
            1e-7,
        ));

        let rim1 =
            Circle3D::new(Point3::new(big_r, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0), rho).unwrap();
        let rim2 = Circle3D::new(
            Point3::new(big_r * cos_a, big_r * sin_a, 0.0),
            Vec3::new(-sin_a, cos_a, 0.0),
            rho,
        )
        .unwrap();
        let seam =
            Circle3D::new(Point3::new(0.0, 0.0, -rho), Vec3::new(0.0, 0.0, 1.0), big_r).unwrap();

        let e_rim1 = topo.add_edge(Edge::new(v1, v1, EdgeCurve::Circle(rim1)));
        let e_rim2 = topo.add_edge(Edge::new(v2, v2, EdgeCurve::Circle(rim2)));
        let e_seam = topo.add_edge(Edge::new(v1, v2, EdgeCurve::Circle(seam)));

        let band_wire = topo.add_wire(
            Wire::new(
                vec![
                    OrientedEdge::new(e_rim1, true),
                    OrientedEdge::new(e_seam, true),
                    OrientedEdge::new(e_rim2, false),
                    OrientedEdge::new(e_seam, false),
                ],
                true,
            )
            .unwrap(),
        );
        let torus = ToroidalSurface::with_axis(
            Point3::new(0.0, 0.0, 0.0),
            big_r,
            rho,
            Vec3::new(0.0, 0.0, 1.0),
        )
        .unwrap();
        let band = topo.add_face(Face::new(band_wire, vec![], FaceSurface::Torus(torus)));

        let cap1_wire =
            topo.add_wire(Wire::new(vec![OrientedEdge::new(e_rim1, false)], true).unwrap());
        let cap1 = topo.add_face(Face::new(
            cap1_wire,
            vec![],
            FaceSurface::Plane {
                normal: Vec3::new(0.0, 1.0, 0.0),
                d: 0.0,
            },
        ));
        let cap2_wire =
            topo.add_wire(Wire::new(vec![OrientedEdge::new(e_rim2, true)], true).unwrap());
        let cap2 = topo.add_face(Face::new(
            cap2_wire,
            vec![],
            FaceSurface::Plane {
                normal: Vec3::new(-sin_a, cos_a, 0.0),
                d: 0.0,
            },
        ));

        let shell = topo.add_shell(Shell::new(vec![band, cap1, cap2]).unwrap());
        topo.add_solid(Solid::new(shell, vec![]))
    }

    /// Regression: interior points of a partial-turn torus band read Outside.
    /// Two stacked roots: the local Ferrari ray-torus quartic missed real
    /// roots and emitted off-surface spurious ones, and `face_aabb` collapsed
    /// each cap disc (single closed-circle wire, one vertex) to a point AABB,
    /// so the BVH prefilter never offered the caps and their crossings were
    /// dropped from the parity count.
    #[test]
    fn partial_torus_band_interior_points() {
        let (big_r, rho, angle) = (6.0_f64, 2.0_f64, 2.0 * std::f64::consts::PI / 3.0);
        let mut topo = Topology::new();
        let solid = make_partial_torus_band(&mut topo, big_r, rho, angle);
        let opts = ClassifyOptions::default();

        let mid = angle / 2.0;
        let inside = [
            Point3::new(big_r * mid.cos(), big_r * mid.sin(), 0.0),
            Point3::new(big_r * mid.cos(), big_r * mid.sin(), 1.0),
            Point3::new(big_r * mid.cos(), big_r * mid.sin(), -1.0),
            Point3::new(big_r * 0.05f64.cos(), big_r * 0.05f64.sin(), 0.0),
            Point3::new(
                big_r * (angle - 0.05).cos(),
                big_r * (angle - 0.05).sin(),
                0.0,
            ),
            Point3::new((big_r - 1.5) * mid.cos(), (big_r - 1.5) * mid.sin(), 0.0),
            Point3::new((big_r + 1.5) * mid.cos(), (big_r + 1.5) * mid.sin(), 0.0),
        ];
        for p in inside {
            let result = classify_point(&topo, solid, p, &opts).unwrap();
            assert_eq!(result, PointClassification::Inside, "probe {p:?}");
        }

        let outside = [
            Point3::new(big_r * mid.cos(), big_r * mid.sin(), 2.5),
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(-big_r, 0.0, 0.0),
            Point3::new(
                big_r * (angle + 0.1).cos(),
                big_r * (angle + 0.1).sin(),
                0.0,
            ),
            Point3::new(big_r * (-0.1f64).cos(), big_r * (-0.1f64).sin(), 0.0),
        ];
        for p in outside {
            let result = classify_point(&topo, solid, p, &opts).unwrap();
            assert_eq!(result, PointClassification::Outside, "probe {p:?}");
        }
    }

    /// A cap disc bounded by a single closed circle edge must get a full-disc
    /// AABB, not a point box at its lone seam vertex (the collapsed box
    /// starved the classifier's BVH prefilter).
    #[test]
    fn face_aabb_covers_closed_circle_boundary() {
        let (big_r, rho, angle) = (6.0_f64, 2.0_f64, 2.0 * std::f64::consts::PI / 3.0);
        let mut topo = Topology::new();
        let solid = make_partial_torus_band(&mut topo, big_r, rho, angle);
        let shell = topo
            .shell(topo.solid(solid).unwrap().outer_shell())
            .unwrap();

        // Face index 1 is the y=0 cap: disc center (6,0,0) radius 2 in the
        // xz-plane, so the AABB must span x in [4,8] and z in [-2,2].
        let cap = shell.faces()[1];
        let aabb = crate::util::face_aabb(&topo, cap).unwrap();
        assert!(
            aabb.min.x() < 4.0 + 1e-9 && aabb.max.x() > 8.0 - 1e-9,
            "cap AABB x-span collapsed: {aabb:?}"
        );
        assert!(
            aabb.min.z() < -2.0 + 1e-9 && aabb.max.z() > 2.0 - 1e-9,
            "cap AABB z-span collapsed: {aabb:?}"
        );
    }
}
