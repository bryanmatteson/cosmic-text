use std::path::PathBuf;

use cosmic_text::{
    fontdb, Attrs, Buffer, Family, FontSystem, Metrics, OpticalSize, Shaping, SwashCache,
};

/// A font system with only Inter Variable, whose `opsz` axis runs from 14
/// (the text design, its default) to 32 (the display design).
fn inter() -> FontSystem {
    let repo_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let fonts_path = PathBuf::from(&repo_dir).join("fonts");
    let mut db = fontdb::Database::new();
    db.load_font_data(std::fs::read(fonts_path.join("InterVariable.ttf")).unwrap());
    FontSystem::new_with_locale_and_db("en-US".into(), db)
}

/// The width of `text` shaped at 24 pixels, and its first glyph's cache key.
fn shape(
    font_system: &mut FontSystem,
    text: &str,
    optical_size: Option<f32>,
) -> (f32, cosmic_text::CacheKey) {
    let mut buffer = Buffer::new(font_system, Metrics::new(24.0, 30.0));
    let mut buffer = buffer.borrow_with(font_system);
    let mut attrs = Attrs::new().family(Family::Name("Inter Variable"));
    if let Some(size) = optical_size {
        attrs = attrs.optical_size(size);
    }
    buffer.set_size(None, None);
    buffer.set_text(text, &attrs, Shaping::Advanced, None);
    buffer.shape_until_scroll(true);
    let run = buffer.layout_runs().next().expect("a run");
    let key = run.glyphs[0].physical((0.0, 0.0), 1.0).cache_key;
    (run.line_w, key)
}

/// The optical size reaches shaping: Inter's display design is set tighter
/// than its text design, and no optical size keeps the font's default.
#[test]
fn optical_size_changes_advances() {
    let mut font_system = inter();
    let text = "Hamburgefonstiv";
    let (default, _) = shape(&mut font_system, text, None);
    let (text_size, _) = shape(&mut font_system, text, Some(14.0));
    let (display, _) = shape(&mut font_system, text, Some(32.0));
    assert_eq!(default, text_size, "no optical size is the default, 14");
    assert!(
        display < text_size - 1.0,
        "display {display} should be narrower than text {text_size}"
    );
}

/// The optical size reaches the glyph cache key and the rasterizer, so the
/// two designs are cached and drawn apart.
#[test]
fn optical_size_reaches_the_rasterizer() {
    let mut font_system = inter();
    let (_, text_key) = shape(&mut font_system, "a", Some(14.0));
    let (_, display_key) = shape(&mut font_system, "a", Some(32.0));
    assert_eq!(text_key.font_optical_size, Some(OpticalSize(14.0)));
    assert_eq!(display_key.font_optical_size, Some(OpticalSize(32.0)));
    assert_ne!(text_key, display_key);

    let mut cache = SwashCache::new();
    let text_image = cache
        .get_image_uncached(&mut font_system, text_key)
        .expect("a text image");
    let display_image = cache
        .get_image_uncached(&mut font_system, display_key)
        .expect("a display image");
    assert_ne!(
        (text_image.placement.width, text_image.data),
        (display_image.placement.width, display_image.data),
        "the two optical sizes draw the same glyph"
    );
}

/// Spans at different optical sizes are shaped apart, as spans at
/// different weights are.
#[test]
fn optical_size_splits_compatibility() {
    let text = Attrs::new().optical_size(14.0);
    let display = Attrs::new().optical_size(32.0);
    assert!(!text.compatible(&display));
    assert!(text.compatible(&Attrs::new().optical_size(14.0)));
    assert_eq!(OpticalSize(0.0), OpticalSize(-0.0));
}
