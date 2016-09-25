# rust-term-grid [![term_grid on crates.io](http://meritbadge.herokuapp.com/term_grid)](https://crates.io/crates/term_grid) [![Build status](https://travis-ci.org/ogham/rust-term-grid.svg?branch=master)](https://travis-ci.org/ogham/rust-term-grid)

This library arranges textual data in a grid format suitable for fixed-width fonts, using an algorithm to minimise the amount of space needed.

### [View the Rustdoc](https://docs.rs/term_grid)


# Installation

This crate works with [Cargo](http://crates.io). Add the following to your `Cargo.toml` dependencies section:

```toml
[dependencies]
term_grid = "0.1"
```


## Usage

This library arranges textual data in a grid format suitable for fixed-width fonts, using an algorithm to minimise the amount of space needed.
For example:

```rust
use term_grid::{Grid, GridOptions, Direction, Filling};
```

```rust
let mut grid = Grid::new(GridOptions {
    filling:     Filling::Spaces(1),
    direction:   Direction::LeftToRight,
});
```

```rust
for s in vec!["one", "two", "three", "four", "five", "six", "seven",
              "eight", "nine", "ten", "eleven", "twelve"] {
    grid.add(s.into());
}
```

```rust
println!("{}", grid.fit_into_width(24).unwrap());
```

Produces the following tabular result:

```rust
one  two three  four
five six seven  eight
nine ten eleven twelve
```


## Creating a grid

To add data to a grid, first create a new `Grid` object, and then add cells to them with the `add` method.

There are two options that must be specified in the `GridOptions` object that dictate how the grid is formatted:

- `filling`: what to put in between two columns - either a number of spaces, or a text string;
- `direction`, which specifies whether the cells should go along rows, or columns:
    - `Direction::LeftToRight` starts them in the top left and moves *rightwards*, going to the start of a new row after reaching the final column;
    - `Direction::TopToBottom` starts them in the top left and moves *downwards*, going to the top of a new column after reaching the final row.


## Displaying a grid

When display a grid, you can either specify the number of columns in advance, or try to find the maximum number of columns that can fit in an area of a given width.

Splitting a series of cells into columns - or, in other words, starting a new row every *n* cells - is achieved with the `fit_into_columns` method on a `Grid` value.
It takes as its argument the number of columns.

Trying to fit as much data onto one screen as possible is the main use case for specifying a maximum width instead.
This is achieved with the `fit_into_width` method.
It takes the maximum allowed width, including separators, as its argument.
However, it returns an *optional* `Display` value, depending on whether any of the cells actually had a width greater than the maximum width!
If this is the case, your best bet is to just output the cells with one per line.


## Cells and data

Grids to not take `String`s or `&str`s - they take `Cells`.

A **Cell** is a struct containing an individual cell’s contents, as a string, and its pre-computed length, which gets used when calculating a grid’s final dimensions.
Usually, you want the *Unicode width* of the string to be used for this, so you can turn a `String` into a `Cell` with the `.into()` method.

However, you may also want to supply your own width: when you already know the width in advance, or when you want to change the measurement, such as skipping over terminal control characters.
For cases like these, the fields on the `Cell` objects are public, meaning you can construct your own instances as necessary.
