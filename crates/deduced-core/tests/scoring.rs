use deduced_core::{Answer, Round, RoundConfig, RoundStatus, score_round};

#[test]
fn unfinished_round_scores_zero() {
    let answers = vec![Answer {
        id: "car_mazda".into(),
        name: "Mazda".into(),
        category: "cars".into(),
        image: None,
        attributes: Vec::new(),
    }];

    let round = Round::new(
        &answers,
        RoundConfig {
            category: "cars".into(),
            seed: 1,
            max_attempts: 5,
        },
    )
    .expect("round should be created");

    assert_eq!(round.status, RoundStatus::Playing);
    assert_eq!(score_round(&round).points, 0);
}
