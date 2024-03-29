//! A simple generic heap-allocated 2D grid struct.

pub mod iterators;

use crate::vector::Vector;

use std::{
    fmt,
    ops::{Index, IndexMut},
};

/// A simple generic heap-allocated 2D grid struct indexed by `Vector`.
///
/// For a position `Vector { x, y }` in the grid:
/// * `x` determines which column the position is in
/// * `y` determines which row the position is in
///
/// There are `width` columns and `height` rows in the grid, and the grid's iterators traverse it in row-major order.
///
/// `Grid<T>` implements the [`Debug`] trait if `T` implements the [`std::fmt::Display`] trait.
///
/// # Examples
///
/// ```
/// use grid::prelude::*;
///
/// let mut grid: Grid<u8> = Grid::new(8, 10, 3);
///
/// grid[v(1, 0)] = 1;
/// grid[v(3, 5)] = 2;
///
/// assert_eq!(grid[v(3, 5)], 2);
/// assert_eq!(grid[v(1, 0)], 1);
/// assert_eq!(grid[v(6, 4)], 3);
///
/// println!("{:?}", grid);
/// ```
#[derive(PartialEq, Eq, Clone, Default, Hash)]
pub struct Grid<T> {
    raw: Vec<T>,
    dim: Vector,
}

impl<T: Clone> Grid<T> {
    /// Constructs a new `Grid<T>` with the given dimensions, initialising all values to `value`.
    ///
    /// Requires that `T` implements the [`Clone`] trait.
    ///
    /// Panics if the dimensions are not positive or too large.
    ///
    /// # Examples
    ///
    /// ```
    /// use grid::prelude::*;
    ///
    /// let grid: Grid<u8> = Grid::new(8, 10, 1);
    ///
    /// assert_eq!(grid[v(2, 4)], 1);
    /// assert_eq!(grid[v(7, 3)], 1);
    /// ```
    #[track_caller]
    pub fn new(width: i64, height: i64, value: T) -> Self {
        let size = size(width, height);
        let mut raw = Vec::with_capacity(size);
        raw.resize(size, value);
        Self {
            raw,
            dim: Vector::new(width, height),
        }
    }
}

impl<T: Default> Grid<T> {
    /// Constructs a new `Grid<T>` with the given dimensions, initialising all values to their default value.
    ///
    /// Requires that `T` implements the [`Default`] trait.
    ///
    /// Panics if the dimensions are not positive or too large.
    ///
    /// # Examples
    ///
    /// ```
    /// use grid::prelude::*;
    ///
    /// let grid: Grid<u8> = Grid::default(9, 3);
    ///
    /// assert_eq!(grid[v(5, 1)], 0);
    /// assert_eq!(grid[v(6, 2)], 0);
    /// ```
    #[track_caller]
    pub fn default(width: i64, height: i64) -> Self {
        let size = size(width, height);
        let mut raw = Vec::with_capacity(size);
        raw.resize_with(size, Default::default);
        Self {
            raw,
            dim: Vector::new(width, height),
        }
    }
}

impl<T> Grid<T> {
    /// Constructs a new `Grid<T>` with the given dimensions, computing all initial values from the closure `f`.
    ///
    /// Panics if the dimensions are not positive or too large.
    ///
    /// # Examples
    ///
    /// ```
    /// use grid::prelude::*;
    ///
    /// let grid: Grid<u8> = Grid::from_simple_fn(8, 10, || 2);
    ///
    /// assert_eq!(grid[v(5, 3)], 2);
    /// assert_eq!(grid[v(1, 8)], 2);
    /// ```
    #[track_caller]
    pub fn from_simple_fn<F>(width: i64, height: i64, f: F) -> Self
    where
        F: FnMut() -> T,
    {
        let size = size(width, height);
        let mut raw = Vec::with_capacity(size);
        raw.resize_with(size, f);
        Self {
            raw,
            dim: Vector::new(width, height),
        }
    }

    /// Constructs a new `Grid<T>` with the given dimensions, computing all initial values from the closure `f` which maps each position to a value.
    ///
    /// Panics if the dimensions are not positive or too large.
    ///
    /// # Examples
    ///
    /// ```
    /// use grid::prelude::*;
    ///
    /// let grid: Grid<i64> = Grid::from_fn(8, 10, |pos| pos.x + pos.y);
    ///
    /// assert_eq!(grid[v(5, 3)], 8);
    /// assert_eq!(grid[v(7, 9)], 16);
    /// ```
    #[track_caller]
    pub fn from_fn<F>(width: i64, height: i64, mut f: F) -> Self
    where
        F: FnMut(Vector) -> T,
    {
        let mut raw = Vec::with_capacity(size(width, height));
        for y in 0..height {
            for x in 0..width {
                raw.push(f(Vector::new(x, y)));
            }
        }
        Self {
            raw,
            dim: Vector::new(width, height),
        }
    }

    /// Constructs a new `Grid<T>` with the given dimensions and values computed by an iterator in row-major order.
    ///
    /// Panics if the dimensions are not positive or too large.
    ///
    /// Panics if the iterator runs out before the grid is filled.
    ///
    /// # Examples
    ///
    /// ```
    /// use grid::prelude::*;
    ///
    /// let grid: Grid<u8> = Grid::from_iter(2, 3, [1, 2, 3, 4, 5, 6]);
    ///
    /// assert_eq!(grid[v(1, 0)], 2);
    /// assert_eq!(grid[v(0, 2)], 5);
    /// assert_eq!(grid[v(1, 2)], 6);
    /// ```
    #[track_caller]
    pub fn from_iter<I>(width: i64, height: i64, values: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let size = size(width, height);
        let mut raw = Vec::with_capacity(size);
        let mut values = values.into_iter();
        for _ in 0..size {
            raw.push(values.next().expect("iterator too short"));
        }
        Self {
            raw,
            dim: Vector::new(width, height),
        }
    }

    /// Constructs a new `Grid<T>` from an iterator of iterators, where each inner iterator defines a row.
    ///
    /// Panics if not all inner iterators are the same length.
    ///
    /// # Examples
    ///
    /// ```
    /// use grid::prelude::*;
    ///
    /// let values = [
    ///     [1, 2, 3],
    ///     [4, 5, 6],
    /// ];
    ///
    /// let grid: Grid<u8> = Grid::from_nested_iter(values);
    ///
    /// assert_eq!(grid.width(), 3);
    /// assert_eq!(grid.height(), 2);
    /// assert_eq!(grid[v(2, 1)], 6);
    /// ```
    #[track_caller]
    pub fn from_nested_iter<I, J>(values: I) -> Self
    where
        I: IntoIterator<Item = J>,
        J: IntoIterator<Item = T>,
    {
        let mut values = values.into_iter();
        let mut raw = Vec::new();
        let Some(first) = values.next() else {
            return Self {
                raw,
                dim: Vector::new(0, 0),
            };
        };
        for value in first.into_iter() {
            raw.push(value);
        }
        let width = raw.len() as i64;
        let mut height = 1;
        for inner in values {
            height += 1;
            let mut count = 0;
            for value in inner.into_iter() {
                count += 1;
                raw.push(value);
            }
            if count != width {
                panic!("not all inner iterators are the same length");
            }
        }
        Self {
            raw,
            dim: Vector::new(width, height),
        }
    }

    /// Returns the width of the grid.
    ///
    /// # Examples
    ///
    /// ```
    /// use grid::prelude::*;
    ///
    /// let grid: Grid<u8> = Grid::new(8, 10, 9);
    ///
    /// assert_eq!(grid.width(), 8);
    /// ```
    #[inline]
    pub fn width(&self) -> i64 {
        self.dim.x
    }

    /// Returns the height of the grid.
    ///
    /// # Examples
    ///
    /// ```
    /// use grid::prelude::*;
    ///
    /// let grid: Grid<u8> = Grid::new(8, 10, 10);
    ///
    /// assert_eq!(grid.height(), 10);
    /// ```
    #[inline]
    pub fn height(&self) -> i64 {
        self.dim.y
    }

    /// Returns the dimensions of the grid.
    ///
    /// # Examples
    ///
    /// ```
    /// use grid::prelude::*;
    ///
    /// let grid: Grid<u8> = Grid::new(8, 10, 11);
    ///
    /// assert_eq!(grid.dim(), v(8, 10));
    /// ```
    #[inline]
    pub fn dim(&self) -> Vector {
        self.dim
    }

    /// Returns a reference to the value at the given position of the grid, or `None` if out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use grid::prelude::*;
    ///
    /// let mut grid: Grid<u8> = Grid::new(8, 10, 3);
    ///
    /// grid[v(1, 1)] = 4;
    ///
    /// assert_eq!(grid.get(v(5, 2)), Some(&3));
    /// assert_eq!(grid.get(v(1, 1)), Some(&4));
    /// assert_eq!(grid.get(v(8, 6)), None);
    /// assert_eq!(grid.get(v(4, 10)), None);
    /// assert_eq!(grid.get(v(-2, 3)), None);
    /// ```
    pub fn get(&self, pos: Vector) -> Option<&T> {
        Some(&self.raw[self.get_index(pos)?])
    }

    /// Returns a mutable reference to the value at the given position of the grid, or `None` if out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use grid::prelude::*;
    ///
    /// let mut grid: Grid<u8> = Grid::new(8, 10, 4);
    ///
    /// grid[v(5, 3)] = 2;
    ///
    /// assert_eq!(grid.get_mut(v(5, 3)), Some(&mut 2));
    /// assert_eq!(grid.get_mut(v(0, 0)), Some(&mut 4));
    /// assert_eq!(grid.get_mut(v(1, 10)), None);
    /// assert_eq!(grid.get_mut(v(9, 7)), None);
    /// assert_eq!(grid.get_mut(v(4, -1)), None);
    /// ```
    pub fn get_mut(&mut self, pos: Vector) -> Option<&mut T> {
        let index = self.get_index(pos)?;
        Some(&mut self.raw[index])
    }

    /// Sets the value at the given position of the grid.
    ///
    /// Returns the old value at that position, or `None` if out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use grid::prelude::*;
    ///
    /// let mut grid: Grid<u8> = Grid::new(8, 10, 5);
    ///
    /// assert_eq!(grid.set(v(2, 3), 7), Some(5));
    /// assert_eq!(grid.set(v(9, 12), 1), None);
    /// assert_eq!(grid.set(v(-4, -7), 3), None);
    ///
    /// assert_eq!(grid[v(2, 3)], 7);
    /// ```
    pub fn set(&mut self, pos: Vector, value: T) -> Option<T> {
        Some(std::mem::replace(self.get_mut(pos)?, value))
    }

    /// Returns `true` if the given position is within the bounds of the grid, or `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use grid::prelude::*;
    ///
    /// let grid: Grid<u8> = Grid::new(15, 14, 11);
    ///
    /// assert_eq!(grid.in_bounds(v(0, 0)), true);
    /// assert_eq!(grid.in_bounds(v(10, 4)), true);
    /// assert_eq!(grid.in_bounds(v(15, 2)), false);
    /// assert_eq!(grid.in_bounds(v(3, 17)), false);
    /// assert_eq!(grid.in_bounds(v(-1, 5)), false);
    /// assert_eq!(grid.in_bounds(v(-15, -14)), false);
    /// ```
    pub fn in_bounds(&self, pos: Vector) -> bool {
        (0..self.width()).contains(&pos.x) && (0..self.height()).contains(&pos.y)
    }

    fn get_index(&self, pos: Vector) -> Option<usize> {
        self.in_bounds(pos)
            .then(|| pos.x as usize + ((pos.y as usize) * (self.width() as usize)))
    }

    /// Maps the values of an existing grid to create a new grid with the same dimensions.
    ///
    /// # Examples
    ///
    /// ```
    /// use grid::prelude::*;
    ///
    /// let grid_a: Grid<u8> = Grid::new(15, 14, 11);
    ///
    /// let grid_b = grid_a.map(|value| *value + 2);
    ///
    /// assert_eq!(grid_b[v(2, 3)], 13);
    ///
    /// let grid_c = grid_b.map(ToString::to_string);
    ///
    /// assert_eq!(&grid_c[v(2, 3)], "13");
    /// ```
    pub fn map<F, U>(&self, mut f: F) -> Grid<U>
    where
        F: FnMut(&T) -> U,
    {
        let mut raw = Vec::with_capacity(self.raw.len());
        for value in self {
            raw.push(f(value));
        }
        Grid { raw, dim: self.dim }
    }

    /// Maps the values and positions of an existing grid to create a new grid with the same dimensions.
    ///
    /// # Examples
    ///
    /// ```
    /// use grid::prelude::*;
    ///
    /// let grid_a: Grid<i64> = Grid::new(5, 6, 3);
    ///
    /// let grid_b = grid_a.pos_map(|pos, value| *value + pos.x);
    ///
    /// assert_eq!(grid_b[v(1, 4)], 4);
    /// assert_eq!(grid_b[v(3, 0)], 6);
    ///
    /// let grid_c = grid_b.pos_map(|pos, value| *value + pos.y);
    ///
    /// assert_eq!(grid_c[v(1, 4)], 8);
    /// ```
    pub fn pos_map<F, U>(&self, mut f: F) -> Grid<U>
    where
        F: FnMut(Vector, &T) -> U,
    {
        let mut raw = Vec::with_capacity(self.raw.len());
        for (pos, value) in self.iter_positions() {
            raw.push(f(pos, value));
        }
        Grid { raw, dim: self.dim }
    }

    /// Maps the values of an existing grid to create a new grid with the same dimensions.
    ///
    /// Consumes `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use grid::prelude::*;
    ///
    /// let grid_a: Grid<u8> = Grid::new(15, 14, 11);
    ///
    /// let grid_b = grid_a.map_into(|value| value + 2);
    ///
    /// assert_eq!(grid_b[v(2, 3)], 13);
    /// ```
    pub fn map_into<F, U>(self, mut f: F) -> Grid<U>
    where
        F: FnMut(T) -> U,
    {
        let mut raw = Vec::with_capacity(self.raw.len());
        let dim = self.dim;
        for value in self {
            raw.push(f(value));
        }
        Grid { raw, dim }
    }

    /// Maps the values and positions of an existing grid to create a new grid with the same dimensions.
    ///
    /// Consumes `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use grid::prelude::*;
    ///
    /// let grid_a: Grid<i64> = Grid::new(5, 6, 3);
    ///
    /// let grid_b = grid_a.pos_map_into(|pos, value| value + pos.x);
    ///
    /// assert_eq!(grid_b[v(1, 4)], 4);
    /// assert_eq!(grid_b[v(3, 0)], 6);
    /// ```
    pub fn pos_map_into<F, U>(self, mut f: F) -> Grid<U>
    where
        F: FnMut(Vector, T) -> U,
    {
        let mut raw = Vec::with_capacity(self.raw.len());
        let dim = self.dim;
        for (pos, value) in self.into_iter_positions() {
            raw.push(f(pos, value));
        }
        Grid { raw, dim }
    }
}

impl<T> Index<Vector> for Grid<T> {
    type Output = T;

    #[track_caller]
    fn index(&self, pos: Vector) -> &Self::Output {
        let dim = self.dim;
        if let Some(r) = self.get(pos) {
            return r;
        }
        panic!("position out of bounds: the dimensions are {dim} but the position is {pos}")
    }
}

impl<T> IndexMut<Vector> for Grid<T> {
    #[track_caller]
    fn index_mut(&mut self, pos: Vector) -> &mut Self::Output {
        let dim = self.dim;
        if let Some(r) = self.get_mut(pos) {
            return r;
        }
        panic!("position out of bounds: the dimensions are {dim} but the position is {pos}")
    }
}

impl<T: fmt::Display> fmt::Debug for Grid<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let strings = self.map(ToString::to_string);
        let longest = strings.iter().map(String::len).max().unwrap();

        writeln!(f, "{}x{}", self.width(), self.height())?;

        for y in 0..strings.height() {
            for x in 0..strings.width() {
                let s = &strings[Vector::new(x, y)];
                write!(f, "{}{s}", " ".repeat(longest - s.len()))?;
                if x != strings.width() - 1 {
                    write!(f, ",")?;
                }
            }
            if y != strings.height() - 1 {
                writeln!(f)?;
            }
        }

        Ok(())
    }
}

#[track_caller]
fn size(width: i64, height: i64) -> usize {
    if width <= 0 || height <= 0 {
        panic!("dimensions must be positive: ({width}, {height})");
    }
    if let Some(size) = (width as usize).checked_mul(height as usize) {
        return size;
    }
    panic!("dimensions are too large: ({width}, {height})");
}
