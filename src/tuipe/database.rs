use crate::Tuipe;
use crate::tuipe::{Language, TestType, db_path, structs::DBdata};
use sqlite::State;

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

    // Get stats for all saved test results in the database
    // Returns all stats as Vec<DBdata>, or an error if there was an error
    pub fn get_stats_from_db(&self) -> Result<Vec<DBdata>, &'static str> {
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
        let connection = sqlite::open(db_path).ok();
        match connection {
            Some(connection) => {
                let mut out_vec = Vec::new();
                // CLDL-ENTRY: title: unwraps, priority: 18, tag: db
                let mut statement = connection.prepare(sql_statement).unwrap();
                while let Ok(State::Row) = statement.next() {
                    let mut data = DBdata::new();
                    data.wpm = statement.read::<f64, _>("wpm").unwrap();
                    data.raw_wpm = statement.read::<f64, _>("raw_wpm").unwrap();
                    data.accuracy = statement.read::<f64, _>("accuracy").unwrap();
                    data.test_type = statement.read::<String, _>("test_type").unwrap();
                    data.language = statement.read::<String, _>("language").unwrap();
                    data.characters_typed =
                        statement.read::<i64, _>("characters_typed").unwrap() as u16;
                    data.time = statement.read::<i64, _>("time").unwrap() as u128;
                    out_vec.push(data);
                }
                Ok(out_vec)
            }
            None => Err("Could not open database connection"),
        }
    }
}
