use regex::Regex;

pub fn count_tokens(text: &str) -> usize {
    // This simple tokenizer counts words as tokens.
    // You can adjust the regex pattern to suit your needs.
    let re = Regex::new(r"\w+").unwrap();
    re.find_iter(text).count()
}

