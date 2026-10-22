//! stub
pub struct Args {
    pub paths: Vec<String>,
    pub code: Option<String>,
    pub dry_run: bool,
    pub include_maybe: bool,
    pub profile: litigate_base::Profile,
    pub lib: Vec<String>,
}
pub fn fix(_a: &Args) -> crate::out::Outcome {
    Err(crate::out::Failure::new("x"))
}
