//! Pan and zoom: how the graph's own coordinates map to the screen.

/// The smallest zoom factor.
pub const MIN_ZOOM: f64 = 0.25;
/// The largest zoom factor.
pub const MAX_ZOOM: f64 = 4.0;

/// Where the canvas is panned to and how far it is zoomed in.
///
/// A graph point `(x, y)` is drawn at `(x * zoom + pan_x, y * zoom + pan_y)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
    pan_x: f64,
    pan_y: f64,
    zoom: f64,
}

impl Default for View {
    fn default() -> View {
        View {
            pan_x: 0.0,
            pan_y: 0.0,
            zoom: 1.0,
        }
    }
}

impl View {
    /// Horizontal offset of the graph origin on screen.
    pub fn pan_x(&self) -> f64 {
        self.pan_x
    }

    /// Vertical offset of the graph origin on screen.
    pub fn pan_y(&self) -> f64 {
        self.pan_y
    }

    /// How many screen pixels one graph unit covers.
    pub fn zoom(&self) -> f64 {
        self.zoom
    }

    /// Move the graph by `(dx, dy)` screen pixels.
    pub fn pan_by(&mut self, dx: f64, dy: f64) {
        self.pan_x += dx;
        self.pan_y += dy;
    }

    /// Multiply the zoom by `factor`, keeping the graph point under the screen
    /// point `(x, y)` where it is. The zoom stays between `MIN_ZOOM` and
    /// `MAX_ZOOM`.
    pub fn zoom_at(&mut self, x: f64, y: f64, factor: f64) {
        let zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
        let ratio = zoom / self.zoom;
        self.pan_x = x - (x - self.pan_x) * ratio;
        self.pan_y = y - (y - self.pan_y) * ratio;
        self.zoom = zoom;
    }

    /// The screen position of the graph point `(x, y)`.
    pub fn to_screen(&self, x: f64, y: f64) -> (f64, f64) {
        (x * self.zoom + self.pan_x, y * self.zoom + self.pan_y)
    }

    /// The graph point under the screen position `(x, y)`.
    pub fn to_graph(&self, x: f64, y: f64) -> (f64, f64) {
        ((x - self.pan_x) / self.zoom, (y - self.pan_y) / self.zoom)
    }
}
