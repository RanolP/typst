use super::*;

/// A container with a horizontal and vertical component.
#[derive(Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Spec<T> {
    /// The horizontal component.
    pub x: T,
    /// The vertical component.
    pub y: T,
}

impl<T> Spec<T> {
    /// Create a new instance from the two components.
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    /// Create a new instance with two equal components.
    pub fn splat(v: T) -> Self
    where
        T: Clone,
    {
        Self { x: v.clone(), y: v }
    }

    /// Maps the individual fields with `f`.
    pub fn map<F, U>(self, mut f: F) -> Spec<U>
    where
        F: FnMut(T) -> U,
    {
        Spec { x: f(self.x), y: f(self.y) }
    }

    /// Convert to the generic representation.
    pub fn to_gen(self, block: SpecAxis) -> Gen<T> {
        match block {
            SpecAxis::Horizontal => Gen::new(self.y, self.x),
            SpecAxis::Vertical => Gen::new(self.x, self.y),
        }
    }

    /// Compares if this instance's field are equal to that of another with
    /// respect to `eq`.
    pub fn eq_by<U, F>(&self, other: &Spec<U>, eq: F) -> bool
    where
        F: Fn(&T, &U) -> bool,
    {
        eq(&self.x, &other.x) && eq(&self.y, &other.y)
    }
}

impl Spec<Length> {
    /// The zero value.
    pub fn zero() -> Self {
        Self { x: Length::zero(), y: Length::zero() }
    }

    /// Convert to a point.
    pub fn to_point(self) -> Point {
        Point::new(self.x, self.y)
    }

    /// Convert to a size.
    pub fn to_size(self) -> Size {
        Size::new(self.x, self.y)
    }
}

impl<T> Spec<Option<T>> {
    /// Unwrap the individual fields.
    pub fn unwrap_or(self, other: Spec<T>) -> Spec<T> {
        Spec {
            x: self.x.unwrap_or(other.x),
            y: self.y.unwrap_or(other.y),
        }
    }
}

impl<T> Get<SpecAxis> for Spec<T> {
    type Component = T;

    fn get(self, axis: SpecAxis) -> T {
        match axis {
            SpecAxis::Horizontal => self.x,
            SpecAxis::Vertical => self.y,
        }
    }

    fn get_mut(&mut self, axis: SpecAxis) -> &mut T {
        match axis {
            SpecAxis::Horizontal => &mut self.x,
            SpecAxis::Vertical => &mut self.y,
        }
    }
}

impl<T: Debug> Debug for Spec<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Spec({:?}, {:?})", self.x, self.y)
    }
}

/// The two specific layouting axes.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SpecAxis {
    /// The horizontal layouting axis.
    Horizontal,
    /// The vertical layouting axis.
    Vertical,
}

impl SpecAxis {
    /// The direction with the given positivity for this axis.
    pub fn dir(self, positive: bool) -> Dir {
        match (self, positive) {
            (Self::Horizontal, true) => Dir::LTR,
            (Self::Horizontal, false) => Dir::RTL,
            (Self::Vertical, true) => Dir::TTB,
            (Self::Vertical, false) => Dir::BTT,
        }
    }

    /// The other axis.
    pub fn other(self) -> Self {
        match self {
            Self::Horizontal => Self::Vertical,
            Self::Vertical => Self::Horizontal,
        }
    }
}
