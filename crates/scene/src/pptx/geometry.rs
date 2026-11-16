//! Sizes: the stage in CSS pixels on one side, English Metric Units on the other.

use crate::model::Size;

/// The width of every slide, in EMU: 13.333 inches, PowerPoint's widescreen.
pub(crate) const SLIDE_WIDTH: i64 = 12_192_000;

const EMU_PER_POINT: f64 = 12_700.0;

/// A rectangle on the stage, in stage pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Frame {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// The stage of one deck and its scale to a slide.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Stage {
    pub width: f64,
    pub height: f64,
}

impl Stage {
    /// The stage of a deck of `size`.
    pub(crate) fn of(size: Size) -> Self {
        Stage { width: f64::from(size.width.max(1)), height: f64::from(size.height.max(1)) }
    }

    /// `pixels` of the stage, in EMU.
    pub(crate) fn emu(&self, pixels: f64) -> i64 {
        (pixels * SLIDE_WIDTH as f64 / self.width).round() as i64
    }

    /// The slide height in EMU: the stage's ratio at [`SLIDE_WIDTH`].
    pub(crate) fn slide_height(&self) -> i64 {
        self.emu(self.height)
    }

    /// A font of `pixels` on the stage, in hundredths of a point.
    pub(crate) fn font(&self, pixels: f64) -> i64 {
        ((self.emu(pixels) as f64 / EMU_PER_POINT * 100.0).round() as i64).max(100)
    }

    /// The area content may use: the stage less the theme's margins.
    pub(crate) fn content(&self) -> Frame {
        let (side, top) = (self.width * 0.0625, self.height * 0.09);
        Frame { x: side, y: top, width: self.width - 2.0 * side, height: self.height - 2.0 * top }
    }

    /// `<a:off/><a:ext/>` for `frame`.
    pub(crate) fn xfrm(&self, frame: &Frame) -> String {
        format!(
            "<a:off x=\"{}\" y=\"{}\"/><a:ext cx=\"{}\" cy=\"{}\"/>",
            self.emu(frame.x),
            self.emu(frame.y),
            self.emu(frame.width).max(1),
            self.emu(frame.height).max(1)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_full_hd_stage_is_a_widescreen_slide_with_half_point_pixels() {
        let stage = Stage::of(Size { width: 1920, height: 1080 });
        assert_eq!((stage.emu(1920.0), stage.slide_height()), (12_192_000, 6_858_000));
        assert_eq!((stage.font(40.0), stage.font(104.0)), (2000, 5200));
        assert_eq!(Stage::of(Size { width: 1024, height: 768 }).slide_height(), 9_144_000);
    }
}
