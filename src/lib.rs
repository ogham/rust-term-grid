#![crate_name = "term_grid"]
#![crate_type = "rlib"]
#![crate_type = "dylib"]

#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]

//! This is a library for formatting strings in a grid layout, for a terminal
//! or anywhere that uses a fixed-width font.

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

    pub fn fit_into_width<'grid>(&'grid self, maximum_width: Width) -> Option<Display<'grid>> {
        self.width_dimensions(maximum_width).map(|dims| Display { grid: &self, dimensions: dims })
    }

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