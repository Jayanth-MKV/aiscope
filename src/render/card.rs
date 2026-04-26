//! Shareable PNG card renderer (tiny-skia + cosmic-text).
//!
//! Produces an 800×420 PNG suitable for tweets / blog posts, summarising:
//! - sources scanned
//! - statements parsed
//! - high-severity conflicts detected
//! - token waste
//!
//! Pure Rust, no headless browser, no system fonts required (cosmic-text
//! falls back to its embedded sans-serif).

use crate::model::{ConflictKind, ContextBundle};
use anyhow::{Context, Result};
use cosmic_text::{
    Attrs, Buffer, Color as CTColor, Family, FontSystem, Metrics, Shaping, SwashCache, Weight,
};
use std::path::Path;
use tiny_skia::{Color, Paint, Pixmap, PixmapMut, Rect, Transform};

const W: u32 = 800;
const H: u32 = 420;

pub fn render(bundle: &ContextBundle, out: &Path) -> Result<()> {
    let mut canvas = Pixmap::new(W, H).context("failed to allocate pixmap")?;
    canvas.fill(Color::from_rgba8(0x0e, 0x10, 0x14, 0xff));

    // Accent stripe along the top.
    draw_rect(&mut canvas.as_mut(), 0.0, 0.0, W as f32, 8.0, 0x7c, 0x3a, 0xed);

    let high = bundle.high_severity_conflicts().count();
    let dups = bundle
        .conflicts
        .iter()
        .filter(|c| matches!(c.kind, ConflictKind::Duplicate))
        .count();
    let clashes = bundle.conflicts.len() - dups;
    let waste = bundle.waste_pct();
    let waste_rgb = match waste {
        0..=10 => (0x10, 0xb9, 0x81),
        11..=25 => (0xf5, 0x9e, 0x0b),
        _ => (0xef, 0x44, 0x44),
    };

    let mut fs = FontSystem::new();
    let mut cache = SwashCache::new();

    draw_text(&mut canvas, &mut fs, &mut cache, "aiscope", 40.0, 30.0,
        Metrics::new(40.0, 48.0), Weight::BOLD, (0xff, 0xff, 0xff))?;
    draw_text(&mut canvas, &mut fs, &mut cache,
        "DevTools for your AI coding tools' memory", 220.0, 42.0,
        Metrics::new(18.0, 22.0), Weight::NORMAL, (0xa0, 0xa6, 0xb4))?;

    draw_text(&mut canvas, &mut fs, &mut cache, &format!("{}%", waste),
        40.0, 110.0, Metrics::new(120.0, 130.0), Weight::BOLD, waste_rgb)?;
    draw_text(&mut canvas, &mut fs, &mut cache,
        "of your context window is stale", 40.0, 270.0,
        Metrics::new(20.0, 24.0), Weight::NORMAL, (0xc0, 0xc6, 0xd4))?;

    draw_stat(&mut canvas, &mut fs, &mut cache, 460.0, 110.0,
        &bundle.sources.len().to_string(), "sources")?;
    draw_stat(&mut canvas, &mut fs, &mut cache, 460.0, 180.0,
        &bundle.statements.len().to_string(), "rules")?;
    draw_stat(&mut canvas, &mut fs, &mut cache, 460.0, 250.0,
        &format!("{}", high),
        if high > 0 { "high-severity conflicts" } else { "conflicts" })?;

    let footer = format!(
        "{} clashes  ·  {} duplicates  ·  {} tokens  ·  {}",
        clashes, dups, bundle.total_tokens,
        bundle.root.file_name().and_then(|s| s.to_str()).unwrap_or("repo"),
    );
    draw_text(&mut canvas, &mut fs, &mut cache, &footer, 40.0, 370.0,
        Metrics::new(15.0, 18.0), Weight::NORMAL, (0x70, 0x76, 0x84))?;

    canvas.save_png(out).context("failed to write PNG")?;
    eprintln!("aiscope: wrote {}", out.display());
    Ok(())
}

fn draw_stat(canvas: &mut Pixmap, fs: &mut FontSystem, cache: &mut SwashCache,
    x: f32, y: f32, big: &str, label: &str) -> Result<()> {
    draw_text(canvas, fs, cache, big, x, y,
        Metrics::new(40.0, 44.0), Weight::BOLD, (0xff, 0xff, 0xff))?;
    draw_text(canvas, fs, cache, label, x + 80.0, y + 14.0,
        Metrics::new(16.0, 20.0), Weight::NORMAL, (0xa0, 0xa6, 0xb4))?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn draw_rect(canvas: &mut PixmapMut, x: f32, y: f32, w: f32, h: f32, r: u8, g: u8, b: u8) {
    let mut paint = Paint::default();
    paint.set_color_rgba8(r, g, b, 0xff);
    if let Some(rect) = Rect::from_xywh(x, y, w, h) {
        canvas.fill_rect(rect, &paint, Transform::identity(), None);
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_text(canvas: &mut Pixmap, fs: &mut FontSystem, cache: &mut SwashCache,
    text: &str, x: f32, y: f32, metrics: Metrics, weight: Weight,
    rgb: (u8, u8, u8)) -> Result<()> {
    let mut buf = Buffer::new(fs, metrics);
    let attrs = Attrs::new().family(Family::SansSerif).weight(weight);
    buf.set_size(fs, Some(W as f32 - x - 10.0), Some(H as f32 - y));
    buf.set_text(fs, text, attrs, Shaping::Advanced);
    buf.shape_until_scroll(fs, false);
    let color = CTColor::rgba(rgb.0, rgb.1, rgb.2, 0xff);

    let canvas_w = canvas.width() as i32;
    let canvas_h = canvas.height() as i32;
    let pixels = canvas.pixels_mut();

    buf.draw(fs, cache, color, |gx, gy, gw, gh, c| {
        let alpha = c.a();
        if alpha == 0 { return; }
        for dy in 0..gh as i32 {
            for dx in 0..gw as i32 {
                let px = x as i32 + gx + dx;
                let py = y as i32 + gy + dy;
                if px < 0 || py < 0 || px >= canvas_w || py >= canvas_h { continue; }
                let idx = (py * canvas_w + px) as usize;
                let dst = &mut pixels[idx];
                let a = alpha as u32;
                let inv = 255 - a;
                let blend = |src: u8, d: u8| -> u8 {
                    (((src as u32 * a) + (d as u32 * inv) + 127) / 255) as u8
                };
                let new = tiny_skia::PremultipliedColorU8::from_rgba(
                    blend(c.r(), dst.red()),
                    blend(c.g(), dst.green()),
                    blend(c.b(), dst.blue()),
                    dst.alpha().saturating_add(alpha.saturating_sub(dst.alpha())),
                ).unwrap_or(*dst);
                *dst = new;
            }
        }
    });

    Ok(())
}
