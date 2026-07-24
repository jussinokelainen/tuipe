use crate::Tuipe;
use crate::tuipe::{Language, TestType, db_path, structs::DBdata};
use sqlite::State;

impl Tuipe {
    // Save the test results into the database
    pub fn save_to_db(&self) -> Result<(), sqlite::Error> {
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
        let connection = sqlite::open(db_path)?;
        connection.execute(sql_statement)
    }

    // Get stats for all saved test results in the database
    // Returns all stats as Vec<DBdata>, or an error if there was an error
    pub fn get_stats_from_db(&self) -> Result<Vec<DBdata>, sqlite::Error> {
        let db_path = db_path();
        let sql_statement = "
            SELECT
                wpm,
                raw_wpm,
                accuracy,
                test_type,
                language,
                characters_typed,
                time
            FROM
                results
            ;";
        let connection = sqlite::open(db_path)?;
        let mut out_vec = Vec::new();
        let mut statement = connection.prepare(sql_statement)?;
        loop {
            match statement.next()? {
                State::Row => {
                    let mut data = DBdata::new();
                    data.wpm = statement.read::<f64, _>("wpm")?;
                    data.raw_wpm = statement.read::<f64, _>("raw_wpm")?;
                    data.accuracy = statement.read::<f64, _>("accuracy")?;
                    data.test_type = statement.read::<String, _>("test_type")?;
                    data.language = statement.read::<String, _>("language")?;
                    data.characters_typed = statement.read::<i64, _>("characters_typed")? as u16;
                    data.time = statement.read::<i64, _>("time")? as u128;
                    out_vec.push(data);
                }
                State::Done => break,
            }
        }

        Ok(out_vec)
    }
}
