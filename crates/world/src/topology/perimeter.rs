use glam::Vec3;
use crate::frame::SurfaceFrame;

/// The four outer perimeter edge boundaries in local tangent space (+U, -U, +V, -V).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PerimeterEdge {
    PositiveU,
    NegativeU,
    PositiveV,
    NegativeV,
}

impl PerimeterEdge {
    /// Opposite perimeter edge along the same coordinate axis.
    pub fn opposite(&self) -> Self {
        match self {
            Self::PositiveU => Self::NegativeU,
            Self::NegativeU => Self::PositiveU,
            Self::PositiveV => Self::NegativeV,
            Self::NegativeV => Self::PositiveV,
        }
    }

    /// All four outer perimeter edge headings.
    pub fn all() -> [Self; 4] {
        [
            Self::PositiveU,
            Self::NegativeU,
            Self::PositiveV,
            Self::NegativeV,
        ]
    }

    /// Dense ordinal index (0..4) for canonical array slot lookup.
    pub fn index(&self) -> usize {
        match self {
            Self::PositiveU => 0,
            Self::NegativeU => 1,
            Self::PositiveV => 2,
            Self::NegativeV => 3,
        }
    }
}

/// Domain object encapsulating the classification of coordinates against a face perimeter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PerimeterCrossing {
    Interior,
    Crossing {
        edge: PerimeterEdge,
        overshoot: f32,
    },
}

impl PerimeterCrossing {
    /// True if coordinates lie within or on the perimeter boundary.
    pub fn is_interior(&self) -> bool {
        matches!(self, Self::Interior)
    }

    /// True if coordinates cross beyond an outer perimeter boundary.
    pub fn is_crossing(&self) -> bool {
        matches!(self, Self::Crossing { .. })
    }

    /// Dispatches behavior polymorphically without exposing raw internal state.
    pub fn dispatch<T>(
        &self,
        on_interior: impl FnOnce() -> T,
        on_crossing: impl FnOnce(PerimeterEdge, f32) -> T,
    ) -> T {
        match self {
            Self::Interior => on_interior(),
            Self::Crossing { edge, overshoot } => on_crossing(*edge, *overshoot),
        }
    }

    /// Exceeded perimeter edge, if any.
    pub fn edge(&self) -> Option<PerimeterEdge> {
        match self {
            Self::Interior => None,
            Self::Crossing { edge, .. } => Some(*edge),
        }
    }

    /// Distance exceeding the perimeter boundary.
    pub fn overshoot(&self) -> f32 {
        match self {
            Self::Interior => 0.0,
            Self::Crossing { overshoot, .. } => *overshoot,
        }
    }
}

/// Encapsulated four-edged perimeter boundary of a planar face in tangent space.
#[derive(Clone, Copy, Debug)]
pub struct FacePerimeter {
    u_limit: f32,
    v_limit: f32,
}

impl FacePerimeter {
    /// Primary constructor: assigns boundary threshold fields directly.
    pub fn new(u_limit: f32, v_limit: f32) -> Self {
        Self { u_limit, v_limit }
    }

    /// Secondary constructor for a square face perimeter with uniform half size.
    pub fn square(half_size: f32) -> Self {
        Self::new(half_size, half_size)
    }

    /// Evaluates the perimeter crossing state for given tangential coordinates.
    pub fn crossing_at(&self, u: f32, v: f32) -> PerimeterCrossing {
        let u_excess = u.abs() - self.u_limit;
        let v_excess = v.abs() - self.v_limit;

        if u_excess <= 0.0 && v_excess <= 0.0 {
            return PerimeterCrossing::Interior;
        }

        if u_excess >= v_excess {
            let edge = if u > 0.0 {
                PerimeterEdge::PositiveU
            } else {
                PerimeterEdge::NegativeU
            };
            PerimeterCrossing::Crossing {
                edge,
                overshoot: u_excess,
            }
        } else {
            let edge = if v > 0.0 {
                PerimeterEdge::PositiveV
            } else {
                PerimeterEdge::NegativeV
            };
            PerimeterCrossing::Crossing {
                edge,
                overshoot: v_excess,
            }
        }
    }

    /// Evaluates the perimeter crossing state directly from a 3D spatial position on a surface frame.
    pub fn crossing_point(&self, frame: &SurfaceFrame, point: Vec3) -> PerimeterCrossing {
        let (u, v) = frame.decompose(point);
        self.crossing_at(u, v)
    }

    /// Determines whether tangential coordinates lie within or on the perimeter bounds.
    pub fn contains(&self, u: f32, v: f32) -> bool {
        u.abs() <= self.u_limit && v.abs() <= self.v_limit
    }

    /// Determines whether a 3D spatial position lies within the face surface perimeter bounds.
    pub fn contains_point(&self, frame: &SurfaceFrame, point: Vec3) -> bool {
        let (u, v) = frame.decompose(point);
        self.contains(u, v)
    }

    /// Clamps tangential coordinates to remain within the perimeter.
    pub fn clamp(&self, u: f32, v: f32) -> (f32, f32) {
        (
            u.clamp(-self.u_limit, self.u_limit),
            v.clamp(-self.v_limit, self.v_limit),
        )
    }
}
