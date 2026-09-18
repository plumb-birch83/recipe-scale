use crate::quantity::Quantity;

pub struct Ingredient {
    pub quantity: Quantity,
    pub unit: Option<String>,
    pub name: String,
}

pub struct Recipe {
    pub name: String,
    pub base_servings: u32,
    pub ingredients: Vec<Ingredient>,
}

// Units we recognize as a unit token rather than the start of an
// ingredient name. Anything not on this list (like "eggs" in "3 eggs")
// is treated as part of the name with no unit.
const UNITS: &[&str] = &[
    "cup", "cups", "tsp", "tbsp", "oz", "lb", "lbs", "g", "kg", "ml", "l",
    "pinch", "pinches", "clove", "cloves", "can", "cans", "slice", "slices",
    "stick", "sticks", "quart", "quarts", "pint", "pints", "gallon", "gallons",
];

pub fn parse(input: &str) -> Result<Recipe, String> {
    let mut name = String::from("Untitled Recipe");
    let mut base_servings: Option<u32> = None;
    let mut ingredients = Vec::new();

    for (line_no, raw_line) in input.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix("name:") {
            name = rest.trim().to_string();
            continue;
        }
        if let Some(rest) = line.strip_prefix("servings:") {
            base_servings = Some(rest.trim().parse().map_err(|_| {
                format!("line {}: bad servings value: {}", line_no + 1, rest.trim())
            })?);
            continue;
        }
        ingredients
            .push(parse_ingredient(line).map_err(|e| format!("line {}: {}", line_no + 1, e))?);
    }

    let base_servings = base_servings.ok_or("recipe is missing a \"servings:\" line")?;
    if base_servings == 0 {
        return Err("servings must be greater than zero".to_string());
    }
    if ingredients.is_empty() {
        return Err("recipe has no ingredient lines".to_string());
    }

    Ok(Recipe {
        name,
        base_servings,
        ingredients,
    })
}

fn parse_ingredient(line: &str) -> Result<Ingredient, String> {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    let (quantity, consumed) = Quantity::parse_tokens(&tokens)?;
    let rest = &tokens[consumed..];
    if rest.is_empty() {
        return Err(format!("ingredient has a quantity but no name: {}", line));
    }

    let lowered = rest[0].to_lowercase();
    let (unit, name_tokens) = if UNITS.contains(&lowered.as_str()) {
        (Some(rest[0].to_string()), &rest[1..])
    } else {
        (None, rest)
    };
    if name_tokens.is_empty() {
        return Err(format!("ingredient has a unit but no name: {}", line));
    }

    Ok(Ingredient {
        quantity,
        unit,
        name: name_tokens.join(" "),
    })
}
