# Content Format

Content is data-defined so new categories do not require new Rust structs.

## Category Definition

```json
{
  "id": "cars",
  "name": "Cars",
  "attempts": 5,
  "attributes": [
    {
      "key": "country",
      "label": "Country",
      "comparison": "exact"
    }
  ]
}
```

Supported comparison rules:

- `exact`: equal values match, unequal values are different.
- `numeric`: guesses lower than the answer show `Higher`; guesses higher than the answer show `Lower`.
- `tags`: identical tag lists match, overlapping tag lists are partial.
- `bool`: same boolean matches, different boolean is different.

## Answer Definition

```json
{
  "id": "car_mazda",
  "name": "Mazda",
  "category": "cars",
  "image": "images/cars/mazda.png",
  "attributes": [
    {
      "key": "country",
      "value": {
        "Text": "Japan"
      }
    }
  ]
}
```

Supported attribute values:

- `Text`
- `Number`
- `Bool`
- `Tags`

## Validation Rules

- Every answer must reference an existing category.
- Every answer must include each attribute declared by its category.
- Answers cannot include attributes that the category does not declare.
- Asset paths are relative to `assets/`.
