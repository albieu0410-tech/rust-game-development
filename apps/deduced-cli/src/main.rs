use std::{
    error::Error,
    io::{self, Write},
    path::Path,
};

use deduced_content::load_content_from_dir;
use deduced_core::{Comparison, Round, RoundConfig, RoundStatus, score_round};

fn main() -> Result<(), Box<dyn Error>> {
    let content = load_content_from_dir(Path::new("content"))?;
    let seed = seed_from_args().unwrap_or(839_183);

    if let Some(category_id) = reveal_answer_category_from_args() {
        let Some(category) = content.category(&category_id) else {
            println!("Unknown category: {category_id}");
            return Ok(());
        };
        let round = Round::new(
            &content.answers,
            RoundConfig {
                category: category.id.clone(),
                seed,
                max_attempts: category.attempts,
            },
        )?;
        println!("{}\t{}", round.answer.id, round.answer.name);
        return Ok(());
    }

    println!("DEDUCED");
    println!();
    println!("Categories:");
    for category in &content.categories {
        println!("- {} ({})", category.name, category.id);
    }

    let category_id = prompt("Category id: ")?;
    let Some(category) = content.category(&category_id) else {
        println!("Unknown category: {category_id}");
        return Ok(());
    };

    let mut round = Round::new(
        &content.answers,
        RoundConfig {
            category: category.id.clone(),
            seed,
            max_attempts: category.attempts,
        },
    )?;

    println!();
    println!("Category: {}", category.name);
    println!("Attempts: {}", round.max_attempts);
    println!("Seed: {seed} (pass --seed <n> to reproduce or force a different round)");
    println!("Type a guess by name or id.");

    while round.status == RoundStatus::Playing {
        println!();
        println!("Available guesses:");
        for answer in content.answers_for_category(&category.id) {
            println!("- {}", answer.name);
        }

        let input = prompt("Guess: ")?;
        let Some(guess) = content.find_answer(&category.id, &input) else {
            println!("No answer matched '{input}'.");
            continue;
        };

        let result = round.submit_guess(category, guess)?.clone();
        println!();
        println!("{}:", result.answer_name);
        for comparison in result.comparisons {
            println!(
                "{:<14} {:<18} {}",
                comparison.label,
                comparison.guessed_value.display_value(),
                comparison_symbol(comparison.comparison)
            );
        }
        println!("Attempts: {}/{}", round.attempts_used(), round.max_attempts);
    }

    println!();
    match round.status {
        RoundStatus::Won => {
            let score = score_round(&round);
            println!(
                "Won. Answer: {}. Score: {}",
                round.answer.name, score.points
            );
        }
        RoundStatus::Lost => {
            println!("Lost. Answer: {}.", round.answer.name);
        }
        RoundStatus::Playing => {}
    }

    Ok(())
}

/// Reads `--seed <n>` from the command line so a developer can force or
/// reproduce a specific round instead of always getting the default one.
fn seed_from_args() -> Option<u64> {
    let mut args = std::env::args();
    while let Some(arg) = args.next() {
        if arg == "--seed" {
            return args.next()?.parse().ok();
        }
    }
    None
}

/// Reads `--reveal-answer <category_id>`: prints `answer_id<TAB>answer_name`
/// for the given (seed, category) and exits immediately, without an
/// interactive loop. Dev/test tool only — lets automation and other clients
/// (e.g. testing the multiplayer server) reproduce a known target instead of
/// guessing blind.
fn reveal_answer_category_from_args() -> Option<String> {
    let mut args = std::env::args();
    while let Some(arg) = args.next() {
        if arg == "--reveal-answer" {
            return args.next();
        }
    }
    None
}

fn prompt(label: &str) -> io::Result<String> {
    print!("{label}");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn comparison_symbol(comparison: Comparison) -> &'static str {
    match comparison {
        Comparison::Match => "MATCH",
        Comparison::Higher => "HIGHER",
        Comparison::Lower => "LOWER",
        Comparison::Different => "DIFFERENT",
        Comparison::Partial => "PARTIAL",
    }
}
