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
    LanguageScreen,
    TestTypeScreen,
    EndScreen,
    Typing,
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
}

pub enum MainMenu {
    StartTest,
    SelectTestType,
    SelectLanguage,
}

impl MainMenu {
    pub const COUNT: usize = 3;

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => MainMenu::StartTest,
            1 => MainMenu::SelectTestType,
            2 => MainMenu::SelectLanguage,
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
}
