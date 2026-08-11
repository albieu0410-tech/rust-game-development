use deduced_core::{AttributeValue, CategoryDefinition, Comparison, ComparisonRule, compare_value};

#[test]
fn numeric_comparison_points_toward_answer() {
    assert_eq!(
        compare_value(
            ComparisonRule::Numeric,
            &AttributeValue::Number(10.0),
            &AttributeValue::Number(5.0),
        ),
        Comparison::Higher
    );
    assert_eq!(
        compare_value(
            ComparisonRule::Numeric,
            &AttributeValue::Number(10.0),
            &AttributeValue::Number(15.0),
        ),
        Comparison::Lower
    );
}

#[test]
fn tags_can_match_partially() {
    assert_eq!(
        compare_value(
            ComparisonRule::Tags,
            &AttributeValue::Tags(vec!["Northern".into(), "Eastern".into()]),
            &AttributeValue::Tags(vec!["Northern".into(), "Western".into()]),
        ),
        Comparison::Partial
    );
}

#[test]
fn category_definition_can_be_built_from_data_shape() {
    let category = CategoryDefinition {
        id: "cars".into(),
        name: "Cars".into(),
        attempts: 5,
        attributes: Vec::new(),
    };

    assert_eq!(category.id, "cars");
}
