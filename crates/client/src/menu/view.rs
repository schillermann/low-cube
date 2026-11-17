use glam::Vec3;
use world::MeshTarget;
use super::glyph::PixelStamp;
use super::version_stamp::VersionStamp;
use super::widget::ScreenBounds;

/// Focused interface for 2D orthographic menu views emitting quads into a `MeshTarget`.
pub trait MenuView {
    fn emit(&self, target: &mut dyn MeshTarget);
}

/// Decorator framing any `MenuView` inside a bordered slate dialog card with a centered gold title.
pub struct TitledCard<'a, V> {
    bounds: ScreenBounds,
    title: &'a str,
    title_y: f32,
    inner: V,
}

impl<'a, V: MenuView> TitledCard<'a, V> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(bounds: ScreenBounds, title: &'a str, title_y: f32, inner: V) -> Self {
        Self { bounds, title, title_y, inner }
    }
}

impl<V: MenuView> MenuView for TitledCard<'_, V> {
    fn emit(&self, target: &mut dyn MeshTarget) {
        let bg = Vec3::new(0.08, 0.10, 0.15);
        let rim = Vec3::new(0.32, 0.39, 0.50);
        let gold = Vec3::new(0.96, 0.82, 0.34);
        self.bounds.emit_framed(bg, rim, 0.008, target);
        PixelStamp::title(gold).emit_centered(self.title, 0.0, self.title_y, target);
        self.inner.emit(target);
    }
}

/// Decorator stamping the release Semantic Version in the bottom-right corner over any open `MenuView`.
pub struct VersionedView<V> {
    stamp: VersionStamp,
    inner: V,
}

impl<V: MenuView> VersionedView<V> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(stamp: VersionStamp, inner: V) -> Self {
        Self { stamp, inner }
    }

    /// Secondary constructor decorating a view with the current compile-time crate version.
    pub const fn current(inner: V) -> Self {
        Self::new(VersionStamp::current(), inner)
    }
}

impl<V: MenuView> MenuView for VersionedView<V> {
    fn emit(&self, target: &mut dyn MeshTarget) {
        self.inner.emit(target);
        self.stamp.emit(target);
    }
}
