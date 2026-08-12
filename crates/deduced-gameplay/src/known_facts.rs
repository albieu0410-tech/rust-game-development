use std::collections::{HashMap, HashSet};

use deduced_core::{AttributeValue, Comparison, GuessResult};

/// A piece of information the player can be certain of, derived from all guesses
/// made so far in a round.
#[derive(Debug, Clone, PartialEq)]
pub enum KnownFact {
    /// The answer's value for this attribute is known exactly (a guess matched it).
    Exact {
        key: String,
        label: String,
        value: AttributeValue,
    },
    /// The answer's numeric value for this attribute is known to sit within these
    /// bounds (inclusive ends are not implied; `None` means unbounded on that side).
    Range {
        key: String,
        label: String,
        min: Option<f64>,
        max: Option<f64>,
    },
}

/// Aggregates every guess made in a round into the set of facts a player can
/// currently deduce about the hidden answer.
///
/// Only `Match` comparisons (exact facts) and `Higher`/`Lower` comparisons on
/// numeric attributes (range bounds) carry durable information; `Different` and
/// `Partial` results eliminate possibilities but don't pin down a value, so they
/// are not surfaced here.
pub fn derive_known_facts(guesses: &[GuessResult]) -> Vec<KnownFact> {
    let mut exact: HashMap<String, (String, AttributeValue)> = HashMap::new();
    let mut ranges: HashMap<String, (String, Option<f64>, Option<f64>)> = HashMap::new();
    let mut seen = HashSet::new();
    let mut order = Vec::new();

    for guess in guesses {
        for comparison in &guess.comparisons {
            if seen.insert(comparison.key.clone()) {
                order.push(comparison.key.clone());
            }

            match comparison.comparison {
                Comparison::Match => {
                    exact.insert(
                        comparison.key.clone(),
                        (comparison.label.clone(), comparison.guessed_value.clone()),
                    );
                }
                Comparison::Higher | Comparison::Lower => {
                    let Some(value) = numeric_value(&comparison.guessed_value) else {
                        continue;
                    };

                    let entry = ranges
                        .entry(comparison.key.clone())
                        .or_insert_with(|| (comparison.label.clone(), None, None));

                    match comparison.comparison {
                        Comparison::Higher => {
                            entry.1 = Some(entry.1.map_or(value, |min: f64| min.max(value)));
                        }
                        Comparison::Lower => {
                            entry.2 = Some(entry.2.map_or(value, |max: f64| max.min(value)));
                        }
                        _ => unreachable!(),
                    }
                }
                Comparison::Different | Comparison::Partial => {}
            }
        }
    }

    order
        .into_iter()
        .filter_map(|key| {
            if let Some((label, value)) = exact.get(&key) {
                Some(KnownFact::Exact {
                    key,
                    label: label.clone(),
                    value: value.clone(),
                })
            } else {
                ranges.get(&key).map(|(label, min, max)| KnownFact::Range {
                    key,
                    label: label.clone(),
                    min: *min,
                    max: *max,
                })
            }
        })
        .collect()
}

fn numeric_value(value: &AttributeValue) -> Option<f64> {
    match value {
        AttributeValue::Number(number) => Some(*number),
        _ => None,
    }
}
