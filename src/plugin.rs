use fyrox::{
	core::{color::Color, pool::Handle, reflect::prelude::*, visitor::prelude::*},
	gui::{
		BuildContext, HorizontalAlignment, Thickness, UiNode, UserInterface, VerticalAlignment,
		border::BorderBuilder,
		brush::Brush,
		grid::{Column, GridBuilder, Row},
		message::{MessageDirection, MouseButton, UiMessage},
		stack_panel::StackPanelBuilder,
		text::{TextBuilder, TextMessage},
		widget::{WidgetBuilder, WidgetMessage},
	},
	plugin::{Plugin, PluginContext},
};

use crate::koala_kombo::{CELL_PX, GAP_PX, GRID_SIZE, KoalaKombo, Piece};

#[derive(Default, Visit, Reflect, Debug)]
pub struct GamePlugin {
	#[visit(skip)]
	#[reflect(hidden)]
	state: Option<KoalaKombo>,

	// UI handles
	board_cells: Vec<Handle<UiNode>>,
	piece_tray: Handle<UiNode>,
	piece_widgets: Vec<Handle<UiNode>>,
	score_text: Handle<UiNode>,

	// Drag state
	#[visit(skip)]
	#[reflect(hidden)]
	dragging: Option<DragState>,
}

#[derive(Debug)]
struct DragState {
	piece_idx: usize,
	hover_cell: Option<usize>,
}

impl GamePlugin {
	fn build_ui(&mut self, ctx: &mut BuildContext) -> Handle<UiNode> {
		self.state = Some(KoalaKombo::new());

		// Title
		let title = TextBuilder::new(WidgetBuilder::new().with_margin(Thickness::uniform(8.0)))
			.with_font_size(80.0.into())
			.with_text("Koala Kombo")
			.build(ctx);

		// Score
		self.score_text = TextBuilder::new(WidgetBuilder::new().with_margin(Thickness::uniform(8.0)))
			.with_text("Score: 0")
			.with_font_size(50.0.into())
			.build(ctx);

		// Board grid
		let board_grid = self.build_board(ctx);
		let board_border =
			BorderBuilder::new(WidgetBuilder::new().with_margin(Thickness::uniform(12.0)).with_child(board_grid))
				.with_stroke_thickness(Thickness::uniform(2.0).into())
				.build(ctx);

		// Piece tray
		let piece_children = self.build_piece_widgets(ctx);
		self.piece_tray = GridBuilder::new(WidgetBuilder::new().with_children(piece_children))
			.add_rows(vec![Row::strict(170.0)])
			.add_columns(vec![Column::strict(170.0); 3])
			.build(ctx);

		let piece_border =
			BorderBuilder::new(WidgetBuilder::new().with_margin(Thickness::uniform(12.0)).with_child(self.piece_tray))
				.with_stroke_thickness(Thickness::uniform(2.0).into())
				.build(ctx);

		StackPanelBuilder::new(WidgetBuilder::new().with_margin(Thickness::uniform(12.0)).with_children([
			title,
			self.score_text,
			board_border,
			piece_border,
		]))
		.build(ctx)
	}

	fn build_board(&mut self, ctx: &mut BuildContext) -> Handle<UiNode> {
		self.board_cells.clear();

		let rows = (0..GRID_SIZE).map(|_| Row::strict(CELL_PX + GAP_PX)).collect::<Vec<_>>();
		let cols = (0..GRID_SIZE).map(|_| Column::strict(CELL_PX + GAP_PX)).collect::<Vec<_>>();

		let mut children = Vec::with_capacity(GRID_SIZE * GRID_SIZE);
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
				children.push(cell);
			}
		}

		GridBuilder::new(WidgetBuilder::new().with_children(children)).add_rows(rows).add_columns(cols).build(ctx)
	}

	fn build_piece_widgets(&mut self, ctx: &mut BuildContext) -> Vec<Handle<UiNode>> {
		self.piece_widgets.clear();
		let state = self.state.as_ref().unwrap();

		let mut children = Vec::with_capacity(3);
		for i in 0..3 {
			let piece = &state.pieces[i];
			let shape_grid = Self::build_piece_shape(ctx, piece);

			// Transparent container
			let widget = BorderBuilder::new(
				WidgetBuilder::new()
					.on_column(i)
					.with_margin(Thickness::uniform(8.0))
					.with_width(150.0)
					.with_height(150.0)
					.with_child(shape_grid)
					.with_background(Brush::Solid(Color::TRANSPARENT).into()),
			)
			.with_stroke_thickness(Thickness::uniform(0.0).into())
			.build(ctx);

			self.piece_widgets.push(widget);
			children.push(widget);
		}

		children
	}

	fn build_piece_shape(ctx: &mut BuildContext, piece: &Piece) -> Handle<UiNode> {
		let (min_x, max_x, min_y, max_y) =
			piece.blocks.iter().fold((i32::MAX, i32::MIN, i32::MAX, i32::MIN), |(min_x, max_x, min_y, max_y), &(x, y)| {
				(min_x.min(x), max_x.max(x), min_y.min(y), max_y.max(y))
			});

		let width = (max_x - min_x + 1) as usize;
		let height = (max_y - min_y + 1) as usize;
		let cell_size = 30.0;
		let gap = 2.0;

		let rows = (0..height).map(|_| Row::strict(cell_size + gap)).collect::<Vec<_>>();
		let cols = (0..width).map(|_| Column::strict(cell_size + gap)).collect::<Vec<_>>();

		let children = piece
			.blocks
			.iter()
			.map(|&(bx, by)| {
				BorderBuilder::new(
					WidgetBuilder::new()
						.on_row((by - min_y) as usize)
						.on_column((bx - min_x) as usize)
						.with_margin(Thickness::uniform(gap * 0.5))
						.with_background(Brush::Solid(Color::from_rgba(100, 150, 255, 255)).into()),
				)
				.with_stroke_thickness(Thickness::uniform(1.0).into())
				.build(ctx)
			})
			.collect::<Vec<_>>();

		GridBuilder::new(
			WidgetBuilder::new()
				.with_children(children)
				.with_horizontal_alignment(HorizontalAlignment::Center)
				.with_vertical_alignment(VerticalAlignment::Center),
		)
		.add_rows(rows)
		.add_columns(cols)
		.build(ctx)
	}

	fn refresh(&self, ui: &mut UserInterface) {
		let state = self.state.as_ref().unwrap();

		// Calculate preview cells if dragging over board
		let (preview_cells, preview_valid) = if let Some(ref drag) = self.dragging
			&& let Some(cell_idx) = drag.hover_cell
		{
			let x = cell_idx % GRID_SIZE;
			let y = cell_idx / GRID_SIZE;
			(state.preview_cells(drag.piece_idx, x, y), state.can_place(drag.piece_idx, x, y))
		} else {
			(vec![], false)
		};

		// Paint board cells
		for y in 0..GRID_SIZE {
			for x in 0..GRID_SIZE {
				let idx = y * GRID_SIZE + x;
				let brush = if preview_cells.contains(&idx) {
					if preview_valid {
						Brush::Solid(Color::from_rgba(100, 200, 100, 180))
					} else {
						Brush::Solid(Color::from_rgba(200, 100, 100, 180))
					}
				} else if state.cell_filled(x, y) {
					Brush::Solid(Color::from_rgba(100, 150, 255, 255))
				} else {
					Brush::Solid(Color::from_rgba(40, 40, 40, 255))
				};

				ui.send_message(WidgetMessage::background(self.board_cells[idx], MessageDirection::ToWidget, brush.into()));
			}
		}

		// Update score
		ui.send_message(TextMessage::text(self.score_text, MessageDirection::ToWidget, format!("Score: {}", state.score)));
	}

	fn rebuild_piece_tray(&mut self, ui: &mut UserInterface) {
		// Remove old pieces
		for &widget in &self.piece_widgets {
			ui.send_message(WidgetMessage::remove(widget, MessageDirection::ToWidget));
		}

		// Build new pieces
		let new_widgets = {
			let mut ctx = ui.build_ctx();
			self.build_piece_widgets(&mut ctx)
		};

		// Link to tray
		for widget in new_widgets {
			ui.send_message(WidgetMessage::link(widget, MessageDirection::ToWidget, self.piece_tray));
		}
	}

	fn update_piece_visibility(&self, ui: &mut UserInterface) {
		let state = self.state.as_ref().unwrap();
		for (i, &widget) in self.piece_widgets.iter().enumerate() {
			ui.send_message(WidgetMessage::visibility(widget, MessageDirection::ToWidget, !state.pieces[i].used));
		}
	}
}

impl Plugin for GamePlugin {
	fn init(&mut self, _scene_path: Option<&str>, context: PluginContext) {
		let ui = context.user_interfaces.first_mut();
		let ui_root = ui.root();

		{
			let mut ctx = ui.build_ctx();
			let root = self.build_ui(&mut ctx);
			ctx.link(root, ui_root);
		}

		self.refresh(ui);
	}

	fn on_ui_message(&mut self, context: &mut PluginContext, message: &UiMessage) {
		if message.direction() != MessageDirection::FromWidget {
			return;
		}

		let ui = context.user_interfaces.first_mut();
		let dest = message.destination();

		// Mouse down on piece - start drag
		if let Some(WidgetMessage::MouseDown {
			button: MouseButton::Left,
			pos,
			..
		}) = message.data()
		{
			if let Some(piece_idx) = self.piece_widgets.iter().position(|&h| h == dest) {
				let state = self.state.as_ref().unwrap();
				if !state.pieces[piece_idx].used {
					self.dragging = Some(DragState {
						piece_idx,
						hover_cell: None,
					});

					let widget = self.piece_widgets[piece_idx];

					// Unlink from grid layout so we can position freely, link to UI root
					let ui_root = ui.root();
					ui.send_message(WidgetMessage::link(widget, MessageDirection::ToWidget, ui_root));

					// Make hit-test invisible so mouse events pass through to board
					ui.send_message(WidgetMessage::hit_test_visibility(widget, MessageDirection::ToWidget, false));

					// Position at cursor
					let offset = *pos - fyrox::core::algebra::Vector2::new(75.0, 75.0);
					ui.send_message(WidgetMessage::desired_position(widget, MessageDirection::ToWidget, offset));

					self.refresh(ui);
				}
			}
			return;
		}

		// Mouse move - update drag position (listen globally while dragging)
		if let Some(WidgetMessage::MouseMove { pos, .. }) = message.data()
			&& let Some(ref drag) = self.dragging
		{
			let widget = self.piece_widgets[drag.piece_idx];
			let offset = *pos - fyrox::core::algebra::Vector2::new(75.0, 75.0);
			ui.send_message(WidgetMessage::desired_position(widget, MessageDirection::ToWidget, offset));
			// Don't return here - let other handlers process this event too
		}

		// Mouse enter board cell - update hover
		if let Some(WidgetMessage::MouseEnter) = message.data()
			&& let Some(ref mut drag) = self.dragging
		{
			if let Some(cell_idx) = self.board_cells.iter().position(|&h| h == dest) {
				drag.hover_cell = Some(cell_idx);
				self.refresh(ui);
			}
			return;
		}

		// Mouse leave board cell - clear hover
		if let Some(WidgetMessage::MouseLeave) = message.data()
			&& let Some(ref mut drag) = self.dragging
		{
			if let Some(cell_idx) = self.board_cells.iter().position(|&h| h == dest)
				&& drag.hover_cell == Some(cell_idx)
			{
				drag.hover_cell = None;
				self.refresh(ui);
			}
			return;
		}

		// Mouse up - place shape
		if let Some(WidgetMessage::MouseUp {
			button: MouseButton::Left,
			..
		}) = message.data()
			&& let Some(drag) = self.dragging.take()
		{
			let state = self.state.as_mut().unwrap();

			let placed = if let Some(cell_idx) = drag.hover_cell {
				let x = cell_idx % GRID_SIZE;
				let y = cell_idx / GRID_SIZE;
				state.place_shape(drag.piece_idx, x, y)
			} else {
				false
			};

			if placed {
				// Check if pieces were regenerated
				if state.pieces.iter().all(|p| !p.used) {
					self.rebuild_piece_tray(ui);
				} else {
					self.update_piece_visibility(ui);
				}
			} else {
				// Rebuild tray to reset positions
				self.rebuild_piece_tray(ui);
			}

			self.refresh(ui);
		}
	}
}
