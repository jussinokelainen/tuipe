use crate::Tuipe;
use crate::tuipe::State;
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

impl Tuipe {
    // Input controls
    pub fn stats_screen_input(&mut self, keycode: crossterm::event::KeyCode) {
        match keycode {
            KeyCode::Esc => self.state = State::MainMenu,
            KeyCode::Char('q') => self.should_exit = true,
            _ => {}
        }
    }

    // Rendering
    pub fn render_stats_screen(&mut self, frame: &mut Frame) {
        let input_area = self.create_layout(40, 16, frame);

        let mut lines: Vec<Line<'_>> = Vec::new();
        let stats_res = self.get_stats_from_db();
        match stats_res {
            Ok(results) => {
                let tests_count = results.len();
                let mut total_wpm = 0.0;
                let mut total_wpm_raw = 0.0;
                let mut total_acc = 0.0;
                let mut record_wpm = 0.0;
                let mut record_language = String::new();
                let mut record_ttype = String::new();
                for result in &results {
                    total_wpm += result.wpm;
                    total_wpm_raw += result.raw_wpm;
                    total_acc += result.accuracy;

                    if result.wpm > record_wpm {
                        record_wpm = result.wpm;
                        record_language = result.language.clone();
                        record_ttype = result.test_type.clone();
                    }
                }
                let avg_wpm = ((total_wpm / tests_count as f64) * 10.0).round() / 10.0;
                let avg_wpm_raw = ((total_wpm_raw / tests_count as f64) * 10.0).round() / 10.0;
                let avg_acc = ((total_acc / tests_count as f64) * 1000.0).round() / 10.0;

                lines.push(Line::from(Span::styled(
                    "Your stats:",
                    Style::default().fg(Color::Cyan),
                )));
                lines.push(Line::from(Span::raw("")));
                lines.push(Line::from(Span::styled(
                    "Personal best:",
                    Style::default().fg(Color::Green),
                )));
                lines.push(Line::from(Span::raw(format!(
                    "Test type: {}",
                    record_ttype
                ))));
                lines.push(Line::from(Span::raw(format!(
                    "wpm: {}",
                    (record_wpm * 10.0).round() / 10.0
                ))));
                lines.push(Line::from(Span::raw(format!(
                    "Language: {}",
                    record_language
                ))));

                lines.push(Line::from(Span::raw("")));
                lines.push(Line::from(Span::styled(
                    "Overall:",
                    Style::default().fg(Color::Green),
                )));
                lines.push(Line::from(Span::raw(format!(
                    "Tests completed: {}",
                    tests_count
                ))));
                lines.push(Line::from(Span::raw(format!("Average wpm: {}", avg_wpm))));
                lines.push(Line::from(Span::raw(format!(
                    "Average raw wpm: {}",
                    avg_wpm_raw
                ))));
                lines.push(Line::from(Span::raw(format!(
                    "Average accuracy: {}%",
                    avg_acc
                ))));
            }
            Err(e) => {
                lines.push(Line::from(Span::styled(
                    "Error:",
                    Style::default().fg(Color::Red),
                )));
                lines.push(Line::from(Span::raw(e.to_string())));
            }
        }

        lines.push(Line::from(Span::raw("")));
        lines.push(Line::from(Span::raw("")));
        lines.push(Line::from(Span::styled(
            "Back: Esc",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::styled(
            "Quit: q",
            Style::default().fg(Color::DarkGray),
        )));

        let input = Paragraph::new(lines)
            .style(Style::default())
            .centered()
            .block(Block::new());
        frame.render_widget(input, input_area);
    }
}
