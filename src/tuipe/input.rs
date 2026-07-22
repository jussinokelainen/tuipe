use crate::tuipe::{Difficulty, MainMenu, State, get_current_time_as_millis};
use crate::{Language, TestType, Tuipe};
use crossterm::event::KeyCode;

impl Tuipe {
    fn set_start_time(&mut self) {
        self.test_start_time = get_current_time_as_millis()
    }

    pub fn get_time_and_wpm(&mut self) {
        // Calculate time
        self.stats.time = (get_current_time_as_millis() - self.test_start_time) as f64;
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

        // Check that the character is correct if on master difficulty
        if self.test_difficulty == Difficulty::Master
            && self.input[self.word_index]
                .chars()
                .nth(self.character_index)
                != self.words[self.word_index]
                    .chars()
                    .nth(self.character_index)
        {
            self.state = State::EndScreen
        }

        self.character_index += 1;
    }

    fn add_word(&mut self) {
        let w_idx = self.word_index;
        if self.words[w_idx].len() > self.input[w_idx].len() {
            let buffer_count = (self.words[w_idx].len() - self.input[w_idx].len()) as u8;
            self.input_buffer[w_idx] = buffer_count;
        }

        // Check that the word is correct if on Expert difficulty
        if self.test_difficulty == Difficulty::Expert && self.words[w_idx] != self.input[w_idx] {
            self.state = State::EndScreen
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

    pub fn main_menu_input(&mut self, keycode: crossterm::event::KeyCode) {
        match keycode {
            KeyCode::Char('k') => {
                self.mainmenu_selection =
                    (self.mainmenu_selection + MainMenu::COUNT - 1) % MainMenu::COUNT;
            }
            KeyCode::Char('j') => {
                self.mainmenu_selection = (self.mainmenu_selection + 1) % MainMenu::COUNT;
            }
            KeyCode::Enter => {
                match MainMenu::from_index(self.mainmenu_selection) {
                    MainMenu::StartTest => self.restart_test(),
                    MainMenu::SelectTestType => self.state = State::TestTypeSelector,
                    MainMenu::SelectLanguage => self.state = State::LanguageSelector,
                    MainMenu::SelectDifficulty => self.state = State::DifficultySelector,
                }
                self.mainmenu_selection = 0;
            }
            KeyCode::Char('q') => self.should_exit = true,
            _ => {}
        }
    }

    pub fn difficulty_selector_input(&mut self, keycode: crossterm::event::KeyCode) {
        match keycode {
            KeyCode::Char('k') => {
                self.difficulty_selection =
                    (self.difficulty_selection + Difficulty::COUNT - 1) % Difficulty::COUNT;
            }
            KeyCode::Char('j') => {
                self.difficulty_selection = (self.difficulty_selection + 1) % Difficulty::COUNT;
            }
            KeyCode::Enter => {
                self.test_difficulty = Difficulty::from_index(self.difficulty_selection);
                self.state = State::MainMenu;
                self.difficulty_selection = 0
            }
            KeyCode::Char('q') => self.should_exit = true,
            KeyCode::Esc => self.state = State::MainMenu,
            _ => {}
        }
    }

    pub fn test_type_selector_input(&mut self, keycode: crossterm::event::KeyCode) {
        match keycode {
            KeyCode::Char('k') => {
                self.testtype_selection =
                    (self.testtype_selection + TestType::COUNT - 1) % TestType::COUNT;
            }
            KeyCode::Char('j') => {
                self.testtype_selection = (self.testtype_selection + 1) % TestType::COUNT;
            }
            KeyCode::Enter => {
                self.test_type = TestType::from_index(self.testtype_selection);
                self.state = State::MainMenu;
                self.testtype_selection = 0
            }
            KeyCode::Char('q') => self.should_exit = true,
            KeyCode::Esc => self.state = State::MainMenu,
            _ => {}
        }
    }

    pub fn language_selector_input(&mut self, keycode: crossterm::event::KeyCode) {
        match keycode {
            KeyCode::Char('k') => {
                self.language_selection =
                    (self.language_selection + Language::COUNT - 1) % Language::COUNT;
            }
            KeyCode::Char('j') => {
                self.language_selection = (self.language_selection + 1) % Language::COUNT;
            }
            KeyCode::Enter => {
                self.language = Language::from_index(self.language_selection);
                self.state = State::MainMenu;
                self.language_selection = 0
            }
            KeyCode::Char('q') => self.should_exit = true,
            KeyCode::Esc => self.state = State::MainMenu,
            _ => {}
        }
    }

    pub fn end_screen_input(&mut self, keycode: crossterm::event::KeyCode) {
        match keycode {
            KeyCode::Tab => self.restart_test(),
            KeyCode::Esc => self.state = State::MainMenu,
            KeyCode::Char('q') => self.should_exit = true,
            _ => {}
        }
    }

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
}
