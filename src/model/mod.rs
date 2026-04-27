//! Domain types: what aiscope actually models about your AI tools' memory.
//!
//! Pipeline shapes:
//! ```text
//!   Source (file on disk)
//!     │
//!     ▼ Layer 1: parse → Statement (one bullet/sentence with span)
//!     │
//!     ▼ Layer 2: canonicalize → CanonStmt (NFKC + caseless + stemmed)
//!     │
//!     ▼ Layer 3: extract → Assertion (axis + value + polarity + condition)
//!     │
//!     ▼ Layer 4: reason → Conflict (group by axis, find disagreements)
//!     │
//!     ▼ Layer 5: render → text/json/TUI/diagnostic
//! ```

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Tool & Source — unchanged from v0.0
// ---------------------------------------------------------------------------

/// Which AI tool a piece of memory belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tool {
    Cursor,
    Claude,
    Copilot,
}

impl Tool {
    pub fn label(&self) -> &'static str {
        match self {
            Tool::Cursor => "Cursor",
            Tool::Claude => "Claude Code",
            Tool::Copilot => "GitHub Copilot",
        }
    }
}

/// Which *subsystem* of a tool this file belongs to. Different subsystems
/// have different intent and so different conflict semantics:
///
/// - `Instructions` — general guidance always-on (copilot-instructions.md,
///   `.cursorrules`, CLAUDE.md, `.github/instructions/*`).
/// - `Prompts` — slash-command actions invoked explicitly by the user.
/// - `Agents` — autonomous task-runners with their own tool allowlists.
/// - `ChatModes` — capability bundles selected per chat session.
/// - `Skills` — reusable knowledge packs (e.g. `.claude/skills/*/SKILL.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Subsystem {
    Instructions,
    Prompts,
    Agents,
    ChatModes,
    Skills,
}

impl Subsystem {
    pub fn label(&self) -> &'static str {
        match self {
            Subsystem::Instructions => "instructions",
            Subsystem::Prompts => "prompts",
            Subsystem::Agents => "agents",
            Subsystem::ChatModes => "chatmodes",
            Subsystem::Skills => "skills",
        }
    }
}

/// Where a memory file's content actually applies. Combines explicit
/// frontmatter (`applyTo`, `globs`, `alwaysApply`) and path-derived scope
/// (e.g. `apps/web/AGENTS.md` is implicitly scoped to `apps/web/**`).
///
/// Two `Scope`s **overlap** if any path matches both. Non-overlapping
/// scopes mean two rules can't both be active for the same file, so a
/// would-be conflict is downgraded to `Severity::Low`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Scope {
    /// Glob patterns the rule applies to. Empty = applies everywhere.
    pub globs: Vec<String>,
    /// Whether `alwaysApply: true` was set in frontmatter.
    pub always_apply: bool,
    /// Path prefix derived from the file's location (e.g. `apps/web/`).
    pub path_prefix: Option<String>,
    /// Optional model restriction from frontmatter.
    pub model: Option<String>,
    /// Tool-allowlist restriction from frontmatter.
    pub tools: Vec<String>,
}

/// One source file (a single rule/instruction file on disk).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub tool: Tool,
    pub subsystem: Subsystem,
    pub path: PathBuf,
    /// Display label (e.g. ".cursorrules", "CLAUDE.md").
    pub label: String,
    /// Optional name extracted from frontmatter (agents/skills/prompts).
    pub name: Option<String>,
    /// Optional one-line description from frontmatter.
    pub description: Option<String>,
    /// Where this file's rules apply (frontmatter + path-derived).
    pub scope: Scope,
}

// ---------------------------------------------------------------------------
// Layer 1 output: Statement
// ---------------------------------------------------------------------------

/// One parsed atomic instruction with byte span back into the source file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statement {
    pub source_index: usize,
    pub text: String,
    /// Byte offset within the source file.
    pub byte_start: usize,
    pub byte_end: usize,
    /// 1-based line number for diagnostics.
    pub line: usize,
}

// ---------------------------------------------------------------------------
// Legacy Rule type — kept for back-compat with old renderers.
// New code should consume Statement + Assertion instead.
// ---------------------------------------------------------------------------

/// One extracted rule line (legacy view used by text/JSON/TUI renderers).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub source_index: usize,
    pub text: String,
    pub tokens: usize,
    pub fingerprint: u64,
}

// ---------------------------------------------------------------------------
// Layer 3 output: Assertion (THE world-class types)
// ---------------------------------------------------------------------------

/// What kind of statement this is — assertion, prohibition, or permission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Polarity {
    /// "use X", "prefer X", "always X", "require X" → +1 vote for X
    Prefer,
    /// "don't use X", "avoid X", "never X", "forbid X" → -1 vote for X
    Forbid,
    /// "X is allowed", "X is fine", "X is acceptable" → no clash signal
    Allow,
}

/// What axis a rule is about. Each axis has a closed set of possible values.
///
/// To add a new axis: add a variant here, add its `AxisValue` variants,
/// add a pattern extractor in `crate::extract::pattern`, and add canonical
/// embedding exemplars in `crate::extract::embedding`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "scope")]
pub enum Axis {
    /// Identifier naming style. Scope distinguishes vars/fns/types/etc.
    Naming(NamingScope),
    /// Indentation style.
    Indentation,
    /// String quote style.
    QuoteStyle,
    /// JS/TS package manager.
    PackageManager,
    /// Async control flow style.
    AsyncStyle,
    /// Where tests live relative to source.
    TestColocation,
    /// Type system strictness.
    TypeStrictness,
    /// Comment density convention.
    CommentDensity,
    /// Error-handling style.
    ErrorHandling,
    /// Import statement style.
    ImportStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NamingScope {
    Variables,
    Functions,
    Types,
    Constants,
    Files,
    Any,
}

/// The concrete value of an axis. Comparing two `AxisValue`s on the same axis
/// is the entire conflict-detection step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "v", content = "x")]
pub enum AxisValue {
    // Naming
    CamelCase,
    SnakeCase,
    PascalCase,
    KebabCase,
    ScreamingSnakeCase,
    // Indentation
    Tabs,
    Spaces2,
    Spaces4,
    Spaces8,
    // QuoteStyle
    SingleQuote,
    DoubleQuote,
    Backtick,
    // PackageManager
    Npm,
    Pnpm,
    Yarn,
    Bun,
    // AsyncStyle
    AsyncAwait,
    PromiseChain,
    Callbacks,
    // TestColocation
    BesideSource,
    DedicatedDir,
    // TypeStrictness
    Strict,
    Loose,
    // CommentDensity
    Heavy,
    Minimal,
    // ErrorHandling
    Throw,
    ResultType,
    // ImportStyle
    NamedImport,
    DefaultImport,
    NamespaceImport,
}

/// Optional condition narrowing the assertion (e.g. "in legacy code only").
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Condition {
    pub raw: String,
}

/// The fully-typed claim extracted from a Statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assertion {
    pub statement_index: usize,
    pub axis: Axis,
    pub value: AxisValue,
    pub polarity: Polarity,
    pub condition: Option<Condition>,
    /// 0.0–1.0. >=0.95 = pattern match. 0.6–0.95 = semantic. <0.6 = filtered.
    pub confidence: f32,
    /// Which extraction stage produced this.
    pub origin: ExtractionOrigin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtractionOrigin {
    Pattern,
    Embedding,
    CrossEncoder,
}

// ---------------------------------------------------------------------------
// Layer 4 output: Conflict
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub kind: ConflictKind,
    /// Indices into `bundle.assertions` (or `bundle.statements` for Duplicate).
    pub left: usize,
    pub right: usize,
    pub axis: Option<Axis>,
    pub note: String,
    pub severity: Severity,
    /// 0.0–1.0 — propagated from the lower-confidence side.
    pub confidence: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictKind {
    /// Same statement appears in two places (waste).
    Duplicate,
    /// Two assertions on the same axis disagree.
    Clash,
    /// One asserts X, another forbids X (explicit polarity conflict).
    PolarityConflict,
    /// An instruction names a tool the agent's allowlist excludes,
    /// or the agent has no allowlist at all.
    AgentToolMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Low confidence or paraphrase-only — surface in TUI but don't fail CI.
    Low,
    /// High confidence — `aiscope check` exits non-zero.
    High,
}

// ---------------------------------------------------------------------------
// The full bundle produced by one scan.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBundle {
    pub root: PathBuf,
    pub sources: Vec<Source>,
    pub statements: Vec<Statement>,
    pub assertions: Vec<Assertion>,
    /// Legacy view consumed by text/JSON/TUI renderers.
    pub rules: Vec<Rule>,
    pub conflicts: Vec<Conflict>,
    pub total_tokens: usize,
    pub stale_tokens: usize,
}

impl ContextBundle {
    pub fn waste_pct(&self) -> u32 {
        if self.total_tokens == 0 {
            return 0;
        }
        ((self.stale_tokens as f64 / self.total_tokens as f64) * 100.0).round() as u32
    }

    /// Conflicts above the configured severity bar (used by `aiscope check`).
    pub fn high_severity_conflicts(&self) -> impl Iterator<Item = &Conflict> {
        self.conflicts
            .iter()
            .filter(|c| c.severity == Severity::High)
    }
}
