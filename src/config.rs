#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub enum TablePreset {
    Nothing,
    AsciiFull,
}

#[derive(Debug, Clone, Copy)]
pub struct StdoutConfig {
    pub table_preset: TablePreset,
    pub plain_output: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum OutputType {
    Stdout(StdoutConfig),
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub output_type: OutputType,
}
