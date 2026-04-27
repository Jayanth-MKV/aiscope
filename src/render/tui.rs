//! Interactive ratatui TUI: split-pane (Sources / Unified / Score).
//!
//! Keys:
//!   q / Esc        quit
//!   c              toggle conflicts-only filter
//!   ↑/↓ or j/k     scroll
//!   PgUp/PgDn      page scroll

use crate::model::{ConflictKind, ContextBundle, Scope, Tool};
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Terminal,
};
use std::io;
use std::time::Duration;

pub fn render(bundle: &ContextBundle) -> Result<()> {
    // If stdout is not a TTY (e.g. piped), fall back to text output.
    if !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
        print!("{}", super::text::render(bundle));
        return Ok(());
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, bundle);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

struct State {
    sources_state: ListState,
    rules_state: ListState,
    conflicts_only: bool,
}

fn run_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    bundle: &ContextBundle,
) -> Result<()> {
    let mut st = State {
        sources_state: ListState::default(),
        rules_state: ListState::default(),
        conflicts_only: false,
    };
    st.sources_state.select(Some(0));
    st.rules_state.select(Some(0));

    loop {
        terminal.draw(|frame| draw(frame, bundle, &mut st))?;

        if !event::poll(Duration::from_millis(150))? {
            continue;
        }
        if let Event::Key(k) = event::read()? {
            if k.kind != KeyEventKind::Press {
                continue;
            }
            match k.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Char('c') => st.conflicts_only = !st.conflicts_only,
                KeyCode::Down | KeyCode::Char('j') => {
                    let i = st.rules_state.selected().unwrap_or(0).saturating_add(1);
                    st.rules_state
                        .select(Some(i.min(bundle.rules.len().saturating_sub(1))));
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let i = st.rules_state.selected().unwrap_or(0);
                    st.rules_state.select(Some(i.saturating_sub(1)));
                }
                KeyCode::PageDown => {
                    let i = st.rules_state.selected().unwrap_or(0).saturating_add(10);
                    st.rules_state
                        .select(Some(i.min(bundle.rules.len().saturating_sub(1))));
                }
                KeyCode::PageUp => {
                    let i = st.rules_state.selected().unwrap_or(0);
                    st.rules_state.select(Some(i.saturating_sub(10)));
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn draw(frame: &mut ratatui::Frame, bundle: &ContextBundle, st: &mut State) {
    use std::collections::HashSet;

    // Each Conflict's left/right may index either statements (Duplicate) or
    // assertions (Clash/PolarityConflict). Resolve to underlying statement
    // indices, which 1:1 match the legacy `rules` view used here.
    let conflict_indices: HashSet<usize> = bundle
        .conflicts
        .iter()
        .flat_map(|c| match c.kind {
            ConflictKind::Duplicate | ConflictKind::AgentToolMismatch => vec![c.left, c.right],
            ConflictKind::Clash | ConflictKind::PolarityConflict => vec![
                bundle.assertions[c.left].statement_index,
                bundle.assertions[c.right].statement_index,
            ],
        })
        .collect();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(5)])
        .split(frame.area());

    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(chunks[0]);

    // ── Left: sources ────────────────────────────────────────────
    let source_items: Vec<ListItem> = bundle
        .sources
        .iter()
        .enumerate()
        .map(|(idx, s)| {
            let icon = match s.tool {
                Tool::Cursor => "○",
                Tool::Claude => "◆",
                Tool::Copilot => "▲",
            };
            let real_count = bundle
                .rules
                .iter()
                .filter(|r| r.source_index == idx)
                .count();
            ListItem::new(Line::from(vec![
                Span::styled(format!("{icon} "), Style::default().fg(tool_color(s.tool))),
                Span::raw(format!("{} ", s.label)),
                Span::styled(
                    format!("({real_count})"),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();

    let sources_widget = List::new(source_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Sources ({}) ", bundle.sources.len())),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    frame.render_stateful_widget(sources_widget, main[0], &mut st.sources_state);

    // ── Right: unified rules ─────────────────────────────────────
    let rule_items: Vec<ListItem> = bundle
        .rules
        .iter()
        .enumerate()
        .filter(|(i, _)| !st.conflicts_only || conflict_indices.contains(i))
        .map(|(i, r)| {
            let src = bundle
                .sources
                .get(r.source_index)
                .map(|s| (s.label.as_str(), s.tool, &s.scope))
                .unwrap_or(("?", Tool::Cursor, &EMPTY_SCOPE));
            let conflict = conflict_indices.contains(&i);
            let prefix = if conflict { "⚠ " } else { "  " };
            let scope_tag = scope_tag(src.2);
            ListItem::new(Line::from(vec![
                Span::styled(
                    prefix.to_string(),
                    Style::default().fg(if conflict { Color::Red } else { Color::Reset }),
                ),
                Span::styled(
                    format!("[{}] ", src.0),
                    Style::default().fg(tool_color(src.1)),
                ),
                Span::raw(r.text.clone()),
                Span::styled(
                    format!("  {}", scope_tag),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();

    let title = if st.conflicts_only {
        format!(
            " Unified context · CONFLICTS ONLY ({} hits) ",
            conflict_indices.len()
        )
    } else {
        format!(" Unified context ({} rules) ", bundle.rules.len())
    };
    let rules_widget = List::new(rule_items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    frame.render_stateful_widget(rules_widget, main[1], &mut st.rules_state);

    // ── Bottom: score ─────────────────────────────────────────────
    let waste = bundle.waste_pct();
    let waste_color = match waste {
        0..=10 => Color::Green,
        11..=25 => Color::Yellow,
        _ => Color::Red,
    };
    let dup_count = bundle
        .conflicts
        .iter()
        .filter(|c| matches!(c.kind, ConflictKind::Duplicate))
        .count();
    let clash_count = bundle.conflicts.len() - dup_count;

    let score = Paragraph::new(vec![
        Line::from(vec![
            Span::raw(format!("{} rules · ", bundle.rules.len())),
            Span::styled(
                format!("{} clashes ⚠", clash_count),
                Style::default().fg(if clash_count > 0 {
                    Color::Red
                } else {
                    Color::Green
                }),
            ),
            Span::raw(format!(" · {} duplicates · ", dup_count)),
            Span::raw(format!("{} tokens · ", bundle.total_tokens)),
            Span::styled(
                format!("{}% wasted", waste),
                Style::default()
                    .fg(waste_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(Span::styled(
            "[q] quit  [c] conflicts only  [↑/↓] scroll  [PgUp/PgDn] page",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .block(Block::default().borders(Borders::ALL).title(" Score "))
    .wrap(Wrap { trim: true });
    frame.render_widget(score, chunks[1]);
}

fn tool_color(t: Tool) -> Color {
    match t {
        Tool::Cursor => Color::Cyan,
        Tool::Claude => Color::Magenta,
        Tool::Copilot => Color::Yellow,
    }
}

/// Empty fallback `Scope` used when a rule's source can't be resolved.
static EMPTY_SCOPE: Scope = Scope {
    globs: Vec::new(),
    always_apply: false,
    path_prefix: None,
    model: None,
    tools: Vec::new(),
};

/// Compact one-line scope label for a rule row.
/// Only shows what is *factual* — applyTo globs, path prefix, or `always`.
fn scope_tag(s: &Scope) -> String {
    if !s.globs.is_empty() {
        let joined = s.globs.join(",");
        let short = if joined.chars().count() > 28 {
            let mut t: String = joined.chars().take(27).collect();
            t.push('…');
            t
        } else {
            joined
        };
        format!("[{short}]")
    } else if let Some(p) = &s.path_prefix {
        format!("[{p}]")
    } else if s.always_apply {
        "[always]".to_string()
    } else {
        String::new()
    }
}
