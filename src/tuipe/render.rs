use crate::Language;
use crate::TestType;
use crate::Tuipe;
use crate::tuipe::State;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Position};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

impl Tuipe {
    fn create_test_lines(&mut self, width: u16) -> (Vec<Line<'_>>, (u16, u16)) {
        let mut lines: Vec<Line<'_>> = Vec::new();
        let mut current_line: Vec<Span<'_>> = Vec::new();
        let mut current_line_width: u16 = 0;

        let mut cursor_row: u16 = 0;
        let mut cursor_col: u16 = 0;
        let mut cursor_found = false;

        for (word_idx, word) in self.words.iter().enumerate() {
            let mut word_spans: Vec<Span<'_>> = Vec::new();

            if word_idx > self.word_index {
                // The typer is not here yet, print the whole word as dark gray
                let color = Color::DarkGray;
                word_spans.push(Span::styled(word, Style::default().fg(color)));
            } else if word_idx < self.word_index && &self.input[word_idx] == word {
                // This word was typed correctly, no need to go character by character
                let color = Color::Reset;
                word_spans.push(Span::styled(word, Style::default().fg(color)));
            } else {
                // Either the typer is at this word or it has typos, check each character
                let cur_input_word = &self.input[word_idx];
                for (char_idx, char) in word.chars().enumerate() {
                    let color = if self.word_index > word_idx || self.character_index > char_idx {
                        match cur_input_word.chars().nth(char_idx) {
                            None => Color::DarkGray,
                            c if c == word.chars().nth(char_idx) => Color::Reset,
                            _ => Color::Red,
                        }
                    } else {
                        Color::DarkGray
                    };

                    let mut tmp = [0; 4];
                    let char_as_str: &str = char.encode_utf8(&mut tmp);
                    word_spans.push(Span::styled(
                        char_as_str.to_string(),
                        Style::default().fg(color),
                    ));
                }
                // If the word at current word_idx has more characters in input
                // as in the real word, print them out here as red
                if cur_input_word.len() > word.len() {
                    let color = Color::Red;
                    let extra_characters = &cur_input_word[word.len()..cur_input_word.len()];
                    word_spans.push(Span::styled(
                        extra_characters.to_string(),
                        Style::default().fg(color),
                    ));
                }
            }

            let word_width = Line::from(word_spans.clone()).width() as u16;

            // Word-wrap: if this word + trailing space won't fit, break to a new line first
            if current_line_width > 0 && current_line_width + word_width + 1 > width {
                let new_line = std::mem::take(&mut current_line);
                if cursor_found {
                    lines.push(Line::from(new_line));
                }
                current_line_width = 0;
            }

            // Record cursor position while we're on the word the typist is currently at
            if !cursor_found && word_idx == self.word_index {
                let word_len = word.chars().count();
                let target_char = self.character_index.min(word_len);
                let mut col_in_word: u16 = word_spans[..target_char]
                    .iter()
                    .map(|s| s.width() as u16)
                    .sum();
                if self.character_index + 1 > word_len {
                    // overtyped past the word into the extra red characters
                    col_in_word = word_width;
                }
                cursor_row = lines.len() as u16;
                cursor_col = current_line_width + col_in_word;
                cursor_found = true;
            }

            current_line.extend(word_spans);
            current_line_width += word_width;
            current_line.push(Span::styled(" ", Style::default()));
            current_line_width += 1;
        }

        lines.push(Line::from(current_line));

        if !cursor_found {
            let last_row = (lines.len() - 1) as u16;
            let last_width = lines.last().map(|l| l.width() as u16).unwrap_or(0);
            cursor_row = last_row;
            cursor_col = last_width;
        }

        (lines, (cursor_row, cursor_col))
    }

    fn render_test(&mut self, frame: &mut Frame) {
        let layout_vert = Layout::default()
            .direction(Direction::Vertical)
            .flex(Flex::Center)
            .constraints([Constraint::Min(0), Constraint::Max(4), Constraint::Min(0)])
            .split(frame.area());
        let layout_horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .flex(Flex::Center)
            .constraints([Constraint::Min(0), Constraint::Max(100), Constraint::Min(0)])
            .split(layout_vert[1]);
        let input_area = layout_horizontal[1];
        let text_width = input_area.width.saturating_sub(2);
        let (lines, (cursor_row, cursor_col)) = self.create_test_lines(text_width);

        // Calculate cursor offset for centered text
        #[expect(clippy::cast_possible_truncation)]
        let cursor_line_width = lines
            .get(cursor_row as usize)
            .map_or(0, ratatui::text::Line::width) as u16;
        let center_offset = (text_width / 2).saturating_sub(cursor_line_width / 2);

        let input = Paragraph::new(lines)
            .style(Style::default())
            .centered()
            .block(Block::new());
        frame.render_widget(input, input_area);

        #[expect(clippy::cast_possible_truncation)]
        frame.set_cursor_position(Position::new(
            input_area.x + 1 + center_offset + cursor_col,
            input_area.y + cursor_row,
        ));
    }

    fn render_endscreen(&mut self, frame: &mut Frame) {
        let layout_vert = Layout::default()
            .direction(Direction::Vertical)
            .flex(Flex::Center)
            .constraints([Constraint::Min(0), Constraint::Max(15), Constraint::Min(0)])
            .split(frame.area());
        let layout_horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .flex(Flex::Center)
            .constraints([Constraint::Min(0), Constraint::Max(40), Constraint::Min(0)])
            .split(layout_vert[1]);
        let input_area = layout_horizontal[1];

        let time_str: String =
            "Your time: ".to_string() + &(self.stats.time / 1000.0).to_string() + " seconds.";
        // Multiplication and division by 100 to enable rounding wpm
        let wpm_str: String =
            "WPM: ".to_string() + &((self.stats.wpm * 100.0).round() / 100.0).to_string();
        let raw_wpm_str: String =
            "raw WPM: ".to_string() + &((self.stats.wpm_raw * 100.0).round() / 100.0).to_string();
        let typed_char_str: String =
            "Characters typed: ".to_string() + &(self.stats.typed_characters).to_string();
        let typed_word_str: String =
            "Words typed: ".to_string() + &(self.stats.typed_words).to_string();
        let mut lines: Vec<Line<'static>> = Vec::new();
        lines.push(Line::from(Span::styled(
            format!("Test done: {}", TestType::as_string(&self.test_type)),
            Style::default().fg(Color::Green),
        )));
        lines.push(Line::from(Span::styled(time_str, Style::default())));
        lines.push(Line::from(Span::styled("", Style::default())));

        lines.push(Line::from(Span::styled(
            wpm_str,
            Style::default().fg(Color::Blue),
        )));
        lines.push(Line::from(Span::styled(raw_wpm_str, Style::default())));
        lines.push(Line::from(Span::styled("", Style::default())));

        lines.push(Line::from(Span::styled(typed_char_str, Style::default())));
        lines.push(Line::from(Span::styled(typed_word_str, Style::default())));

        lines.push(Line::from(Span::styled("", Style::default())));
        lines.push(Line::from(Span::styled("", Style::default())));
        lines.push(Line::from(Span::styled(
            "Quit: q",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::styled(
            "Back to main menu: Esc",
            Style::default().fg(Color::DarkGray),
        )));
        let input = Paragraph::new(lines)
            .style(Style::default())
            .centered()
            .block(Block::new());
        frame.render_widget(input, input_area);
    }

    fn render_test_type_screen(&mut self, frame: &mut Frame) {
        let layout_vert = Layout::default()
            .direction(Direction::Vertical)
            .flex(Flex::Center)
            .constraints([Constraint::Min(0), Constraint::Max(16), Constraint::Min(0)])
            .split(frame.area());
        let layout_horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .flex(Flex::Center)
            .constraints([Constraint::Min(0), Constraint::Max(40), Constraint::Min(0)])
            .split(layout_vert[1]);
        let input_area = layout_horizontal[1];

        let mut lines: Vec<Line<'static>> = Vec::new();
        lines.push(Line::from(Span::styled(
            "Available tests:",
            Style::default().fg(Color::Green),
        )));
        lines.push(Line::from(Span::styled("", Style::default())));

        for (i, name) in TestType::as_vec().iter().enumerate() {
            let style = if i == self.test_selection {
                Style::default().fg(Color::Blue)
            } else {
                Style::default()
            };
            let label = if i == self.test_selection {
                format!("> {name}")
            } else {
                format!("{name}")
            };
            lines.push(Line::from(Span::styled(label, style)));
        }

        lines.push(Line::from(Span::styled("", Style::default())));
        lines.push(Line::from(Span::styled("", Style::default())));
        lines.push(Line::from(Span::styled(
            "Move: j/k",
            Style::default().fg(Color::DarkGray),
        )));
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

    fn render_language_screen(&mut self, frame: &mut Frame) {
        let layout_vert = Layout::default()
            .direction(Direction::Vertical)
            .flex(Flex::Center)
            .constraints([Constraint::Min(0), Constraint::Max(16), Constraint::Min(0)])
            .split(frame.area());
        let layout_horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .flex(Flex::Center)
            .constraints([Constraint::Min(0), Constraint::Max(40), Constraint::Min(0)])
            .split(layout_vert[1]);
        let input_area = layout_horizontal[1];

        let mut lines: Vec<Line<'static>> = Vec::new();
        lines.push(Line::from(Span::styled(
            "Available languages:",
            Style::default().fg(Color::Green),
        )));
        lines.push(Line::from(Span::styled("", Style::default())));

        for (i, name) in Language::as_vec().iter().enumerate() {
            let style = if i == self.language_selection {
                Style::default().fg(Color::Blue)
            } else {
                Style::default()
            };
            let label = if i == self.language_selection {
                format!("> {name}")
            } else {
                format!("{name}")
            };
            lines.push(Line::from(Span::styled(label, style)));
        }

        lines.push(Line::from(Span::styled("", Style::default())));
        lines.push(Line::from(Span::styled("", Style::default())));
        lines.push(Line::from(Span::styled(
            "Move: j/k",
            Style::default().fg(Color::DarkGray),
        )));
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

    fn render_main_menu(&mut self, frame: &mut Frame) {
        let layout_vert = Layout::default()
            .direction(Direction::Vertical)
            .flex(Flex::Center)
            .constraints([Constraint::Min(0), Constraint::Max(16), Constraint::Min(0)])
            .split(frame.area());
        let layout_horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .flex(Flex::Center)
            .constraints([Constraint::Min(0), Constraint::Max(40), Constraint::Min(0)])
            .split(layout_vert[1]);
        let input_area = layout_horizontal[1];

        let mut lines: Vec<Line<'static>> = Vec::new();
        lines.push(Line::from(Span::styled(
            "Tuipe",
            Style::default().fg(Color::Magenta),
        )));
        lines.push(Line::from(Span::styled("", Style::default())));
        let ttype = TestType::as_string(&self.test_type);
        lines.push(Line::from(Span::styled(
            format!("Current Test type: {ttype}"),
            Style::default().fg(Color::LightCyan),
        )));
        let lang = Language::as_string(&self.language);
        lines.push(Line::from(Span::styled(
            format!("Current Language: {lang}"),
            Style::default().fg(Color::LightCyan),
        )));
        lines.push(Line::from(Span::styled("", Style::default())));

        let options = ["Start test", "Select test type", "Select language"];
        for (i, name) in options.iter().enumerate() {
            let style = if i == self.mainmenu_selection {
                Style::default().fg(Color::Blue)
            } else {
                Style::default()
            };
            let label = if i == self.mainmenu_selection {
                format!("> {name}")
            } else {
                format!("{name}")
            };
            lines.push(Line::from(Span::styled(label, style)));
        }

        lines.push(Line::from(Span::styled("", Style::default())));
        lines.push(Line::from(Span::styled("", Style::default())));
        lines.push(Line::from(Span::styled(
            "Move: j/k",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::styled(
            "Select: Enter",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::styled(
            "Quit: q",
            Style::default().fg(Color::DarkGray),
        )));
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

    pub fn render(&mut self, frame: &mut Frame) {
        // check if test done instead of self.words == self.input
        if !self.stats.time_is_set && self.check_is_test_done() {
            self.get_time_and_wpm();
            self.state = State::EndScreen;
        }

        match self.state {
            State::MainMenu => self.render_main_menu(frame),
            State::LanguageScreen => self.render_language_screen(frame),
            State::TestTypeScreen => self.render_test_type_screen(frame),
            State::Typing => self.render_test(frame),
            State::EndScreen => self.render_endscreen(frame),
        }
    }
}
