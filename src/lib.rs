#![crate_name = "term_grid"]
#![crate_type = "rlib"]
#![crate_type = "dylib"]

#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]

//! ## Introduction
//!
//! This library arranges textual data in a grid format suitable for
//! fixed-width fonts, using an algorithm to minimise the amount of space
//! needed. For example:
//!
//! ```rust
//! use term_grid::{Grid, GridOptions, Direction};
//!
//! let mut grid = Grid::new(GridOptions {
//!     separator_width: 1,
//!     direction: Direction::LeftToRight,
//! });
//!
//! for s in vec!["one", "two", "three", "four", "five", "six", "seven",
//!               "eight", "nine", "ten", "eleven", "twelve"] {
//!     grid.add(s.into());
//! }
//!
//! println!("{}", grid.fit_into_width(24).unwrap());
//! ```
//!
//! Produces the following tabular result:
//!
//! ```text
//! one  two three  four
//! five six seven  eight
//! nine ten eleven twelve
//! ```
//!
//!
//! ## Creating a Grid
//!
//! To add data to a grid, first create a new `Grid` object, and then add cells to
//! them with the `add` method.
//!
//! There are two options that must be specified in the `GridOptions` object that
//! dictate how the grid is formatted:
//!
//! - `separator_width`: the number of space characters to be placed between two
//!   columns;
//! - `direction`, which specifies whether the cells should go along
//!   rows, or columns:
//!   - `Direction::LeftToRight` starts them in the top left and
//!     moves *rightwards*, going to the start of a new row after reaching the
//!     final column;
//!   - `Direction::TopToBottom` starts them in the top left and moves
//!     *downwards*, going to the top of a new column after reaching the final
//!     row.
//!
//!
//! ## Displaying a Grid
//!
//! When display a grid, you can either specify the number of columns in advance,
//! or try to find the maximum number of columns that can fit in an area of a
//! given width.
//!
//! Splitting a series of cells into columns - or, in other words, starting a new
//! row every *n* cells - is achieved with the `fit_into_columns` method on a
//! `Grid` value. It takes as its argument the number of columns.
//!
//! Trying to fit as much data onto one screen as possible is the main use case
//! for specifying a maximum width instead. This is achieved with the
//! `fit_into_width` method. It takes the maximum allowed width, including
//! separators, as its argument. However, it returns an *optional* `Display`
//! value, depending on whether any of the cells actually had a width greater than
//! the maximum width! If this is the case, your best bet is to just output the
//! cells with one per line.
//!
//!
//! ## Cells and Data
//!
//! Grids to not take `String`s or `&str`s - they take `Cells`.
//!
//! A **Cell** is a struct containing an individual cell's contents, as a string,
//! and its pre-computed length, which gets used when calculating a grid's final
//! dimensions. Usually, you want the *Unicode width* of the string to be used for
//! this, so you can turn a `String` into a `Cell` with the `.into()` method.
//!
//! However, you may also want to supply your own width: when you already know the
//! width in advance, or when you want to change the measurement, such as skipping
//! over terminal control characters. For cases like these, the fields on the
//! `Cell` objects are public, meaning you can construct your own instances as
//! necessary.

use std::cmp::max;
use std::convert;
use std::fmt;
use std::iter::repeat;

extern crate unicode_width;
use unicode_width::UnicodeWidthStr;


/// A **Cell** is the combination of a string and its pre-computed length.
///
/// The easiest way to create a Cell is just by using `string.into()`, which
/// uses the **unicode width** of the string (see the `unicode_width` crate).
/// However, the fields are public, if you wish to provide your own length.
#[derive(PartialEq, Debug)]
pub struct Cell {

    /// The string to display when this cell gets rendered.
    pub contents: String,

    /// The pre-computed length of the string.
    pub width: Width,
}

impl convert::From<String> for Cell {
    fn from(string: String) -> Self {
        Cell {
            width: UnicodeWidthStr::width(&*string),
            contents: string,
        }
    }
}

impl<'_> convert::From<&'_ str> for Cell {
    fn from(string: &'_ str) -> Self {
        Cell {
            width: UnicodeWidthStr::width(&*string),
            contents: string.into(),
        }
    }
}


/// Direction cells should be written in - either across, or downwards.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Direction {

    /// Starts at the top left and moves rightwards, going back to the first
    /// column for a new row - like a typewriter.
    LeftToRight,

    /// Starts at the top left and moves downwards, going back to the first
    /// row for a new column - like how `ls` lists files by default.
    TopToBottom,
}


/// The width of a cell, in columns.
pub type Width = usize;


/// The user-assignable options for a grid view that should be passed into
/// `Grid::new()`.
#[derive(PartialEq, Debug, Copy, Clone)]
pub struct GridOptions {

    /// Direction that the cells should be written in - either across, or
    /// downwards.
    pub direction: Direction,

    /// Number of spaces to put in between each column of cells.
    pub separator_width: Width,
}


#[derive(PartialEq, Debug)]
struct Dimensions {

    /// The number of lines in the grid.
    num_lines: Width,

    /// The width of each column in the grid. The length of this vector serves
    /// as the number of columns.
    widths: Vec<Width>,
}


/// Everything needed to format the cells with the grid options.
///
/// For more information, see the module-level documentation.
#[derive(PartialEq, Debug)]
pub struct Grid {

    /// Options used in constructing the grid.
    options: GridOptions,

    /// Vector of cells
    cells: Vec<Cell>,
}

impl Grid {

    /// Creates a new grid view with the given options.
    pub fn new(options: GridOptions) -> Grid {
        Grid {
            options: options,
            cells: Vec::new(),
        }
    }

    /// Reserves space in the vector for the given number of additional cells
    /// to be added. (See `vec#reserve`)
    pub fn reserve(&mut self, additional: usize) {
        self.cells.reserve(additional)
    }

    /// Adds another cell onto the vector.
    pub fn add(&mut self, cell: Cell) {
        self.cells.push(cell)
    }

    /// Returns a displayable grid that's been packed to fit into the given
    /// width in the fewest number of rows.
    ///
    /// Returns `None` if any of the cells has a width greater than the
    /// maximum width.
    pub fn fit_into_width<'grid>(&'grid self, maximum_width: Width) -> Option<Display<'grid>> {
        self.width_dimensions(maximum_width).map(|dims| Display { grid: &self, dimensions: dims })
    }

    /// Returns a displayable grid with the given number of columns, and no
    /// maximum width.
    pub fn fit_into_columns<'grid>(&'grid self, num_columns: usize) -> Display<'grid> {
        Display { grid: &self, dimensions: self.columns_dimensions(num_columns) }
    }

    fn columns_dimensions(&self, num_columns: usize) -> Dimensions {
        let mut num_lines = self.cells.len() / num_columns;
        if self.cells.len() % num_columns != 0 {
            num_lines += 1;
        }

        self.column_widths(num_lines, num_columns)
    }

    fn column_widths(&self, num_lines: usize, num_columns: usize) -> Dimensions {
        let mut column_widths: Vec<Width> = repeat(0).take(num_columns).collect();
        for (index, cell) in self.cells.iter().enumerate() {
            let index = match self.options.direction {
                Direction::LeftToRight  => index % num_columns,
                Direction::TopToBottom  => index / num_lines,
            };
            column_widths[index] = max(column_widths[index], cell.width);
        }

        Dimensions {
            num_lines: num_lines,
            widths: column_widths,
        }
    }

    fn width_dimensions(&self, maximum_width: Width) -> Option<Dimensions> {

        // TODO: this function could almost certainly be optimised...
        // surely not *all* of the numbers of lines are worth searching through!

        let cell_count = self.cells.len();

        // Instead of numbers of columns, try to find the fewest number of *lines*
        // that the output will fit in.
        for num_lines in 1 .. cell_count {

            // The number of columns is the number of cells divided by the number
            // of lines, *rounded up*.
            let mut num_columns = cell_count / num_lines;
            if cell_count % num_lines != 0 {
                num_columns += 1;
            }

            // Early abort: if there are so many columns that the width of the
            // *column separators* is bigger than the width of the screen, then
            // don't even try to tabulate it.
            // This is actually a necessary check, because the width is stored as
            // a usize, and making it go negative makes it huge instead, but it
            // also serves as a speed-up.
            let total_separator_width = (num_columns - 1) * self.options.separator_width;
            if maximum_width < total_separator_width {
                continue;
            }

            // Remove the separator width from the available space.
            let adjusted_width = maximum_width - total_separator_width;

            let potential_dimensions = self.column_widths(num_lines, num_columns);
            if sum(potential_dimensions.widths.iter().map(|&x| x)) < adjusted_width {
                return Some(potential_dimensions);
            }
        }

        // If you get here you have really wide cells.
        return None;
    }
}

/// A displayable representation of a Grid.
#[derive(PartialEq, Debug)]
pub struct Display<'grid> {
    grid: &'grid Grid,
    dimensions: Dimensions,
}

impl<'grid> fmt::Display for Display<'grid> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        for y in 0 .. self.dimensions.num_lines {
            for x in 0 .. self.dimensions.widths.len() {
                let num = match self.grid.options.direction {
                    Direction::LeftToRight   => y * self.dimensions.widths.len() + x,
                    Direction::TopToBottom   => y + self.dimensions.num_lines * x,
                };

                // Abandon a line mid-way through if that's where the cells end
                if num >= self.grid.cells.len() {
                    continue;
                }

                let ref cell = self.grid.cells[num];
                if x == self.dimensions.widths.len() - 1 {
                    // The final column doesn't need to have trailing spaces
                    try!(write!(f, "{}", cell.contents));
                }
                else {
                    assert!(self.dimensions.widths[x] >= cell.width);
                    let extra_spaces = self.dimensions.widths[x] - cell.width + self.grid.options.separator_width;
                    try!(write!(f, "{}", pad_string(&cell.contents, extra_spaces)));
                }
            }
            try!(write!(f, "\n"));
        }

        Ok(())
	}
}


/// Pad a string with the given number of spaces.
fn spaces(length: usize) -> String {
    repeat(" ").take(length).collect()
}

/// Pad a string with the given alignment and number of spaces.
///
/// This doesn't take the width the string *should* be, rather the number
/// of spaces to add.
fn pad_string(string: &str, padding: usize) -> String {
    format!("{}{}", string, spaces(padding))
}

// TODO: This function can be replaced with `Iterator#sum` once the
// `iter_arith` feature flag cools down.
fn sum<I: Iterator<Item=Width>>(iterator: I) -> Width {
    iterator.fold(0, |s, e| s + e)
}