mod database;
mod menu_main;
mod menu_stats;
mod opt_capitalization;
mod opt_difficulty;
mod opt_language;
mod opt_main;
mod opt_numbers;
mod opt_testtype;
mod test_over;
mod test_running;
use crate::tuipe::opt_main::OptMenu;
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

// Main state enum for the program
enum State {
    MainMenu,
    OptMenu,
    StatsScreen,
    TestFinished,
    TestInterrupted,
    Typing,
}

// Main struct for the program
pub struct Tuipe {
    save_success: Result<(), sqlite::Error>,
    should_exit: bool,
    state: State,
    version: &'static str,

    menu_selection: usize,

    opts: Opts,
    stats: FinalStats,
    test: Test,

    input: Vec<String>,
    input_buffer: Vec<u8>,

    character_index: usize,
    word_index: usize,
    words: Vec<String>,
}

struct Opts {
    capitals: bool,
    difficulty: Difficulty,
    language: Language,
    numbers: bool,
    ttype: TestType,

    state: OptMenu,
}

struct Test {
    correct_chars: u16,
    incorrect_chars: u16,
    is_started: bool,
    is_timed: bool,
    start_time: u128,
    time_limit: usize,
}

struct FinalStats {
    accuracy: f64,
    time: f64,
    time_is_set: bool,
    typed_characters: usize,
    typed_words: usize,
    wpm: f64,
    wpm_raw: f64,
}

pub struct DBdata {
    pub accuracy: f64,
    pub characters_typed: u16,
    pub language: String,
    pub raw_wpm: f64,
    pub test_type: String,
    pub time: u128,
    pub wpm: f64,
}

// Opts struct but every element is a String or bool so they can be saved
// into the json file
#[derive(Serialize, Deserialize)]
struct Config {
    capitals: bool,
    difficulty: String,
    language: String,
    numbers: bool,
    test_type: String,
}

impl Opts {
    fn new() -> Self {
        Self {
            capitals: false,
            difficulty: Difficulty::Normal,
            language: Language::English,
            numbers: false,
            ttype: TestType::Words25,

            state: OptMenu::Main,
        }
    }
}

impl Test {
    fn new() -> Self {
        Self {
            correct_chars: 0,
            incorrect_chars: 0,
            is_started: false,
            is_timed: false,
            start_time: 0,
            time_limit: 0,
        }
    }
}

impl FinalStats {
    fn new() -> Self {
        Self {
            accuracy: 0.0,
            time: 0.0,
            time_is_set: false,
            typed_characters: 0,
            typed_words: 0,
            wpm: 0.0,
            wpm_raw: 0.0,
        }
    }
}

impl DBdata {
    pub fn new() -> Self {
        Self {
            accuracy: 0.0,
            characters_typed: 0,
            language: String::new(),
            raw_wpm: 0.0,
            test_type: String::new(),
            time: 0,
            wpm: 0.0,
        }
    }
}

impl Tuipe {
    pub fn new() -> Self {
        let opts_struct = match load_configs() {
            Ok(loaded_opts) => loaded_opts,
            Err(_) => Opts::new(),
        };
        Self {
            save_success: Ok(()),
            // This is a weird way to do this but it should work fine,
            // since if creating the database fails i want the program
            // to exit atleast for now, maybe later this will change
            should_exit: !database_exists(),
            state: State::MainMenu,
            version: match option_env!("VERSION") {
                Some(version_num) => version_num,
                None => "UNKNOWN",
            },

            menu_selection: 0,

            opts: opts_struct,
            stats: FinalStats::new(),
            test: Test::new(),

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

        (self.test.is_timed, self.test.time_limit) = TestType::is_timed(&self.opts.ttype);
        self.test.is_started = false;
        self.test.start_time = 0;
        self.test.correct_chars = 0;
        self.test.incorrect_chars = 0;

        self.stats = FinalStats::new();

        self.input = vec![String::new()];
        self.input_buffer = vec![0];

        self.character_index = 0;
        self.word_index = 0;
        self.words = get_words_as_vector(&self.opts);
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
            State::MainMenu => self.render_main_menu(frame),
            State::OptMenu => self.render_opt(frame),
            State::StatsScreen => self.render_stats_screen(frame),
            State::TestFinished => self.render_test_finished(frame),
            State::TestInterrupted => self.render_test_interrupted(frame),
            State::Typing => self.render_test(frame),
        }
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;

            if let Some(key) = event::read()?.as_key_press_event() {
                match self.state {
                    State::MainMenu => self.main_menu_input(key.code),
                    State::OptMenu => self.opt_menu_controls(key.code),
                    State::StatsScreen => self.stats_screen_input(key.code),
                    State::TestFinished => self.end_screen_input(key.code),
                    State::TestInterrupted => self.end_screen_input(key.code),
                    State::Typing if key.kind == KeyEventKind::Press => {
                        self.typing_test_input(key.code)
                    }
                    State::Typing => {}
                }
            }
            if self.should_exit {
                save_configs(self.opts)?;
                log::info!("Application stopped");
                return Ok(());
            }
        }
    }
}

// Returns a vector containing the words for the typing test
// Takes the test language and the test type as parameters
fn get_words_as_vector(opts: &Opts) -> Vec<String> {
    // Get directory for word files at compile time from the DATADIR argument,
    // or use /usr/share/tuipe as default fallback
    const DATADIR: &str = match option_env!("DATADIR") {
        Some(path) => path,
        None => "/usr/share/tuipe",
    };

    let count = TestType::word_count(&opts.ttype);
    let wordfile = match opts.language {
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

    // Take capitals as an extra bool so when given a word that is numbers
    // it doesnt try to do weird stuff with them, just adds them to the vector
    // (thanks borrow checker)
    let mut add_word = |mut word: String, capitals: bool| {
        if capitals {
            let mut char_rng = rand::rng();
            word = word
                .chars()
                .map(|c| {
                    if char_rng.random_bool(0.25) {
                        c.to_ascii_uppercase()
                    } else {
                        c
                    }
                })
                .collect();
            words.push(word);
        } else {
            words.push(word);
        }
    };

    while i < count {
        if let Some(word) = word_vector.choose(&mut rng) {
            let new_word = word.to_lowercase();
            if new_word != prev_word {
                if opts.numbers {
                    let mut num_rng = rand::rng();
                    if num_rng.random_bool(0.10) {
                        let num_count = new_word.len();
                        let number_string: String = (0..num_count)
                            .map(|_| num_rng.random_range(0..10).to_string())
                            .collect();
                        add_word(number_string, false)
                    } else {
                        add_word(new_word.clone(), opts.capitals)
                    }
                } else {
                    add_word(new_word.clone(), opts.capitals);
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
fn load_configs() -> Result<Opts> {
    // Check if the config file exists
    config_file_exists()?;

    let json = std::fs::read_to_string(config_path()?)?;
    let config: Config = serde_json::from_str(&json)?;

    log::info!(
        "Loaded configs: language:{}, type:{}, difficulty:{}, capitals:{}, numbers:{}",
        config.language,
        config.test_type,
        config.difficulty,
        config.capitals,
        config.numbers
    );

    Ok(Opts {
        capitals: config.capitals,
        difficulty: Difficulty::from_string(config.difficulty.as_str()),
        language: Language::from_string(config.language.as_str()),
        numbers: config.numbers,
        ttype: TestType::from_string(config.test_type.as_str()),

        state: OptMenu::Main,
    })
}

// Function to save the user's options to a json file, Takes the Opts struct
// that has the values to be saved as an argument
// Either returns nothing, or an error if one occurred
fn save_configs(conf: Opts) -> Result<()> {
    let config = Config {
        language: String::from(Language::as_string(&conf.language)),
        test_type: String::from(TestType::as_string(&conf.ttype)),
        difficulty: String::from(Difficulty::as_string(&conf.difficulty)),
        numbers: conf.numbers,
        capitals: conf.capitals,
    };
    let json = serde_json::to_string_pretty(&config)?;
    std::fs::write(config_path()?, json)?;

    log::info!(
        "Saved configs: language:{}, type:{}, difficulty:{}, capitals:{}, numbers: {}",
        config.language,
        config.test_type,
        config.difficulty,
        config.capitals,
        config.numbers
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
                numbers BOOLEAN,
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
