use std::{
	collections::hash_map::RandomState,
	hash::{BuildHasher, Hash},
};

use fyrox::{
	core::{color::Color, pool::Handle, reflect::prelude::*, visitor::prelude::*},
	gui::{
		BuildContext, Thickness, UiNode, UserInterface,
		border::BorderBuilder,
		brush::Brush,
		button::{ButtonBuilder, ButtonMessage},
		grid::{Column, GridBuilder, Row},
		message::{MessageDirection, MouseButton, UiMessage},
		stack_panel::StackPanelBuilder,
		text::{TextBuilder, TextMessage},
		widget::{WidgetBuilder, WidgetMessage},
	},
	plugin::{Plugin, PluginContext},
};

const GRID_SIZE: usize = 8;
const CELL_PX: f32 = 80.0;
const GAP_PX: f32 = 4.0;

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
struct Cell {
	filled: bool,
}

#[derive(Clone, Debug)]
struct Shape {
	blocks: &'static [(i32, i32)],
}

#[derive(Debug)]
struct GameState {
	board: Vec<Cell>,
	available_pieces: [Shape; 3],
	selected_piece: Option<usize>,
	score: u32,
}

impl GameState {
	fn new() -> Self {
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

	fn idx(x: usize, y: usize) -> usize {
		y * GRID_SIZE + x
	}

	fn in_bounds(x: i32, y: i32) -> bool {
		x >= 0 && y >= 0 && (x as usize) < GRID_SIZE && (y as usize) < GRID_SIZE
	}

	fn can_place(&self, shape: &Shape, anchor_x: usize, anchor_y: usize) -> bool {
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

	fn place(&mut self, shape: &Shape, anchor_x: usize, anchor_y: usize) {
		let ax = anchor_x as i32;
		let ay = anchor_y as i32;

		for (dx, dy) in shape.blocks {
			let x = (ax + dx) as usize;
			let y = (ay + dy) as usize;
			let idx = Self::idx(x, y);
			self.board[idx].filled = true;
		}
	}

	fn clear_complete_lines(&mut self) -> u32 {
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

	fn generate_new_pieces(&mut self) {
		let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();

		let hasher = RandomState::new();
		let mut h1 = hasher.build_hasher();
		time.hash(&mut h1);
		let idx1 = (hasher.hash_one(time) as usize) % PIECES.len();

		let mut h2 = hasher.build_hasher();
		(time + 1).hash(&mut h2);
		let idx2 = (hasher.hash_one(time + 1) as usize) % PIECES.len();

		let mut h3 = hasher.build_hasher();
		(time + 2).hash(&mut h3);
		let idx3 = (hasher.hash_one(time + 2) as usize) % PIECES.len();

		self.available_pieces = [
			Shape { blocks: PIECES[idx1] },
			Shape { blocks: PIECES[idx2] },
			Shape { blocks: PIECES[idx3] },
		];
	}
}

#[derive(Default, Visit, Reflect, Debug)]
pub struct GamePlugin {
	ui_root: Handle<UiNode>,
	board_cells: Vec<Handle<UiNode>>,
	piece_buttons: Vec<Handle<UiNode>>,
	score_text: Handle<UiNode>,

	#[visit(skip)]
	#[reflect(hidden)]
	state: Option<GameState>,
}

impl GamePlugin {
	fn build_ui(&mut self, ctx: &mut BuildContext) -> Handle<UiNode> {
		if self.state.is_none() {
			self.state = Some(GameState::new());
		}

		self.board_cells.clear();
		self.piece_buttons.clear();

		// Title
		let title = TextBuilder::new(WidgetBuilder::new().with_margin(Thickness::uniform(8.0)))
			.with_font_size(80.0.into())
			.with_text("Koala Kombo")
			.build(ctx);

		// Score text
		self.score_text = TextBuilder::new(WidgetBuilder::new().with_margin(Thickness::uniform(8.0)))
			.with_text("Score: 0")
			.with_font_size(50.0.into())
			.build(ctx);

		// Board grid (8x8 borders instead of buttons so we can change background)
		let rows = (0..GRID_SIZE).map(|_| Row::strict(CELL_PX + GAP_PX)).collect::<Vec<_>>();
		let cols = (0..GRID_SIZE).map(|_| Column::strict(CELL_PX + GAP_PX)).collect::<Vec<_>>();

		let mut board_children = Vec::with_capacity(GRID_SIZE * GRID_SIZE);
		for y in 0..GRID_SIZE {
			for x in 0..GRID_SIZE {
				let cell = BorderBuilder::new(
					WidgetBuilder::new()
						.on_row(y)
						.on_column(x)
						.with_margin(Thickness::uniform(GAP_PX * 0.5))
						.with_background(Brush::Solid(Color::from_rgba(40, 40, 40, 255)).into()),
				)
				.with_stroke_thickness(Thickness::uniform(1.0).into())
				.build(ctx);

				self.board_cells.push(cell);
				board_children.push(cell);
			}
		}

		let board_grid =
			GridBuilder::new(WidgetBuilder::new().with_children(board_children)).add_rows(rows).add_columns(cols).build(ctx);

		let board_border =
			BorderBuilder::new(WidgetBuilder::new().with_margin(Thickness::uniform(12.0)).with_child(board_grid))
				.with_stroke_thickness(Thickness::uniform(2.0).into())
				.build(ctx);

		// Piece tray (3 buttons)
		let mut piece_children = Vec::with_capacity(3);
		for i in 0..3 {
			let btn = ButtonBuilder::new(
				WidgetBuilder::new().with_margin(Thickness::uniform(8.0)).with_width(150.0).with_height(60.0).on_column(i),
			)
			.with_text(&format!("Piece {}", i + 1))
			.build(ctx);

			self.piece_buttons.push(btn);
			piece_children.push(btn);
		}

		let piece_grid = GridBuilder::new(WidgetBuilder::new().with_children(piece_children))
			.add_rows(vec![Row::strict(80.0)])
			.add_columns(vec![Column::strict(170.0), Column::strict(170.0), Column::strict(170.0)])
			.build(ctx);

		let piece_border =
			BorderBuilder::new(WidgetBuilder::new().with_margin(Thickness::uniform(12.0)).with_child(piece_grid))
				.with_stroke_thickness(Thickness::uniform(2.0).into())
				.build(ctx);

		// Root layout
		StackPanelBuilder::new(WidgetBuilder::new().with_margin(Thickness::uniform(12.0)).with_children([
			title,
			self.score_text,
			board_border,
			piece_border,
		]))
		.build(ctx)
	}

	fn paint_board_cell(ui: &mut UserInterface, handle: Handle<UiNode>, filled: bool) {
		let brush = if filled {
			Brush::Solid(Color::from_rgba(100, 150, 255, 255))
		} else {
			Brush::Solid(Color::from_rgba(40, 40, 40, 255))
		};

		ui.send_message(WidgetMessage::background(handle, MessageDirection::ToWidget, brush.into()));
	}

	fn paint_piece_button(ui: &mut UserInterface, handle: Handle<UiNode>, selected: bool) {
		let brush = if selected {
			Brush::Solid(Color::from_rgba(70, 170, 255, 255))
		} else {
			Brush::Solid(Color::from_rgba(30, 30, 30, 255))
		};
		ui.send_message(WidgetMessage::background(handle, MessageDirection::ToWidget, brush.into()));
	}

	fn refresh_ui(&self, ui: &mut UserInterface) {
		let state = self.state.as_ref().unwrap();

		// Paint board
		for y in 0..GRID_SIZE {
			for x in 0..GRID_SIZE {
				let idx = GameState::idx(x, y);
				let handle = self.board_cells[idx];
				let filled = state.board[idx].filled;
				Self::paint_board_cell(ui, handle, filled);
			}
		}

		// Paint piece selection
		for i in 0..3 {
			let selected = state.selected_piece == Some(i);
			Self::paint_piece_button(ui, self.piece_buttons[i], selected);
		}

		// Update score
		ui.send_message(TextMessage::text(self.score_text, MessageDirection::ToWidget, format!("Score: {}", state.score)));
	}
}

impl Plugin for GamePlugin {
	fn init(&mut self, _scene_path: Option<&str>, context: PluginContext) {
		let ui = context.user_interfaces.first_mut();
		let ui_root = ui.root();

		{
			let mut build_ctx = ui.build_ctx();
			self.ui_root = self.build_ui(&mut build_ctx);
			build_ctx.link(self.ui_root, ui_root);
		}

		self.refresh_ui(ui);
	}

	fn on_ui_message(&mut self, context: &mut PluginContext, message: &UiMessage) {
		let ui = context.user_interfaces.first_mut();

		let dest = message.destination();
		let state = self.state.as_mut().unwrap();

		// Handle piece button clicks (these are still buttons)
		if let Some(btn_msg) = message.data::<ButtonMessage>()
			&& matches!(btn_msg, ButtonMessage::Click)
			&& let Some(piece_idx) = self.piece_buttons.iter().position(|h| *h == dest)
		{
			state.selected_piece = Some(piece_idx);
			self.refresh_ui(ui);
			return;
		}

		// Handle board cell clicks (these are now borders, so use WidgetMessage)
		if let Some(widget_msg) = message.data::<WidgetMessage>()
			&& let WidgetMessage::MouseDown { button, .. } = widget_msg
			&& *button == MouseButton::Left
			&& let Some(cell_idx) = self.board_cells.iter().position(|h| *h == dest)
		{
			let Some(sel) = state.selected_piece else {
				return;
			};

			let x = cell_idx % GRID_SIZE;
			let y = cell_idx / GRID_SIZE;

			let shape_blocks = state.available_pieces[sel].blocks;
			let shape = Shape { blocks: shape_blocks };

			if state.can_place(&shape, x, y) {
				state.place(&shape, x, y);

				let line_score = state.clear_complete_lines();
				state.score += line_score;

				state.selected_piece = None;
				state.generate_new_pieces();

				self.refresh_ui(ui);
			}
		}
	}
}
