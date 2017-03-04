pub mod json;

pub fn join<S: ToString>(vec: Vec<S>, sep: &str) -> String {
    vec
        .iter()
        .fold("".to_string(), |a, b| if a.len() > 0 { a + sep } else { a } + &b.to_string())
}