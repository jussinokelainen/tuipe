use crate::Tuipe;
use crate::tuipe::opt_difficulty::Difficulty;
use crate::tuipe::{State, get_current_time_as_millis};
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Position;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

impl Tuipe {
    // Input controls during the test
    pub fn typing_test_input(&mut self, keycode: crossterm::event::KeyCode) {
        match keycode {
            KeyCode::Char(' ') => self.add_word(),
            KeyCode::Char(to_insert) => self.enter_char(to_insert),
            KeyCode::Backspace => self.delete_char(),
            KeyCode::Esc => self.state = State::MainMenu,
            KeyCode::Tab => self.restart_test(),
            _ => {}
        }
    }

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

    fn set_start_time(&mut self) {
        self.test.start_time = get_current_time_as_millis()
    }

    // Sets the test results into the stats struct and saves the results
    // into the database if given save variable is true
    pub fn set_final_stats(&mut self, save: bool) {
        // Calculate and set time
        self.stats.time = (get_current_time_as_millis() - self.test.start_time) as f64;
        self.stats.time_is_set = true;

        // Calculate and set wpm
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
        self.stats.accuracy = self.test.correct_chars as f64
            / (self.test.correct_chars + self.test.incorrect_chars) as f64;
        if save {
            self.save_success = self.save_to_db();
        }
    }

    // Enter a new character
    fn enter_char(&mut self, new_char: char) {
        if !self.test.is_started {
            self.test.is_started = true;
            self.set_start_time()
        }
        self.input[self.word_index] += new_char.to_string().as_str();

        // Check whether the character given is correct or not
        if self.input[self.word_index]
            .chars()
            .nth(self.character_index)
            != self.words[self.word_index]
                .chars()
                .nth(self.character_index)
        {
            if self.test.difficulty == Difficulty::Master {
                // End the test if on Master difficulty and input has
                // incorrect character
                self.state = State::TestInterrupted;
                self.set_final_stats(false);
            } else {
                self.test.incorrect_chars += 1;
            }
        } else {
            self.test.correct_chars += 1;
        }

        self.character_index += 1;
    }

    // Add a new word
    fn add_word(&mut self) {
        let w_idx = self.word_index;
        if self.words[w_idx].len() > self.input[w_idx].len() {
            let buffer_count = (self.words[w_idx].len() - self.input[w_idx].len()) as u8;
            self.input_buffer[w_idx] = buffer_count;
        }

        // Check that the word is correct
        if self.words[w_idx] != self.input[w_idx] {
            if self.test.difficulty == Difficulty::Master
                || self.test.difficulty == Difficulty::Expert
            {
                // If on master or expert, end the test
                self.state = State::TestInterrupted;
                self.set_final_stats(false);
            } else {
                // else increment incorrect characters
                self.test.incorrect_chars += 1;
            }
        }

        self.character_index = 0;
        self.word_index += 1;
        self.input.push(String::from(""));
        self.input_buffer.push(0);
    }

    // Delete a character. Handles going back words if deleting characters at
    // the start of a word
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
    // Main rendering function
    pub fn render_test(&mut self, frame: &mut Frame) {
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
}
