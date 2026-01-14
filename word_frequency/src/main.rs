use std::collections::HashMap;
use std::env;
use std::io::{self, Read};

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut top_n = 10;
    let mut min_length = 1;
    let mut ignore_case = false;
    let mut input_text = String::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--top" => {
                if let Some(val) = args.get(i + 1) {
                    top_n = val.parse().unwrap_or(10);
                    i += 1;
                }
            }
            "--min-length" => {
                if let Some(val) = args.get(i + 1) {
                    min_length = val.parse().unwrap_or(1);
                    i += 1;
                }
            }
            "--ignore-case" => ignore_case = true,
            "-h" | "--help" => {
                println!("Usage: wordfreq [OPTIONS] [TEXT]");
                return;
            }
            _ => input_text = args[i].clone(),
        }
        i += 1;
    }

    if input_text.is_empty() {
        let _ = io::stdin().read_to_string(&mut input_text);
    }

    let mut counts = HashMap::new();

    let text_to_process = if ignore_case {
        input_text.to_lowercase()
    } else {
        input_text
    };

    for word in text_to_process.split_whitespace() {
        let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric());

        if clean_word.len() >= min_length {
            *counts.entry(clean_word.to_string()).or_insert(0) += 1;
        }
    }

    let mut sorted_counts: Vec<(&String, &i32)> = counts.iter().collect();
    sorted_counts.sort_by(|a, b| b.1.cmp(a.1));

    println!("Word frequency:");
    for (word, count) in sorted_counts.into_iter().take(top_n) {
        println!("{}: {}", word, count);
    }
}
