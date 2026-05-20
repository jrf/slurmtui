use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Row, Table, Wrap},
};

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

pub fn render_job_detail(frame: &mut Frame, area: Rect, detail: &JobDetail, scroll: u16) {
    let popup_area = centered_rect(70, 75, area);
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors::BLUE))
        .title(Span::styled(
            " Job Detail (q:close, j/k:scroll) ",
            Style::default().fg(colors::BLUE).add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().fg(colors::FG).bg(colors::BG_DARK));
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
                        .fg(colors::YELLOW)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(value.as_str(), Style::default().fg(colors::FG)),
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
        .border_style(Style::default().fg(colors::YELLOW))
        .title(Span::styled(
            " Confirm ",
            Style::default().fg(colors::YELLOW).add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().fg(colors::FG).bg(colors::BG_DARK));
    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let text = vec![
        Line::from(Span::styled(message, Style::default().fg(colors::FG))),
        Line::from(""),
        Line::from(vec![
            Span::styled("y", Style::default().fg(colors::GREEN).add_modifier(Modifier::BOLD)),
            Span::styled(":yes  ", Style::default().fg(colors::DARK5)),
            Span::styled("n", Style::default().fg(colors::RED).add_modifier(Modifier::BOLD)),
            Span::styled(":no", Style::default().fg(colors::DARK5)),
        ]),
    ];

    let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner);
}

pub fn render_result(frame: &mut Frame, area: Rect, success: bool, message: &str) {
    let height = 5u16;
    let width = (message.len() as u16 + 6).max(30).min(area.width - 4);
    let popup_area = small_centered_rect(width, height, area);
    frame.render_widget(Clear, popup_area);

    let (title, color) = if success {
        (" Success ", colors::GREEN)
    } else {
        (" Error ", colors::RED)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .title(Span::styled(title, Style::default().fg(color).add_modifier(Modifier::BOLD)))
        .style(Style::default().fg(colors::FG).bg(colors::BG_DARK));
    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let text = vec![
        Line::from(Span::styled(message, Style::default().fg(colors::FG))),
        Line::from(""),
        Line::from(Span::styled("Press Enter or q to close", Style::default().fg(colors::DARK5))),
    ];

    let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner);
}
