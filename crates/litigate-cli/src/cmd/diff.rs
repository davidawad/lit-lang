//! stub
pub struct Args {
    pub a: String,
    pub b: Option<String>,
    pub against: Option<String>,
    pub as_of: Option<String>,
    pub to_as_of: Option<String>,
    pub check: bool,
    pub lib: Vec<String>,
}
pub fn diff(_a: &Args) -> crate::out::Outcome {
    Err(crate::out::Failure::new("x"))
}
