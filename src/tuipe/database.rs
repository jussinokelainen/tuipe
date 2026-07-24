use crate::Tuipe;
use crate::tuipe::db_path;
use crate::tuipe::{Language, TestType};

impl Tuipe {
    // Save the test results into the database
    pub fn save_to_db(&self) -> bool {
        // results(wpm REAL, raw_wpm REAL, accuracy REAL, test_type TEXT, language TEXT, characters_typed INTEGER, time INTEGER)
        let db_path = db_path();
        let sql_statement = format!(
            "
            INSERT INTO
                results(wpm, raw_wpm, accuracy, test_type, language, characters_typed, time)
                VALUES ({}, {}, {}, '{}', '{}', {}, {});",
            self.stats.wpm,
            self.stats.wpm_raw,
            self.stats.accuracy,
            TestType::as_string(&self.test.ttype),
            Language::as_string(&self.language),
            self.stats.typed_characters,
            self.stats.time
        );
        let connection = sqlite::open(db_path).ok();
        // CLDL-ENTRY: title: error handling, priority: 5, tag: db
        match connection {
            Some(connection) => {
                let res = connection.execute(sql_statement);
                if res.is_ok() { true } else { false }
            }
            None => false,
        }
    }
}
