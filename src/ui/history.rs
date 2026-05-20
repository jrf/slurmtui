use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use crate::app::{cmp_history_col, App, HistorySort, SortDir};
use crate::colors;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    let show_search = app.history_search_active || !app.history_search.is_empty();
    let chunks = if show_search {
        Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(area)
    } else {
        Layout::vertical([Constraint::Length(0), Constraint::Min(0)]).split(area)
    };

    if show_search {
        let search_text = format!("/{}", app.history_search);
        let cursor_style = if app.history_search_active {
            Style::default().fg(colors::YELLOW)
        } else {
            Style::default().fg(colors::COMMENT)
        };
        let search = Paragraph::new(search_text)
            .style(cursor_style)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(colors::FG_GUTTER))
                    .title(Span::styled(" Search ", Style::default().fg(colors::BLUE))),
            );
        frame.render_widget(search, chunks[0]);

        if app.history_search_active {
            frame.set_cursor_position((
                chunks[0].x + 1 + app.history_search.len() as u16 + 1,
                chunks[0].y + 1,
            ));
        }
    }

    let search = app.history_search.to_lowercase();
    let mut filtered: Vec<_> = app.history.iter().filter(|h| {
        if search.is_empty() { return true; }
        h.job_id.to_lowercase().contains(&search)
            || h.job_name.to_lowercase().contains(&search)
            || h.partition.to_lowercase().contains(&search)
            || h.state.to_lowercase().contains(&search)
    }).collect();
    let (hcol, hdir) = app.history_sort;
    filtered.sort_by(|a, b| {
        let ord = cmp_history_col(a, b, hcol);
        if hdir == SortDir::Asc { ord } else { ord.reverse() }
    });
    let title = format!(
        " History - {} ({}) ",
        app.history_range.label(),
        filtered.len()
    );

    let (sort_col, sort_dir) = app.history_sort;
    let hdr = |label: &str, col: HistorySort| -> String {
        if col == sort_col {
            format!("{} {}", label, sort_dir.arrow())
        } else {
            label.to_string()
        }
    };
    let header = Row::new(vec![
        Cell::from(hdr("Job ID", HistorySort::JobId)),
        Cell::from(hdr("Name", HistorySort::Name)),
        Cell::from(hdr("Partition", HistorySort::Partition)),
        Cell::from(hdr("State", HistorySort::State)),
        Cell::from(hdr("Elapsed", HistorySort::Elapsed)),
        Cell::from(hdr("CPUTime", HistorySort::CpuTime)),
        Cell::from(hdr("MaxRSS", HistorySort::MaxRss)),
        Cell::from("Exit"),
    ])
    .style(
        Style::default()
            .fg(colors::BLUE)
            .add_modifier(Modifier::BOLD),
    );

    let rows: Vec<Row> = filtered
        .iter()
        .map(|entry| {
            let state_color = match entry.state.as_str() {
                "COMPLETED" => colors::GREEN,
                "FAILED" => colors::RED,
                "CANCELLED" | "CANCELLED+" => colors::COMMENT,
                "TIMEOUT" => colors::ORANGE,
                "RUNNING" => colors::TEAL,
                "PENDING" => colors::YELLOW,
                _ => colors::FG,
            };
            let state_indicator = match entry.state.as_str() {
                "COMPLETED" => "✓ COMP",
                "FAILED" => "✗ FAIL",
                "CANCELLED" | "CANCELLED+" => "⊘ CA",
                "TIMEOUT" => "⏰ TO",
                "RUNNING" => "● RUN",
                "PENDING" => "○ PD",
                _ => &entry.state,
            };

            let rss_display = format_rss(&entry.max_rss);

            Row::new(vec![
                Cell::from(Span::styled(entry.job_id.as_str(), Style::default().fg(colors::FG))),
                Cell::from(Span::styled(entry.job_name.as_str(), Style::default().fg(colors::FG))),
                Cell::from(Span::styled(entry.partition.as_str(), Style::default().fg(colors::FG_DARK))),
                Cell::from(Span::styled(state_indicator, Style::default().fg(state_color))),
                Cell::from(Span::styled(entry.elapsed.as_str(), Style::default().fg(colors::CYAN))),
                Cell::from(Span::styled(entry.cpu_time.as_str(), Style::default().fg(colors::DARK5))),
                Cell::from(Span::styled(rss_display, Style::default().fg(colors::MAGENTA))),
                Cell::from(Span::styled(entry.exit_code.as_str(), Style::default().fg(colors::FG_DARK))),
            ])
        })
        .collect();

    let table_area = chunks[1];
    app.history_viewport = table_area.height.saturating_sub(3).max(1);
    if rows.is_empty() {
        let msg = Paragraph::new(Line::from(Span::styled("No history entries", Style::default().fg(colors::COMMENT))).centered())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(colors::FG_GUTTER))
                    .title(Span::styled(title, Style::default().fg(colors::BLUE))),
            );
        frame.render_widget(msg, table_area);
    } else {
        let widths = [
            Constraint::Length(10),
            Constraint::Length(22),
            Constraint::Length(14),
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Length(14),
            Constraint::Length(10),
            Constraint::Length(8),
        ];

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(colors::FG_GUTTER))
                    .title(Span::styled(title, Style::default().fg(colors::BLUE))),
            )
            .row_highlight_style(
                Style::default()
                    .bg(colors::BG_HIGHLIGHT)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_stateful_widget(table, table_area, &mut app.history_table_state);
    }
}

fn format_rss(rss: &str) -> String {
    if rss.is_empty() {
        return "-".to_string();
    }
    if let Some(kb_str) = rss.strip_suffix('K') {
        if let Ok(kb) = kb_str.parse::<u64>() {
            if kb >= 1_048_576 {
                return format!("{:.1}G", kb as f64 / 1_048_576.0);
            } else if kb >= 1024 {
                return format!("{:.0}M", kb as f64 / 1024.0);
            } else {
                return format!("{}K", kb);
            }
        }
    }
    rss.to_string()
}
