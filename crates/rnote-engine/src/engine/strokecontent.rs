// Imports
use crate::document::Background;
use crate::render::Svg;
use crate::strokes::Stroke;
use crate::Drawable;
use p2d::bounding_volume::{Aabb, BoundingVolume};
use rnote_compose::shapes::Shapeable;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::warn;

/// Represents a collection of strokes with optional bounds and background.
///
/// Used for exporting and clipboard operations.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename = "stroke_content")]
pub struct StrokeContent {
    /// The strokes contained in this `StrokeContent`.
    #[serde(rename = "strokes")]
    pub strokes: Vec<Arc<Stroke>>,
    /// The optional bounding box of the content. If `None`, the bounds are calculated from the strokes.
    #[serde(rename = "bounds")]
    pub bounds: Option<Aabb>,
    /// The optional background associated with the content.
    #[serde(rename = "background")]
    pub background: Option<Background>,
}

impl StrokeContent {
    /// The MIME type used to identify `StrokeContent` data.
    pub const MIME_TYPE: &'static str = "application/rnote-stroke-content";
    /// The margin used when exporting content to the clipboard.
    pub const CLIPBOARD_EXPORT_MARGIN: f64 = 6.0;

    /// Creates a new `StrokeContent` with the specified bounds.
    pub fn with_bounds(mut self, bounds: Option<Aabb>) -> Self {
        self.bounds = bounds;
        self
    }

    /// Creates a new `StrokeContent` with the specified strokes.
    pub fn with_strokes(mut self, strokes: Vec<Arc<Stroke>>) -> Self {
        self.strokes = strokes;
        self
    }

    /// Creates a new `StrokeContent` with the specified background.
    pub fn with_background(mut self, background: Option<Background>) -> Self {
        self.background = background;
        self
    }

    /// Calculates the bounding box of the `StrokeContent`.
    ///
    /// If `bounds` is `Some`, it is returned directly. Otherwise, the bounding box is calculated
    /// by merging the bounding boxes of all strokes. Returns `None` if there are no strokes and
    /// `bounds` is `None`.
    pub fn bounds(&self) -> Option<Aabb> {
        if self.bounds.is_some() {
            return self.bounds;
        }
        if self.strokes.is_empty() {
            return None;
        }
        Some(
            self.strokes
                .iter()
                .map(|s| s.bounds())
                .fold(Aabb::new_invalid(), |acc, x| acc.merged(&x)),
        )
    }

    /// Returns the size of the `StrokeContent`'s bounding box, if available.
    pub fn size(&self) -> Option<na::Vector2<f64>> {
        self.bounds().map(|b| b.extents())
    }

    /// Generates an SVG representation of the `StrokeContent`.
    ///
    /// The generated SVG will have its bounds moved to the origin (0, 0).
    ///
    /// # Arguments
    ///
    /// * `draw_background` - Whether to draw the background in the SVG.
    /// * `draw_pattern` - Whether to draw the background pattern (if applicable).
    /// * `optimize_printing` - Whether to apply optimizations for printing.
    /// * `margin` - The margin to add around the content.
    ///
    /// # Returns
    ///
    /// An `Svg` object representing the content, or `None` if the content has no bounds.
    pub fn generate_svg(
        &self,
        draw_background: bool,
        draw_pattern: bool,
        optimize_printing: bool,
        margin: f64,
    ) -> anyhow::Result<Option<Svg>> {
        let Some(bounds_loosened) = self.bounds().map(|b| b.loosened(margin)) else {
            return Ok(None);
        };
        let mut svg = Svg::gen_with_cairo(
            |cairo_cx| {
                self.draw_to_cairo(
                    cairo_cx,
                    draw_background,
                    draw_pattern,
                    optimize_printing,
                    margin,
                    1.0,
                )
            },
            bounds_loosened,
        )?;
        // The simplification also moves the bounds to mins: [0.0, 0.0], maxs: extents
        if let Err(e) = svg.simplify() {
            warn!("Simplifying Svg while generating StrokeContent Svg failed, Err: {e:?}");
        };
        Ok(Some(svg))
    }

    /// Draws the `StrokeContent` to a Cairo context.
    ///
    /// # Arguments
    ///
    /// * `cairo_cx` - The Cairo context to draw to.
    /// * `draw_background` - Whether to draw the background.
    /// * `draw_pattern` - Whether to draw the background pattern (if applicable).
    /// * `optimize_printing` - Whether to apply optimizations for printing.
    ///                           When true it draws only the darkest color of a vector stroke,
    ///                           if the stroke is not inside of an image.
    /// * `margin` - The margin to add around the content when drawing.
    /// * `image_scale` - The scaling factor for images.
    pub fn draw_to_cairo(
        &self,
        cairo_cx: &cairo::Context,
        draw_background: bool,
        draw_pattern: bool,
        optimize_printing: bool,
        margin: f64,
        image_scale: f64,
    ) -> anyhow::Result<()> {
        let Some(bounds) = self.bounds() else {
            return Ok(());
        };
        let bounds_loosened = bounds.loosened(margin);

        cairo_cx.save()?;
        cairo_cx.rectangle(
            bounds_loosened.mins[0],
            bounds_loosened.mins[1],
            bounds_loosened.extents()[0],
            bounds_loosened.extents()[1],
        );
        cairo_cx.clip();

        if draw_background {
            if let Some(background) = &self.background {
                background.draw_to_cairo(
                    cairo_cx,
                    bounds_loosened,
                    draw_pattern,
                    optimize_printing,
                )?;
            }
        }

        cairo_cx.restore()?;
        cairo_cx.save()?;
        cairo_cx.rectangle(
            bounds.mins[0],
            bounds.mins[1],
            bounds.extents()[0],
            bounds.extents()[1],
        );
        cairo_cx.clip();

        let image_bounds = self
            .strokes
            .iter()
            .filter_map(|stroke| match stroke.as_ref() {
                Stroke::BitmapImage(image) => Some(image.rectangle.bounds()),
                Stroke::VectorImage(image) => Some(image.rectangle.bounds()),
                _ => None,
            })
            .collect::<Vec<Aabb>>();

        for stroke in self.strokes.iter() {
            let stroke_bounds = stroke.bounds();

            if optimize_printing
                && image_bounds
                    .iter()
                    .all(|bounds| !bounds.contains(&stroke_bounds))
            {
                // Using the stroke's bounds instead of hitboxes works for inclusion.
                // If this is changed to intersection, all hitboxes must be checked individually.

                let mut darkest_color_stroke = stroke.as_ref().clone();
                darkest_color_stroke.set_to_darkest_color();

                darkest_color_stroke.draw_to_cairo(cairo_cx, image_scale)?;
            } else {
                stroke.draw_to_cairo(cairo_cx, image_scale)?;
            }
        }

        cairo_cx.restore()?;

        Ok(())
    }
}
