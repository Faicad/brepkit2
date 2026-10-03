//! Geometric properties: volume, area, center of mass, inertia tensor.

pub mod accumulator;
pub mod analytic;
pub mod bbox;
pub mod face_integrator;

pub use accumulator::GProps;

use brepkit_math::aabb::Aabb3;
use brepkit_math::vec::{Point3, Vec3};
use brepkit_topology::Topology;
use brepkit_topology::face::FaceId;
use brepkit_topology::solid::SolidId;

use crate::CheckError;

/// Options for property computation.
#[derive(Debug, Clone)]
pub struct PropertiesOptions {
    /// Gauss quadrature order (default 5).
    pub gauss_order: usize,
    /// Adaptive integration tolerance (default 1e-6).
    pub adaptive_eps: f64,
    /// Maximum adaptive subdivision depth (default 8).
    pub max_depth: usize,
}

impl Default for PropertiesOptions {
    fn default() -> Self {
        Self {
            gauss_order: 5,
            adaptive_eps: 1e-6,
            max_depth: 8,
        }
    }
}

/// Compute the bounding box of a solid.
///
/// # Errors
///
/// Returns an error if any topology entity is missing or the solid has no vertices.
pub fn bounding_box(topo: &Topology, solid: SolidId) -> Result<Aabb3, CheckError> {
    bbox::bounding_box(topo, solid)
}

/// Compute the volume of a solid via face integration.
///
/// Uses the divergence theorem: V = (1/3) sum of integral P dot N dA
/// over every face of the solid's boundary. A cavity (inner shell) is part
/// of that boundary; its faces are oriented away from the material, so
/// summing them removes the void instead of adding it.
///
/// # Errors
///
/// Returns an error if any topology entity is missing or integration fails.
pub fn solid_volume(
    topo: &Topology,
    solid: SolidId,
    options: &PropertiesOptions,
) -> Result<f64, CheckError> {
    let mut total_volume = 0.0;
    for fid in crate::util::solid_face_ids(topo, solid)? {
        let contrib = face_integrator::integrate_face(topo, fid, options.gauss_order)?;
        total_volume += contrib.volume;
    }
    Ok(total_volume)
}

/// Compute the total surface area of a solid.
///
/// Sums the area of every face on the boundary, including the walls of any
/// cavity (inner shell) — those are surface too, facing into the void.
///
/// # Errors
///
/// Returns an error if any topology entity is missing or integration fails.
pub fn solid_area(
    topo: &Topology,
    solid: SolidId,
    options: &PropertiesOptions,
) -> Result<f64, CheckError> {
    let mut total_area = 0.0;
    for fid in crate::util::solid_face_ids(topo, solid)? {
        let contrib = face_integrator::integrate_face(topo, fid, options.gauss_order)?;
        total_area += contrib.area;
    }
    Ok(total_area)
}

/// Compute the center of mass of a solid.
///
/// Uses the divergence theorem: for each coordinate axis, integrates
/// `(1/2) x_i^2 * n_i` over the solid's whole boundary (outer shell plus
/// any cavity shells), then divides by total volume to obtain the
/// volumetric centroid (solid CoM).
///
/// # Errors
///
/// Returns an error if any topology entity is missing, integration fails,
/// or the solid has zero volume.
pub fn center_of_mass(
    topo: &Topology,
    solid: SolidId,
    options: &PropertiesOptions,
) -> Result<Point3, CheckError> {
    let mut total_volume = 0.0;
    let mut mx = 0.0;
    let mut my = 0.0;
    let mut mz = 0.0;

    for fid in crate::util::solid_face_ids(topo, solid)? {
        let contrib = face_integrator::integrate_face(topo, fid, options.gauss_order)?;
        total_volume += contrib.volume;
        mx += contrib.volume_moment_x;
        my += contrib.volume_moment_y;
        mz += contrib.volume_moment_z;
    }

    if total_volume.abs() < 1e-30 {
        return Err(CheckError::IntegrationFailed(
            "solid has zero volume".into(),
        ));
    }

    Ok(Point3::new(
        mx / total_volume,
        my / total_volume,
        mz / total_volume,
    ))
}

/// Compute the v-range for an analytic surface by projecting face wire
/// vertices onto the given axis.
///
/// Iterates over all wires (outer + inner) of `face_id`, projects each vertex
/// position onto `axis` relative to `origin`, and returns `(v_min, v_max)`.
/// If the face has no distinguishable range (e.g. a single vertex),
/// returns `(-1.0, 1.0)` as a fallback.
///
/// # Errors
///
/// Returns an error if any topology entity is missing.
pub fn axial_v_range(
    topo: &Topology,
    face_id: FaceId,
    origin: Point3,
    axis: Vec3,
) -> Result<(f64, f64), CheckError> {
    let face_data = topo.face(face_id)?;
    let outer = topo.wire(face_data.outer_wire())?;

    let mut v_min = f64::MAX;
    let mut v_max = f64::MIN;

    // Chain outer wire and inner wires.
    let inner_wires: Vec<_> = face_data
        .inner_wires()
        .iter()
        .filter_map(|&wid| topo.wire(wid).ok())
        .collect();

    for wire in std::iter::once(outer).chain(inner_wires.iter().copied()) {
        for oe in wire.edges() {
            let edge = topo.edge(oe.edge())?;
            for vid in [oe.oriented_start(edge), oe.oriented_end(edge)] {
                let pt = topo.vertex(vid)?.point();
                let to_pt = Vec3::new(
                    pt.x() - origin.x(),
                    pt.y() - origin.y(),
                    pt.z() - origin.z(),
                );
                let v = axis.dot(to_pt);
                v_min = v_min.min(v);
                v_max = v_max.max(v);
            }
        }
    }

    if v_min < v_max {
        Ok((v_min, v_max))
    } else {
        Ok((-1.0, 1.0))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use brepkit_math::vec::Point3;
    use brepkit_topology::Topology;
    use brepkit_topology::test_utils::make_unit_cube_manifold;

    #[test]
    fn gprops_accumulator_two_cubes() {
        // Two unit cubes side by side along x-axis
        let a = analytic::box_props(1.0, 1.0, 1.0);
        let mut b = analytic::box_props(1.0, 1.0, 1.0);
        // Shift b's center to (1.5, 0.5, 0.5) — as if placed at x=1
        b.center = Point3::new(1.5, 0.5, 0.5);

        let mut combined = a;
        combined.add(&b);

        // Total volume = 2
        assert!((combined.mass - 2.0).abs() < 1e-12);
        // Combined center = (1.0, 0.5, 0.5)
        assert!((combined.center.x() - 1.0).abs() < 1e-12);
        assert!((combined.center.y() - 0.5).abs() < 1e-12);
        assert!((combined.center.z() - 0.5).abs() < 1e-12);
    }

    #[test]
    fn box_props_volume_and_com() {
        let props = analytic::box_props(2.0, 3.0, 4.0);
        assert!((props.mass - 24.0).abs() < 1e-12);
        assert!((props.center.x() - 1.0).abs() < 1e-12);
        assert!((props.center.y() - 1.5).abs() < 1e-12);
        assert!((props.center.z() - 2.0).abs() < 1e-12);
        // Ixx = 24/12 * (9 + 16) = 50
        assert!((props.inertia[0] - 50.0).abs() < 1e-12);
    }

    #[test]
    fn sphere_props_volume() {
        let props = analytic::sphere_props(1.0);
        let expected = 4.0 / 3.0 * std::f64::consts::PI;
        assert!((props.mass - expected).abs() < 1e-12);
        assert!((props.center.x()).abs() < 1e-12);
        assert!((props.center.y()).abs() < 1e-12);
        assert!((props.center.z()).abs() < 1e-12);
    }

    #[test]
    fn cylinder_props_volume_and_com() {
        let props = analytic::cylinder_props(1.0, 2.0);
        let expected_v = std::f64::consts::PI * 2.0;
        assert!((props.mass - expected_v).abs() < 1e-12);
        assert!((props.center.z() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn cone_full_volume() {
        let props = analytic::cone_props(1.0, 0.0, 3.0);
        let expected_v = std::f64::consts::PI * 3.0 / 3.0; // pi * h/3 * r^2
        assert!((props.mass - expected_v).abs() < 1e-12);
        // CoM of full cone at h/4 from base
        assert!((props.center.z() - 0.75).abs() < 1e-12);
    }

    #[test]
    fn torus_props_volume() {
        let props = analytic::torus_props(3.0, 1.0);
        let expected_v = 2.0 * std::f64::consts::PI * std::f64::consts::PI * 3.0;
        assert!((props.mass - expected_v).abs() < 1e-12);
    }

    #[test]
    fn box_surface_area() {
        let area = analytic::box_area(2.0, 3.0, 4.0);
        // 2*(6 + 12 + 8) = 52
        assert!((area - 52.0).abs() < 1e-12);
    }

    #[test]
    fn sphere_surface_area() {
        let area = analytic::sphere_area(2.0);
        let expected = 4.0 * std::f64::consts::PI * 4.0;
        assert!((area - expected).abs() < 1e-12);
    }

    #[test]
    fn inertia_matrix_symmetric() {
        let mut props = GProps::new();
        props.inertia = [10.0, 20.0, 30.0, 1.0, 2.0, 3.0];
        let mat = props.matrix_of_inertia();
        // Off-diagonal symmetry
        assert!((mat[0][1] - mat[1][0]).abs() < 1e-15);
        assert!((mat[0][2] - mat[2][0]).abs() < 1e-15);
        assert!((mat[1][2] - mat[2][1]).abs() < 1e-15);
        // Diagonal values
        assert!((mat[0][0] - 10.0).abs() < 1e-15);
        assert!((mat[1][1] - 20.0).abs() < 1e-15);
        assert!((mat[2][2] - 30.0).abs() < 1e-15);
    }

    #[test]
    fn bounding_box_unit_cube() {
        let mut topo = Topology::new();
        let solid = make_unit_cube_manifold(&mut topo);
        let aabb = bounding_box(&topo, solid).unwrap();
        // Unit cube at origin: min=(0,0,0), max=(1,1,1)
        assert!((aabb.min.x()).abs() < 1e-12);
        assert!((aabb.min.y()).abs() < 1e-12);
        assert!((aabb.min.z()).abs() < 1e-12);
        assert!((aabb.max.x() - 1.0).abs() < 1e-12);
        assert!((aabb.max.y() - 1.0).abs() < 1e-12);
        assert!((aabb.max.z() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn gauss_volume_matches_analytic() {
        let mut topo = Topology::new();
        let solid = make_unit_cube_manifold(&mut topo);
        let options = PropertiesOptions::default();
        let vol = solid_volume(&topo, solid, &options).unwrap();
        // Unit cube volume = 1.0
        assert!((vol - 1.0).abs() < 1e-10, "expected volume 1.0, got {vol}");
    }

    #[test]
    fn gauss_area_matches_analytic() {
        let mut topo = Topology::new();
        let solid = make_unit_cube_manifold(&mut topo);
        let options = PropertiesOptions::default();
        let area = solid_area(&topo, solid, &options).unwrap();
        // Unit cube surface area = 6.0
        assert!((area - 6.0).abs() < 1e-10, "expected area 6.0, got {area}");
    }

    #[test]
    fn gauss_com_matches_analytic() {
        let mut topo = Topology::new();
        let solid = make_unit_cube_manifold(&mut topo);
        let options = PropertiesOptions::default();
        let com = center_of_mass(&topo, solid, &options).unwrap();
        // Unit cube CoM at (0.5, 0.5, 0.5)
        assert!((com.x() - 0.5).abs() < 1e-10, "com.x = {}", com.x());
        assert!((com.y() - 0.5).abs() < 1e-10, "com.y = {}", com.y());
        assert!((com.z() - 0.5).abs() < 1e-10, "com.z = {}", com.z());
    }

    #[test]
    fn accumulator_default_is_zero() {
        let props = GProps::default();
        assert!((props.mass).abs() < 1e-15);
        assert!((props.center.x()).abs() < 1e-15);
        assert!((props.center.y()).abs() < 1e-15);
        assert!((props.center.z()).abs() < 1e-15);
        for &c in &props.inertia {
            assert!(c.abs() < 1e-15);
        }
    }

    // ── inner (cavity) shells are part of the boundary ──────────────────
    //
    // `solid_volume` / `solid_area` / `center_of_mass` integrate only the
    // faces of `Solid::outer_shell()`. A cavity lives in `inner_shells()`,
    // so its six walls contribute nothing: the void is measured as material
    // (volume too large) and the cavity's walls are missing from the area.
    // Every other traversal in this crate (`classify`, `validate`,
    // `distance`) already walks outer + inner shells; these three did not.

    /// One cube face: its four wire edges as (edge index, forward), the
    /// plane normal, and the plane offset.
    type CubeFaceSpec = ([(usize, bool); 4], Vec3, f64);

    /// An axis-aligned cube shell spanning `[ox, ox+size]` (etc.).
    ///
    /// `reversed` builds it with inward-pointing orientation — the sense a
    /// cavity shell carries, whose "outward" side faces the void. This is
    /// the same fixture `operations::measure` pins its cavity cases with.
    fn cube_shell(
        topo: &mut Topology,
        ox: f64,
        oy: f64,
        oz: f64,
        size: f64,
        reversed: bool,
    ) -> brepkit_topology::shell::ShellId {
        use brepkit_topology::edge::{Edge, EdgeCurve};
        use brepkit_topology::face::{Face, FaceSurface};
        use brepkit_topology::shell::Shell;
        use brepkit_topology::vertex::Vertex;
        use brepkit_topology::wire::{OrientedEdge, Wire};

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

    /// A cube of side `size` at `(ox, oy, oz)` and — optionally — a cavity
    /// cube inside it, registered as an inner shell.
    fn cube_with_cavity(topo: &mut Topology, cavity: Option<(f64, f64, f64, f64)>) -> SolidId {
        use brepkit_topology::solid::Solid;

        let outer = cube_shell(topo, 0.0, 0.0, 0.0, 4.0, false);
        let inner = cavity.map(|(cx, cy, cz, size)| cube_shell(topo, cx, cy, cz, size, true));
        topo.add_solid(Solid::new(outer, inner.into_iter().collect()))
    }

    /// Main case: a hollow cube — outer `[0,4]³` minus cavity `[1,3]³` —
    /// measures `4³ − 2³ = 56`, not 64.
    #[test]
    fn cavity_shell_subtracts_volume() {
        let mut topo = Topology::new();
        let solid = cube_with_cavity(&mut topo, Some((1.0, 1.0, 1.0, 2.0)));

        let v = solid_volume(&topo, solid, &PropertiesOptions::default()).unwrap();
        assert!(
            (v - 56.0).abs() < 1e-6,
            "hollow cube volume: got {v}, want 56"
        );
    }

    /// Control case: the same fixture with the cavity removed measures the
    /// full 64. Passing here while the case above fails localises the
    /// defect to the shell traversal, not to the fixture's winding.
    #[test]
    fn solid_cube_volume_ignores_no_shell() {
        let mut topo = Topology::new();
        let solid = cube_with_cavity(&mut topo, None);

        let v = solid_volume(&topo, solid, &PropertiesOptions::default()).unwrap();
        assert!(
            (v - 64.0).abs() < 1e-6,
            "solid cube volume: got {v}, want 64"
        );
    }

    /// The cavity's walls are boundary too: `6·16 + 6·4 = 120`.
    #[test]
    fn cavity_shell_adds_surface_area() {
        let mut topo = Topology::new();
        let solid = cube_with_cavity(&mut topo, Some((1.0, 1.0, 1.0, 2.0)));

        let a = solid_area(&topo, solid, &PropertiesOptions::default()).unwrap();
        assert!(
            (a - 120.0).abs() < 1e-6,
            "hollow cube area: got {a}, want 120"
        );
    }

    /// Control case: no cavity, so no inner walls — `6·16 = 96`.
    #[test]
    fn solid_cube_area_has_no_inner_walls() {
        let mut topo = Topology::new();
        let solid = cube_with_cavity(&mut topo, None);

        let a = solid_area(&topo, solid, &PropertiesOptions::default()).unwrap();
        assert!((a - 96.0).abs() < 1e-6, "solid cube area: got {a}, want 96");
    }

    /// Main case: an off-centre cavity shifts the CoM. Analytic value for
    /// outer `[0,4]³` minus a unit cavity centred at `(3,3,3)`:
    /// `(64·(2,2,2) − 1·(3,3,3)) / 63 = 125/63`.
    #[test]
    fn off_centre_cavity_shifts_center_of_mass() {
        let mut topo = Topology::new();
        let solid = cube_with_cavity(&mut topo, Some((2.5, 2.5, 2.5, 1.0)));

        let expected = 125.0 / 63.0;
        let com = center_of_mass(&topo, solid, &PropertiesOptions::default()).unwrap();
        assert!(
            (com.x() - expected).abs() < 1e-6,
            "hollow cube com.x: got {}, want {expected}",
            com.x()
        );
        assert!(
            (com.y() - expected).abs() < 1e-6,
            "hollow cube com.y: got {}, want {expected}",
            com.y()
        );
        assert!(
            (com.z() - expected).abs() < 1e-6,
            "hollow cube com.z: got {}, want {expected}",
            com.z()
        );
    }

    /// Control case: a centred cavity removes material symmetrically, so
    /// the CoM stays at the cube's centre `(2,2,2)`.
    #[test]
    fn centred_cavity_keeps_center_of_mass() {
        let mut topo = Topology::new();
        let solid = cube_with_cavity(&mut topo, Some((1.0, 1.0, 1.0, 2.0)));

        let com = center_of_mass(&topo, solid, &PropertiesOptions::default()).unwrap();
        assert!(
            (com.x() - 2.0).abs() < 1e-6,
            "centred cavity com.x: got {}, want 2",
            com.x()
        );
        assert!(
            (com.z() - 2.0).abs() < 1e-6,
            "centred cavity com.z: got {}, want 2",
            com.z()
        );
    }
}
