use rand::Rng;

pub const GRID_SIZE: usize = 8;

#[derive(Debug, Clone, Copy)]
pub enum Shape {
	// original tetris shapes
	OrangeRicky,  // ▄▄█
	BlueRicky,    // █▄▄
	ClevelandZ,   // ▀█▄
	RhodeIslandZ, // ▄█▀
	Hero,         // ▄▄▄▄
	Teewee,       // ▄█▄
	Smashboy,     // ██

	// additional shapes
	// █
	// █▄
	OrganeRickyUp,
	// █▀
	// █
	BlueRickyUp,
	// ▄█
	// █
	ClevelandZUp,
	// █▄
	//  █
	RhodeIslandZUp,
	// █
	// █
	// █
	// █
	HeroUp,
	LongHero, // ▄▄▄▄▄
	// █
	// █
	// █
	// █
	// █
	LongHeroUp,
	ShortHero, // ▄▄▄
	// █
	// █
	// █
	ShortHeroUp,
	Duce, // ▄▄
	// █
	// █
	DuceUp,
	Single,     // ▄
	TeeweeDown, // ▀█▀
	// █
	// ██
	// █
	TeeweeRight,
	//  █
	// ██
	//  █
	TeeweeLeft,
	LongSmashboy, // ███
	// ██
	// ██
	// ██
	LongSmashboyUp,
	// ███
	// ███
	// ███
	Huge,
}

impl Shape {
	pub fn get_anchors(&self) -> &'static [(i32, i32)] {
		match self {
			Shape::OrangeRicky => &[(0, 1), (1, 1), (2, 0), (2, 1)],
			Shape::BlueRicky => &[(0, 0), (0, 1), (1, 1), (2, 1)],
			Shape::ClevelandZ => &[(0, 0), (1, 0), (1, 1), (2, 1)],
			Shape::RhodeIslandZ => &[(0, 1), (1, 0), (1, 1), (2, 0)],
			Shape::Hero => &[(0, 0), (1, 0), (2, 0), (3, 0)],
			Shape::Teewee => &[(0, 1), (1, 0), (1, 1), (2, 1)],
			Shape::Smashboy => &[(0, 0), (0, 1), (1, 0), (1, 1)],
			Shape::OrganeRickyUp => &[(0, 0), (0, 1), (0, 2), (1, 2)],
			Shape::BlueRickyUp => &[(0, 0), (0, 1), (0, 2), (1, 0)],
			Shape::ClevelandZUp => &[(1, 0), (1, 1), (0, 1), (0, 2)],
			Shape::RhodeIslandZUp => &[(0, 0), (0, 1), (1, 1), (1, 2)],
			Shape::HeroUp => &[(0, 0), (0, 1), (0, 2), (0, 3)],
			Shape::LongHero => &[(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)],
			Shape::LongHeroUp => &[(0, 0), (0, 1), (0, 2), (0, 3), (0, 4)],
			Shape::ShortHero => &[(0, 0), (1, 0), (2, 0)],
			Shape::ShortHeroUp => &[(0, 0), (0, 1), (0, 2)],
			Shape::Duce => &[(0, 0), (1, 0)],
			Shape::DuceUp => &[(0, 0), (0, 1)],
			Shape::Single => &[(0, 0)],
			Shape::TeeweeDown => &[(0, 0), (1, 0), (1, 1), (2, 0)],
			Shape::TeeweeRight => &[(0, 0), (0, 1), (1, 1), (0, 2)],
			Shape::TeeweeLeft => &[(1, 0), (0, 1), (1, 1), (1, 2)],
			Shape::LongSmashboy => &[(0, 0), (0, 1), (1, 0), (1, 1), (2, 0), (2, 1)],
			Shape::LongSmashboyUp => &[(0, 0), (1, 0), (0, 1), (1, 1), (0, 2), (1, 2)],
			Shape::Huge => &[(0, 0), (0, 1), (0, 2), (1, 0), (1, 1), (1, 2), (2, 0), (2, 1), (2, 2)],
		}
	}
}

#[derive(Clone, Copy, Debug)]
pub struct Piece {
	pub shape: Shape,
	pub used: bool,
}

impl Piece {
	fn random() -> Self {
		const VARIANTS: &[Shape] = &[
			Shape::OrangeRicky,
			Shape::BlueRicky,
			Shape::ClevelandZ,
			Shape::RhodeIslandZ,
			Shape::Hero,
			Shape::Teewee,
			Shape::Smashboy,
			Shape::OrganeRickyUp,
			Shape::BlueRickyUp,
			Shape::ClevelandZUp,
			Shape::RhodeIslandZUp,
			Shape::HeroUp,
			Shape::LongHero,
			Shape::LongHeroUp,
			Shape::ShortHero,
			Shape::ShortHeroUp,
			Shape::Duce,
			Shape::DuceUp,
			Shape::Single,
			Shape::TeeweeDown,
			Shape::TeeweeRight,
			Shape::TeeweeLeft,
			Shape::LongSmashboy,
			Shape::LongSmashboyUp,
			Shape::Huge,
		];

		let idx = rand::rng().random_range(0..VARIANTS.len());
		Self {
			shape: VARIANTS[idx],
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

		for &(dx, dy) in piece.shape.get_anchors() {
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
			.shape
			.get_anchors()
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

		for &(dx, dy) in piece.shape.get_anchors() {
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

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn new_test() {
		let game = KoalaKombo::new();
		assert_eq!(game.score, 0);
		assert_eq!(game.board.len(), GRID_SIZE * GRID_SIZE);
		assert_eq!(game.pieces.len(), 3);
	}
}
