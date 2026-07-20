mod tuipe;
use color_eyre::Result;
use rand::rng;
use rand::seq::IndexedRandom;
use std::fs;
use tuipe::Tuipe;

fn main() -> Result<()> {
    color_eyre::install()?;
    ratatui::run(|terminal| Tuipe::new().run(terminal))
}

fn get_words_as_vector(count: usize) -> Vec<String> {
    let mut words = Vec::new();

    let data =
        fs::read_to_string("/usr/share/tuipe/words/english.json").expect("Failed to read file");
    let word_vector: Vec<String> = serde_json::from_str(&data).expect("Failed to parse JSON");

    let mut rng = rng();
    let mut prev_word = "";
    let mut i = 0;
    while i < count {
        if let Some(new_word) = word_vector.choose(&mut rng) {
            if new_word != prev_word {
                words.push(new_word.clone());
                prev_word = new_word;
                i += 1;
            }
        };
    }

    words
}
