pub struct Test {
    pub ttype: TestType,
    pub difficulty: Difficulty,
    pub is_started: bool,
    pub start_time: u128,
    pub is_timed: bool,
    pub time_limit: usize,
}

impl Test {
    pub fn new() -> Self {
        Self {
            ttype: TestType::Words10,
            difficulty: Difficulty::Expert,
            is_started: false,
            start_time: 0,
            is_timed: false,
            time_limit: 0,
        }
    }
}

pub struct FinalStats {
    pub wpm: f64,
    pub wpm_raw: f64,
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
            time: 0.0,
            time_is_set: false,
            typed_words: 0,
            typed_characters: 0,
        }
    }
}

pub enum State {
    MainMenu,
    LanguageSelector,
    TestTypeSelector,
    DifficultySelector,
    EndScreen,
    Typing,
}

#[derive(PartialEq)]
pub enum Difficulty {
    Normal,
    Expert,
    Master,
}

impl Difficulty {
    pub const COUNT: usize = 3;

    pub fn as_vec() -> Vec<&'static str> {
        vec!["Normal", "Expert", "Master"]
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Difficulty::Normal,
            1 => Difficulty::Expert,
            2 => Difficulty::Master,
            _ => Difficulty::Normal,
        }
    }
}

pub enum TestType {
    Words10,
    Words25,
    Words50,
    Time10,
    Time30,
    Time60,
}

impl TestType {
    pub const COUNT: usize = 6;

    pub fn word_count(&self) -> usize {
        match &self {
            TestType::Words10 => 10,
            TestType::Words25 => 25,
            TestType::Words50 => 50,
            TestType::Time10 => 250,
            TestType::Time30 => 250,
            TestType::Time60 => 500,
        }
    }
    pub fn is_timed(&self) -> (bool, usize) {
        match &self {
            TestType::Words10 => (false, 0),
            TestType::Words25 => (false, 0),
            TestType::Words50 => (false, 0),
            TestType::Time10 => (true, 10),
            TestType::Time30 => (true, 30),
            TestType::Time60 => (true, 60),
        }
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => TestType::Words10,
            1 => TestType::Words25,
            2 => TestType::Words50,
            3 => TestType::Time10,
            4 => TestType::Time30,
            5 => TestType::Time60,
            _ => TestType::Words10,
        }
    }

    pub fn as_string(&self) -> &'static str {
        match &self {
            TestType::Words10 => "10 Words",
            TestType::Words25 => "25 Words",
            TestType::Words50 => "50 Words",
            TestType::Time10 => "10 Seconds",
            TestType::Time30 => "30 Seconds",
            TestType::Time60 => "60 Seconds",
        }
    }

    pub fn as_vec() -> Vec<&'static str> {
        vec![
            "10 Words",
            "25 Words",
            "50 Words",
            "10 Seconds",
            "30 Seconds",
            "60 Seconds",
        ]
    }
}

pub enum MainMenu {
    StartTest,
    SelectTestType,
    SelectLanguage,
    SelectDifficulty,
}

impl MainMenu {
    pub const COUNT: usize = 4;

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => MainMenu::StartTest,
            1 => MainMenu::SelectTestType,
            2 => MainMenu::SelectLanguage,
            3 => MainMenu::SelectDifficulty,
            _ => MainMenu::StartTest,
        }
    }
}

pub enum Language {
    English,
    English1k,
    English5k,
    English10k,
    English25k,
}

impl Language {
    pub const COUNT: usize = 5;

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Language::English,
            1 => Language::English1k,
            2 => Language::English5k,
            3 => Language::English10k,
            4 => Language::English25k,
            _ => Language::English,
        }
    }

    pub fn as_vec() -> Vec<&'static str> {
        vec![
            "English",
            "English 1k",
            "English 5k",
            "English 10k",
            "English 25k",
        ]
    }

    pub fn as_string(&self) -> &'static str {
        match &self {
            Language::English => "English",
            Language::English1k => "English 1k",
            Language::English5k => "English 5k",
            Language::English10k => "English 10k",
            Language::English25k => "English 25k",
        }
    }
}
