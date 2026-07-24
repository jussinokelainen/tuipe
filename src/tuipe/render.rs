use crate::Tuipe;
use crate::tuipe::{Difficulty, Language, State, TestType};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Position, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

impl Tuipe {
    // Creates a vector of lines containing the words of the test that need
    // to be drawn on the screen with their correct colors
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
                } else {
                    lines = vec![Line::from(new_line)];
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

    // Function for rendering screens, returns a Rect positioned at the center of
    // the screen with maximum width and height of given parameters
    fn create_layout(&self, width: u16, height: u16, frame: &mut Frame) -> Rect {
        let layout_vert = Layout::default()
            .direction(Direction::Vertical)
            .flex(Flex::Center)
            .constraints([
                Constraint::Min(0),
                Constraint::Max(height),
                Constraint::Min(0),
            ])
            .split(frame.area());
        let layout_horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .flex(Flex::Center)
            .constraints([
                Constraint::Min(0),
                Constraint::Max(width),
                Constraint::Min(0),
            ])
            .split(layout_vert[1]);
        layout_horizontal[1]
    }

    // Renders the typing test screen
    fn render_test(&mut self, frame: &mut Frame) {
        let input_area = self.create_layout(100, 4, frame);
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

    // Renders the main menu
    fn render_main_menu(&mut self, frame: &mut Frame) {
        let input_area = self.create_layout(40, 18, frame);

        let mut lines: Vec<Line<'static>> = Vec::new();
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
        lines.push(Line::from(Span::raw("")));

        let options = [
            "Start test",
            "Select test type",
            "Select language",
            "Select difficulty",
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

        lines.push(Line::from(Span::raw("")));
        lines.push(Line::from(Span::raw("")));
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

    // Renders the difficulty selector screen
    fn render_stats_screen(&mut self, frame: &mut Frame) {
        let input_area = self.create_layout(40, 16, frame);

        let mut lines: Vec<Line<'static>> = Vec::new();
        let stats_res = self.get_stats_from_db();
        match stats_res {
            Ok(v) => {
                lines.push(Line::from(Span::styled(
                    "Your stats:",
                    Style::default().fg(Color::Cyan),
                )));
                lines.push(Line::from(Span::raw("")));
                lines.push(Line::from(Span::raw(format!(
                    "Tests completed: {}",
                    v.len()
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

    // Renders the difficulty selector screen
    fn render_difficulty_selector(&mut self, frame: &mut Frame) {
        let input_area = self.create_layout(40, 16, frame);

        let mut lines: Vec<Line<'static>> = Vec::new();
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

        lines.push(Line::from(Span::raw("")));
        lines.push(Line::from(Span::raw("")));
        self.add_select_menu_controls(&mut lines);

        let input = Paragraph::new(lines)
            .style(Style::default())
            .centered()
            .block(Block::new());
        frame.render_widget(input, input_area);
    }

    // Renders the test type selector screen
    fn render_test_type_selector(&mut self, frame: &mut Frame) {
        let input_area = self.create_layout(40, 16, frame);

        let mut lines: Vec<Line<'static>> = Vec::new();
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

        lines.push(Line::from(Span::raw("")));
        lines.push(Line::from(Span::raw("")));
        self.add_select_menu_controls(&mut lines);

        let input = Paragraph::new(lines)
            .style(Style::default())
            .centered()
            .block(Block::new());
        frame.render_widget(input, input_area);
    }

    // Renders the language selector screen
    fn render_language_selector(&mut self, frame: &mut Frame) {
        let input_area = self.create_layout(40, 16, frame);

        let mut lines: Vec<Line<'static>> = Vec::new();
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

        lines.push(Line::from(Span::raw("")));
        lines.push(Line::from(Span::raw("")));
        self.add_select_menu_controls(&mut lines);

        let input = Paragraph::new(lines)
            .style(Style::default())
            .centered()
            .block(Block::new());
        frame.render_widget(input, input_area);
    }

    // Adds the main menu controls as dark gray to the lines vector
    fn add_menu_controls(&self, lines: &mut Vec<Line<'static>>) {
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
    }

    // Adds the control info for the option menus as dark gray to given lines vector
    fn add_select_menu_controls(&self, lines: &mut Vec<Line<'static>>) {
        self.add_menu_controls(lines);
        lines.push(Line::from(Span::styled(
            "Back: Esc",
            Style::default().fg(Color::DarkGray),
        )));
    }

    // Renders the end screen
    fn render_endscreen(&mut self, frame: &mut Frame) {
        let input_area = self.create_layout(40, 16, frame);

        let mut lines: Vec<Line<'static>> = Vec::new();
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
        let input = Paragraph::new(lines)
            .style(Style::default())
            .centered()
            .block(Block::new());
        frame.render_widget(input, input_area);
    }

    // The main render function of the program
    pub fn render(&mut self, frame: &mut Frame) {
        // check if test done instead of self.words == self.input
        if !self.stats.time_is_set && self.check_is_test_done() {
            self.set_final_stats();
            self.state = State::EndScreen;
        }

        match self.state {
            State::MainMenu => self.render_main_menu(frame),
            State::StatsScreen => self.render_stats_screen(frame),
            State::LanguageSelector => self.render_language_selector(frame),
            State::TestTypeSelector => self.render_test_type_selector(frame),
            State::DifficultySelector => self.render_difficulty_selector(frame),
            State::Typing => self.render_test(frame),
            State::EndScreen => self.render_endscreen(frame),
        }
    }
}
