use crate::Tuipe;
use crate::tuipe::State;
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

pub enum OptMenu {
    TestType,
    Difficulty,
    Language,
    Capitals,
}

impl OptMenu {
    pub const COUNT: usize = 4;

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => OptMenu::TestType,
            1 => OptMenu::Difficulty,
            2 => OptMenu::Language,
            3 => OptMenu::Capitals,
            _ => OptMenu::TestType,
        }
    }
}
impl Tuipe {
    // Input controls
    pub fn opt_menu_controls(&mut self, keycode: crossterm::event::KeyCode) {
        match keycode {
            KeyCode::Char('k') => {
                self.menu_selection = (self.menu_selection + OptMenu::COUNT - 1) % OptMenu::COUNT;
            }
            KeyCode::Char('j') => {
                self.menu_selection = (self.menu_selection + 1) % OptMenu::COUNT;
            }
            KeyCode::Esc => self.state = State::MainMenu,
            KeyCode::Enter => {
                match OptMenu::from_index(self.menu_selection) {
                    OptMenu::TestType => self.state = State::TestTypeSelector,
                    OptMenu::Difficulty => self.state = State::DifficultySelector,
                    OptMenu::Language => self.state = State::LanguageSelector,
                    OptMenu::Capitals => self.state = State::CapitalizationSelector,
                }
                self.menu_selection = 0;
            }
            KeyCode::Char('q') => self.should_exit = true,
            _ => {}
        }
    }

    // Rendering
    pub fn render_opt_menu(&mut self, frame: &mut Frame) {
        let input_area = self.create_layout(40, 21, frame);

        let mut lines: Vec<Line<'_>> = Vec::new();
        lines.push(Line::from(Span::styled(
            "Tuipe Options",
            Style::default().fg(Color::Magenta),
        )));
        lines.push(Line::from(Span::raw("")));

        let options = ["Test Type", "Difficulty", "Language", "Capital letters"];
        for (i, name) in options.iter().enumerate() {
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
