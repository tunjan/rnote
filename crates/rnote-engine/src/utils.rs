// Imports
use crate::fileformats::xoppformat;
use geo::line_string;
use p2d::bounding_volume::Aabb;
use rnote_compose::Color;
use std::ops::Range;

/// Returns the current crate version as defined in `Cargo.toml`.
pub const fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Converts an `xoppformat::XoppColor` to an internal `Color` representation.
///
/// `xoppformat::XoppColor` components are in the range 0-255.
/// `Color` components are in the range 0.0-1.0.
pub fn color_from_xopp(xopp_color: xoppformat::XoppColor) -> Color {
    Color {
        r: f64::from(xopp_color.red) / 255.0,
        g: f64::from(xopp_color.green) / 255.0,
        b: f64::from(xopp_color.blue) / 255.0,
        a: f64::from(xopp_color.alpha) / 255.0,
    }
}

/// Converts an internal `Color` representation to an `xoppformat::XoppColor`.
///
/// `Color` components are in the range 0.0-1.0.
/// `xoppformat::XoppColor` components are in the range 0-255.
pub fn xoppcolor_from_color(color: Color) -> xoppformat::XoppColor {
    xoppformat::XoppColor {
        red: (color.r * 255.0).floor() as u8,
        green: (color.g * 255.0).floor() as u8,
        blue: (color.b * 255.0).floor() as u8,
        alpha: (color.a * 255.0).floor() as u8,
    }
}

/// Returns the current local date and time formatted as "YYYY-MM-DD_HH:MM:SS".
pub fn now_formatted_string() -> String {
    chrono::Local::now().format("%Y-%m-%d_%H:%M:%S").to_string()
}

/// Generates a filename for a specific page of a document.
///
/// The filename is formatted as "{file_stem_name} - Page {i:02}", where `i` is the page number.
pub fn format_page_filename(file_stem_name: &str, i: usize) -> String {
    file_stem_name.to_string() + &format!(" - Page {i:02}")
}

/// Converts a value from one DPI to another.
///
/// # Arguments
///
/// * `value` - The value to convert.
/// * `current_dpi` - The current DPI of the value.
/// * `target_dpi` - The target DPI to convert to.
///
/// # Returns
///
/// The converted value in the target DPI.
pub fn convert_value_dpi(value: f64, current_dpi: f64, target_dpi: f64) -> f64 {
    (value / current_dpi) * target_dpi
}

/// Converts a 2D coordinate from one DPI to another.
///
/// # Arguments
///
/// * `coord` - The coordinate to convert.
/// * `current_dpi` - The current DPI of the coordinate.
/// * `target_dpi` - The target DPI to convert to.
///
/// # Returns
///
/// The converted coordinate in the target DPI.
pub fn convert_coord_dpi(
    coord: na::Vector2<f64>,
    current_dpi: f64,
    target_dpi: f64,
) -> na::Vector2<f64> {
    (coord / current_dpi) * target_dpi
}

#[cfg(feature = "ui")]
/// Converts an `rnote_compose::Transform` to a `gtk4::gsk::Transform`.
pub fn transform_to_gsk(transform: &rnote_compose::Transform) -> gtk4::gsk::Transform {
    gtk4::gsk::Transform::new().matrix(>k4::graphene::Matrix::from_2d(
        transform.affine[(0, 0)],
        transform.affine[(1, 0)],
        transform.affine[(0, 1)],
        transform.affine[(1, 1)],
        transform.affine[(0, 2)],
        transform.affine[(1, 2)],
    ))
}

/// Converts a `p2d::bounding_volume::Aabb` to a `geo::Polygon<f64>`.
pub fn p2d_aabb_to_geo_polygon(aabb: Aabb) -> geo::Polygon<f64> {
    let line_string = line_string![
        (x: aabb.mins[0], y: aabb.mins[1]),
        (x: aabb.maxs[0], y: aabb.mins[1]),
        (x: aabb.maxs[0], y: aabb.maxs[1]),
        (x: aabb.mins[0], y: aabb.maxs[1]),
        (x: aabb.mins[0], y: aabb.mins[1]),
    ];
    geo::Polygon::new(line_string, vec![])
}

/// Returns a range where the start is always less than or equal to the end.
pub fn positive_range<I>(first: I, second: I) -> Range<I>
where
    I: PartialOrd,
{
    if first < second {
        first..second
    } else {
        second..first
    }
}

/// (De)serializes a [glib::Bytes] with base64 encoding.
pub mod glib_bytes_base64 {
    use serde::{Deserializer, Serializer};

    /// Serializes a [`Vec<u8>`] as base64 encoded.
    pub fn serialize<S: Serializer>(v: &glib::Bytes, s: S) -> Result<S::Ok, S::Error> {
        rnote_compose::serialize::sliceu8_base64::serialize(v, s)
    }

    /// Deserializes base64 encoded [glib::Bytes].
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<glib::Bytes, D::Error> {
        rnote_compose::serialize::sliceu8_base64::deserialize(d).map(glib::Bytes::from_owned)
    }
}
