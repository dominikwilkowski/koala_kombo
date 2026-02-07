use rand::Rng;

pub const GRID_SIZE: usize = 8;

/// Creates a `&'static [Coord]` from an ASCII grid.
/// Use `x` for filled cells and `.` for empty cells.
/// Wrap each row in `[]`.
///
/// ```skip
/// shape![
///     [. . x]
///     [x x x]
/// ],
/// ```
macro_rules! shape {
	// Entry: count x's for array size, then flatten rows into @build
	($([ $($cell:tt)+ ])+) => {{
		const SHAPE: [Coord; 0 $($(+ shape!(@is_x $cell))+)+] =
			shape!(@build [] ; 0 ; 0 ; $($($cell)+ ;)+);
		&SHAPE
	}};

	(@is_x x) => { 1 };
	(@is_x .) => { 0 };

	// Done: no more tokens
	(@build [$($coords:expr),*] ; $row:expr ; $col:expr ;) => {
		[$($coords),*]
	};

	// `x` cell
	(@build [$($coords:expr),*] ; $row:expr ; $col:expr ; x $($rest:tt)*) => {
		shape!(@build [$($coords,)* Coord::new($col, $row)] ; $row ; ($col + 1) ; $($rest)*)
	};

	// `.` cell
	(@build [$($coords:expr),*] ; $row:expr ; $col:expr ; . $($rest:tt)*) => {
		shape!(@build [$($coords),*] ; $row ; ($col + 1) ; $($rest)*)
	};

	// `; ;` = end of one row's cells hitting the next row's start `;` — reset column, bump row
	(@build [$($coords:expr),*] ; $row:expr ; $col:expr ; ; $($rest:tt)*) => {
		shape!(@build [$($coords),*] ; ($row + 1) ; 0 ; $($rest)*)
	};
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coord {
	pub column: usize,
	pub row: usize,
}

impl Coord {
	pub const fn new(column: usize, row: usize) -> Self {
		Self { column, row }
	}

	pub fn from_index(idx: usize) -> Self {
		Self {
			column: idx % GRID_SIZE,
			row: idx / GRID_SIZE,
		}
	}

	pub fn to_index(self) -> usize {
		self.row * GRID_SIZE + self.column
	}

	/// Returns the offset coordinate if it's within bounds, otherwise `None`.
	pub fn offset(self, dc: usize, dr: usize) -> Option<Self> {
		let column = self.column + dc;
		let row = self.row + dr;
		if column < GRID_SIZE && row < GRID_SIZE {
			Some(Self { column, row })
		} else {
			None
		}
	}
}

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
	pub const ALL: &[Shape] = &[
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

	pub fn get_coords(&self) -> &'static [Coord] {
		match self {
			Shape::OrangeRicky => shape![
				[. . x]
				[x x x]
			],
			Shape::BlueRicky => shape![
				[x . .]
				[x x x]
			],
			Shape::ClevelandZ => shape![
				[x x]
				[. x x]
			],
			Shape::RhodeIslandZ => shape![
				[. x x]
				[x x .]
			],
			Shape::Hero => shape![[x x x x]],
			Shape::Teewee => shape![
				[. x .]
				[x x x]
			],
			Shape::Smashboy => shape![
				[x x]
				[x x]
			],
			Shape::OrganeRickyUp => shape![
				[x .]
				[x .]
				[x x]
			],
			Shape::BlueRickyUp => shape![
				[x x]
				[x .]
				[x .]
			],
			Shape::ClevelandZUp => shape![
				[. x]
				[x x]
				[x .]
			],
			Shape::RhodeIslandZUp => shape![
				[x .]
				[x x]
				[. x]
			],
			Shape::HeroUp => shape![[x][x][x][x]],
			Shape::LongHero => shape![[x x x x x]],
			Shape::LongHeroUp => shape![[x][x][x][x][x]],
			Shape::ShortHero => shape![[x x x]],
			Shape::ShortHeroUp => shape![[x][x][x]],
			Shape::Duce => shape![[x x]],
			Shape::DuceUp => shape![[x][x]],
			Shape::Single => shape![[x]],
			Shape::TeeweeDown => shape![
				[x x x]
				[. x .]
			],
			Shape::TeeweeRight => shape![
				[x .]
				[x x]
				[x .]
			],
			Shape::TeeweeLeft => shape![
				[. x]
				[x x]
				[. x]
			],
			Shape::LongSmashboy => shape![
				[x x]
				[x x]
				[x x]
			],
			Shape::LongSmashboyUp => shape![
				[x x x]
				[x x x]
			],
			Shape::Huge => shape![
				[x x x]
				[x x x]
				[x x x]
			],
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
		let idx = rand::rng().random_range(0..Shape::ALL.len());
		Self {
			shape: Shape::ALL[idx],
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

	/// Returns if a cell on board is filled at a given coordinate.
	pub fn cell_filled(&self, coord: Coord) -> bool {
		self.board[coord.to_index()]
	}

	/// Get the cells that would be occupied if placing a piece at the coordinate.
	/// Returns `None` if any cell would be out of bounds.
	pub fn can_place(&self, piece_idx: usize, coord: Coord) -> Option<Vec<Coord>> {
		let piece = &self.pieces[piece_idx];

		piece.shape.get_coords().iter().map(|delta| coord.offset(delta.column, delta.row)).collect::<Option<Vec<Coord>>>()
	}

	/// Place a piece on the board. Returns true if successful.
	/// Handles: placement, marking used, clearing lines, score, and regenerating pieces.
	pub fn place_shape(&mut self, piece_idx: usize, coord: Coord) -> bool {
		let cells = match self.can_place(piece_idx, coord) {
			Some(cells) if cells.iter().all(|&c| !self.board[c.to_index()]) => cells,
			_ => return false,
		};

		// Place the blocks
		for &c in &cells {
			self.board[c.to_index()] = true;
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
		for row in 0..GRID_SIZE {
			let start = row * GRID_SIZE;
			if self.board[start..start + GRID_SIZE].iter().all(|&filled| filled) {
				for column in 0..GRID_SIZE {
					self.board[Coord::new(column, row).to_index()] = false;
				}
				score += GRID_SIZE as u32;
			}
		}

		// Check columns
		for column in 0..GRID_SIZE {
			if (0..GRID_SIZE).all(|row| self.board[Coord::new(column, row).to_index()]) {
				for row in 0..GRID_SIZE {
					self.board[Coord::new(column, row).to_index()] = false;
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

	#[test]
	fn shape_macro_coords() {
		// OrangeRicky: ..x / xxx → (2,0), (0,1), (1,1), (2,1)
		let coords = Shape::OrangeRicky.get_coords();
		assert_eq!(coords.len(), 4);
		assert!(coords.contains(&Coord::new(2, 0)));
		assert!(coords.contains(&Coord::new(0, 1)));
		assert!(coords.contains(&Coord::new(1, 1)));
		assert!(coords.contains(&Coord::new(2, 1)));

		// Hero: xxxx → (0,0), (1,0), (2,0), (3,0)
		let coords = Shape::Hero.get_coords();
		assert_eq!(coords.len(), 4);
		for col in 0..4 {
			assert!(coords.contains(&Coord::new(col, 0)));
		}

		// HeroUp: 4 rows of x → (0,0), (0,1), (0,2), (0,3)
		let coords = Shape::HeroUp.get_coords();
		assert_eq!(coords.len(), 4);
		for row in 0..4 {
			assert!(coords.contains(&Coord::new(0, row)));
		}

		// Single: x → (0,0)
		assert_eq!(Shape::Single.get_coords(), &[Coord::new(0, 0)]);

		// Huge: 3x3 → 9 cells
		let coords = Shape::Huge.get_coords();
		assert_eq!(coords.len(), 9);
		for row in 0..3 {
			for col in 0..3 {
				assert!(coords.contains(&Coord::new(col, row)));
			}
		}

		// TeeweeDown: xxx / .x. → (0,0), (1,0), (2,0), (1,1)
		let coords = Shape::TeeweeDown.get_coords();
		assert_eq!(coords.len(), 4);
		assert!(coords.contains(&Coord::new(0, 0)));
		assert!(coords.contains(&Coord::new(1, 0)));
		assert!(coords.contains(&Coord::new(2, 0)));
		assert!(coords.contains(&Coord::new(1, 1)));
	}
}
