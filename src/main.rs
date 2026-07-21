mod tuipe;
use crate::tuipe::Language;
use crate::tuipe::TestType;
use color_eyre::Result;
use tuipe::Tuipe;

fn main() -> Result<()> {
    color_eyre::install()?;
    ratatui::run(|terminal| Tuipe::new().run(terminal))
}
