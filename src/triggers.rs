// triggers.rs
pub fn trigger_words() -> Vec<&'static str> {
    vec![
        "generate", "create", "make", "produce", "design", "draw", "build", "elaborate",
        // ... (include all the words from the provided list)
        "change the", "make another", "one more", "replace the", "try another",
    ]
}

pub fn contains_trigger_word(input: &str) -> bool {
    let triggers = trigger_words();
    triggers.iter().any(|&trigger| input.to_lowercase().contains(trigger))
}
