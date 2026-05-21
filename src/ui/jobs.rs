use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use crate::app::{cmp_job_col, App, JobFilter, JobSort, SortDir};
use crate::colors;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    let show_search = app.job_search_active || !app.job_search.is_empty();
    let chunks = if show_search {
        Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(area)
    } else {
        Layout::vertical([Constraint::Length(0), Constraint::Min(0)]).split(area)
    };

    if show_search {
        let search_text = format!("/{}", app.job_search);
        let cursor_style = if app.job_search_active {
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

        if app.job_search_active {
            frame.set_cursor_position((
                chunks[0].x + 1 + app.job_search.len() as u16 + 1,
                chunks[0].y + 1,
            ));
        }
    }

    let search = app.job_search.to_lowercase();
    let mut filtered: Vec<_> = app.jobs.iter().filter(|j| {
        if search.is_empty() { return true; }
        j.job_id.to_lowercase().contains(&search)
            || j.name.to_lowercase().contains(&search)
            || j.partition.to_lowercase().contains(&search)
            || j.state.to_lowercase().contains(&search)
            || j.user.to_lowercase().contains(&search)
            || j.reason_or_nodelist.to_lowercase().contains(&search)
    }).collect();
    let (jcol, jdir) = app.jobs_sort;
    filtered.sort_by(|a, b| {
        let ord = cmp_job_col(a, b, jcol);
        if jdir == SortDir::Asc { ord } else { ord.reverse() }
    });
    let filter_label = match app.job_filter {
        JobFilter::MyJobs => "My Jobs",
        JobFilter::AllJobs => "All Jobs",
    };
    let title = format!(" {} ({}) ", filter_label, filtered.len());

    let (sort_col, sort_dir) = app.jobs_sort;
    let hdr = |label: &str, col: JobSort| -> String {
        if col == sort_col {
            format!("{} {}", label, sort_dir.arrow())
        } else {
            label.to_string()
        }
    };
    let header = Row::new(vec![
        Cell::from(hdr("Job ID", JobSort::JobId)),
        Cell::from(hdr("Name", JobSort::Name)),
        Cell::from(hdr("Partition", JobSort::Partition)),
        Cell::from(hdr("State", JobSort::State)),
        Cell::from(hdr("CPUs", JobSort::Cpus)),
        Cell::from(hdr("Memory", JobSort::Memory)),
        Cell::from(hdr("Elapsed", JobSort::Elapsed)),
        Cell::from(hdr("TimeLimit", JobSort::TimeLimit)),
        Cell::from("Node/Reason"),
        Cell::from(hdr("User", JobSort::User)),
    ])
    .style(
        Style::default()
            .fg(colors::BLUE)
            .add_modifier(Modifier::BOLD),
    );

    let rows: Vec<Row> = filtered
        .iter()
        .map(|job| {
            let state_color = match job.state.as_str() {
                "RUNNING" => colors::GREEN,
                "PENDING" => colors::YELLOW,
                "COMPLETING" => colors::BLUE1,
                "COMPLETED" => colors::TEAL,
                "FAILED" => colors::RED,
                "CANCELLED" => colors::COMMENT,
                "TIMEOUT" => colors::ORANGE,
                _ => colors::FG,
            };
            let state_indicator = match job.state.as_str() {
                "RUNNING" => "● RUN",
                "PENDING" => "○ PD",
                "COMPLETING" => "◑ CG",
                "COMPLETED" => "✓ CD",
                "FAILED" => "✗ F",
                "CANCELLED" => "⊘ CA",
                "TIMEOUT" => "⏰ TO",
                _ => &job.state,
            };

            Row::new(vec![
                Cell::from(Span::styled(job.job_id.as_str(), Style::default().fg(colors::FG))),
                Cell::from(Span::styled(job.name.as_str(), Style::default().fg(colors::FG))),
                Cell::from(Span::styled(job.partition.as_str(), Style::default().fg(colors::FG_DARK))),
                Cell::from(Span::styled(state_indicator, Style::default().fg(state_color))),
                Cell::from(Span::styled(job.cpus.to_string(), Style::default().fg(colors::MAGENTA))),
                Cell::from(Span::styled(job.memory.as_str(), Style::default().fg(colors::MAGENTA))),
                Cell::from(Span::styled(job.elapsed.as_str(), Style::default().fg(colors::CYAN))),
                Cell::from(Span::styled(job.time_limit.as_str(), Style::default().fg(colors::DARK5))),
                Cell::from(Span::styled(job.reason_or_nodelist.as_str(), Style::default().fg(colors::FG_DARK))),
                Cell::from(Span::styled(job.user.as_str(), Style::default().fg(colors::FG_DARK))),
            ])
        })
        .collect();

    let empty_msg = if app.jobs.is_empty() {
        "No jobs found"
    } else {
        "No matching jobs"
    };

    let table_area = chunks[1];
    app.jobs_viewport = table_area.height.saturating_sub(3).max(1);
    if rows.is_empty() {
        let msg = Paragraph::new(Line::from(Span::styled(empty_msg, Style::default().fg(colors::COMMENT))).centered())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(colors::FG_GUTTER))
                    .title(Span::styled(title, Style::default().fg(colors::BLUE))),
            );
        frame.render_widget(msg, table_area);
    } else {
        let widths = [
            Constraint::Length(12),
            Constraint::Length(20),
            Constraint::Length(12),
            Constraint::Length(8),
            Constraint::Length(5),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(12),
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

        frame.render_stateful_widget(table, table_area, &mut app.jobs_table_state);
    }
}
