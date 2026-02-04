use rand::Rng;

pub const GRID_SIZE: usize = 8;
pub const CELL_PX: f32 = 80.0;
pub const GAP_PX: f32 = 4.0;

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
pub struct Piece {
	pub blocks: &'static [(i32, i32)],
	pub used: bool,
}

impl Piece {
	fn random() -> Self {
		let idx = rand::rng().random_range(0..PIECES.len());
		Self {
			blocks: PIECES[idx],
			used: false,
		}
	}
}

#[derive(Debug)]
pub struct KoalaKombo {
	board: [bool; GRID_SIZE * GRID_SIZE],
	pub pieces: [Piece; 3],
	pub score: u32,
}

impl KoalaKombo {
	pub fn new() -> Self {
		Self {
			board: [false; GRID_SIZE * GRID_SIZE],
			pieces: [Piece::random(), Piece::random(), Piece::random()],
			score: 0,
		}
	}

	pub fn cell_filled(&self, x: usize, y: usize) -> bool {
		self.board[y * GRID_SIZE + x]
	}

	/// Check if a piece can be placed at the given anchor position.
	pub fn can_place(&self, piece_idx: usize, anchor_x: usize, anchor_y: usize) -> bool {
		let piece = &self.pieces[piece_idx];
		if piece.used {
			return false;
		}

		let ax = anchor_x as i32;
		let ay = anchor_y as i32;

		for &(dx, dy) in piece.blocks {
			let x = ax + dx;
			let y = ay + dy;

			if x < 0 || y < 0 || x >= GRID_SIZE as i32 || y >= GRID_SIZE as i32 {
				return false;
			}

			if self.board[y as usize * GRID_SIZE + x as usize] {
				return false;
			}
		}
		true
	}

	/// Get the cells that would be occupied if placing a piece at the anchor.
	/// Returns empty Vec if out of bounds.
	pub fn preview_cells(&self, piece_idx: usize, anchor_x: usize, anchor_y: usize) -> Vec<usize> {
		let piece = &self.pieces[piece_idx];
		let ax = anchor_x as i32;
		let ay = anchor_y as i32;

		piece
			.blocks
			.iter()
			.filter_map(|&(dx, dy)| {
				let x = ax + dx;
				let y = ay + dy;
				if x >= 0 && y >= 0 && x < GRID_SIZE as i32 && y < GRID_SIZE as i32 {
					Some(y as usize * GRID_SIZE + x as usize)
				} else {
					None
				}
			})
			.collect()
	}

	/// Place a piece on the board. Returns true if successful.
	/// Handles: placement, marking used, clearing lines, score, and regenerating pieces.
	pub fn place_shape(&mut self, piece_idx: usize, anchor_x: usize, anchor_y: usize) -> bool {
		if !self.can_place(piece_idx, anchor_x, anchor_y) {
			return false;
		}

		// Place the blocks
		let piece = &self.pieces[piece_idx];
		let ax = anchor_x as i32;
		let ay = anchor_y as i32;

		for &(dx, dy) in piece.blocks {
			let x = (ax + dx) as usize;
			let y = (ay + dy) as usize;
			self.board[y * GRID_SIZE + x] = true;
		}

		// Mark piece as used
		self.pieces[piece_idx].used = true;

		// Clear complete lines and update score
		self.score += self.clear_lines();

		// Regenerate pieces if all used
		if self.pieces.iter().all(|p| p.used) {
			self.pieces = [Piece::random(), Piece::random(), Piece::random()];
		}

		true
	}

	fn clear_lines(&mut self) -> u32 {
		let mut score = 0;

		// Check rows
		for y in 0..GRID_SIZE {
			let start = y * GRID_SIZE;
			if self.board[start..start + GRID_SIZE].iter().all(|&filled| filled) {
				for x in 0..GRID_SIZE {
					self.board[y * GRID_SIZE + x] = false;
				}
				score += GRID_SIZE as u32;
			}
		}

		// Check columns
		for x in 0..GRID_SIZE {
			if (0..GRID_SIZE).all(|y| self.board[y * GRID_SIZE + x]) {
				for y in 0..GRID_SIZE {
					self.board[y * GRID_SIZE + x] = false;
				}
				score += GRID_SIZE as u32;
			}
		}

		score
	}
}
