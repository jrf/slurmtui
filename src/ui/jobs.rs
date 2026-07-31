use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::app::{cmp_job_col, job_total_gpus, App, JobFilter, JobSort, SortDir};
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
        let search = Paragraph::new(search_text).style(cursor_style).block(
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
    let mut filtered: Vec<_> = app
        .jobs
        .iter()
        .filter(|j| {
            if search.is_empty() {
                return true;
            }
            j.job_id.to_lowercase().contains(&search)
                || j.name.to_lowercase().contains(&search)
                || j.partition.to_lowercase().contains(&search)
                || j.state.to_lowercase().contains(&search)
                || j.user.to_lowercase().contains(&search)
                || j.reason_or_nodelist.to_lowercase().contains(&search)
        })
        .collect();
    let (jcol, jdir) = app.jobs_sort;
    filtered.sort_by(|a, b| {
        let ord = cmp_job_col(a, b, jcol);
        if jdir == SortDir::Asc {
            ord
        } else {
            ord.reverse()
        }
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

    let col_specs = [
        ("Job ID", Some(JobSort::JobId), Constraint::Length(10), 0),
        ("Name", Some(JobSort::Name), Constraint::Fill(1), 70),
        (
            "Partition",
            Some(JobSort::Partition),
            Constraint::Length(12),
            95,
        ),
        ("State", Some(JobSort::State), Constraint::Length(8), 0),
        ("GPUs", Some(JobSort::Gpus), Constraint::Length(6), 115),
        ("CPUs", Some(JobSort::Cpus), Constraint::Length(6), 90),
        ("Memory", Some(JobSort::Memory), Constraint::Length(8), 110),
        (
            "Elapsed",
            Some(JobSort::Elapsed),
            Constraint::Length(10),
            80,
        ),
        (
            "TimeLimit",
            Some(JobSort::TimeLimit),
            Constraint::Length(10),
            105,
        ),
        ("Node/Reason", None, Constraint::Fill(1), 0),
        ("User", Some(JobSort::User), Constraint::Length(10), 55),
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
            .fg(colors::BLUE)
            .add_modifier(Modifier::BOLD),
    );

    let rows: Vec<Row> = filtered
        .iter()
        .map(|job| {
            let mut cells = Vec::with_capacity(visible_cols.len());
            for &(label, _, _, _) in &visible_cols {
                let cell = match *label {
                    "Job ID" => Cell::from(Span::styled(
                        job.job_id.as_str(),
                        Style::default().fg(colors::FG),
                    )),
                    "Name" => Cell::from(Span::styled(
                        job.name.as_str(),
                        Style::default().fg(colors::FG),
                    )),
                    "Partition" => Cell::from(Span::styled(
                        job.partition.as_str(),
                        Style::default().fg(colors::FG_DARK),
                    )),
                    "State" => {
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
                        Cell::from(Span::styled(
                            state_indicator,
                            Style::default().fg(state_color),
                        ))
                    }
                    "GPUs" => {
                        let total_gpus = job_total_gpus(job);
                        let gpu_text = if total_gpus == 0 {
                            "-".to_string()
                        } else if job.num_nodes > 1 {
                            format!("{} ({}/node)", total_gpus, job.gpus_per_node)
                        } else {
                            total_gpus.to_string()
                        };
                        let gpu_style = if total_gpus == 0 {
                            Style::default().fg(colors::DARK5)
                        } else {
                            Style::default().fg(colors::MAGENTA)
                        };
                        Cell::from(Span::styled(gpu_text, gpu_style))
                    }
                    "CPUs" => Cell::from(Span::styled(
                        job.cpus.to_string(),
                        Style::default().fg(colors::MAGENTA),
                    )),
                    "Memory" => Cell::from(Span::styled(
                        job.memory.as_str(),
                        Style::default().fg(colors::MAGENTA),
                    )),
                    "Elapsed" => Cell::from(Span::styled(
                        job.elapsed.as_str(),
                        Style::default().fg(colors::CYAN),
                    )),
                    "TimeLimit" => Cell::from(Span::styled(
                        job.time_limit.as_str(),
                        Style::default().fg(colors::DARK5),
                    )),
                    "Node/Reason" => Cell::from(Span::styled(
                        job.reason_or_nodelist.as_str(),
                        Style::default().fg(colors::FG_DARK),
                    )),
                    "User" => Cell::from(Span::styled(
                        job.user.as_str(),
                        Style::default().fg(colors::FG_DARK),
                    )),
                    _ => Cell::from(""),
                };
                cells.push(cell);
            }
            Row::new(cells)
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
        let msg = Paragraph::new(
            Line::from(Span::styled(
                empty_msg,
                Style::default().fg(colors::COMMENT),
            ))
            .centered(),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors::FG_GUTTER))
                .title(Span::styled(title, Style::default().fg(colors::BLUE))),
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
