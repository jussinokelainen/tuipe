use crate::get_words_as_vector;
use color_eyre::Result;
use crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;
use std::time::{SystemTime, UNIX_EPOCH};

/// App holds the state of the application
pub struct Tuipe {
    pub input_mode: State,

    pub test_is_started: bool,
    pub test_start_time: u128,
    pub test_final_time: f64,

    pub cursor_index: usize,
    pub character_index: usize,
    pub word_index: usize,

    pub input: Vec<String>,
    pub input_buffer: Vec<u8>,
    pub words: Vec<String>,
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
            cursor_index: 0,
            character_index: 0,
            word_index: 0,
            input: vec!["".to_string()],
            input_buffer: vec![0],
            words: Vec::new(),
        }
    }

    const fn move_cursor_left(&mut self) {
        if self.cursor_index > 0 {
            self.cursor_index -= 1
        }
    }

    const fn move_cursor_right(&mut self) {
        self.cursor_index += 1;
    }

    fn set_start_time(&mut self) {
        let time_now = SystemTime::now();
        let start_time = time_now
            .duration_since(UNIX_EPOCH)
            .expect("time should go forward");
        self.test_start_time = start_time.as_millis()
    }

    pub fn get_test_time(&mut self) {
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
        self.move_cursor_right();
    }

    fn add_word(&mut self) {
        let w_idx = self.word_index;
        if self.words[w_idx].len() > self.input[w_idx].len() {
            let buffer_count = (self.words[w_idx].len() - self.input[w_idx].len()) as u8;
            self.input_buffer[w_idx] = buffer_count;
            // Move the cursor forward the required amount of characters
            for _ in 0..buffer_count {
                self.move_cursor_right();
            }
        }

        self.character_index = 0;
        self.word_index += 1;
        self.input.push(String::from(""));
        self.input_buffer.push(0);
        self.move_cursor_right();
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
                if self.input_buffer[self.word_index] != 0 {
                    // Move the cursor back the required amount of characters
                    for _ in 1..self.input_buffer[self.word_index] {
                        self.move_cursor_left();
                    }
                }
                self.move_cursor_left();
                // Move left one more time to account for the extra space between words
                self.move_cursor_left();
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
            self.move_cursor_left();
            self.character_index -= 1
        }
    }

    const fn reset_cursor(&mut self) {
        self.cursor_index = 0;
    }

    fn restart_test(&mut self) {
        const COUNT: usize = 15;
        self.words = get_words_as_vector(COUNT);
        self.input = vec!["".to_string()];
        self.input_buffer = vec![0];
        self.input_mode = State::Typing;
        self.cursor_index = 0;
        self.character_index = 0;
        self.word_index = 0;
        self.reset_cursor();
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
                        KeyCode::Left => self.move_cursor_left(),
                        KeyCode::Right => self.move_cursor_right(),
                        KeyCode::Esc => self.input_mode = State::TestOver,
                        KeyCode::Tab => self.restart_test(),
                        _ => {}
                    },
                    State::Typing => {}
                }
            }
        }
    }
}
