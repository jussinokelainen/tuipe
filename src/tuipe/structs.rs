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
    StartScreen,
    EndScreen,
    Typing,
}

#[derive(Clone)]
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
