# recipe-scale

A recipe in a cookbook is written for one serving count. As soon as you want
a different one, you're doing fraction arithmetic by hand: 1 1/2 tsp baking
powder for 4 servings becomes what, exactly, for 6? Most of the "recipe
scaler" tools online round to decimals and lose the fraction you actually
want to measure with. This one keeps quantities as exact fractions the
whole way through, so 1 1/2 scaled by 3/2 comes out as 2 1/4, not 2.25.

It answers one question: given a recipe file and a target serving count (or
an explicit multiplier), what are the scaled ingredient quantities?

## Recipe file format

Plain text. Two metadata lines, then one ingredient per line.

```
name: Fluffy Pancakes
servings: 4

1 1/2 cups all-purpose flour
3 1/2 tsp baking powder
1 tsp salt
1 tbsp sugar
1 1/4 cups milk
1 egg
3 tbsp butter, melted
```

Each ingredient line starts with a quantity: a whole number (`3`), a decimal
(`0.5`), a fraction (`1/2`), or a mixed number (`1 1/2`). After the quantity
comes an optional unit (cup, tsp, tbsp, oz, g, clove, and so on — see
`src/recipe.rs` for the full list) and then the ingredient name. If the word
after the quantity isn't a recognized unit, it's treated as part of the
name, so `3 eggs` scales fine with no unit at all.

## Usage

Scale to a target serving count:

```
$ recipe-scale examples/pancakes.txt --servings 6
Fluffy Pancakes (scaled from 4 to 6 servings, x1.500)

  2 1/4 cups all-purpose flour
  5 1/4 tsp baking powder
  1 1/2 tsp salt
  1 1/2 tbsp sugar
  1 7/8 cups milk
  1 1/2 egg
  4 1/2 tbsp butter, melted
```

Or scale by an explicit multiplier instead of a target serving count:

```
$ recipe-scale examples/pancakes.txt --factor 1/3
```

Scaled quantities can get awkward (1/3 of 1 7/8 cups is 5/8 cup, but 1/7 of
it is not something you can measure). `--round-to` snaps every quantity to the
nearest multiple of a step, ties going up:

```
$ recipe-scale examples/pancakes.txt --factor 1/3 --round-to 1/8
```

The step is any positive fraction or decimal. A small quantity can round to
0 if the step is coarse, so pick a step that suits the smallest measure in the
recipe. It applies to `--json` output too.

Add `--json` to either mode for machine-readable output:

```
$ recipe-scale examples/pancakes.txt --servings 6 --json
{"name":"Fluffy Pancakes","base_servings":4,"factor":1.500000,"ingredients":[{"quantity":"2 1/4","quantity_decimal":2.2500,"unit":"cups","name":"all-purpose flour"},...]}
```

`quantity` is the exact fraction as a string, formatted the same way as the
human output; `quantity_decimal` is the same value as a float, for callers
that would rather not parse fractions.

## Building

Standard library only, no dependencies to fetch:

```
cargo build --release
```

Run the test suite with `cargo test`.

## Status

Early skeleton. Parsing is strict about the two-line metadata header and
whitespace-separated ingredient lines; see the roadmap for what's next.
