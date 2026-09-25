use crate::Tuipe;
use crate::tuipe::{
    State, opt_difficulty::Difficulty, opt_language::Language, opt_testtype::TestType,
};
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

pub enum MainMenu {
    StartTest,
    SelectTestType,
    SelectLanguage,
    SelectDifficulty,
    SelectCapitalization,
    ViewStats,
}

impl MainMenu {
    pub const COUNT: usize = 6;

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => MainMenu::StartTest,
            1 => MainMenu::SelectTestType,
            2 => MainMenu::SelectLanguage,
            3 => MainMenu::SelectDifficulty,
            4 => MainMenu::SelectCapitalization,
            5 => MainMenu::ViewStats,
            _ => MainMenu::StartTest,
        }
    }
}

impl Tuipe {
    // Input controls
    pub fn main_menu_input(&mut self, keycode: crossterm::event::KeyCode) {
        match keycode {
            KeyCode::Char('k') => {
                self.menu_selection = (self.menu_selection + MainMenu::COUNT - 1) % MainMenu::COUNT;
            }
            KeyCode::Char('j') => {
                self.menu_selection = (self.menu_selection + 1) % MainMenu::COUNT;
            }
            KeyCode::Enter => {
                match MainMenu::from_index(self.menu_selection) {
                    MainMenu::StartTest => self.restart_test(),
                    MainMenu::SelectTestType => self.state = State::TestTypeSelector,
                    MainMenu::SelectLanguage => self.state = State::LanguageSelector,
                    MainMenu::SelectDifficulty => self.state = State::DifficultySelector,
                    MainMenu::SelectCapitalization => self.state = State::CapitalizationSelector,
                    MainMenu::ViewStats => self.state = State::StatsScreen,
                }
                self.menu_selection = 0;
            }
            KeyCode::Char('q') => self.should_exit = true,
            _ => {}
        }
    }

    // Rendering
    pub fn render_main_menu(&mut self, frame: &mut Frame) {
        let input_area = self.create_layout(40, 21, frame);

        let mut lines: Vec<Line<'_>> = Vec::new();
        lines.push(Line::from(Span::styled(
            "Tuipe",
            Style::default().fg(Color::Magenta),
        )));
        lines.push(Line::from(Span::raw("")));
        lines.push(Line::from(Span::styled(
            "Current settings:",
            Style::default().fg(Color::Cyan),
        )));
        let ttype = TestType::as_string(&self.test.ttype);
        lines.push(Line::from(Span::styled(
            format!("Test type: {ttype}"),
            Style::default().fg(Color::LightCyan),
        )));
        let lang = Language::as_string(&self.language);
        lines.push(Line::from(Span::styled(
            format!("Language: {lang}"),
            Style::default().fg(Color::LightCyan),
        )));
        let difficulty = Difficulty::as_string(&self.test.difficulty);
        lines.push(Line::from(Span::styled(
            format!("Difficulty: {difficulty}"),
            Style::default().fg(Color::LightCyan),
        )));
        let capitals = self.test.capitals;
        lines.push(Line::from(Span::styled(
            format!("Capital letters: {capitals}"),
            Style::default().fg(Color::LightCyan),
        )));
        lines.push(Line::from(Span::raw("")));

        let options = [
            "Start test",
            "Select test type",
            "Select language",
            "Select difficulty",
            "Select capitalization",
            "View stats history",
        ];
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

        self.add_menu_controls(&mut lines);
        // Print the program version at the bottom
        lines.push(Line::from(Span::styled("", Style::default())));
        lines.push(Line::from(Span::styled(
            format!("version: {}", self.version),
            Style::default().fg(Color::DarkGray),
        )));

        let input = Paragraph::new(lines)
            .style(Style::default())
            .centered()
            .block(Block::new());
        frame.render_widget(input, input_area);
    }
}
