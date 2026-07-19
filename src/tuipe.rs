use crate::get_words_as_vector;
use color_eyre::Result;
use crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Flex, Layout, Position};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use ratatui::{DefaultTerminal, Frame};
use std::time::{SystemTime, UNIX_EPOCH};

/// App holds the state of the application
pub struct Tuipe {
    input_mode: State,

    test_is_started: bool,
    test_start_time: u128,
    test_final_time: f64,

    character_index: usize,
    word_index: usize,

    input: Vec<String>,
    input_buffer: Vec<u8>,
    words: Vec<String>,

    input_width: u16,
    input_height: u16,
}

pub enum State {
    TestOver,
    Typing,
}

impl Tuipe {
    pub fn new() -> Self {
        Self {
            input_mode: State::Typing,
            test_is_started: false,
            test_start_time: 0,
            test_final_time: 0.0,
            character_index: 0,
            word_index: 0,
            input: vec!["".to_string()],
            input_buffer: vec![0],
            words: Vec::new(),
            input_width: 0,
            input_height: 0,
        }
    }

    fn set_start_time(&mut self) {
        let time_now = SystemTime::now();
        let start_time = time_now
            .duration_since(UNIX_EPOCH)
            .expect("time should go forward");
        self.test_start_time = start_time.as_millis()
    }

    fn get_test_time(&mut self) {
        // Calculate time
        let time_now = SystemTime::now();
        let end_time = time_now
            .duration_since(UNIX_EPOCH)
            .expect("time should go forward");
        self.test_final_time = (end_time.as_millis() - self.test_start_time) as f64;
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
                self.character_index = self.input[self.word_index].len() - 1;
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
        const COUNT: usize = 50;
        self.words = get_words_as_vector(COUNT);
        self.input = vec!["".to_string()];
        self.input_buffer = vec![0];
        self.input_mode = State::Typing;
        self.character_index = 0;
        self.word_index = 0;
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        self.restart_test();
        loop {
            terminal.draw(|frame| self.render(frame))?;

            if let Some(key) = event::read()?.as_key_press_event() {
                match self.input_mode {
                    State::TestOver => match key.code {
                        KeyCode::Tab => self.restart_test(),
                        KeyCode::Char('e') => {
                            self.input_mode = State::Typing;
                        }
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
                        _ => {}
                    },
                    State::Typing if key.kind == KeyEventKind::Press => match key.code {
                        KeyCode::Char(' ') => self.add_word(),
                        KeyCode::Char(to_insert) => self.enter_char(to_insert),
                        KeyCode::Backspace => self.delete_char(),
                        KeyCode::Esc => self.input_mode = State::TestOver,
                        KeyCode::Tab => self.restart_test(),
                        _ => {}
                    },
                    State::Typing => {}
                }
            }
        }
    }

    fn create_render_lines(&mut self, width: u16) -> (Vec<Line<'static>>, (u16, u16)) {
        if self.words == self.input {
            self.input_mode = State::TestOver;
            self.get_test_time();
            println!("\n{}", self.test_final_time / 1000.0)
        }

        let mut lines: Vec<Line<'static>> = Vec::new();
        let mut current_line: Vec<Span<'static>> = Vec::new();
        let mut current_line_width: u16 = 0;

        let mut cursor_row: u16 = 0;
        let mut cursor_col: u16 = 0;
        let mut cursor_found = false;

        for (word_idx, word) in self.words.clone().into_iter().enumerate() {
            let mut word_spans: Vec<Span<'static>> = Vec::new();

            if word_idx > self.word_index {
                // The typer is not here yet, print the whole word as dark gray
                let color = Color::DarkGray;
                word_spans.push(Span::styled(word.clone(), Style::default().fg(color)));
            } else {
                // The typer has been at this word, check each character
                for (char_idx, char) in word.chars().enumerate() {
                    let mut color = Color::DarkGray;
                    if self.word_index > word_idx || self.character_index >= char_idx {
                        if self.input[word_idx].chars().nth(char_idx)
                            == self.words[word_idx].chars().nth(char_idx)
                        {
                            color = Color::Reset
                        } else {
                            color = Color::Red
                        }
                    }
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

    fn render(&mut self, frame: &mut Frame) {
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
        self.input_width = input_area.width;
        self.input_height = input_area.height;

        let text_width = input_area.width.saturating_sub(2); // minus left/right border
        let (lines, (cursor_row, cursor_col)) = self.create_render_lines(text_width);

        let input = Paragraph::new(lines)
            .style(Style::default())
            .block(Block::bordered());
        frame.render_widget(input, input_area);

        match self.input_mode {
            State::TestOver => {}
            #[expect(clippy::cast_possible_truncation)]
            State::Typing => frame.set_cursor_position(Position::new(
                input_area.x + cursor_col + 1,
                input_area.y + cursor_row + 1,
            )),
        }
    }
}
