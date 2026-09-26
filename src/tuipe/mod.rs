mod database;
mod menu_main;
mod menu_stats;
mod opt_capitalization;
mod opt_difficulty;
mod opt_language;
mod opt_main;
mod opt_testtype;
mod test_over;
mod test_running;
pub use crate::tuipe::{
    opt_difficulty::Difficulty, opt_language::Language, opt_testtype::TestType,
};
use color_eyre::Result;
use crossterm::event::{self, KeyEventKind};
use rand::{RngExt, rng, seq::IndexedRandom};
use ratatui::DefaultTerminal;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, fs, fs::File, fs::create_dir_all, io::Error};

pub struct Test {
    pub capitals: bool,
    pub correct_chars: u16,
    pub difficulty: Difficulty,
    pub incorrect_chars: u16,
    pub is_started: bool,
    pub is_timed: bool,
    pub start_time: u128,
    pub time_limit: usize,
    pub ttype: TestType,
}

impl Test {
    pub fn new() -> Self {
        Self {
            capitals: false,
            correct_chars: 0,
            difficulty: Difficulty::Normal,
            incorrect_chars: 0,
            is_started: false,
            is_timed: false,
            start_time: 0,
            time_limit: 0,
            ttype: TestType::Words25,
        }
    }
}

pub struct FinalStats {
    pub wpm: f64,
    pub wpm_raw: f64,
    pub accuracy: f64,
    pub time: f64,
    pub time_is_set: bool,
    pub typed_words: usize,
    pub typed_characters: usize,
}

impl FinalStats {
    pub fn new() -> Self {
        Self {
            wpm: 0.0,
            wpm_raw: 0.0,
            accuracy: 0.0,
            time: 0.0,
            time_is_set: false,
            typed_words: 0,
            typed_characters: 0,
        }
    }
}

pub struct DBdata {
    pub wpm: f64,
    pub raw_wpm: f64,
    pub accuracy: f64,
    pub test_type: String,
    pub language: String,
    pub characters_typed: u16,
    pub time: u128,
}

impl DBdata {
    pub fn new() -> Self {
        Self {
            wpm: 0.0,
            raw_wpm: 0.0,
            accuracy: 0.0,
            test_type: String::new(),
            language: String::new(),
            characters_typed: 0,
            time: 0,
        }
    }
}

// Main state enum for the program
pub enum State {
    MainMenu,
    OptMenu,
    StatsScreen,
    LanguageSelector,
    TestTypeSelector,
    DifficultySelector,
    CapitalizationSelector,
    TestFinished,
    TestInterrupted,
    Typing,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub language: String,
    pub test_type: String,
    pub difficulty: String,
    pub capitals: bool,
}

// Returns a vector containing the words for the typing test
// Takes the test language and the test type as parameters
fn get_words_as_vector(language: &Language, test_type: &TestType, capitals: bool) -> Vec<String> {
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

    let data = fs::read_to_string(&wordfile).expect("Failed to read file");
    let word_vector: Vec<String> = serde_json::from_str(&data).expect("Failed to parse JSON");

    log::info!("Read wordfile: {}", &wordfile);

    let mut words = Vec::new();
    let mut rng = rng();
    let mut prev_word = String::from("");
    let mut i = 0;
    while i < count {
        if let Some(word) = word_vector.choose(&mut rng) {
            let mut new_word = word.to_lowercase();
            if new_word != prev_word {
                if capitals {
                    let mut char_rng = rand::rng();
                    new_word = new_word
                        .chars()
                        .map(|c| {
                            if char_rng.random_bool(0.25) {
                                c.to_ascii_uppercase()
                            } else {
                                c
                            }
                        })
                        .collect();
                    words.push(new_word.clone());
                } else {
                    words.push(new_word.clone());
                }
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

fn get_share_dir() -> Result<PathBuf, Error> {
    let path = PathBuf::from(env::var("HOME").expect("$HOME not set")).join(".local/share/tuipe/");
    match create_dir_all(&path) {
        Ok(_) => Ok(path),
        Err(e) => Err(e),
    }
}

// Return the filepath to the config file, or an error
fn config_path() -> Result<PathBuf, Error> {
    let local_dir = get_share_dir()?;
    Ok(local_dir.join("config.json"))
}

// Check if the config file exists, and try to create it if it doesn't
fn config_file_exists() -> Result<()> {
    let path = config_path()?;
    if !path.is_file() {
        File::create(path)?;
    }
    Ok(())
}

// Function to read the config json file.
// Either returns an error, the read values or default values if the read
// values are something unexpected
fn load_configs() -> Result<(Language, TestType, Difficulty, bool)> {
    // Check if the config file exists
    config_file_exists()?;

    let json = std::fs::read_to_string(config_path()?)?;
    let config: Config = serde_json::from_str(&json)?;

    log::info!(
        "Loaded configs: language:{}, type:{}, difficulty:{}, capitals:{}",
        config.language,
        config.test_type,
        config.difficulty,
        config.capitals
    );

    Ok((
        Language::from_string(config.language.as_str()),
        TestType::from_string(config.test_type.as_str()),
        Difficulty::from_string(config.difficulty.as_str()),
        config.capitals,
    ))
}

fn save_configs(lang: Language, ttype: TestType, diff: Difficulty, capitals: bool) -> Result<()> {
    let config = Config {
        language: String::from(Language::as_string(&lang)),
        test_type: String::from(TestType::as_string(&ttype)),
        difficulty: String::from(Difficulty::as_string(&diff)),
        capitals: capitals,
    };
    let json = serde_json::to_string_pretty(&config)?;
    std::fs::write(config_path()?, json)?;

    log::info!(
        "Saved configs: language:{}, type:{}, difficulty:{}, capitals:{}",
        config.language,
        config.test_type,
        config.difficulty,
        config.capitals
    );

    Ok(())
}

// Returns the filepath of the local results database
// TODO: maybe return a result instead?
pub fn db_path() -> (PathBuf, bool) {
    let local_dir = get_share_dir();
    match local_dir {
        Ok(path) => (path.join("results.db"), true),
        Err(_) => (PathBuf::from(""), false),
    }
}

// Create the database table if it doesn't exist
// returns true if the table already existed or it was successfully created
fn database_exists() -> bool {
    let (db_path, success) = db_path();
    // If getting the path for the database was not successful, the database
    // does not exist to the program
    if !success {
        return false;
    }
    let table_create_query = "
            CREATE TABLE IF NOT EXISTS results(
                wpm REAL,
                raw_wpm REAL,
                accuracy REAL,
                test_type TEXT,
                language TEXT,
                capital_letters BOOLEAN,
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

// Main struct for the program
pub struct Tuipe {
    version: &'static str,
    state: State,
    should_exit: bool,
    language: Language,
    save_success: Result<(), sqlite::Error>,

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
        let mut test_struct = Test::new();
        let mut test_lang = Language::English;
        match load_configs() {
            Ok((lang, ttype, diff, caps)) => {
                test_lang = lang;
                test_struct.ttype = ttype;
                test_struct.difficulty = diff;
                test_struct.capitals = caps
            }
            Err(_) => {}
        }
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
            language: test_lang,
            save_success: Ok(()),

            menu_selection: 0,

            test: test_struct,
            stats: FinalStats::new(),

            input: vec![String::new()],
            input_buffer: vec![0],

            character_index: 0,
            word_index: 0,
            words: vec![String::new()],
        }
    }

    // Reset all required data fields for a new game
    fn restart_test(&mut self) {
        self.state = State::Typing;
        self.save_success = Ok(());

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
        self.words = get_words_as_vector(&self.language, &self.test.ttype, self.test.capitals);
    }

    // Checks whether the test is over, by either the time being up in a timed
    // test, or in a normal words test typing the last word correct or entering
    // a new word after the last word (pressing space during the last word)
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

        // Check if the time is up if typing a timed test
        if self.test.is_timed && self.test.is_started {
            let elapsed_test_time = (get_current_time_as_millis() - self.test.start_time) as f64;
            // Divide elapsed time by 1000 to convert it from milliseconds to seconds
            if (elapsed_test_time / 1000.0) >= self.test.time_limit as f64 {
                return true;
            }
        }

        false
    }

    // Function for rendering screens, returns a Rect positioned at the center of
    // the screen with maximum width and height of given parameters
    fn create_layout(&self, width: u16, height: u16, frame: &mut Frame) -> Rect {
        let layout_vert = Layout::default()
            .direction(Direction::Vertical)
            .flex(Flex::Center)
            .constraints([
                Constraint::Min(0),
                Constraint::Max(height),
                Constraint::Min(0),
            ])
            .split(frame.area());
        let layout_horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .flex(Flex::Center)
            .constraints([
                Constraint::Min(0),
                Constraint::Max(width),
                Constraint::Min(0),
            ])
            .split(layout_vert[1]);
        layout_horizontal[1]
    }

    // Adds the main menu controls as dark gray to the lines vector
    fn add_menu_controls(&self, lines: &mut Vec<Line<'_>>) {
        lines.push(Line::from(Span::raw("")));
        lines.push(Line::from(Span::raw("")));
        lines.push(Line::from(Span::styled(
            "Move: j/k",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::styled(
            "Select: Enter",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::styled(
            "Quit: q",
            Style::default().fg(Color::DarkGray),
        )));
    }

    // Adds the control info for the option menus as dark gray to given lines vector
    fn add_opt_menu_controls(&self, lines: &mut Vec<Line<'_>>) {
        self.add_menu_controls(lines);
        lines.push(Line::from(Span::styled(
            "Back: Esc",
            Style::default().fg(Color::DarkGray),
        )));
    }

    // The main render function of the program
    pub fn render(&mut self, frame: &mut Frame) {
        // check if test done instead of self.words == self.input
        if !self.stats.time_is_set && self.check_is_test_done() {
            self.set_final_stats(true);
            self.state = State::TestFinished;
        }

        match self.state {
            State::CapitalizationSelector => self.render_capitalization_selector(frame),
            State::DifficultySelector => self.render_difficulty_selector(frame),
            State::LanguageSelector => self.render_language_selector(frame),
            State::MainMenu => self.render_main_menu(frame),
            State::OptMenu => self.render_opt_menu(frame),
            State::StatsScreen => self.render_stats_screen(frame),
            State::TestFinished => self.render_test_finished(frame),
            State::TestInterrupted => self.render_test_interrupted(frame),
            State::TestTypeSelector => self.render_test_type_selector(frame),
            State::Typing => self.render_test(frame),
        }
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;

            if let Some(key) = event::read()?.as_key_press_event() {
                match self.state {
                    State::CapitalizationSelector => self.capitalization_selector_input(key.code),
                    State::DifficultySelector => self.difficulty_selector_input(key.code),
                    State::LanguageSelector => self.language_selector_input(key.code),
                    State::MainMenu => self.main_menu_input(key.code),
                    State::OptMenu => self.opt_menu_controls(key.code),
                    State::StatsScreen => self.stats_screen_input(key.code),
                    State::TestFinished => self.end_screen_input(key.code),
                    State::TestInterrupted => self.end_screen_input(key.code),
                    State::TestTypeSelector => self.test_type_selector_input(key.code),
                    State::Typing if key.kind == KeyEventKind::Press => {
                        self.typing_test_input(key.code)
                    }
                    State::Typing => {}
                }
            }
            if self.should_exit {
                save_configs(
                    self.language,
                    self.test.ttype,
                    self.test.difficulty,
                    self.test.capitals,
                )?;
                log::info!("Application stopped");
                return Ok(());
            }
        }
    }
}
