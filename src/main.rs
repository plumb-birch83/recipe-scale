mod quantity;
mod recipe;

use quantity::Quantity;
use recipe::Recipe;
use std::env;
use std::fs;
use std::process::ExitCode;

struct Options {
    path: String,
    target_servings: Option<u32>,
    factor: Option<Quantity>,
    json: bool,
    round_to: Option<Quantity>,
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let opts = match parse_args(&args) {
        Ok(opts) => opts,
        Err(msg) => {
            eprintln!("error: {}", msg);
            eprintln!();
            print_usage();
            return ExitCode::FAILURE;
        }
    };

    let text = match fs::read_to_string(&opts.path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: could not read {}: {}", opts.path, e);
            return ExitCode::FAILURE;
        }
    };

    let recipe = match recipe::parse(&text) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: {}", e);
            return ExitCode::FAILURE;
        }
    };

    let factor = match resolve_factor(&recipe, &opts) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("error: {}", e);
            return ExitCode::FAILURE;
        }
    };

    if opts.json {
        print_json(&recipe, factor, opts.round_to);
    } else {
        print_human(&recipe, factor, opts.round_to);
    }

    ExitCode::SUCCESS
}

fn resolve_factor(recipe: &Recipe, opts: &Options) -> Result<Quantity, String> {
    match (opts.target_servings, opts.factor) {
        (Some(target), None) => {
            if target == 0 {
                return Err("--servings must be greater than zero".to_string());
            }
            Ok(Quantity::new(target as i64, recipe.base_servings as i64))
        }
        (None, Some(f)) => Ok(f),
        (None, None) => Err("give either --servings N or --factor X".to_string()),
        (Some(_), Some(_)) => Err("use --servings or --factor, not both".to_string()),
    }
}

fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut path = None;
    let mut target_servings = None;
    let mut factor = None;
    let mut json = false;
    let mut round_to = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json = true,
            "--servings" => {
                i += 1;
                let val = args.get(i).ok_or("--servings needs a value")?;
                target_servings = Some(
                    val.parse::<u32>()
                        .map_err(|_| format!("bad --servings value: {}", val))?,
                );
            }
            "--factor" => {
                i += 1;
                let val = args.get(i).ok_or("--factor needs a value")?;
                let (q, _) = Quantity::parse_tokens(&[val.as_str()])?;
                factor = Some(q);
            }
            "--round-to" => {
                i += 1;
                let val = args.get(i).ok_or("--round-to needs a value")?;
                let (step, _) = Quantity::parse_tokens(&[val.as_str()])?;
                if step.num <= 0 {
                    return Err(format!("--round-to must be positive: {}", val));
                }
                round_to = Some(step);
            }
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            other if path.is_none() && !other.starts_with('-') => {
                path = Some(other.to_string());
            }
            other => return Err(format!("unrecognized argument: {}", other)),
        }
        i += 1;
    }

    let path = path.ok_or("missing recipe file")?;
    Ok(Options {
        path,
        target_servings,
        factor,
        json,
        round_to,
    })
}

fn print_usage() {
    eprintln!(
        "usage: recipe-scale <recipe-file> (--servings N | --factor X) [--round-to STEP] [--json]"
    );
}

fn scaled_quantity(quantity: Quantity, factor: Quantity, round_to: Option<Quantity>) -> Quantity {
    let scaled = quantity.scaled_by(factor);
    match round_to {
        Some(step) => scaled.round_to(step),
        None => scaled,
    }
}

fn print_human(recipe: &Recipe, factor: Quantity, round_to: Option<Quantity>) {
    let target = factor.as_f64() * recipe.base_servings as f64;
    println!(
        "{} (scaled from {} to {:.0} servings, x{:.3})",
        recipe.name,
        recipe.base_servings,
        target,
        factor.as_f64()
    );
    println!();
    for ing in &recipe.ingredients {
        let scaled = scaled_quantity(ing.quantity, factor, round_to);
        match &ing.unit {
            Some(unit) => println!("  {} {} {}", scaled, unit, ing.name),
            None => println!("  {} {}", scaled, ing.name),
        }
    }
}

fn print_json(recipe: &Recipe, factor: Quantity, round_to: Option<Quantity>) {
    let mut out = String::new();
    out.push('{');
    out.push_str(&format!("\"name\":{},", json_string(&recipe.name)));
    out.push_str(&format!("\"base_servings\":{},", recipe.base_servings));
    out.push_str(&format!("\"factor\":{:.6},", factor.as_f64()));
    out.push_str("\"ingredients\":[");
    for (idx, ing) in recipe.ingredients.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        let scaled = scaled_quantity(ing.quantity, factor, round_to);
        out.push('{');
        out.push_str(&format!("\"quantity\":\"{}\",", scaled));
        out.push_str(&format!("\"quantity_decimal\":{:.4},", scaled.as_f64()));
        match &ing.unit {
            Some(unit) => out.push_str(&format!("\"unit\":{},", json_string(unit))),
            None => out.push_str("\"unit\":null,"),
        }
        out.push_str(&format!("\"name\":{}", json_string(&ing.name)));
        out.push('}');
    }
    out.push_str("]}");
    println!("{}", out);
}

fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
