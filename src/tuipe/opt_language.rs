use crate::Tuipe;
use crate::tuipe::State;
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

pub enum Language {
    English,
    English1k,
    English5k,
    English10k,
    English25k,
}

impl Language {
    pub const COUNT: usize = 5;

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Language::English,
            1 => Language::English1k,
            2 => Language::English5k,
            3 => Language::English10k,
            4 => Language::English25k,
            _ => Language::English,
        }
    }

    pub fn as_vec() -> Vec<&'static str> {
        vec![
            "English",
            "English 1k",
            "English 5k",
            "English 10k",
            "English 25k",
        ]
    }

    pub fn as_string(&self) -> &'static str {
        match &self {
            Language::English => "English",
            Language::English1k => "English 1k",
            Language::English5k => "English 5k",
            Language::English10k => "English 10k",
            Language::English25k => "English 25k",
        }
    }

    pub fn from_string(str: &str) -> Self {
        match str {
            "English" => Language::English,
            "English 1k" => Language::English1k,
            "English 5k" => Language::English5k,
            "English 10k" => Language::English10k,
            "English 25k" => Language::English25k,
            _ => Language::English,
        }
    }
}

impl Tuipe {
    // Input controls
    pub fn language_selector_input(&mut self, keycode: crossterm::event::KeyCode) {
        match keycode {
            KeyCode::Char('k') => {
                self.menu_selection = (self.menu_selection + Language::COUNT - 1) % Language::COUNT;
            }
            KeyCode::Char('j') => {
                self.menu_selection = (self.menu_selection + 1) % Language::COUNT;
            }
            KeyCode::Enter => {
                self.language = Language::from_index(self.menu_selection);
                self.state = State::MainMenu;
                self.menu_selection = 0
            }
            KeyCode::Char('q') => self.should_exit = true,
            KeyCode::Esc => self.state = State::MainMenu,
            _ => {}
        }
    }

    // Rendering
    pub fn render_language_selector(&mut self, frame: &mut Frame) {
        let input_area = self.create_layout(40, 16, frame);

        let mut lines: Vec<Line<'_>> = Vec::new();
        lines.push(Line::from(Span::styled(
            "Available languages:",
            Style::default().fg(Color::Green),
        )));
        lines.push(Line::from(Span::raw("")));

        for (i, name) in Language::as_vec().iter().enumerate() {
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
