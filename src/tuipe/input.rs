use crate::Tuipe;
use std::time::{SystemTime, UNIX_EPOCH};

impl Tuipe {
    fn set_start_time(&mut self) {
        let time_now = SystemTime::now();
        let start_time = time_now
            .duration_since(UNIX_EPOCH)
            .expect("time should go forward");
        self.test_start_time = start_time.as_millis()
    }

    pub fn get_time_and_wpm(&mut self) {
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

    pub fn enter_char(&mut self, new_char: char) {
        if !self.test_is_started {
            self.test_is_started = true;
            self.set_start_time()
        }
        self.input[self.word_index] += new_char.to_string().as_str();
        self.character_index += 1;
    }

    pub fn add_word(&mut self) {
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

    pub fn delete_char(&mut self) {
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
}
