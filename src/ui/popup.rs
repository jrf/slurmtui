use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Row, Table, Wrap},
};

use crate::app::{FilePicker, LogView, ThemePicker};
use crate::colors;
use crate::slurm::JobDetail;

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}

fn small_centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

/// Truncate a path to fit a char budget, keeping the tail (which is usually
/// the most informative — basename). Prefixes with `…` when shortened.
fn truncate_path(path: &str, max_chars: usize) -> String {
    let len = path.chars().count();
    if len <= max_chars {
        return path.to_string();
    }
    if max_chars <= 1 {
        return "…".to_string();
    }
    let keep = max_chars - 1;
    let skip = len - keep;
    let tail: String = path.chars().skip(skip).collect();
    format!("…{}", tail)
}

pub fn render_job_detail(frame: &mut Frame, area: Rect, detail: &JobDetail, scroll: u16) {
    let popup_area = centered_rect(70, 75, area);
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors::blue()))
        .title(Span::styled(
            " Job Detail (q:close, j/k:scroll, g/G:top/bot) ",
            Style::default()
                .fg(colors::blue())
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().fg(colors::fg()).bg(colors::bg_dark()));
    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let widths = [Constraint::Length(20), Constraint::Fill(1)];

    let visible_rows: Vec<Row> = detail
        .fields
        .iter()
        .skip(scroll as usize)
        .map(|(key, value)| {
            Row::new(vec![
                Span::styled(
                    key.as_str(),
                    Style::default()
                        .fg(colors::purple())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(value.as_str(), Style::default().fg(colors::fg())),
            ])
        })
        .collect();

    let table = Table::new(visible_rows, widths);
    frame.render_widget(table, inner);
}

pub fn render_confirm(frame: &mut Frame, area: Rect, message: &str) {
    let height = 5u16;
    let width = (message.len() as u16 + 6).max(30).min(area.width - 4);
    let popup_area = small_centered_rect(width, height, area);
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors::purple()))
        .title(Span::styled(
            " Confirm ",
            Style::default()
                .fg(colors::purple())
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().fg(colors::fg()).bg(colors::bg_dark()));
    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let text = vec![
        Line::from(Span::styled(message, Style::default().fg(colors::fg()))),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "y",
                Style::default()
                    .fg(colors::green())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(":yes  ", Style::default().fg(colors::dark5())),
            Span::styled(
                "n",
                Style::default()
                    .fg(colors::red())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(":no", Style::default().fg(colors::dark5())),
        ]),
    ];

    let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner);
}

pub fn render_log_view(frame: &mut Frame, area: Rect, view: &LogView) {
    let popup_area = centered_rect(85, 85, area);
    frame.render_widget(Clear, popup_area);

    let follow_label = if view.follow { "ON" } else { "off" };
    let popup_inner_w = popup_area.width.saturating_sub(2) as usize;
    let prefix = format!(" Log [{}] job {} — ", view.kind.label(), view.job_id,);
    let suffix = format!(" (follow:{}) ", follow_label);
    let path_raw = if view.path.is_empty() {
        "(no path)".to_string()
    } else {
        view.path.clone()
    };
    let path_budget = popup_inner_w.saturating_sub(prefix.chars().count() + suffix.chars().count());
    let path_display = truncate_path(&path_raw, path_budget);
    let title = format!("{}{}{}", prefix, path_display, suffix);

    let hints = " f:toggle  r:reload  t:switch  j/k:scroll  g/G:top/bot  q:close ";
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors::blue()))
        .title(Span::styled(
            title,
            Style::default()
                .fg(colors::blue())
                .add_modifier(Modifier::BOLD),
        ))
        .title_bottom(
            Line::from(Span::styled(hints, Style::default().fg(colors::dark5()))).right_aligned(),
        )
        .style(Style::default().fg(colors::fg()).bg(colors::bg_dark()));
    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    if let Some(err) = &view.error {
        let msg = Paragraph::new(Line::from(Span::styled(
            err.as_str(),
            Style::default().fg(colors::red()),
        )))
        .wrap(Wrap { trim: false });
        frame.render_widget(msg, inner);
        return;
    }

    if view.contents.is_empty() {
        let msg = Paragraph::new(Line::from(Span::styled(
            "<empty>",
            Style::default().fg(colors::dark5()),
        )));
        frame.render_widget(msg, inner);
        return;
    }

    let lines: Vec<Line> = view
        .contents
        .lines()
        .map(|l| {
            Line::from(Span::styled(
                l.to_string(),
                Style::default().fg(colors::fg()),
            ))
        })
        .collect();
    let paragraph = Paragraph::new(lines)
        .scroll((view.scroll, 0))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner);
}

pub fn render_file_picker(frame: &mut Frame, area: Rect, picker: &FilePicker) {
    let popup_area = centered_rect(70, 70, area);
    frame.render_widget(Clear, popup_area);

    let mode = if picker.show_all { "all" } else { "scripts" };
    let title = if picker.query_active {
        format!(
            " Find [{}] in {} (Up/Down  Enter:pick  Esc:cancel) ",
            mode,
            picker.current_dir.display()
        )
    } else {
        format!(
            " Browse [{}]: {} (j/k:nav  Enter:open  h/Bksp:up  /:find  a:toggle  Esc:cancel) ",
            mode,
            picker.current_dir.display()
        )
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors::blue()))
        .title(Span::styled(
            title,
            Style::default()
                .fg(colors::blue())
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().fg(colors::fg()).bg(colors::bg_dark()));
    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    if picker.query_active {
        render_query_mode(frame, inner, picker);
    } else {
        render_browse_mode(frame, inner, picker);
    }
}

pub fn render_theme_picker(frame: &mut Frame, area: Rect, picker: &ThemePicker) {
    let desired_height = picker.names.len().saturating_add(2) as u16;
    let height = desired_height.min(area.height.saturating_sub(4)).max(5);
    let popup_area = small_centered_rect(48, height, area);
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors::blue()))
        .title(Span::styled(
            " Theme ",
            Style::default()
                .fg(colors::blue())
                .add_modifier(Modifier::BOLD),
        ))
        .title_bottom(
            Line::from(Span::styled(
                " j/k:preview  enter:save  esc:cancel ",
                Style::default().fg(colors::dark5()),
            ))
            .centered(),
        )
        .style(Style::default().fg(colors::fg()).bg(colors::bg_dark()));
    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let visible = inner.height as usize;
    let start = if picker.selected >= visible {
        picker.selected + 1 - visible
    } else {
        0
    };
    let end = (start + visible).min(picker.names.len());
    let width = inner.width as usize;
    let lines: Vec<Line> = picker.names[start..end]
        .iter()
        .enumerate()
        .map(|(offset, name)| {
            let selected = start + offset == picker.selected;
            let marker = if selected { "› " } else { "  " };
            let display = format!("{marker}{}", name.replace('-', " "));
            let padded = format!("{display:<width$}");
            let style = if selected {
                Style::default()
                    .fg(colors::purple())
                    .bg(colors::bg_highlight())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(colors::fg())
            };
            Line::from(Span::styled(padded, style))
        })
        .collect();
    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_browse_mode(frame: &mut Frame, inner: Rect, picker: &FilePicker) {
    if picker.entries.is_empty() {
        let empty = Paragraph::new(Line::from(Span::styled(
            "<no matching entries — press `a` to show all files>",
            Style::default().fg(colors::dark5()),
        )));
        frame.render_widget(empty, inner);
        return;
    }

    let height = inner.height as usize;
    let total = picker.entries.len();
    let start = if picker.selected >= height {
        picker.selected + 1 - height
    } else {
        0
    };
    let end = (start + height).min(total);

    let lines: Vec<Line> = picker.entries[start..end]
        .iter()
        .enumerate()
        .map(|(i, ent)| {
            let global_i = start + i;
            let display = if ent.is_dir {
                format!("{}/", ent.name)
            } else {
                ent.name.clone()
            };
            let style = if global_i == picker.selected {
                Style::default()
                    .fg(colors::fg())
                    .bg(colors::bg_highlight())
                    .add_modifier(Modifier::BOLD)
            } else if ent.is_dir {
                Style::default().fg(colors::blue())
            } else {
                Style::default().fg(colors::fg())
            };
            Line::from(Span::styled(display, style))
        })
        .collect();

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn render_query_mode(frame: &mut Frame, inner: Rect, picker: &FilePicker) {
    let chunks = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).split(inner);
    let input_area = chunks[0];
    let list_area = chunks[1];

    let prompt = Line::from(vec![
        Span::styled(
            "> ",
            Style::default()
                .fg(colors::purple())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(picker.query.as_str(), Style::default().fg(colors::fg())),
        Span::styled("▌", Style::default().fg(colors::fg())),
    ]);
    frame.render_widget(Paragraph::new(prompt), input_area);

    if picker.query.is_empty() {
        let hint = Paragraph::new(Line::from(Span::styled(
            "<type to fuzzy-find files under this directory>",
            Style::default().fg(colors::dark5()),
        )));
        frame.render_widget(hint, list_area);
        return;
    }

    if picker.matches.is_empty() {
        let empty = Paragraph::new(Line::from(Span::styled(
            "<no matches>",
            Style::default().fg(colors::dark5()),
        )));
        frame.render_widget(empty, list_area);
        return;
    }

    let height = list_area.height as usize;
    let total = picker.matches.len();
    let start = if picker.selected >= height {
        picker.selected + 1 - height
    } else {
        0
    };
    let end = (start + height).min(total);

    let lines: Vec<Line> = picker.matches[start..end]
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let global_i = start + i;
            let style = if global_i == picker.selected {
                Style::default()
                    .fg(colors::fg())
                    .bg(colors::bg_highlight())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(colors::fg())
            };
            Line::from(Span::styled(m.display.as_str(), style))
        })
        .collect();

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, list_area);
}

pub fn render_result(frame: &mut Frame, area: Rect, success: bool, message: &str) {
    let height = 5u16;
    let width = (message.len() as u16 + 6).max(30).min(area.width - 4);
    let popup_area = small_centered_rect(width, height, area);
    frame.render_widget(Clear, popup_area);

    let (title, color) = if success {
        (" Success ", colors::green())
    } else {
        (" Error ", colors::red())
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .title(Span::styled(
            title,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().fg(colors::fg()).bg(colors::bg_dark()));
    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let text = vec![
        Line::from(Span::styled(message, Style::default().fg(colors::fg()))),
        Line::from(""),
        Line::from(Span::styled(
            "Press Enter or q to close",
            Style::default().fg(colors::dark5()),
        )),
    ];

    let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner);
}
