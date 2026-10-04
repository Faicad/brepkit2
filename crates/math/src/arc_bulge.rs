//! The sliver between one chord of a polygonised arc and the arc itself.

use crate::curves::Circle3D;
use crate::vec::{Point3, Vec3};

/// The region between a chord of a polygonised circular arc and the arc.
///
/// A polygon can only approach a curved boundary from one side: the sampled
/// points lie *on* the arc, so every chord cuts inside it. Polygon and arc
/// therefore enclose different regions, and they disagree on exactly these
/// slivers — for a minor arc (which is what sampling produces) the sliver is
/// the part of the disc lying beyond the chord.
///
/// Since the true region and the polygon differ by precisely the union of
/// these slivers, a containment answer built on the polygon is exact once it
/// is inverted for a point falling inside one. Carrying the slivers lets the
/// polygon stay cheap (a few dozen points) while the test stays exact for
/// circular boundaries — no amount of extra sampling gets there, because the
/// error is only ever shrunk, never removed.
///
/// This lives in `math` (rather than beside each polygoniser) because more
/// than one crate polygonises wires, and a silently different sliver test in
/// one of them produces a disagreement with no build-time signal.
#[derive(Debug, Clone, Copy)]
pub struct ArcBulge {
    /// Midpoint of the chord.
    chord_mid: Point3,
    /// Unit vector from the arc's centre through the chord midpoint.
    outward: Vec3,
    center: Point3,
    radius_sq: f64,
}

impl ArcBulge {
    /// The sliver bounded by the chord `a → b` and the arc of `circle`.
    ///
    /// Returns `None` when the chord degenerates (it passes through the
    /// centre, so "beyond the chord" has no meaning) — such a chord stands
    /// for a semicircle, which sampling a circle at a few dozen points never
    /// produces.
    #[must_use]
    pub fn new(a: Point3, b: Point3, circle: &Circle3D) -> Option<Self> {
        let center = circle.center();
        let chord_mid = Point3::new(
            (a.x() + b.x()) * 0.5,
            (a.y() + b.y()) * 0.5,
            (a.z() + b.z()) * 0.5,
        );
        let out = chord_mid - center;
        let len = out.length();
        if len < 1e-12 {
            return None;
        }
        Some(Self {
            chord_mid,
            outward: out * (1.0 / len),
            center,
            radius_sq: circle.radius() * circle.radius(),
        })
    }

    /// Whether `p` lies in the sliver: inside the circle, beyond the chord.
    #[must_use]
    pub fn contains(&self, p: Point3) -> bool {
        (p - self.center).length_squared() < self.radius_sq
            && (p - self.chord_mid).dot(self.outward) > 0.0
    }
}
