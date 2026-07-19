use crate::tuipe::State;
use crate::tuipe::Tuipe;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Position};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

impl Tuipe {
    pub fn create_render_string(&mut self) -> Line<'_> {
        let mut output_as_vec = Vec::new();
        // See if the game is over, this is most likely not the right place
        // for this
        if self.words == self.input {
            self.input_mode = State::TestOver;
            self.get_test_time();
            println!("\n{}", self.test_final_time / 1000.0)
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

    pub fn render(&mut self, frame: &mut Frame) {
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
