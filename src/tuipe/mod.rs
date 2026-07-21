mod input;
mod render;
mod structs;
use color_eyre::Result;
use crossterm::event::{self, KeyEventKind};
use rand::rng;
use rand::seq::IndexedRandom;
use ratatui::DefaultTerminal;
use std::fs;
use structs::{FinalStats, State};
pub use structs::{Language, MainMenu, TestType};

fn get_words_as_vector(language: &Language, test_type: &TestType) -> Vec<String> {
    // Get directory for word files at compile time from the DATADIR argument,
    // or use /usr/share/tuipe as default fallback
    const DATA_DIR: &str = match option_env!("DATADIR") {
        Some(path) => path,
        None => "/usr/share/tuipe",
    };
    let count = match test_type {
        TestType::Words10 => 10,
        TestType::Words25 => 25,
        TestType::Words50 => 50,
    };
    let wordfile = match language {
        Language::English => DATA_DIR.to_string() + "/languages/english.json",
        Language::English1k => DATA_DIR.to_string() + "/languages/english_1k.json",
        Language::English5k => DATA_DIR.to_string() + "/languages/english_5k.json",
        Language::English10k => DATA_DIR.to_string() + "/languages/english_10k.json",
        Language::English25k => DATA_DIR.to_string() + "/languages/english_25k.json",
    };

    let data = fs::read_to_string(wordfile).expect("Failed to read file");
    let word_vector: Vec<String> = serde_json::from_str(&data).expect("Failed to parse JSON");

    let mut words = Vec::new();
    let mut rng = rng();
    let mut prev_word = "";
    let mut i = 0;
    while i < count {
        if let Some(new_word) = word_vector.choose(&mut rng) {
            if new_word != prev_word {
                words.push(new_word.clone().to_lowercase());
                prev_word = new_word;
                i += 1;
            }
        };
    }

    words
}

pub struct Tuipe {
    state: State,
    should_exit: bool,
    pub language: Language,
    pub test_type: TestType,

    language_selection: usize,
    test_selection: usize,
    mainmenu_selection: usize,

    test_is_started: bool,
    test_start_time: u128,

    stats: FinalStats,

    input: Vec<String>,
    input_buffer: Vec<u8>,

    character_index: usize,
    word_index: usize,
    words: Vec<String>,
}

impl Tuipe {
    pub fn new() -> Self {
        Self {
            state: State::MainMenu,
            should_exit: false,
            language: Language::English,
            test_type: TestType::Words10,
            language_selection: 0,
            test_selection: 0,
            mainmenu_selection: 0,

            test_is_started: false,
            test_start_time: 0,

            stats: FinalStats::new(),

            input: vec![String::new()],
            input_buffer: vec![0],

            character_index: 0,
            word_index: 0,
            words: vec![String::new()],
        }
    }

    fn restart_test(&mut self) {
        self.state = State::Typing;

        self.test_is_started = false;
        self.test_start_time = 0;

        self.stats = FinalStats::new();

        self.input = vec![String::new()];
        self.input_buffer = vec![0];

        self.character_index = 0;
        self.word_index = 0;
        self.words = get_words_as_vector(&self.language, &self.test_type);
    }

    fn check_is_test_done(&self) -> bool {
        let words_len = self.words.len();

        // This check is needed so the program doesn't index the
        // vectors on startup, since both vectors are initialized with
        // the same length
        if words_len > 1 {
            if words_len < self.input.len() {
                return true;
            }
            if words_len == self.input.len() {
                if self.words[words_len - 1] == self.input[words_len - 1] {
                    return true;
                }
            }
        }

        false
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;

            if let Some(key) = event::read()?.as_key_press_event() {
                match self.state {
                    State::TestTypeScreen => self.test_type_screen_input(key.code),
                    State::LanguageScreen => self.language_screen_input(key.code),
                    State::MainMenu => self.main_menu_input(key.code),
                    State::EndScreen => self.end_screen_input(key.code),
                    State::Typing if key.kind == KeyEventKind::Press => {
                        self.typing_test_input(key.code)
                    }
                    State::Typing => {}
                }
            }
            if self.should_exit {
                return Ok(());
            }
        }
    }
}
