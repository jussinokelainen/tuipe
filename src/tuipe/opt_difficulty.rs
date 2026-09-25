use crate::Tuipe;
use crate::tuipe::State;
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

#[derive(PartialEq)]
pub enum Difficulty {
    Normal,
    Expert,
    Master,
}

impl Difficulty {
    pub const COUNT: usize = 3;

    pub fn as_vec() -> Vec<&'static str> {
        vec!["Normal", "Expert", "Master"]
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Difficulty::Normal,
            1 => Difficulty::Expert,
            2 => Difficulty::Master,
            _ => Difficulty::Normal,
        }
    }

    pub fn as_string(&self) -> &'static str {
        match &self {
            Difficulty::Normal => "Normal",
            Difficulty::Expert => "Expert",
            Difficulty::Master => "Master",
        }
    }

    pub fn from_string(str: &str) -> Self {
        match str {
            "Normal" => Difficulty::Normal,
            "Expert" => Difficulty::Expert,
            "Master" => Difficulty::Master,
            _ => Difficulty::Normal,
        }
    }
}

impl Tuipe {
    // Input controls
    pub fn difficulty_selector_input(&mut self, keycode: crossterm::event::KeyCode) {
        match keycode {
            KeyCode::Char('k') => {
                self.menu_selection =
                    (self.menu_selection + Difficulty::COUNT - 1) % Difficulty::COUNT;
            }
            KeyCode::Char('j') => {
                self.menu_selection = (self.menu_selection + 1) % Difficulty::COUNT;
            }
            KeyCode::Enter => {
                self.test.difficulty = Difficulty::from_index(self.menu_selection);
                self.state = State::MainMenu;
                self.menu_selection = 0
            }
            KeyCode::Char('q') => self.should_exit = true,
            KeyCode::Esc => self.state = State::MainMenu,
            _ => {}
        }
    }

    // Rendering
    pub fn render_difficulty_selector(&mut self, frame: &mut Frame) {
        let input_area = self.create_layout(40, 16, frame);

        let mut lines: Vec<Line<'_>> = Vec::new();
        lines.push(Line::from(Span::styled(
            "Available difficulties:",
            Style::default().fg(Color::Green),
        )));
        lines.push(Line::from(Span::raw("")));

        for (i, name) in Difficulty::as_vec().iter().enumerate() {
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

        self.add_select_menu_controls(&mut lines);

        let input = Paragraph::new(lines)
            .style(Style::default())
            .centered()
            .block(Block::new());
        frame.render_widget(input, input_area);
    }
}
