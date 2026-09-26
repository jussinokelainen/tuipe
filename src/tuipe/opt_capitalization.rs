use crate::Tuipe;
use crate::tuipe::OptMenu;
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

impl Tuipe {
    // Input controls
    pub fn input_opt_capitals(&mut self, keycode: crossterm::event::KeyCode) {
        match keycode {
            KeyCode::Enter => {
                self.test.capitals = !self.test.capitals;
            }
            KeyCode::Char('q') => self.should_exit = true,
            KeyCode::Esc => self.opt_state = OptMenu::Main,
            _ => {}
        }
    }

    // Rendering
    pub fn render_opt_capitals(&mut self, frame: &mut Frame) {
        let input_area = self.create_layout(40, 16, frame);

        let mut lines: Vec<Line<'_>> = Vec::new();
        lines.push(Line::from(Span::styled(
            format!("Current capitalization: {}", self.test.capitals),
            Style::default().fg(Color::Green),
        )));
        lines.push(Line::from(Span::raw("")));

        lines.push(Line::from(Span::styled(
            format!("> Toggle capitals"),
            Style::default().fg(Color::Blue),
        )));

        lines.push(Line::from(Span::raw("")));
        lines.push(Line::from(Span::raw("")));
        lines.push(Line::from(Span::styled(
            "Select: Enter",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::styled(
            "Quit: q",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::styled(
            "Back: Esc",
            Style::default().fg(Color::DarkGray),
        )));

        let input = Paragraph::new(lines)
            .style(Style::default())
            .centered()
            .block(Block::new());
        frame.render_widget(input, input_area);
    }
}
