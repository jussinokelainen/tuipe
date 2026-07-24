mod input;
mod render;
mod structs;
use color_eyre::Result;
use crossterm::event::{self, KeyEventKind};
use rand::rng;
use rand::seq::IndexedRandom;
use ratatui::DefaultTerminal;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, fs};
use structs::{Difficulty, FinalStats, State, Test};
pub use structs::{Language, MainMenu, TestType};

// Returns a vector containing the words for the typing test
// Takes the test language and the test type as parameters
fn get_words_as_vector(language: &Language, test_type: &TestType) -> Vec<String> {
    // Get directory for word files at compile time from the DATADIR argument,
    // or use /usr/share/tuipe as default fallback
    const DATADIR: &str = match option_env!("DATADIR") {
        Some(path) => path,
        None => "/usr/share/tuipe",
    };

    let count = TestType::word_count(&test_type);
    let wordfile = match language {
        Language::English => DATADIR.to_string() + "/languages/english.json",
        Language::English1k => DATADIR.to_string() + "/languages/english_1k.json",
        Language::English5k => DATADIR.to_string() + "/languages/english_5k.json",
        Language::English10k => DATADIR.to_string() + "/languages/english_10k.json",
        Language::English25k => DATADIR.to_string() + "/languages/english_25k.json",
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

// Returns the current time since the epoch in milliseconds
fn get_current_time_as_millis() -> u128 {
    let time_now = SystemTime::now();
    let current_time = time_now
        .duration_since(UNIX_EPOCH)
        .expect("time should go forward");
    current_time.as_millis()
}

// Returns the filepath of the local results database
fn db_path() -> PathBuf {
    let path = PathBuf::from(env::var("HOME").expect("$HOME not set"))
        .join(".local/share/tuipe/results.db");
    path
}

// Create the database table if it doesn't exist
// returns true if the table already existed or it was successfully created
fn database_exists() -> bool {
    let db_path = db_path();
    let table_create_query = "
            CREATE TABLE IF NOT EXISTS results(
                wpm REAL,
                raw_wpm REAL,
                accuracy REAL,
                test_type TEXT,
                language TEXT,
                characters_typed INTEGER,
                time INTEGER
            );";
    let connection = sqlite::open(db_path).ok();
    match connection {
        Some(connection) => {
            let res = connection.execute(table_create_query);
            if res.is_ok() { true } else { false }
        }
        None => false,
    }
}

pub struct Tuipe {
    version: &'static str,
    state: State,
    should_exit: bool,
    language: Language,

    menu_selection: usize,

    test: Test,
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
            version: match option_env!("VERSION") {
                Some(version_num) => version_num,
                None => "UNKNOWN",
            },
            state: State::MainMenu,
            // This is a weird way to do this but it should work fine,
            // since if creating the database fails i want the program
            // to exit atleast for now, maybe later this will change
            should_exit: !database_exists(),
            language: Language::English,

            menu_selection: 0,

            test: Test::new(),
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

        (self.test.is_timed, self.test.time_limit) = TestType::is_timed(&self.test.ttype);
        self.test.is_started = false;
        self.test.start_time = 0;
        self.test.correct_chars = 0;
        self.test.incorrect_chars = 0;

        self.stats = FinalStats::new();

        self.input = vec![String::new()];
        self.input_buffer = vec![0];

        self.character_index = 0;
        self.word_index = 0;
        self.words = get_words_as_vector(&self.language, &self.test.ttype);
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

        if self.test.is_timed && self.test.is_started {
            let elapsed_test_time = (get_current_time_as_millis() - self.test.start_time) as f64;
            // Divide elapsed time by 1000 to convert it from milliseconds to seconds
            if (elapsed_test_time / 1000.0) >= self.test.time_limit as f64 {
                return true;
            }
        }

        false
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;

            if let Some(key) = event::read()?.as_key_press_event() {
                match self.state {
                    State::TestTypeSelector => self.test_type_selector_input(key.code),
                    State::LanguageSelector => self.language_selector_input(key.code),
                    State::DifficultySelector => self.difficulty_selector_input(key.code),
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
