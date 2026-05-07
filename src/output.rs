use std::io::Write;

// ── ANSI color/style constants ──────────────────────────────

pub const GREEN: &str = "\x1b[32m";
pub const RED: &str = "\x1b[31m";
pub const YELLOW: &str = "\x1b[33m";
pub const BLUE: &str = "\x1b[34m";
pub const CYAN: &str = "\x1b[36m";
pub const MAGENTA: &str = "\x1b[35m";
pub const WHITE: &str = "\x1b[37m";
pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";
pub const RESET: &str = "\x1b[0m";

// ── Icon constants ──────────────────────────────────────────

pub const ICON_OK: &str = "✓";
pub const ICON_ERR: &str = "✗";
pub const ICON_WARN: &str = "⚠";
pub const ICON_INFO: &str = "ℹ";
pub const ICON_BULLET: &str = "•";
pub const ICON_ARROW: &str = "→";
pub const ICON_DOT_ON: &str = "●";
pub const ICON_DOT_OFF: &str = "○";
pub const ICON_PROMPT: &str = "▸";
pub const ICON_HEADER: &str = "▸";
pub const ICON_SEP: &str = "│";

// ── Styled string builders ──────────────────────────────────

#[inline]
pub fn style(text: &str, color: &str) -> String {
    format!("{}{}{}", color, text, RESET)
}

#[inline]
pub fn green(text: &str) -> String { style(text, GREEN) }
#[inline]
pub fn red(text: &str) -> String { style(text, RED) }
#[inline]
pub fn yellow(text: &str) -> String { style(text, YELLOW) }
#[inline]
pub fn blue(text: &str) -> String { style(text, BLUE) }
#[inline]
pub fn cyan(text: &str) -> String { style(text, CYAN) }
#[inline]
pub fn magenta(text: &str) -> String { style(text, MAGENTA) }
#[inline]
pub fn bold(text: &str) -> String { style(text, BOLD) }
#[inline]
pub fn dim(text: &str) -> String { style(text, DIM) }

// ── Terminal output helpers ─────────────────────────────────

/// Print a success message with a green checkmark.
pub fn print_success(msg: &str) {
    println!("{} {}{}", green(ICON_OK), msg, RESET);
}

/// Print an error message with a red cross.
pub fn print_error(msg: &str) {
    eprintln!("{} {}{}", red(ICON_ERR), msg, RESET);
}

/// Print a warning message with a yellow triangle.
pub fn print_warning(msg: &str) {
    eprintln!("{} {}{}", yellow(ICON_WARN), msg, RESET);
}

/// Print an info message with a blue info icon.
pub fn print_info(msg: &str) {
    println!("{} {}{}", blue(ICON_INFO), msg, RESET);
}

/// Print a bullet point with dim styling.
pub fn print_bullet(msg: &str) {
    println!("  {} {}{}", dim(ICON_BULLET), msg, RESET);
}

/// Print an arrow step with dim styling.
pub fn print_arrow(msg: &str) {
    println!("  {} {}{}", dim(ICON_ARROW), msg, RESET);
}

/// Print a numbered step.
pub fn print_step(n: usize, msg: &str) {
    println!("  {}[{}]{} {}", DIM, n, RESET, msg);
}

/// Print a section header.
pub fn print_header(title: &str) {
    println!("\n  {} {} {}{}", bold(ICON_HEADER), bold(title), RESET, "");
}

/// Print a sub-section title.
pub fn print_section(title: &str) {
    println!("  {}{}:{}{}", BOLD, title, RESET, "");
}

/// Print a prompt marker.
pub fn print_prompt(msg: &str) {
    print!("{} {} {}{} ", cyan(ICON_PROMPT), msg, RESET, "");
}

/// Print a dimmed label/value pair.
pub fn print_kv(label: &str, value: &str) {
    println!("  {}{}:{} {}", DIM, label, RESET, value);
}

/// Print an empty state with a message and hint.
pub fn print_empty(title: &str, message: &str, hint: &str) {
    println!();
    println!("  {} {}{}", DIM, title, RESET);
    println!("  {}", message);
    println!("  {}{}{}", DIM, hint, RESET);
    println!();
}

/// Print a horizontal separator line.
pub fn print_separator() {
    println!("  {}{}", DIM, "─".repeat(60));
}

/// Print a confirmation prompt and return the result.
pub fn confirm(msg: &str) -> bool {
    print_prompt(msg);
    let _ = std::io::stdout().flush();
    let mut input = String::new();
    let _ = std::io::stdin().read_line(&mut input);
    input.trim().eq_ignore_ascii_case("y")
}

/// Print a destructive confirmation prompt with warning styling.
pub fn confirm_danger(msg: &str) -> bool {
    print!("  {} {}{} {}{} ", yellow(ICON_WARN), BOLD, "WARNING:", RESET, msg);
    let _ = std::io::stdout().flush();
    let mut input = String::new();
    let _ = std::io::stdin().read_line(&mut input);
    input.trim().eq_ignore_ascii_case("y")
}

// ── Status icon helpers ─────────────────────────────────────

/// Render a colored status icon (● for active, ○ for inactive).
pub fn status_icon(active: bool) -> &'static str {
    if active {
        "\x1b[32m●\x1b[0m"  // green circle
    } else {
        "\x1b[31m○\x1b[0m"  // red circle
    }
}

/// Render a status label with color.
pub fn status_label(active: bool) -> String {
    if active {
        green("running")
    } else {
        red("stopped")
    }
}

// ── Status table ────────────────────────────────────────────

pub struct StatusRow {
    pub user: String,
    pub icon: String,
    pub status: String,
    pub projects: String,
    pub workspace: String,
}

impl StatusRow {
    pub fn new(
        user: impl Into<String>,
        active: bool,
        status_text: impl Into<String>,
        projects: &[String],
        workspace: Option<&std::path::Path>,
    ) -> Self {
        StatusRow {
            user: user.into(),
            icon: status_icon(active).to_string(),
            status: status_text.into(),
            projects: if projects.is_empty() {
                dim("—").to_string()
            } else {
                projects.join(", ")
            },
            workspace: workspace
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| dim("—").to_string()),
        }
    }
}

/// Render a formatted status table with colored headers and borders.
pub fn render_status_table<W: Write>(
    writer: &mut W,
    rows: &[StatusRow],
) -> std::io::Result<()> {
    if rows.is_empty() {
        writeln!(writer)?;
        writeln!(writer, "  {}No runtimes found{}", DIM, RESET)?;
        writeln!(writer, "  There are no AI runtimes configured on this system.")?;
        writeln!(writer, "  {}Hint: usw create <user>  —  Create a new isolated runtime{}", DIM, RESET)?;
        writeln!(writer)?;
        return Ok(());
    }

    // Column widths
    let (max_user_w, max_status_w, max_project_w, max_workspace_w) = rows.iter().fold(
        (8, 10, 12, 14), // minimum widths
        |(uw, sw, pw, ww), row| {
            (
                uw.max(row.user.len()),
                sw.max(row.status.len()),
                pw.max(row.projects.len()),
                ww.max(row.workspace.len()),
            )
        },
    );

    let total_w = max_user_w + max_status_w + max_project_w + max_workspace_w + 13;

    // Title bar
    writeln!(
        writer,
        "{}┌─{}{} {:<total_w$} {}─┐{}",
        DIM,
        RESET,
        BOLD,
        format!("RUNTIMES ({})", rows.len()),
        RESET,
        DIM,
        total_w = total_w.saturating_sub(12),
    )?;

    // Top border
    let top = format!(
        "┬─{user:─<user_w$}─┬─{status:─<status_w$}─┬─{projects:─<project_w$}─┬─{workspace:─<workspace_w$}─┐",
        user = "", user_w = max_user_w,
        status = "", status_w = max_status_w,
        projects = "", project_w = max_project_w,
        workspace = "", workspace_w = max_workspace_w,
    );
    writeln!(writer, "{}{}{}", DIM, top, RESET)?;

    // Header row
    let header = format!(
        "│ {:<user_w$} │ {:<status_w$} │ {:<project_w$} │ {:<workspace_w$} │",
        bold("USER"),
        bold("STATUS"),
        bold("PROJECTS"),
        bold("WORKSPACE"),
        user_w = max_user_w,
        status_w = max_status_w,
        project_w = max_project_w,
        workspace_w = max_workspace_w,
    );
    writeln!(writer, "{}{}{}", BLUE, header, RESET)?;

    // Separator
    let sep = format!(
        "├─{user:─<user_w$}─┼─{status:─<status_w$}─┼─{projects:─<project_w$}─┼─{workspace:─<workspace_w$}─┤",
        user = "", user_w = max_user_w,
        status = "", status_w = max_status_w,
        projects = "", project_w = max_project_w,
        workspace = "", workspace_w = max_workspace_w,
    );
    writeln!(writer, "{}{}{}", DIM, sep, RESET)?;

    // Data rows
    for row in rows {
        let status_colored = if row.status == "running" {
            green(&row.status)
        } else {
            red(&row.status)
        };
        let line = format!(
            "│ {} {:<user_w$} │ {:<status_w$} │ {:<project_w$} │ {:<workspace_w$} │",
            row.icon,
            row.user,
            status_colored,
            row.projects,
            row.workspace,
            user_w = max_user_w,
            status_w = max_status_w,
            project_w = max_project_w,
            workspace_w = max_workspace_w,
        );
        writeln!(writer, "{}", line)?;
    }

    // Bottom border
    let bottom = format!(
        "└─{user:─<user_w$}─┴─{status:─<status_w$}─┴─{projects:─<project_w$}─┴─{workspace:─<workspace_w$}─┘",
        user = "", user_w = max_user_w,
        status = "", status_w = max_status_w,
        projects = "", project_w = max_project_w,
        workspace = "", workspace_w = max_workspace_w,
    );
    writeln!(writer, "{}{}{}", DIM, bottom, RESET)?;

    // Summary
    let running = rows.iter().filter(|r| r.status == "running").count();
    let stopped = rows.len() - running;
    writeln!(
        writer,
        "  {}{} running{}  {}{} stopped{}",
        GREEN, running, RESET,
        RED, stopped, RESET,
    )?;

    Ok(())
}

// ── Legacy helpers (kept for compatibility) ─────────────────

/// Print a success message to a writer.
pub fn success<W: Write>(writer: &mut W, msg: &str) -> std::io::Result<()> {
    writeln!(writer, "{}✓{} {msg}", GREEN, RESET)
}

/// Print an info message to a writer.
pub fn info<W: Write>(writer: &mut W, msg: &str) -> std::io::Result<()> {
    writeln!(writer, "{}ℹ{} {msg}", BLUE, RESET)
}

/// Print a warning message to a writer.
pub fn warn<W: Write>(writer: &mut W, msg: &str) -> std::io::Result<()> {
    writeln!(writer, "{}⚠{} {msg}", YELLOW, RESET)
}

// ── Tests ───────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_table_output() {
        let mut buf = Vec::new();
        let rows = vec![
            StatusRow::new("client-a", true, "running".to_string(), &["project-a".into()], Some(std::path::Path::new("/home/client-a/projects/project-a"))),
            StatusRow::new("client-b", false, "stopped".to_string(), &["project-a".into(), "project-b".into()], None),
        ];
        render_status_table(&mut buf, &rows).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("client-a"));
        assert!(output.contains("client-b"));
        assert!(output.contains("running"));
        assert!(output.contains("stopped"));
        assert!(output.contains("RUNTIMES"));
    }

    #[test]
    fn test_empty_table() {
        let mut buf = Vec::new();
        render_status_table(&mut buf, &[]).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("No runtimes found"));
    }
}
