# rust-term-grid [![Build Status](https://travis-ci.org/ogham/rust-term-grid.svg?branch=master)](https://travis-ci.org/ogham/rust-term-grid)

This is a library for formatting strings in a grid layout, for a terminal - or anywhere that uses a fixed-width font.

### [View the Rustdoc](http://bsago.me/doc/term_grid/)

## Installation

It uses [Cargo](http://crates.io/), Rust's package manager. You can
depend on this library by adding `term_grid` to your Cargo dependencies:

```toml
[dependencies]
term_grid = "*"
```

Or, to use the Git repo directly:

```toml
[dependencies.term_grid]
git = "https://github.com/ogham/rust-term-grid.git"
```


## Introduction

This library arranges textual data in a grid format suitable for fixed-width fonts, using an algorithm to minimise the amount of space needed. For example:

```rust
let mut grid = Grid::new(GridOptions {
    separator_width:  1,
    direction:        Direction::LeftToRight,
});

for s in vec!["one", "two", "three", "four", "five", "six", "seven",
              "eight", "nine", "ten", "eleven", "twelve"] {
    grid.add(s.into());
}

println!("{}", grid.fit_into_width(24).unwrap());
```

Produces the following tabular result:

    one  two three  four
    five six seven  eight
    nine ten eleven twelve


## Creating a Grid

To add data to a grid, first create a new `Grid` object, and then add cells to them with the `add` method.

There are two options that must be specified in the `GridOptions` object that dictate how the grid is formatted:

- `separator_width`: the number of space characters to be placed between two columns;
- `direction`, which specifies whether the cells should go along rows, or columns:
    - `Direction::LeftToRight` starts them in the top left and moves *rightwards*, going to the start of a new row after reaching the final column;
    - `Direction::TopToBottom` starts them in the top left and moves *downwards*, going to the top of a new column after reaching the final row.


## Displaying a Grid

When display a grid, you can either specify the number of columns in advance, or try to find the maximum number of columns that can fit in an area of a given width.

Splitting a series of cells into columns - or, in other words, starting a new row every *n* cells - is achieved with the `fit_into_columns` method on a `Grid` value. It takes as its argument the number of columns.

Trying to fit as much data onto one screen as possible is the main use case for specifying a maximum width instead. This is achieved with the `fit_into_width` method. It takes the maximum allowed width, including separators, as its argument. However, it returns an *optional* `Display` value, depending on whether any of the cells actually had a width greater than the maximum width! If this is the case, your best bet is to just output the cells with one per line.


## Cells and Data

Grids to not take `String`s or `&str`s - they take `Cells`.

A **Cell** is a struct containing an individual cell's contents, as a string, and its pre-computed length, which gets used when calculating a grid's final dimensions. Usually, you want the *Unicode width* of the string to be used for this, so you can turn a `String` into a `Cell` with the `.into()` method.

However, you may also want to supply your own width: when you already know the width in advance, or when you want to change the measurement, such as skipping over terminal control characters. For cases like these, the fields on the `Cell` objects are public, meaning you can construct your own instances as necessary.