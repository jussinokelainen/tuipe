mod tuipe;
use color_eyre::Result;
use rand::rng;
use rand::seq::IndexedRandom;
use std::fs;
use tuipe::Language;
use tuipe::Tuipe;

fn main() -> Result<()> {
    color_eyre::install()?;
    ratatui::run(|terminal| Tuipe::new().run(terminal))
}

fn get_words_as_vector(language: tuipe::Language) -> Vec<String> {
    let mut words = Vec::new();
    let count = 50;
    let wordfile;
    match language {
        Language::English => wordfile = "/usr/share/tuipe/words/english.json",
        Language::English1k => wordfile = "/usr/share/tuipe/words/english_1k.json",
        Language::English5k => wordfile = "/usr/share/tuipe/words/english_5k.json",
        Language::English10k => wordfile = "/usr/share/tuipe/words/english_10k.json",
        Language::English25k => wordfile = "/usr/share/tuipe/words/english_25k.json",
    }

    let data = fs::read_to_string(wordfile).expect("Failed to read file");
    let word_vector: Vec<String> = serde_json::from_str(&data).expect("Failed to parse JSON");

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
