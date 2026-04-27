//! Summary PNG card: 1280×720 dark-terminal aesthetic, JetBrains Mono,
//! shows the **top 3 conflicts** with actual rule excerpts side-by-side
//! plus the headline waste% and footer totals.
//!
//! All text positioning uses cosmic-text's actual shaped width — no
//! fudge factors, no hardcoded column widths. Truncation is performed
//! by re-shaping until the result fits the available pixel width.

use crate::model::{Conflict, ConflictKind, ContextBundle, Severity};
use anyhow::{Context, Result};
use cosmic_text::{
    Attrs, Buffer, Color as CosColor, Family, FontSystem, Metrics, Shaping, SwashCache,
    fontdb::Database,
};
use std::path::Path;
use tiny_skia::{Color, Pixmap, Rect, Transform};

const W: u32 = 1280;
const H: u32 = 720;

const BG: (u8, u8, u8) = (0x0E, 0x10, 0x14);
const PANEL: (u8, u8, u8) = (0x14, 0x17, 0x1F);
const PANEL_HEAD: (u8, u8, u8) = (0x1A, 0x1E, 0x28);
const BORDER: (u8, u8, u8) = (0x2A, 0x2F, 0x3B);
const FG: (u8, u8, u8) = (0xE6, 0xE6, 0xE6);
const DIM: (u8, u8, u8) = (0x7F, 0x84, 0x90);
const CYAN: (u8, u8, u8) = (0x00, 0xD7, 0xFF);
const YELLOW: (u8, u8, u8) = (0xFF, 0xC8, 0x57);
const RED: (u8, u8, u8) = (0xFF, 0x5F, 0x5F);
const GREEN: (u8, u8, u8) = (0x5F, 0xFF, 0xB5);
const MAGENTA: (u8, u8, u8) = (0xFF, 0x6F, 0xCF);

const FONT_TTF: &[u8] = include_bytes!("../../assets/fonts/JetBrainsMono-Regular.ttf");
const FAMILY: &str = "JetBrains Mono";

const PAD: f32 = 32.0;
const PANEL_Y: f32 = 130.0;
const PANEL_H: f32 = 470.0;
const PANEL_HEAD_H: f32 = 36.0;
const ROW_BLOCK_H: f32 = 130.0;
const ROW_INSET_X: f32 = 20.0;
const ROW_GAP_AFTER_BADGE: f32 = 16.0;
const ELLIPSIS: char = '…';

pub fn render(bundle: &ContextBundle, out_path: &Path) -> Result<()> {
    let mut pixmap = Pixmap::new(W, H).context("alloc pixmap")?;
    pixmap.fill(rgb(BG));

    // Cyan accent stripe along the top.
    fill(&mut pixmap, 0.0, 0.0, W as f32, 4.0, CYAN);

    let mut db = Database::new();
    db.load_font_data(FONT_TTF.to_vec());
    let mut fs = FontSystem::new_with_locale_and_db("en-US".into(), db);
    let mut cache = SwashCache::new();

    // ── Header ────────────────────────────────────────────────────────────
    let logo = "aiscope";
    let logo_w = measure(&mut fs, logo, 36.0);
    draw(&mut pixmap, &mut fs, &mut cache, logo, PAD, PAD, 36.0, CYAN)?;

    let repo_name = bundle
        .root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(".");
    let repo = format!("· {repo_name}");
    draw(
        &mut pixmap,
        &mut fs,
        &mut cache,
        &repo,
        PAD + logo_w + 12.0,
        PAD + 8.0,
        24.0,
        FG,
    )?;

    // Big waste % on the right. Cap at 100 so the display never overflows
    // even on pathological inputs (e.g. stale_tokens > total_tokens).
    let waste = bundle.waste_pct().min(100);
    let high_count = bundle.high_severity_conflicts().count();
    let waste_color = if high_count == 0 && waste <= 10 {
        GREEN
    } else if high_count <= 2 && waste <= 25 {
        YELLOW
    } else {
        RED
    };
    let waste_str = format!("{waste}%");
    draw_right(
        &mut pixmap,
        &mut fs,
        &mut cache,
        &waste_str,
        W as f32 - PAD,
        PAD - 8.0,
        72.0,
        waste_color,
    )?;
    draw_right(
        &mut pixmap,
        &mut fs,
        &mut cache,
        "context wasted",
        W as f32 - PAD,
        PAD + 64.0,
        16.0,
        DIM,
    )?;

    // ── Top 3 conflicts panel ─────────────────────────────────────────────
    let panel_x = PAD;
    let panel_w = W as f32 - PAD * 2.0;
    fill(&mut pixmap, panel_x, PANEL_Y, panel_w, PANEL_H, PANEL);
    border(&mut pixmap, panel_x, PANEL_Y, panel_w, PANEL_H, BORDER);
    fill(
        &mut pixmap,
        panel_x,
        PANEL_Y,
        panel_w,
        PANEL_HEAD_H,
        PANEL_HEAD,
    );
    draw(
        &mut pixmap,
        &mut fs,
        &mut cache,
        "Top conflicts",
        panel_x + 16.0,
        PANEL_Y + 8.0,
        18.0,
        CYAN,
    )?;

    let mut sorted: Vec<&Conflict> = bundle.conflicts.iter().collect();
    sorted.sort_by_key(|c| {
        (
            match c.severity {
                Severity::High => 0,
                Severity::Low => 1,
            },
            match c.kind {
                ConflictKind::PolarityConflict => 0,
                ConflictKind::AgentToolMismatch => 1,
                ConflictKind::Clash => 2,
                ConflictKind::Duplicate => 3,
            },
        )
    });

    let mut row_y = PANEL_Y + PANEL_HEAD_H + 20.0;
    let row_x = panel_x + ROW_INSET_X;
    let row_w = panel_w - ROW_INSET_X * 2.0;
    let shown_count = sorted.len().min(3);
    for c in sorted.iter().take(3) {
        draw_conflict_row(
            &mut pixmap,
            &mut fs,
            &mut cache,
            bundle,
            c,
            row_x,
            row_y,
            row_w,
        )?;
        row_y += ROW_BLOCK_H;
    }
    if shown_count == 0 {
        draw(
            &mut pixmap,
            &mut fs,
            &mut cache,
            "✓ no conflicts detected — your AI tools agree",
            row_x,
            PANEL_Y + PANEL_H / 2.0 - 12.0,
            22.0,
            GREEN,
        )?;
    } else if bundle.conflicts.len() > shown_count {
        let extra = bundle.conflicts.len() - shown_count;
        draw(
            &mut pixmap,
            &mut fs,
            &mut cache,
            &format!("+{extra} more — run `aiscope` for the full list"),
            row_x,
            PANEL_Y + PANEL_H - 26.0,
            14.0,
            DIM,
        )?;
    }

    // ── Footer totals ─────────────────────────────────────────────────────
    let dups = bundle
        .conflicts
        .iter()
        .filter(|c| matches!(c.kind, ConflictKind::Duplicate))
        .count();
    let clashes = bundle.conflicts.len() - dups;
    let footer_y = H as f32 - PAD - 24.0;
    let footer = format!(
        "{} sources  ·  {} rules  ·  {} clashes  ·  {} duplicates  ·  {} tokens",
        bundle.sources.len(),
        bundle.statements.len(),
        clashes,
        dups,
        bundle.total_tokens
    );
    draw(
        &mut pixmap,
        &mut fs,
        &mut cache,
        &footer,
        PAD,
        footer_y,
        16.0,
        DIM,
    )?;
    let repo_url = env!("CARGO_PKG_REPOSITORY")
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let attribution = format!("aiscope · v{} · {}", env!("CARGO_PKG_VERSION"), repo_url,);
    draw_right(
        &mut pixmap,
        &mut fs,
        &mut cache,
        &attribution,
        W as f32 - PAD,
        footer_y,
        14.0,
        DIM,
    )?;

    pixmap.save_png(out_path).context("save png")?;
    println!("aiscope: wrote {}", out_path.display());
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn draw_conflict_row(
    pixmap: &mut Pixmap,
    fs: &mut FontSystem,
    cache: &mut SwashCache,
    bundle: &ContextBundle,
    c: &Conflict,
    x: f32,
    y: f32,
    w: f32,
) -> Result<()> {
    let (badge, badge_col) = match (c.kind, c.severity) {
        (ConflictKind::PolarityConflict, Severity::High) => ("HIGH polarity", RED),
        (ConflictKind::Clash, Severity::High) => ("HIGH clash", RED),
        (ConflictKind::Duplicate, Severity::High) => ("HIGH dup", YELLOW),
        (ConflictKind::AgentToolMismatch, Severity::High) => ("HIGH agent", RED),
        (ConflictKind::PolarityConflict, Severity::Low) => ("low polarity", MAGENTA),
        (ConflictKind::Clash, Severity::Low) => ("low clash", MAGENTA),
        (ConflictKind::Duplicate, Severity::Low) => ("low dup", DIM),
        (ConflictKind::AgentToolMismatch, Severity::Low) => ("low agent", MAGENTA),
    };

    // Badge — measured, not assumed.
    let badge_size = 16.0;
    let badge_w = measure(fs, badge, badge_size);
    draw(pixmap, fs, cache, badge, x, y, badge_size, badge_col)?;

    // Headline (left ⇄ right). Truncated to whatever pixel space remains.
    let l_label = bundle
        .statements
        .get(c.left)
        .and_then(|s| bundle.sources.get(s.source_index))
        .map(|s| s.label.as_str())
        .unwrap_or("?");
    let r_label = bundle
        .statements
        .get(c.right)
        .and_then(|s| bundle.sources.get(s.source_index))
        .map(|s| s.label.as_str())
        .unwrap_or("?");
    let head_x = x + badge_w + ROW_GAP_AFTER_BADGE;
    let head_w = (x + w) - head_x;
    let head = fit_to_width(fs, &format!("{l_label}  vs  {r_label}"), 16.0, head_w);
    draw(pixmap, fs, cache, &head, head_x, y, 16.0, FG)?;

    // Note (one-line description), truncated to pixel width.
    let note = fit_to_width(fs, &c.note, 14.0, w);
    draw(pixmap, fs, cache, &note, x, y + 28.0, 14.0, DIM)?;

    // Rule excerpts (left + right), prefixed with ▸
    let l_text = bundle
        .statements
        .get(c.left)
        .map(|s| s.text.as_str())
        .unwrap_or("");
    let r_text = bundle
        .statements
        .get(c.right)
        .map(|s| s.text.as_str())
        .unwrap_or("");
    let excerpt_l = fit_to_width(fs, &format!("▸ {l_text}"), 14.0, w);
    let excerpt_r = fit_to_width(fs, &format!("▸ {r_text}"), 14.0, w);
    draw(pixmap, fs, cache, &excerpt_l, x, y + 56.0, 14.0, FG)?;
    draw(pixmap, fs, cache, &excerpt_r, x, y + 82.0, 14.0, FG)?;

    Ok(())
}

/// Measure the rendered width of `text` at `size` using cosmic-text shaping.
fn measure(fs: &mut FontSystem, text: &str, size: f32) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    let metrics = Metrics::new(size, size * 1.3);
    let mut buf = Buffer::new(fs, metrics);
    let attrs = Attrs::new().family(Family::Name(FAMILY));
    // Generous fixed bound so cosmic-text doesn't wrap during measurement.
    buf.set_size(fs, Some(W as f32 * 4.0), Some(size * 4.0));
    buf.set_text(fs, text, attrs, Shaping::Advanced);
    buf.shape_until_scroll(fs, false);
    buf.layout_runs().map(|r| r.line_w).fold(0.0_f32, f32::max)
}

/// Truncate `text` so its shaped width fits within `max_w` pixels, adding an
/// ellipsis when characters are dropped. Pure measurement — no fudge factors.
fn fit_to_width(fs: &mut FontSystem, text: &str, size: f32, max_w: f32) -> String {
    if max_w <= 0.0 {
        return String::new();
    }
    if measure(fs, text, size) <= max_w {
        return text.to_string();
    }
    let chars: Vec<char> = text.chars().collect();
    // Binary-search the largest prefix length whose `prefix + …` still fits.
    let mut lo = 0_usize;
    let mut hi = chars.len();
    while lo < hi {
        let mid = (lo + hi).div_ceil(2);
        let mut candidate: String = chars[..mid].iter().collect();
        candidate.push(ELLIPSIS);
        if measure(fs, &candidate, size) <= max_w {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    let mut out: String = chars[..lo].iter().collect();
    out.push(ELLIPSIS);
    out
}

fn rgb((r, g, b): (u8, u8, u8)) -> Color {
    Color::from_rgba8(r, g, b, 255)
}

fn fill(p: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, c: (u8, u8, u8)) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    if let Some(rect) = Rect::from_xywh(x, y, w, h) {
        let mut paint = tiny_skia::Paint::default();
        paint.set_color(rgb(c));
        p.fill_rect(rect, &paint, Transform::identity(), None);
    }
}

fn border(p: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, c: (u8, u8, u8)) {
    fill(p, x, y, w, 1.0, c);
    fill(p, x, y + h - 1.0, w, 1.0, c);
    fill(p, x, y, 1.0, h, c);
    fill(p, x + w - 1.0, y, 1.0, h, c);
}

#[allow(clippy::too_many_arguments)]
fn draw(
    pixmap: &mut Pixmap,
    fs: &mut FontSystem,
    cache: &mut SwashCache,
    text: &str,
    x: f32,
    y: f32,
    size: f32,
    rgb_c: (u8, u8, u8),
) -> Result<()> {
    if text.is_empty() {
        return Ok(());
    }
    let metrics = Metrics::new(size, size * 1.3);
    let mut buf = Buffer::new(fs, metrics);
    let attrs = Attrs::new().family(Family::Name(FAMILY));
    buf.set_size(fs, Some(W as f32 * 4.0), Some(H as f32 * 4.0));
    buf.set_text(fs, text, attrs, Shaping::Advanced);
    buf.shape_until_scroll(fs, false);
    let color = CosColor::rgb(rgb_c.0, rgb_c.1, rgb_c.2);
    let pw = pixmap.width() as i32;
    let ph = pixmap.height() as i32;
    let pixels = pixmap.pixels_mut();
    buf.draw(fs, cache, color, |gx, gy, _w, _h, c| {
        let px = x as i32 + gx;
        let py = y as i32 + gy;
        if px < 0 || py < 0 || px >= pw || py >= ph {
            return;
        }
        let idx = (py * pw + px) as usize;
        let a = c.a() as u32;
        if a == 0 {
            return;
        }
        let dst = pixels[idx];
        let dr = dst.red() as u32;
        let dg = dst.green() as u32;
        let db = dst.blue() as u32;
        let sr = c.r() as u32;
        let sg = c.g() as u32;
        let sb = c.b() as u32;
        let nr = ((sr * a + dr * (255 - a)) / 255) as u8;
        let ng = ((sg * a + dg * (255 - a)) / 255) as u8;
        let nb = ((sb * a + db * (255 - a)) / 255) as u8;
        pixels[idx] = tiny_skia::PremultipliedColorU8::from_rgba(nr, ng, nb, 255).unwrap();
    });
    Ok(())
}

/// Right-anchored draw: measures the actual shaped width and offsets `x` by
/// it, so the right edge of the text lands exactly at `x_right`.
#[allow(clippy::too_many_arguments)]
fn draw_right(
    pixmap: &mut Pixmap,
    fs: &mut FontSystem,
    cache: &mut SwashCache,
    text: &str,
    x_right: f32,
    y: f32,
    size: f32,
    rgb_c: (u8, u8, u8),
) -> Result<()> {
    let w = measure(fs, text, size);
    draw(pixmap, fs, cache, text, x_right - w, y, size, rgb_c)
}
