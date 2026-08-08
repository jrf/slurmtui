use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
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
            Style::default().fg(colors::purple())
        } else {
            Style::default().fg(colors::comment())
        };
        let search = Paragraph::new(search_text).style(cursor_style).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors::fg_gutter()))
                .title(Span::styled(" Search ", Style::default().fg(colors::blue()))),
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
    let mut filtered: Vec<_> = app
        .history
        .iter()
        .filter(|h| {
            if search.is_empty() {
                return true;
            }
            h.job_id.to_lowercase().contains(&search)
                || h.job_name.to_lowercase().contains(&search)
                || h.partition.to_lowercase().contains(&search)
                || h.state.to_lowercase().contains(&search)
        })
        .collect();
    let (hcol, hdir) = app.history_sort;
    filtered.sort_by(|a, b| {
        let ord = cmp_history_col(a, b, hcol);
        if hdir == SortDir::Asc {
            ord
        } else {
            ord.reverse()
        }
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

    let col_specs = [
        (
            "Job ID",
            Some(HistorySort::JobId),
            Constraint::Length(10),
            0,
        ),
        ("Name", Some(HistorySort::Name), Constraint::Fill(1), 0),
        (
            "Partition",
            Some(HistorySort::Partition),
            Constraint::Length(12),
            85,
        ),
        ("State", Some(HistorySort::State), Constraint::Length(10), 0),
        (
            "Elapsed",
            Some(HistorySort::Elapsed),
            Constraint::Length(12),
            0,
        ),
        (
            "CPUTime",
            Some(HistorySort::CpuTime),
            Constraint::Length(14),
            100,
        ),
        (
            "MaxRSS",
            Some(HistorySort::MaxRss),
            Constraint::Length(10),
            55,
        ),
        ("Exit", None, Constraint::Length(8), 0),
    ];

    let visible_cols: Vec<_> = col_specs
        .iter()
        .filter(|&&(_, _, _, min_w)| area.width >= min_w)
        .collect();

    let header_cells = visible_cols.iter().map(|&(label, sort_key, _, _)| {
        if let Some(col) = sort_key {
            Cell::from(hdr(label, *col))
        } else {
            Cell::from(label.to_string())
        }
    });

    let header = Row::new(header_cells).style(
        Style::default()
            .fg(colors::blue())
            .add_modifier(Modifier::BOLD),
    );

    let rows: Vec<Row> = filtered
        .iter()
        .map(|entry| {
            let state_color = match entry.state.as_str() {
                "COMPLETED" => colors::green(),
                "FAILED" => colors::red(),
                "CANCELLED" | "CANCELLED+" => colors::comment(),
                "TIMEOUT" => colors::orange(),
                "RUNNING" => colors::teal(),
                "PENDING" => colors::purple(),
                _ => colors::fg(),
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

            let mut cells = Vec::with_capacity(visible_cols.len());
            for &(label, _, _, _) in &visible_cols {
                let cell = match *label {
                    "Job ID" => Cell::from(Span::styled(
                        entry.job_id.as_str(),
                        Style::default().fg(colors::fg()),
                    )),
                    "Name" => Cell::from(Span::styled(
                        entry.job_name.as_str(),
                        Style::default().fg(colors::fg()),
                    )),
                    "Partition" => Cell::from(Span::styled(
                        entry.partition.as_str(),
                        Style::default().fg(colors::fg_dark()),
                    )),
                    "State" => Cell::from(Span::styled(
                        state_indicator,
                        Style::default().fg(state_color),
                    )),
                    "Elapsed" => Cell::from(Span::styled(
                        entry.elapsed.as_str(),
                        Style::default().fg(colors::cyan()),
                    )),
                    "CPUTime" => Cell::from(Span::styled(
                        entry.cpu_time.as_str(),
                        Style::default().fg(colors::dark5()),
                    )),
                    "MaxRSS" => Cell::from(Span::styled(
                        rss_display.clone(),
                        Style::default().fg(colors::magenta()),
                    )),
                    "Exit" => Cell::from(Span::styled(
                        entry.exit_code.as_str(),
                        Style::default().fg(colors::fg_dark()),
                    )),
                    _ => Cell::from(""),
                };
                cells.push(cell);
            }
            Row::new(cells)
        })
        .collect();

    let table_area = chunks[1];
    app.history_viewport = table_area.height.saturating_sub(3).max(1);
    if rows.is_empty() {
        let msg = Paragraph::new(
            Line::from(Span::styled(
                "No history entries",
                Style::default().fg(colors::comment()),
            ))
            .centered(),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors::fg_gutter()))
                .title(Span::styled(title, Style::default().fg(colors::blue()))),
        );
        frame.render_widget(msg, table_area);
    } else {
        let widths: Vec<Constraint> = visible_cols
            .iter()
            .map(|&(_, _, width, _)| *width)
            .collect();

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(colors::fg_gutter()))
                    .title(Span::styled(title, Style::default().fg(colors::blue()))),
            )
            .row_highlight_style(
                Style::default()
                    .bg(colors::bg_highlight())
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
