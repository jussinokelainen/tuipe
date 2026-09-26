use crate::Tuipe;
use crate::tuipe::State;
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

pub enum TestType {
    Words10,
    Words25,
    Words50,
    Time10,
    Time30,
    Time60,
}

impl TestType {
    pub const COUNT: usize = 6;

    pub fn word_count(&self) -> usize {
        match &self {
            TestType::Words10 => 10,
            TestType::Words25 => 25,
            TestType::Words50 => 50,
            TestType::Time10 => 250,
            TestType::Time30 => 250,
            TestType::Time60 => 500,
        }
    }
    pub fn is_timed(&self) -> (bool, usize) {
        match &self {
            TestType::Words10 => (false, 0),
            TestType::Words25 => (false, 0),
            TestType::Words50 => (false, 0),
            TestType::Time10 => (true, 10),
            TestType::Time30 => (true, 30),
            TestType::Time60 => (true, 60),
        }
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => TestType::Words10,
            1 => TestType::Words25,
            2 => TestType::Words50,
            3 => TestType::Time10,
            4 => TestType::Time30,
            5 => TestType::Time60,
            _ => TestType::Words10,
        }
    }

    pub fn as_string(&self) -> &'static str {
        match &self {
            TestType::Words10 => "10 Words",
            TestType::Words25 => "25 Words",
            TestType::Words50 => "50 Words",
            TestType::Time10 => "10 Seconds",
            TestType::Time30 => "30 Seconds",
            TestType::Time60 => "60 Seconds",
        }
    }

    pub fn from_string(str: &str) -> Self {
        match str {
            "10 Words" => TestType::Words10,
            "25 Words" => TestType::Words25,
            "50 Words" => TestType::Words50,
            "10 Seconds" => TestType::Time10,
            "30 Seconds" => TestType::Time30,
            "60 Seconds" => TestType::Time60,
            _ => TestType::Words10,
        }
    }

    pub fn as_vec() -> Vec<&'static str> {
        vec![
            "10 Words",
            "25 Words",
            "50 Words",
            "10 Seconds",
            "30 Seconds",
            "60 Seconds",
        ]
    }
}

impl Tuipe {
    // Input controls
    pub fn test_type_selector_input(&mut self, keycode: crossterm::event::KeyCode) {
        match keycode {
            KeyCode::Char('k') => {
                self.menu_selection = (self.menu_selection + TestType::COUNT - 1) % TestType::COUNT;
            }
            KeyCode::Char('j') => {
                self.menu_selection = (self.menu_selection + 1) % TestType::COUNT;
            }
            KeyCode::Enter => {
                self.test.ttype = TestType::from_index(self.menu_selection);
                self.state = State::OptMenu;
                self.menu_selection = 0
            }
            KeyCode::Char('q') => self.should_exit = true,
            KeyCode::Esc => self.state = State::OptMenu,
            _ => {}
        }
    }

    // Rendering
    pub fn render_test_type_selector(&mut self, frame: &mut Frame) {
        let input_area = self.create_layout(40, 16, frame);

        let mut lines: Vec<Line<'_>> = Vec::new();
        lines.push(Line::from(Span::styled(
            "Available tests:",
            Style::default().fg(Color::Green),
        )));
        lines.push(Line::from(Span::raw("")));

        for (i, name) in TestType::as_vec().iter().enumerate() {
            let style = if i == self.menu_selection {
                Style::default().fg(Color::Blue)
            } else {
                Style::default()
            };
            let label = if i == self.menu_selection {
                format!("> {name}")
            } else {
                format!("{name}")
            };
            lines.push(Line::from(Span::styled(label, style)));
        }

        self.add_opt_menu_controls(&mut lines);

        let input = Paragraph::new(lines)
            .style(Style::default())
            .centered()
            .block(Block::new());
        frame.render_widget(input, input_area);
    }
}
