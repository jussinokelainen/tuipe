use crate::Tuipe;
use crate::tuipe::State;
use crate::tuipe::opt_testtype::TestType;
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

impl Tuipe {
    // Input controls
    pub fn end_screen_input(&mut self, keycode: crossterm::event::KeyCode) {
        match keycode {
            KeyCode::Tab => self.restart_test(),
            KeyCode::Esc => self.state = State::MainMenu,
            KeyCode::Char('q') => self.should_exit = true,
            _ => {}
        }
    }

    // Renders the interrupted test end screen
    pub fn render_test_interrupted(&mut self, frame: &mut Frame) {
        let input_area = self.create_layout(40, 16, frame);

        let mut lines: Vec<Line<'_>> = Vec::new();
        lines.push(Line::from(Span::styled(
            format!("Test failed: {}", TestType::as_string(&self.test.ttype)),
            Style::default().fg(Color::Red),
        )));

        lines.push(Line::from(Span::raw("")));

        lines.push(Line::from(Span::styled(
            format!("WPM: {}", (self.stats.wpm * 100.0).round() / 100.0),
            Style::default().fg(Color::Blue),
        )));
        lines.push(Line::from(Span::styled(
            format!("Accuracy: {}%", (self.stats.accuracy * 100.0).round()),
            Style::default().fg(Color::Blue),
        )));
        lines.push(Line::from(Span::raw("")));

        lines.push(Line::from(Span::raw(format!(
            "Time: {} seconds",
            self.stats.time / 1000.0
        ))));
        lines.push(Line::from(Span::raw(format!(
            "raw WPM: {}",
            (self.stats.wpm_raw * 100.0).round() / 100.0
        ))));
        lines.push(Line::from(Span::raw(format!(
            "Characters: {}",
            self.stats.typed_characters
        ))));
        lines.push(Line::from(Span::raw(format!(
            "Words: {}",
            self.stats.typed_words
        ))));

        self.add_endscreen_controls(&mut lines);

        let input = Paragraph::new(lines)
            .style(Style::default())
            .centered()
            .block(Block::new());
        frame.render_widget(input, input_area);
    }

    // Renders the successful test end screen
    pub fn render_test_finished(&mut self, frame: &mut Frame) {
        let input_area = self.create_layout(40, 16, frame);

        let mut lines: Vec<Line<'_>> = Vec::new();
        lines.push(Line::from(Span::styled(
            format!("Test done: {}", TestType::as_string(&self.test.ttype)),
            Style::default().fg(Color::Green),
        )));

        lines.push(Line::from(Span::raw("")));

        lines.push(Line::from(Span::styled(
            format!("WPM: {}", (self.stats.wpm * 100.0).round() / 100.0),
            Style::default().fg(Color::Blue),
        )));
        lines.push(Line::from(Span::styled(
            format!("Accuracy: {}%", (self.stats.accuracy * 100.0).round()),
            Style::default().fg(Color::Blue),
        )));
        lines.push(Line::from(Span::raw("")));

        lines.push(Line::from(Span::raw(format!(
            "Time: {} seconds",
            self.stats.time / 1000.0
        ))));
        lines.push(Line::from(Span::raw(format!(
            "raw WPM: {}",
            (self.stats.wpm_raw * 100.0).round() / 100.0
        ))));
        lines.push(Line::from(Span::raw(format!(
            "Characters: {}",
            self.stats.typed_characters
        ))));
        lines.push(Line::from(Span::raw(format!(
            "Words: {}",
            self.stats.typed_words
        ))));

        lines.push(Line::from(Span::raw("")));
        match self.save_success {
            Ok(_) => lines.push(Line::from(Span::styled(
                "Results saved successfully!",
                Style::default().fg(Color::Green),
            ))),
            Err(_) => lines.push(Line::from(Span::styled(
                "Failed to save results",
                Style::default().fg(Color::Red),
            ))),
        }

        self.add_endscreen_controls(&mut lines);

        let input = Paragraph::new(lines)
            .style(Style::default())
            .centered()
            .block(Block::new());
        frame.render_widget(input, input_area);
    }

    // Adds the end screen controls as dark gray to the lines vector
    fn add_endscreen_controls(&self, lines: &mut Vec<Line<'_>>) {
        lines.push(Line::from(Span::raw("")));
        lines.push(Line::from(Span::raw("")));
        lines.push(Line::from(Span::styled(
            "Restart test: Tab",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::styled(
            "Back to main menu: Esc",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::styled(
            "Quit: q",
            Style::default().fg(Color::DarkGray),
        )));
    }
}
