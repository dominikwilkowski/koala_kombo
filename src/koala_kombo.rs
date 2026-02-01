use rand::Rng;

pub const GRID_SIZE: usize = 8;
pub const CELL_PX: f32 = 80.0;
pub const GAP_PX: f32 = 4.0;

// The pieces that can spawn
const PIECES: [&[(i32, i32)]; 9] = [
	&[(0, 0)],                                 // Single block
	&[(0, 0), (1, 0)],                         // Horizontal 2
	&[(0, 0), (0, 1)],                         // Vertical 2
	&[(0, 0), (1, 0), (2, 0)],                 // Horizontal 3
	&[(0, 0), (0, 1), (0, 2)],                 // Vertical 3
	&[(0, 0), (1, 0), (0, 1), (1, 1)],         // 2x2 square
	&[(0, 0), (1, 0), (1, 1)],                 // L shape
	&[(0, 0), (1, 0), (2, 0), (1, 1)],         // T shape
	&[(0, 0), (1, 0), (2, 0), (0, 1), (0, 2)], // Big L
];

#[derive(Clone, Copy, Debug)]
pub struct Cell {
	pub filled: bool,
}

#[derive(Clone, Debug)]
pub struct Shape {
	pub blocks: &'static [(i32, i32)],
}

#[derive(Debug)]
pub struct KoalaKombo {
	pub board: Vec<Cell>,
	pub available_pieces: [Shape; 3],
	pub selected_piece: Option<usize>,
	pub score: u32,
}

impl KoalaKombo {
	pub fn new() -> Self {
		Self {
			board: vec![Cell { filled: false }; GRID_SIZE * GRID_SIZE],
			available_pieces: [
				Shape { blocks: PIECES[0] },
				Shape { blocks: PIECES[1] },
				Shape { blocks: PIECES[2] },
			],
			selected_piece: None,
			score: 0,
		}
	}

	pub fn idx(x: usize, y: usize) -> usize {
		y * GRID_SIZE + x
	}

	fn in_bounds(x: i32, y: i32) -> bool {
		x >= 0 && y >= 0 && (x as usize) < GRID_SIZE && (y as usize) < GRID_SIZE
	}

	pub fn can_place(&self, shape: &Shape, anchor_x: usize, anchor_y: usize) -> bool {
		let ax = anchor_x as i32;
		let ay = anchor_y as i32;

		for (dx, dy) in shape.blocks {
			let x = ax + dx;
			let y = ay + dy;

			if !Self::in_bounds(x, y) {
				return false;
			}

			let idx = Self::idx(x as usize, y as usize);
			if self.board[idx].filled {
				return false;
			}
		}
		true
	}

	pub fn place(&mut self, shape: &Shape, anchor_x: usize, anchor_y: usize) {
		let ax = anchor_x as i32;
		let ay = anchor_y as i32;

		for (dx, dy) in shape.blocks {
			let x = (ax + dx) as usize;
			let y = (ay + dy) as usize;
			let idx = Self::idx(x, y);
			self.board[idx].filled = true;
		}
	}

	pub fn clear_complete_lines(&mut self) -> u32 {
		let mut score = 0;

		// Check rows
		for y in 0..GRID_SIZE {
			let row_start = y * GRID_SIZE;
			if self.board[row_start..row_start + GRID_SIZE].iter().all(|cell| cell.filled) {
				for x in 0..GRID_SIZE {
					self.board[Self::idx(x, y)].filled = false;
				}
				score += GRID_SIZE as u32;
			}
		}

		// Check columns
		for x in 0..GRID_SIZE {
			if (0..GRID_SIZE).all(|y| self.board[Self::idx(x, y)].filled) {
				for y in 0..GRID_SIZE {
					self.board[Self::idx(x, y)].filled = false;
				}
				score += GRID_SIZE as u32;
			}
		}

		score
	}

	pub fn generate_new_pieces(&mut self) {
		let mut rng = rand::rng();
		let idx1 = rng.random_range(0..PIECES.len());
		let idx2 = rng.random_range(0..PIECES.len());
		let idx3 = rng.random_range(0..PIECES.len());

		self.available_pieces = [
			Shape { blocks: PIECES[idx1] },
			Shape { blocks: PIECES[idx2] },
			Shape { blocks: PIECES[idx3] },
		];
	}
}
