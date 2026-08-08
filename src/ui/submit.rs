use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::colors;
use crate::slurm::SubmitForm;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(5)]).split(area);

    render_form(frame, chunks[0], &app.submit_form);
    render_preview(frame, chunks[1], &app.submit_form);
}

fn render_form(frame: &mut Frame, area: Rect, form: &SubmitForm) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors::fg_gutter()))
        .title(Span::styled(
            " Submit Job ",
            Style::default().fg(colors::blue()),
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let field_height = 1u16;
    let spacing = 1u16;
    let mut y = inner.y;

    for i in 0..SubmitForm::FIELD_COUNT {
        if y >= inner.y + inner.height {
            break;
        }

        let label = form.field_label(i);
        let value = form.field_value(i);
        let is_active = i == form.active_field;
        let is_editing = is_active && form.editing;

        let label_width = 14u16;
        let label_area = ratatui::layout::Rect::new(inner.x, y, label_width, field_height);
        let value_area = ratatui::layout::Rect::new(
            inner.x + label_width + 1,
            y,
            inner.width.saturating_sub(label_width + 1),
            field_height,
        );

        let label_style = if is_active {
            Style::default()
                .fg(colors::purple())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(colors::dark5())
        };

        let label_widget = Paragraph::new(format!("{}:", label)).style(label_style);
        frame.render_widget(label_widget, label_area);

        let value_display = if i == 2 && !form.available_partitions.is_empty() {
            if value.is_empty() {
                "<Enter to select>".to_string()
            } else {
                format!("{} (Enter to cycle)", value)
            }
        } else if is_editing {
            format!("{}▌", value)
        } else if value.is_empty() {
            "<empty>".to_string()
        } else {
            value.to_string()
        };

        let value_style = if is_editing {
            Style::default().fg(colors::fg()).bg(colors::bg_highlight())
        } else if is_active {
            Style::default().fg(colors::fg())
        } else {
            Style::default().fg(colors::fg_dark())
        };

        let value_widget = Paragraph::new(value_display).style(value_style);
        frame.render_widget(value_widget, value_area);

        if is_editing {
            frame.set_cursor_position((value_area.x + value.len() as u16, value_area.y));
        }

        y += field_height + spacing;
    }
}

fn render_preview(frame: &mut Frame, area: Rect, form: &SubmitForm) {
    let cmd = form.to_command_string();
    let preview = Paragraph::new(Line::from(cmd))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors::fg_gutter()))
                .title(Span::styled(
                    " Command Preview ",
                    Style::default().fg(colors::blue()),
                )),
        )
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(colors::teal()));
    frame.render_widget(preview, area);
}
