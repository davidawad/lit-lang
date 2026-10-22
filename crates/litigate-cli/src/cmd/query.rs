//! stub
pub struct Args {
    pub kind: crate::cli::QueryKind,
    pub text: Option<String>,
    pub cite: Option<String>,
    pub effective_on: Option<String>,
    pub overrides: Option<String>,
    pub package: Option<String>,
    pub limit: usize,
    pub cursor: Option<String>,
    pub paths: Vec<String>,
    pub lib: Vec<String>,
}
pub fn query(_a: &Args) -> crate::out::Outcome {
    Err(crate::out::Failure::new("x"))
}
