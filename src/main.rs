use color_eyre::Result;
use crossterm::event::{self, KeyCode, KeyEventKind};
use rand::rng;
use rand::seq::IndexedRandom;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Position};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use ratatui::{DefaultTerminal, Frame};
use std::fs;

fn main() -> Result<()> {
    color_eyre::install()?;
    ratatui::run(|terminal| App::new().run(terminal))
}

fn get_words_as_vector(count: usize) -> Vec<String> {
    let mut words = Vec::new();

    let data =
        fs::read_to_string("/usr/share/tuipe/words/english.json").expect("Failed to read file");
    let word_vector: Vec<String> = serde_json::from_str(&data).expect("Failed to parse JSON");

    let mut rng = rng();
    for _ in 0..count {
        if let Some(new_word) = word_vector.choose(&mut rng) {
            words.push(new_word.clone());
        };
    }

    words
}

/// App holds the state of the application
struct App {
    input_mode: State,

    cursor_index: usize,
    character_index: usize,
    word_index: usize,

    input: Vec<String>,
    words: Vec<String>,
}

enum State {
    TestOver,
    Typing,
}

impl App {
    fn new() -> Self {
        Self {
            input_mode: State::Typing,
            cursor_index: 0,
            character_index: 0,
            word_index: 0,
            input: vec!["".to_string()],
            words: Vec::new(),
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_index > 0 {
            self.cursor_index -= 1
        }
    }

    fn move_cursor_right(&mut self) {
        self.cursor_index += 1;
    }

    fn enter_char(&mut self, new_char: char) {
        self.input[self.word_index] += new_char.to_string().as_str();
        self.character_index += 1;
        self.move_cursor_right();
    }

    fn add_word(&mut self) {
        self.character_index = 0;
        self.word_index += 1;
        self.input.push(String::from(""));
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
        self.input_mode = State::Typing;
        self.cursor_index = 0;
        self.character_index = 0;
        self.word_index = 0;
        self.reset_cursor();
    }

    fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
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

    fn create_render_string(&mut self) -> Line<'_> {
        let mut output_as_vec = Vec::new();
        if self.words == self.input {
            self.input_mode = State::TestOver
        }

        for (word_idx, word) in self.words.clone().into_iter().enumerate() {
            if word_idx > self.word_index {
                // The typer is not here yet, print the whole word as dark gray
                let color = Color::DarkGray;
                output_as_vec.push(Span::styled(word.clone(), Style::default().fg(color)))
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

                    output_as_vec.push(Span::styled(
                        char_as_str.to_string(),
                        Style::default().fg(color),
                    ))
                }
                // If the word at current word_idx has more characters in input
                // as in the real word, print them out here as red
                if self.input[word_idx].len() > word.len() {
                    let color = Color::Red;
                    let extra_characters =
                        &self.input[word_idx][word.len()..self.input[word_idx].len()];
                    output_as_vec.push(Span::styled(
                        extra_characters.to_string(),
                        Style::default().fg(color),
                    ))
                }
            }
            output_as_vec.push(Span::styled(" ", Style::default()))
        }

        Line::from(output_as_vec)
    }

    fn render(&mut self, frame: &mut Frame) {
        let layout_vert = Layout::default()
            .direction(Direction::Vertical)
            .flex(Flex::Center)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(50),
                Constraint::Percentage(25),
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
        let input = Paragraph::new(self.create_render_string())
            .style(Style::default())
            .block(Block::bordered());
        frame.render_widget(input, input_area);
        match self.input_mode {
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            State::TestOver => {}

            // Make the cursor visible and ask ratatui to put it at the specified coordinates after
            // rendering
            #[expect(clippy::cast_possible_truncation)]
            State::Typing => frame.set_cursor_position(Position::new(
                // Draw the cursor at the current position in the input field.
                // This position can be controlled via the left and right arrow key
                input_area.x + self.cursor_index as u16 + 1,
                // Move one line down, from the border to the input line
                input_area.y + 1,
            )),
        }
    }
}
