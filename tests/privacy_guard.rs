//! Privacy guard: aiscope v0.1 must not contain code paths that read
//! anything under `~/.claude/projects/`. This test greps the source tree.

use std::fs;
use std::path::Path;

fn walk(dir: &Path, out: &mut Vec<String>) {
    if !dir.is_dir() {
        return;
    }
    for e in fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, out);
        } else if p.extension().map(|x| x == "rs").unwrap_or(false) {
            if let Ok(s) = fs::read_to_string(&p) {
                out.push(format!("{}\n{}", p.display(), s));
            }
        }
    }
}

#[test]
fn no_session_log_reads() {
    let mut sources = Vec::new();
    walk(Path::new("src"), &mut sources);
    let combined = sources.join("\n");

    // Allowed mentions of "projects" in source: comments, the privacy assertion,
    // and any future opt-in flag (which v0.1 does NOT have). The single
    // `assert!(!global.starts_with(...projects)...)` guard is the only runtime
    // reference. Any unexpected read attempt would add `read_to_string`,
    // `File::open`, or similar near a `projects` literal \u2014 we forbid that.
    let bad_patterns = [
        ("read_to_string", "projects"),
        ("File::open", "projects"),
        ("std::fs::read", "projects"),
    ];

    for (call, ctx) in bad_patterns {
        let mut idx = 0;
        while let Some(pos) = combined[idx..].find(call) {
            let abs = idx + pos;
            let window_start = abs.saturating_sub(200);
            let window_end = (abs + 200).min(combined.len());
            let window = &combined[window_start..window_end];
            assert!(
                !window.contains(ctx),
                "privacy guard violated: `{}` near `{}` \u{2014} v0.1 must not read session logs",
                call,
                ctx
            );
            idx = abs + call.len();
        }
    }
}
