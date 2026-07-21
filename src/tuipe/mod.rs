mod structs;
use crate::get_words_as_vector;
use color_eyre::Result;
use crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Flex, Layout, Position};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use ratatui::{DefaultTerminal, Frame};
use std::time::{SystemTime, UNIX_EPOCH};
pub use structs::Language;
use structs::{FinalStats, State};

pub struct Tuipe {
    state: State,
    pub language: Language,
    language_selection: usize,

    test_is_started: bool,
    test_start_time: u128,

    stats: FinalStats,

    input: Vec<String>,
    input_buffer: Vec<u8>,

    character_index: usize,
    word_index: usize,
    words: Vec<String>,
    words_count: usize,
}

impl Tuipe {
    pub fn new() -> Self {
        Self {
            state: State::StartScreen,
            language: Language::English,
            language_selection: 0,

            test_is_started: false,
            test_start_time: 0,

            stats: FinalStats::new(),

            input: vec!["".to_string()],
            input_buffer: vec![0],

            character_index: 0,
            word_index: 0,
            words: Vec::new(),
            words_count: 50,
        }
    }

    fn set_start_time(&mut self) {
        let time_now = SystemTime::now();
        let start_time = time_now
            .duration_since(UNIX_EPOCH)
            .expect("time should go forward");
        self.test_start_time = start_time.as_millis()
    }

    fn get_time_and_wpm(&mut self) {
        // Calculate time
        let time_now = SystemTime::now();
        let end_time = time_now
            .duration_since(UNIX_EPOCH)
            .expect("time should go forward");
        self.stats.time = (end_time.as_millis() - self.test_start_time) as f64;
        self.stats.time_is_set = true;

        // Calculate wpm
        let mut correct_characters = 0;
        let mut raw_extra_chars = 0;
        let mut typed_words = 0;
        for (idx, word) in self.input.iter().enumerate() {
            if self.words.len() > idx {
                let input_word_len = word.chars().count();
                let actual_word = &self.words[idx];
                if actual_word == word {
                    correct_characters += input_word_len;
                } else {
                    if input_word_len < actual_word.chars().count() {
                        raw_extra_chars += input_word_len;
                    } else {
                        raw_extra_chars += actual_word.chars().count();
                    }
                }
                typed_words += 1;
                // Add one character to accont for the space after the word
                correct_characters += 1;
            }
        }
        // Remove one since there is no space after the last word
        correct_characters -= 1;
        let correct_words = correct_characters as f64 / 5 as f64;
        let raw_words = (correct_characters + raw_extra_chars) as f64 / 5 as f64;

        // times 60 to get words per minute instead of words per second,
        // and divide self.test_final_time by 1000 to convert it from
        // milliseconds to seconds
        self.stats.wpm = (correct_words * 60.0) / (self.stats.time / 1000.0);
        self.stats.wpm_raw = (raw_words * 60.0) / (self.stats.time / 1000.0);
        self.stats.typed_words = typed_words;
        self.stats.typed_characters = correct_characters;
    }

    fn enter_char(&mut self, new_char: char) {
        if !self.test_is_started {
            self.test_is_started = true;
            self.set_start_time()
        }
        self.input[self.word_index] += new_char.to_string().as_str();
        self.character_index += 1;
    }

    fn add_word(&mut self) {
        let w_idx = self.word_index;
        if self.words[w_idx].len() > self.input[w_idx].len() {
            let buffer_count = (self.words[w_idx].len() - self.input[w_idx].len()) as u8;
            self.input_buffer[w_idx] = buffer_count;
        }

        self.character_index = 0;
        self.word_index += 1;
        self.input.push(String::from(""));
        self.input_buffer.push(0);
    }

    fn delete_char(&mut self) {
        if self.character_index == 0 {
            // We are at the first character of a word, go back a word if possible
            if self.word_index > 0 {
                // Remove the word from the input vector
                let new_length = self.input.len().saturating_sub(1);
                self.input.truncate(new_length);

                self.word_index -= 1;
                self.character_index = self.input[self.word_index].len();
                if self.input_buffer[self.word_index] != 0 {}
            }
        } else {
            // We can remove a character from the current word
            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input[self.word_index]
                .chars()
                .take(from_left_to_current_index);
            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input[self.word_index] = before_char_to_delete.collect();
            self.character_index -= 1
        }
    }

    fn restart_test(&mut self) {
        self.state = State::Typing;

        self.test_is_started = false;
        self.test_start_time = 0;

        self.stats = FinalStats::new();

        self.input = vec!["".to_string()];
        self.input_buffer = vec![0];

        self.character_index = 0;
        self.word_index = 0;
        self.words = get_words_as_vector(self.language.clone());
    }

    fn check_is_test_done(&self) -> bool {
        if self.words.len() < self.input.len() {
            return true;
        }
        if self.words.len() == self.input.len() {
            if self.words[self.words_count - 1] == self.input[self.words_count - 1] {
                return true;
            }
        }

        false
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        // CLDL-ENTRY: title: useless call, priority: 12, tag: NONE
        self.restart_test();
        self.state = State::StartScreen;
        loop {
            terminal.draw(|frame| self.render(frame))?;

            if let Some(key) = event::read()?.as_key_press_event() {
                match self.state {
                    State::StartScreen => match key.code {
                        KeyCode::Char('k') => {
                            self.language_selection =
                                (self.language_selection + Language::COUNT - 1) % Language::COUNT;
                        }
                        KeyCode::Char('j') => {
                            self.language_selection =
                                (self.language_selection + 1) % Language::COUNT;
                        }
                        KeyCode::Enter => {
                            self.language = Language::from_index(self.language_selection);
                            self.restart_test();
                        }
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
                        _ => {}
                    },
                    State::EndScreen => match key.code {
                        KeyCode::Tab => self.restart_test(),
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
                        _ => {}
                    },
                    State::Typing if key.kind == KeyEventKind::Press => match key.code {
                        KeyCode::Char(' ') => self.add_word(),
                        KeyCode::Char(to_insert) => self.enter_char(to_insert),
                        KeyCode::Backspace => self.delete_char(),
                        KeyCode::Esc => self.state = State::StartScreen,
                        KeyCode::Tab => self.restart_test(),
                        _ => {}
                    },
                    State::Typing => {}
                }
            }
        }
    }

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
            } else {
                // The typer has been at this word, check each character
                for (char_idx, char) in word.chars().enumerate() {
                    let cur_input_char = self.input[word_idx].chars().nth(char_idx);
                    let color = if self.word_index > word_idx || self.character_index > char_idx {
                        match cur_input_char {
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
                if self.input[word_idx].len() > word.len() {
                    let color = Color::Red;
                    let extra_characters =
                        &self.input[word_idx][word.len()..self.input[word_idx].len()];
                    word_spans.push(Span::styled(
                        extra_characters.to_string(),
                        Style::default().fg(color),
                    ));
                }
            }

            let word_width = Line::from(word_spans.clone()).width() as u16;

            // Word-wrap: if this word + trailing space won't fit, break to a new line first
            if current_line_width > 0 && current_line_width + word_width + 1 > width {
                lines.push(Line::from(std::mem::take(&mut current_line)));
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
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(20),
                Constraint::Percentage(40),
            ])
            .split(frame.area());
        let layout_horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .flex(Flex::Center)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(50),
                Constraint::Percentage(25),
            ])
            .split(layout_vert[1]);
        let input_area = layout_horizontal[1];
        let text_width = input_area.width.saturating_sub(2); // minus left/right border
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
            .block(Block::bordered());
        frame.render_widget(input, input_area);

        #[expect(clippy::cast_possible_truncation)]
        frame.set_cursor_position(Position::new(
            input_area.x + 1 + center_offset + cursor_col,
            input_area.y + 1 + cursor_row,
        ));
    }

    fn render_endscreen(&mut self, frame: &mut Frame) {
        let layout_vert = Layout::default()
            .direction(Direction::Vertical)
            .flex(Flex::Center)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(20),
                Constraint::Percentage(40),
            ])
            .split(frame.area());
        let layout_horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .flex(Flex::Center)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(20),
                Constraint::Percentage(40),
            ])
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
            "Test over.",
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

        let input = Paragraph::new(lines)
            .style(Style::default())
            .centered()
            .block(Block::bordered());
        frame.render_widget(input, input_area);
    }

    fn render_start(&mut self, frame: &mut Frame) {
        let layout_vert = Layout::default()
            .direction(Direction::Vertical)
            .flex(Flex::Center)
            .constraints([
                Constraint::Percentage(35),
                Constraint::Percentage(30),
                Constraint::Percentage(35),
            ])
            .split(frame.area());
        let layout_horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .flex(Flex::Center)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(20),
                Constraint::Percentage(40),
            ])
            .split(layout_vert[1]);
        let input_area = layout_horizontal[1];

        let mut lines: Vec<Line<'static>> = Vec::new();
        lines.push(Line::from(Span::styled(
            "Tuipe, TUI typing test",
            Style::default().fg(Color::Magenta),
        )));
        lines.push(Line::from(Span::styled("", Style::default())));

        lines.push(Line::from(Span::styled(
            "Available languages:",
            Style::default().fg(Color::Green),
        )));

        let languages = [
            "English",
            "English 1k",
            "English 5k",
            "English 10k",
            "English 25k",
        ];
        for (i, name) in languages.iter().enumerate() {
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
            "Start test: Enter",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::styled(
            "Quit: q",
            Style::default().fg(Color::DarkGray),
        )));

        let input = Paragraph::new(lines)
            .style(Style::default())
            .centered()
            .block(Block::bordered());
        frame.render_widget(input, input_area);
    }

    fn render(&mut self, frame: &mut Frame) {
        // check if test done instead of self.words == self.input
        if !self.stats.time_is_set && self.check_is_test_done() {
            self.get_time_and_wpm();
            self.state = State::EndScreen;
        }

        match self.state {
            State::StartScreen => self.render_start(frame),
            State::Typing => self.render_test(frame),
            State::EndScreen => self.render_endscreen(frame),
        }
    }
}
