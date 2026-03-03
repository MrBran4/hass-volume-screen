use core::fmt::Write;
use embedded_graphics::Drawable;
use embedded_graphics::primitives::Primitive;
use embedded_graphics::primitives::PrimitiveStyle;
use embedded_graphics::{
    geometry::Point, pixelcolor::Rgb565, prelude::RgbColor, primitives::Circle,
};

use heapless::String;
use u8g2_fonts::{
    FontRenderer,
    fonts::{u8g2_font_logisoso20_tr, u8g2_font_logisoso46_tr},
    types::{FontColor, HorizontalAlignment},
};

// Initialize the font renderer with the desired font
pub const FONT_LITTLE: FontRenderer = FontRenderer::new::<u8g2_font_logisoso20_tr>();
pub const FONT_BIG: FontRenderer = FontRenderer::new::<u8g2_font_logisoso46_tr>();

pub async fn draw_circle(display: &mut super::Display, centre: Point, diameter: u32) {
    Circle::with_center(centre, diameter)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
        .draw(display)
        .expect("Couldn't draw circle to screen");
}

/// Draw a reading to 0dp + '%' to the display.
pub async fn draw_percentage(
    display: &mut super::Display,
    value: f32,
    origin: Point,
    horizontal_align: HorizontalAlignment,
) {
    let mut new_buf = String::<8>::new();
    write!(&mut new_buf, "{:.0}%", value).unwrap();

    FONT_BIG
        .render_aligned(
            new_buf.as_str(),
            origin,
            u8g2_fonts::types::VerticalPosition::Top,
            horizontal_align,
            FontColor::Transparent(Rgb565::WHITE),
            display,
        )
        .expect("couldn't draw text to screen");
}

/// Draw a literal string
pub async fn draw_text(
    display: &mut super::Display,
    value: &str,
    origin: Point,
    horizontal_align: HorizontalAlignment,
) {
    FONT_LITTLE
        .render_aligned(
            value,
            origin,
            u8g2_fonts::types::VerticalPosition::Top,
            horizontal_align,
            FontColor::Transparent(Rgb565::WHITE),
            display,
        )
        .expect("couldn't draw text to screen");
}
