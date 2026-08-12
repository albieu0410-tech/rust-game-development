use deduced_core::{
    Answer, Attribute, AttributeDefinition, AttributeValue, CategoryDefinition, ComparisonRule,
    GuessResult,
};
use deduced_gameplay::{GameController, GameStatus, KnownFact, reveal_state};

fn category() -> CategoryDefinition {
    CategoryDefinition {
        id: "cars".to_string(),
        name: "Cars".to_string(),
        attempts: 5,
        attributes: vec![
            AttributeDefinition {
                key: "country".to_string(),
                label: "Country".to_string(),
                comparison: ComparisonRule::Exact,
            },
            AttributeDefinition {
                key: "founded".to_string(),
                label: "Founded".to_string(),
                comparison: ComparisonRule::Numeric,
            },
        ],
    }
}

fn answer(id: &str, name: &str, country: &str, founded: f64) -> Answer {
    Answer {
        id: id.to_string(),
        name: name.to_string(),
        category: "cars".to_string(),
        image: None,
        attributes: vec![
            Attribute {
                key: "country".to_string(),
                value: AttributeValue::Text(country.to_string()),
            },
            Attribute {
                key: "founded".to_string(),
                value: AttributeValue::Number(founded),
            },
        ],
    }
}

fn answers() -> Vec<Answer> {
    vec![
        answer("car_honda", "Honda", "Japan", 1948.0),
        answer("car_volvo", "Volvo", "Sweden", 1927.0),
        answer("car_bmw", "BMW", "Germany", 1916.0),
    ]
}

#[test]
fn known_facts_track_exact_matches_and_numeric_bounds() {
    let guesses = vec![
        GuessResult {
            answer_id: "car_volvo".to_string(),
            answer_name: "Volvo".to_string(),
            comparisons: vec![
                deduced_core::AttributeComparison {
                    key: "country".to_string(),
                    label: "Country".to_string(),
                    guessed_value: AttributeValue::Text("Sweden".to_string()),
                    comparison: deduced_core::Comparison::Different,
                },
                deduced_core::AttributeComparison {
                    key: "founded".to_string(),
                    label: "Founded".to_string(),
                    guessed_value: AttributeValue::Number(1927.0),
                    comparison: deduced_core::Comparison::Higher,
                },
            ],
        },
        GuessResult {
            answer_id: "car_bmw".to_string(),
            answer_name: "BMW".to_string(),
            comparisons: vec![
                deduced_core::AttributeComparison {
                    key: "country".to_string(),
                    label: "Country".to_string(),
                    guessed_value: AttributeValue::Text("Germany".to_string()),
                    comparison: deduced_core::Comparison::Different,
                },
                deduced_core::AttributeComparison {
                    key: "founded".to_string(),
                    label: "Founded".to_string(),
                    guessed_value: AttributeValue::Number(1916.0),
                    comparison: deduced_core::Comparison::Higher,
                },
            ],
        },
        GuessResult {
            answer_id: "car_honda".to_string(),
            answer_name: "Honda".to_string(),
            comparisons: vec![
                deduced_core::AttributeComparison {
                    key: "country".to_string(),
                    label: "Country".to_string(),
                    guessed_value: AttributeValue::Text("Japan".to_string()),
                    comparison: deduced_core::Comparison::Match,
                },
                deduced_core::AttributeComparison {
                    key: "founded".to_string(),
                    label: "Founded".to_string(),
                    guessed_value: AttributeValue::Number(1948.0),
                    comparison: deduced_core::Comparison::Match,
                },
            ],
        },
    ];

    let facts = deduced_gameplay::derive_known_facts(&guesses);

    assert_eq!(
        facts,
        vec![
            KnownFact::Exact {
                key: "country".to_string(),
                label: "Country".to_string(),
                value: AttributeValue::Text("Japan".to_string()),
            },
            KnownFact::Exact {
                key: "founded".to_string(),
                label: "Founded".to_string(),
                value: AttributeValue::Number(1948.0),
            },
        ]
    );
}

#[test]
fn known_facts_narrow_a_numeric_range_across_guesses_without_a_match() {
    let guesses = vec![
        GuessResult {
            answer_id: "car_volvo".to_string(),
            answer_name: "Volvo".to_string(),
            comparisons: vec![deduced_core::AttributeComparison {
                key: "founded".to_string(),
                label: "Founded".to_string(),
                guessed_value: AttributeValue::Number(1900.0),
                comparison: deduced_core::Comparison::Higher,
            }],
        },
        GuessResult {
            answer_id: "car_bmw".to_string(),
            answer_name: "BMW".to_string(),
            comparisons: vec![deduced_core::AttributeComparison {
                key: "founded".to_string(),
                label: "Founded".to_string(),
                guessed_value: AttributeValue::Number(1950.0),
                comparison: deduced_core::Comparison::Lower,
            }],
        },
        GuessResult {
            answer_id: "car_honda".to_string(),
            answer_name: "Honda".to_string(),
            comparisons: vec![deduced_core::AttributeComparison {
                key: "founded".to_string(),
                label: "Founded".to_string(),
                guessed_value: AttributeValue::Number(1920.0),
                comparison: deduced_core::Comparison::Higher,
            }],
        },
    ];

    let facts = deduced_gameplay::derive_known_facts(&guesses);

    assert_eq!(
        facts,
        vec![KnownFact::Range {
            key: "founded".to_string(),
            label: "Founded".to_string(),
            min: Some(1920.0),
            max: Some(1950.0),
        }]
    );
}

#[test]
fn reveal_state_progresses_from_first_attempt_to_completion() {
    assert_eq!(reveal_state(0, 5).level, 1);
    assert_eq!(reveal_state(4, 5).level, 5);
    assert_eq!(reveal_state(4, 5).max_level, 5);
    // Guessing beyond max_attempts (should not happen, but must not panic or overflow).
    assert_eq!(reveal_state(10, 5).level, 5);
}

#[test]
fn controller_tracks_attempts_and_reaches_a_win() {
    let mut controller =
        GameController::new_solo(&answers(), category(), 12345).expect("round should start");

    let target_id = controller.round().answer.id.clone();

    assert!(controller.result().is_none());
    assert_eq!(controller.state().status, GameStatus::Playing);
    assert_eq!(controller.state().attempts_used, 0);

    let target_answer = answers()
        .into_iter()
        .find(|answer| answer.id == target_id)
        .expect("target answer must exist in the fixture");

    controller
        .submit_guess(&target_answer)
        .expect("guess should be accepted");

    let state = controller.state();
    assert_eq!(state.status, GameStatus::Won);
    assert_eq!(state.attempts_used, 1);
    assert_eq!(state.guesses.len(), 1);

    let result = controller.result().expect("round should be finished");
    assert!(result.won);
    assert_eq!(result.answer_id, target_id);
    assert_eq!(result.category_id, "cars");
    assert_eq!(result.attempts_used, 1);
    assert!(result.score.points > 0);
}

#[test]
fn controller_reaches_a_loss_after_exhausting_attempts() {
    let mut controller =
        GameController::new_solo(&answers(), category(), 99).expect("round should start");

    let target_id = controller.round().answer.id.clone();
    let wrong_answers: Vec<Answer> = answers()
        .into_iter()
        .filter(|answer| answer.id != target_id)
        .collect();

    // The category allows 5 attempts but the fixture only has 2 wrong answers, so
    // repeat them to exhaust every attempt.
    for i in 0..5 {
        let guess = &wrong_answers[i % wrong_answers.len()];
        let outcome = controller
            .submit_guess(guess)
            .expect("guess should be accepted");
        assert_eq!(outcome.answer_id, guess.id);
    }

    let state = controller.state();
    assert_eq!(state.status, GameStatus::Lost);
    assert_eq!(state.attempts_used, 5);

    let result = controller.result().expect("round should be finished");
    assert!(!result.won);
    assert_eq!(result.answer_id, target_id);
    assert_eq!(result.category_id, "cars");
    assert_eq!(result.score.points, 0);
}

#[test]
fn repeated_guesses_do_not_duplicate_known_facts() {
    let mut controller =
        GameController::new_solo(&answers(), category(), 7).expect("round should start");

    let target_id = controller.round().answer.id.clone();
    let wrong_answer = answers()
        .into_iter()
        .find(|answer| answer.id != target_id)
        .expect("fixture has more than one answer");

    controller.submit_guess(&wrong_answer).unwrap();
    controller.submit_guess(&wrong_answer).unwrap();

    let facts = controller.state().known_facts;
    let founded_facts = facts
        .iter()
        .filter(|fact| matches!(fact, KnownFact::Range { key, .. } | KnownFact::Exact { key, .. } if key == "founded"))
        .count();

    assert!(
        founded_facts <= 1,
        "expected at most one fact per attribute key, got {founded_facts}"
    );
    assert_eq!(controller.state().attempts_used, 2);
}
