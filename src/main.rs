mod tuipe;
mod tuipe_render;
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
    for _ in 0..count {
        if let Some(new_word) = word_vector.choose(&mut rng) {
            words.push(new_word.clone());
        };
    }

    words
}
