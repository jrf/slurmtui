mod history;
mod jobs;
mod nodes;
mod popup;
mod submit;

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
};

use crate::app::{App, Popup, Tab};
use crate::colors;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(frame.area());

    render_tab_bar(frame, chunks[0], app);

    match app.active_tab {
        Tab::Jobs => jobs::render(frame, chunks[1], app),
        Tab::Nodes => nodes::render(frame, chunks[1], app),
        Tab::Submit => submit::render(frame, chunks[1], app),
        Tab::History => history::render(frame, chunks[1], app),
    }

    render_status_bar(frame, chunks[2], app);

    match &app.popup {
        Popup::None => {}
        Popup::JobDetail(detail) => popup::render_job_detail(frame, frame.area(), detail, app.popup_scroll),
        Popup::ConfirmCancel { job_id } => {
            popup::render_confirm(frame, frame.area(), &format!("Cancel job {}?", job_id));
        }
        Popup::SubmitConfirm => {
            let cmd = app.submit_form.to_command_string();
            popup::render_confirm(frame, frame.area(), &format!("Submit: {}?", cmd));
        }
        Popup::SubmitResult { success, message } => {
            popup::render_result(frame, frame.area(), *success, message);
        }
        Popup::FilePicker(picker) => {
            popup::render_file_picker(frame, frame.area(), picker);
        }
    }
}

fn render_tab_bar(frame: &mut Frame, area: Rect, app: &App) {
    let titles: Vec<Line> = Tab::ALL
        .iter()
        .enumerate()
        .map(|(i, tab)| {
            Line::from(format!(" {} {} ", i + 1, tab.title()))
        })
        .collect();

    let refresh_secs = app.time_until_refresh().as_secs();
    let right_text = if let Some(status) = app.status_text() {
        status.to_string()
    } else {
        format!("↻ {}s", refresh_secs)
    };

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors::FG_GUTTER))
                .title(Span::styled(" slurmtop ", Style::default().fg(colors::BLUE).add_modifier(Modifier::BOLD)))
                .title_bottom(Line::from(Span::styled(format!(" {} ", right_text), Style::default().fg(colors::DARK5))).right_aligned()),
        )
        .select(app.active_tab.index())
        .style(Style::default().fg(colors::DARK5))
        .highlight_style(
            Style::default()
                .fg(colors::BLUE)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(tabs, area);
}

fn render_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let spans = match app.active_tab {
        Tab::Jobs => {
            if app.job_search_active {
                vec![
                    key_span("Esc"), desc_span(":Done  "),
                    key_span("Type"), desc_span(":Search"),
                ]
            } else {
                vec![
                    key_span("j/k"), desc_span(":Nav  "),
                    key_span("Enter"), desc_span(":Detail  "),
                    key_span("d"), desc_span(":Cancel  "),
                    key_span("/"), desc_span(":Search  "),
                    key_span("f"), desc_span(":Filter  "),
                    key_span("r"), desc_span(":Refresh  "),
                    key_span("q"), desc_span(":Quit"),
                ]
            }
        }
        Tab::Nodes => vec![
            key_span("j/k"), desc_span(":Nav  "),
            key_span("r"), desc_span(":Refresh  "),
            key_span("q"), desc_span(":Quit"),
        ],
        Tab::Submit => {
            if app.submit_form.editing {
                vec![
                    key_span("Type"), desc_span(":Input  "),
                    key_span("Enter/Esc"), desc_span(":Done"),
                ]
            } else if app.submit_form.active_field == 0 {
                vec![
                    key_span("j/k"), desc_span(":Field  "),
                    key_span("Enter"), desc_span(":Edit  "),
                    key_span("b"), desc_span(":Browse  "),
                    key_span("C-s"), desc_span(":Submit  "),
                    key_span("q"), desc_span(":Quit"),
                ]
            } else {
                vec![
                    key_span("j/k"), desc_span(":Field  "),
                    key_span("Enter"), desc_span(":Edit  "),
                    key_span("C-s"), desc_span(":Submit  "),
                    key_span("q"), desc_span(":Quit"),
                ]
            }
        }
        Tab::History => {
            if app.history_search_active {
                vec![
                    key_span("Esc"), desc_span(":Done  "),
                    key_span("Type"), desc_span(":Search"),
                ]
            } else {
                vec![
                    key_span("j/k"), desc_span(":Nav  "),
                    key_span("Enter"), desc_span(":Detail  "),
                    key_span("/"), desc_span(":Search  "),
                    key_span("f"), desc_span(":Range  "),
                    key_span("r"), desc_span(":Refresh  "),
                    key_span("q"), desc_span(":Quit"),
                ]
            }
        }
    };

    let paragraph = Paragraph::new(Line::from(spans));
    frame.render_widget(paragraph, area);
}

fn key_span(text: &str) -> Span<'_> {
    Span::styled(text, Style::default().fg(colors::YELLOW).add_modifier(Modifier::BOLD))
}

fn desc_span(text: &str) -> Span<'_> {
    Span::styled(text, Style::default().fg(colors::DARK5))
}
