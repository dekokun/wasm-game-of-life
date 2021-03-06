extern crate cfg_if;
extern crate wasm_bindgen;
extern crate js_sys;
extern crate web_sys;

mod utils;

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;

macro_rules! log {
  ($($t:tt)* ) => {
    web_sys::console::log_1(&format!( $($t) * )).into();
  };
}

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

pub struct Timer<'a> {
    name: &'a str,
}

impl<'a> Timer<'a> {
    pub fn new(name: &'a str) -> Timer<'a> {
        web_sys::console::time_with_label(name);
        Timer { name }
    }
}

impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        web_sys::console::time_end_with_label(self.name);
    }
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
  Dead = 0,
  Alive = 1,
}

impl Cell {
  fn toggle(&mut self) {
    *self = match *self {
      Cell::Dead => Cell::Alive,
      Cell::Alive => Cell::Dead,
    };
  }
}

#[wasm_bindgen]
pub struct Universe {
  width: u32,
  height: u32,
  cells: Vec<Cell>,
}

#[wasm_bindgen]
impl Universe {
  pub fn tick(&mut self) {
    let _timer = Timer::new("Universe::tick");
    let mut next = self.cells.clone();
    for row in 0..self.height {
      for col in 0..self.width {
        let idx = self.get_index(row, col);
        let cell = self.cells[idx];
        let live_neighbors = self.live_neighbor_count(row, col);

        let next_cell = match (cell, live_neighbors) {
          (Cell::Alive, x) if x < 2 => Cell::Dead,
          (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
          (Cell::Alive, x) if x > 3 => Cell::Dead,
          (Cell::Dead, 3) => Cell::Alive,
          (otherwise, _) => otherwise,
        };
        next[idx] = next_cell;
      }
    }
    self.cells = next;
  }
  pub fn new() -> Universe {
    let width = 128;
    let height = 128;

    let cells = (0..width * height)
    .map(|_| {
      if js_sys::Math::random() < 0.5 {
        Cell::Alive
      } else {
        Cell::Dead
      }
    }).collect();
    Universe {
      width,
      height,
      cells,
    }
  }

  pub fn render(&self) -> String {
    self.to_string()
  }

  pub fn width(&self) -> u32 {
    self.width
  }
  pub fn height(&self) -> u32 {
    self.height
  }
  pub fn cells(&self) -> *const Cell {
    self.cells.as_ptr()
  }
  pub fn toggle_cell(&mut self, row: u32, column: u32) {
    let idx = self.get_index(row, column);
    self.cells[idx].toggle();
  }

  pub fn all_kill(&mut self) {
    let cells = (0..self.width * self.height)
    .map(|_| {
      Cell::Dead
    }).collect();
    self.cells = cells;
  }

  pub fn reset(&mut self) {
    let cells = (0..self.width * self.height)
    .map(|_| {
      if js_sys::Math::random() < 0.5 {
        Cell::Alive
      } else {
        Cell::Dead
      }
    }).collect();
    self.cells = cells;
  }

  pub fn insert_glider(&mut self, row: u32, column: u32) {
    let neighbor_indexes = self.neighbor_indexs(row, column);
    let center_idx = self.get_index(row, column);
    self.cells[center_idx] = Cell::Dead;
    let neighbor_values = [Cell::Dead, Cell::Alive, Cell::Dead, Cell::Dead, Cell::Alive, Cell::Alive, Cell::Alive, Cell::Alive];
    for (i, item) in neighbor_indexes.iter().enumerate() {
      self.cells[*item] = neighbor_values[i];
    }
  }
}
impl Universe {
  fn get_index(&self, row: u32, column: u32) ->usize {
    (row * self.width + column) as usize
  }
  fn neighbor_indexs(&self, row: u32, column: u32) -> [usize; 8] {
    let mut neighbors = [0; 8];
    let mut neighbor_index = 0;
    for delta_row in [self.height - 1, 0, 1].iter().cloned() {
      for delta_col in [self.width - 1, 0, 1].iter().cloned() {
        if delta_row == 0 && delta_col == 0 {
          continue;
        }
        let neighbor_row = (row + delta_row) % self.height;
        let neighbor_col = (column + delta_col) % self.width;
        let idx = self.get_index(neighbor_row, neighbor_col);
        neighbors[neighbor_index] = idx;
        neighbor_index += 1;
      }
    }
    neighbors
  }
  fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
    let mut count = 0;
    for idx in &self.neighbor_indexs(row, column) {
        count += self.cells[*idx] as u8;
    }
    count
  }
}

use std::fmt;
impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == Cell::Dead { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}
