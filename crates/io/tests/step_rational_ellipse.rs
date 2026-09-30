//! STEP round-trip regressions for rational B-spline weights (F-01) and
//! ELLIPSE placement axes (F-02).
//!
//! Both write a minimal solid and read it back, comparing against the exact
//! analytic geometry — no external reference values are involved.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use brepkit_io::step::reader::read_step;
use brepkit_io::step::writer::write_step;
use brepkit_math::curves::Ellipse3D;
use brepkit_math::nurbs::NurbsCurve;
use brepkit_math::vec::{Point3, Vec3};
use brepkit_topology::Topology;
use brepkit_topology::edge::{Edge, EdgeCurve};
use brepkit_topology::face::{Face, FaceSurface};
use brepkit_topology::shell::Shell;
use brepkit_topology::solid::{Solid, SolidId};
use brepkit_topology::vertex::Vertex;
use brepkit_topology::wire::{OrientedEdge, Wire};

const TOL: f64 = 1e-7;

/// A single planar face (z = 0) bounded by `wire`, wrapped in a solid.
fn solid_from_wire(topo: &mut Topology, wire: brepkit_topology::wire::WireId) -> SolidId {
    let face = topo.add_face(Face::new(
        wire,
        vec![],
        FaceSurface::Plane {
            normal: Vec3::new(0.0, 0.0, 1.0),
            d: 0.0,
        },
    ));
    let shell = topo.add_shell(Shell::new(vec![face]).expect("shell"));
    topo.add_solid(Solid::new(shell, vec![]))
}

/// Every edge curve of the solid's boundary, in wire order.
fn edge_curves(topo: &Topology, solid: SolidId) -> Vec<EdgeCurve> {
    let shell_id = topo.solid(solid).unwrap().outer_shell();
    let shell = topo.shell(shell_id).unwrap();
    let mut curves = Vec::new();
    for &fid in shell.faces() {
        let face = topo.face(fid).unwrap();
        let wire = topo.wire(face.outer_wire()).unwrap();
        for oe in wire.edges() {
            curves.push(topo.edge(oe.edge()).unwrap().curve().clone());
        }
    }
    curves
}

/// F-02: an ELLIPSE's major axis comes from the placement's reference
/// direction, not from an arbitrary frame built off the plane normal.
#[test]
fn ellipse_round_trip_preserves_major_axis() {
    let ref_dir = Vec3::new(1.0, 1.0, 0.0); // 45° in the xy-plane
    let ellipse = Ellipse3D::new_with_ref(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        3.0,
        1.0,
        ref_dir,
    )
    .expect("ellipse");

    let seam = Point3::new(3.0 / 2_f64.sqrt(), 3.0 / 2_f64.sqrt(), 0.0);
    let mut topo = Topology::new();
    let v = topo.add_vertex(Vertex::new(seam, TOL));
    let e = topo.add_edge(Edge::new(v, v, EdgeCurve::Ellipse(ellipse)));
    let wire = topo.add_wire(Wire::new(vec![OrientedEdge::new(e, true)], true).unwrap());
    let solid = solid_from_wire(&mut topo, wire);

    let text = write_step(&topo, &[solid]).expect("write");
    let mut read_topo = Topology::new();
    let solids = read_step(&text, &mut read_topo).expect("read");
    let curves = edge_curves(&read_topo, solids[0]);

    let Some(EdgeCurve::Ellipse(got)) = curves.first() else {
        panic!("expected an ELLIPSE edge, got {curves:?}");
    };
    assert!(
        (got.semi_major() - 3.0).abs() < 1e-9 && (got.semi_minor() - 1.0).abs() < 1e-9,
        "semi-axes changed: {} / {}",
        got.semi_major(),
        got.semi_minor()
    );
    let expected = ref_dir.normalize().unwrap();
    let axis = got.u_axis();
    assert!(
        (axis - expected).length() < 1e-6 || (axis + expected).length() < 1e-6,
        "major axis lost: got {axis:?}, want {expected:?}"
    );
    // The long axis must actually carry the 3.0 extent.
    let on_curve = got.evaluate(0.0);
    assert!(
        (on_curve - Point3::new(0.0, 0.0, 0.0)).length() > 2.9,
        "evaluate(0) should sit on the major axis end, got {on_curve:?}"
    );
}

/// F-01: a rational NURBS curve keeps its weights through a STEP round-trip.
///
/// The boundary is an exact quarter circle of radius 1 — the classic rational
/// quadratic (weights `1, √2/2, 1`). Dropping the weights turns it into a
/// parabola that bulges ~6% off the arc.
#[test]
fn rational_nurbs_round_trip_preserves_weights() {
    let sqrt2_2 = 2_f64.sqrt() / 2.0;
    let arc = NurbsCurve::new(
        2,
        vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        vec![
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ],
        vec![1.0, sqrt2_2, 1.0],
    )
    .expect("arc");

    let mut topo = Topology::new();
    let p_end = topo.add_vertex(Vertex::new(Point3::new(1.0, 0.0, 0.0), TOL));
    let p_mid = topo.add_vertex(Vertex::new(Point3::new(0.0, 1.0, 0.0), TOL));
    let p_origin = topo.add_vertex(Vertex::new(Point3::new(0.0, 0.0, 0.0), TOL));
    let arc_edge = topo.add_edge(Edge::new(p_end, p_mid, EdgeCurve::NurbsCurve(arc)));
    let e1 = topo.add_edge(Edge::new(p_mid, p_origin, EdgeCurve::Line));
    let e2 = topo.add_edge(Edge::new(p_origin, p_end, EdgeCurve::Line));
    let wire = topo.add_wire(
        Wire::new(
            vec![
                OrientedEdge::new(arc_edge, true),
                OrientedEdge::new(e1, true),
                OrientedEdge::new(e2, true),
            ],
            true,
        )
        .unwrap(),
    );
    let solid = solid_from_wire(&mut topo, wire);

    let text = write_step(&topo, &[solid]).expect("write");
    let mut read_topo = Topology::new();
    let solids = read_step(&text, &mut read_topo).expect("read");
    let curves = edge_curves(&read_topo, solids[0]);

    let Some(EdgeCurve::NurbsCurve(got)) = curves.first() else {
        panic!("expected a NURBS edge, got {curves:?}");
    };
    let (u0, u1) = got.domain();
    for i in 1..4 {
        let t = u0 + (u1 - u0) * (i as f64) / 4.0;
        let p = got.evaluate(t);
        let r = p.x().hypot(p.y());
        assert!(
            (r - 1.0).abs() < 1e-3,
            "rational arc off the circle at t={t}: radius {r} (weights dropped?)"
        );
    }
}
