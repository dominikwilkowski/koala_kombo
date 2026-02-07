use fyrox::{
	core::{algebra::Vector2, color::Color, pool::Handle, reflect::prelude::*, visitor::prelude::*},
	dpi::LogicalSize,
	engine::{GraphicsContext, GraphicsContextParams, executor::Executor},
	event_loop::EventLoop,
	gui::{
		BuildContext, HorizontalAlignment, Thickness, UiNode, UserInterface, VerticalAlignment,
		border::BorderBuilder,
		brush::Brush,
		grid::{Column, GridBuilder, Row},
		message::{MessageDirection, MouseButton, UiMessage},
		text::{TextBuilder, TextMessage},
		widget::{WidgetBuilder, WidgetMessage},
	},
	plugin::{Plugin, PluginContext},
	renderer::framework::core::log::{Log, MessageKind},
	window::WindowAttributes,
};

use crate::koala_kombo::{Coord, GRID_SIZE, KoalaKombo, Piece};

const GAP_PX: f32 = 1.0;

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

	// Layout sizes (stored for rebuilding)
	#[visit(skip)]
	#[reflect(hidden)]
	piece_widget_size: f32,

	// Drag state
	#[visit(skip)]
	#[reflect(hidden)]
	dragging: Option<DragState>,
}

#[derive(Debug)]
struct DragState {
	shape: usize,
	hover_cell: Option<Coord>,
}

impl GamePlugin {
	fn build_ui(&mut self, ctx: &mut BuildContext, screen_size: (f32, f32)) -> Handle<UiNode> {
		self.state = Some(KoalaKombo::new());

		let (width, height) = screen_size;
		let margin = 10.0;

		// Calculate sizes based on screen
		self.piece_widget_size = 160.0;
		let piece_widget_size = self.piece_widget_size;
		let piece_tray_height = piece_widget_size + margin * 2.0;

		// Board takes remaining height after title, score, and piece tray
		let title_height = 120.0;
		let score_height = 80.0;
		let header_height = title_height + score_height;
		let available_for_board = height - header_height - piece_tray_height - margin * 4.0;
		let board_size = available_for_board.min(width - margin * 2.0); // Keep square, fit in width

		// Title
		let title = TextBuilder::new(
			WidgetBuilder::new()
				.on_row(0)
				.with_margin(Thickness::uniform(8.0))
				.with_horizontal_alignment(HorizontalAlignment::Center),
		)
		.with_font_size(100.0.into())
		.with_text("Koala Kombo")
		.build(ctx);

		// Score
		self.score_text = TextBuilder::new(
			WidgetBuilder::new()
				.on_row(1)
				.with_margin(Thickness::uniform(8.0))
				.with_horizontal_alignment(HorizontalAlignment::Center),
		)
		.with_text("Score: 0")
		.with_font_size(48.0.into())
		.build(ctx);

		// Board grid
		let board_grid = self.build_board(ctx, board_size);
		let board_border = BorderBuilder::new(
			WidgetBuilder::new()
				.on_row(2)
				.with_margin(Thickness::uniform(margin))
				.with_horizontal_alignment(HorizontalAlignment::Center)
				.with_child(board_grid),
		)
		.with_stroke_thickness(Thickness::uniform(2.0).into())
		.build(ctx);

		// Piece tray
		let piece_children = self.build_piece_widgets(ctx, piece_widget_size);
		self.piece_tray = GridBuilder::new(
			WidgetBuilder::new().with_horizontal_alignment(HorizontalAlignment::Center).with_children(piece_children),
		)
		.add_rows(vec![Row::strict(piece_widget_size)])
		.add_columns(vec![Column::strict(piece_widget_size); 3])
		.build(ctx);

		let piece_border = BorderBuilder::new(
			WidgetBuilder::new()
				.on_row(3)
				.with_margin(Thickness::uniform(margin))
				.with_horizontal_alignment(HorizontalAlignment::Center)
				.with_child(self.piece_tray),
		)
		.with_stroke_thickness(Thickness::uniform(2.0).into())
		.build(ctx);

		// Main layout grid
		GridBuilder::new(WidgetBuilder::new().with_width(width).with_height(height).with_children([
			title,
			self.score_text,
			board_border,
			piece_border,
		]))
		.add_rows(vec![
			Row::strict(title_height),              // Title
			Row::strict(score_height),              // Score
			Row::strict(board_size + margin * 2.0), // Board + margins
			Row::strict(piece_tray_height),         // Piece tray
		])
		.add_columns(vec![Column::stretch()])
		.build(ctx)
	}

	fn build_board(&mut self, ctx: &mut BuildContext, board_size: f32) -> Handle<UiNode> {
		self.board_cells.clear();

		let cell_size = board_size / GRID_SIZE as f32;
		let rows = (0..GRID_SIZE).map(|_| Row::strict(cell_size)).collect::<Vec<_>>();
		let columns = (0..GRID_SIZE).map(|_| Column::strict(cell_size)).collect::<Vec<_>>();

		let mut children = Vec::with_capacity(GRID_SIZE * GRID_SIZE);
		for row in 0..GRID_SIZE {
			for column in 0..GRID_SIZE {
				let cell = BorderBuilder::new(
					WidgetBuilder::new()
						.on_row(row)
						.on_column(column)
						.with_margin(Thickness::uniform(GAP_PX * 0.5))
						.with_background(Brush::Solid(Color::from_rgba(40, 40, 40, 255)).into()),
				)
				.with_stroke_thickness(Thickness::uniform(1.0).into())
				.build(ctx);

				self.board_cells.push(cell);
				children.push(cell);
			}
		}

		GridBuilder::new(WidgetBuilder::new().with_children(children)).add_rows(rows).add_columns(columns).build(ctx)
	}

	fn build_piece_widgets(&mut self, ctx: &mut BuildContext, widget_size: f32) -> Vec<Handle<UiNode>> {
		self.piece_widgets.clear();
		let state = self.state.as_ref().unwrap();

		let mut children = Vec::with_capacity(3);
		for i in 0..3 {
			let piece = &state.pieces[i];
			let shape_grid = Self::build_piece_shape(ctx, piece);

			let widget = BorderBuilder::new(
				WidgetBuilder::new()
					.on_column(i)
					.with_margin(Thickness::uniform(4.0))
					.with_width(widget_size - 8.0)
					.with_height(widget_size - 8.0)
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
		let (min_column, max_column, min_row, max_row) = piece.shape.get_coords().iter().fold(
			(usize::MAX, 0, usize::MAX, 0),
			|(min_column, max_column, min_row, max_row), a| {
				(min_column.min(a.column), max_column.max(a.column), min_row.min(a.row), max_row.max(a.row))
			},
		);

		let width = max_column - min_column + 1;
		let height = max_row - min_row + 1;
		let cell_size = 30.0;
		let gap = 2.0;

		let rows = (0..height).map(|_| Row::strict(cell_size + gap)).collect::<Vec<_>>();
		let columns = (0..width).map(|_| Column::strict(cell_size + gap)).collect::<Vec<_>>();

		let children = piece
			.shape
			.get_coords()
			.iter()
			.map(|a| {
				BorderBuilder::new(
					WidgetBuilder::new()
						.on_row(a.row - min_row)
						.on_column(a.column - min_column)
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
				.with_vertical_alignment(VerticalAlignment::Center)
				.with_hit_test_visibility(false),
		)
		.add_rows(rows)
		.add_columns(columns)
		.build(ctx)
	}

	fn refresh(&self, ui: &mut UserInterface) {
		let state = self.state.as_ref().unwrap();

		// Calculate preview cells if dragging over board
		let (preview_cells, preview_valid) = if let Some(ref drag) = self.dragging
			&& let Some(hover) = drag.hover_cell
		{
			match state.can_place(drag.shape, hover) {
				Some(cells) => {
					let valid = !cells.iter().any(|&c| state.cell_filled(c));
					(cells, valid)
				},
				None => (vec![], false),
			}
		} else {
			(vec![], false)
		};

		// Paint board cells
		for row in 0..GRID_SIZE {
			for column in 0..GRID_SIZE {
				let pos = Coord::new(column, row);
				let brush = if preview_cells.contains(&pos) {
					if preview_valid {
						Brush::Solid(Color::from_rgba(100, 200, 100, 180))
					} else {
						Brush::Solid(Color::from_rgba(200, 100, 100, 180))
					}
				} else if state.cell_filled(pos) {
					Brush::Solid(Color::from_rgba(100, 150, 255, 255))
				} else {
					Brush::Solid(Color::from_rgba(40, 40, 40, 255))
				};

				ui.send_message(WidgetMessage::background(
					self.board_cells[pos.to_index()],
					MessageDirection::ToWidget,
					brush.into(),
				));
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
		let widget_size = self.piece_widget_size;
		let new_widgets = {
			let mut ctx = ui.build_ctx();
			self.build_piece_widgets(&mut ctx, widget_size)
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
		// Get actual screen size from graphics context - use physical pixels for UI
		let screen_size = if let GraphicsContext::Initialized(ctx) = &context.graphics_context {
			let size = ctx.window.inner_size();
			(size.width as f32, size.height as f32)
		} else {
			(1000.0, 1300.0) // Default for Retina 500x650
		};

		let ui = context.user_interfaces.first_mut();
		let ui_root = ui.root();

		{
			let mut ctx = ui.build_ctx();
			let root = self.build_ui(&mut ctx, screen_size);
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
						shape: piece_idx,
						hover_cell: None,
					});

					let widget = self.piece_widgets[piece_idx];

					// Unlink from grid layout so we can position freely, link to UI root
					let ui_root = ui.root();
					ui.send_message(WidgetMessage::link(widget, MessageDirection::ToWidget, ui_root));

					// Make hit-test invisible so mouse events pass through to board
					ui.send_message(WidgetMessage::hit_test_visibility(widget, MessageDirection::ToWidget, false));

					// Position at cursor (center the widget on cursor)
					let half_size = (self.piece_widget_size - 8.0) / 2.0;
					let offset = *pos - Vector2::new(half_size, half_size);
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
			let widget = self.piece_widgets[drag.shape];
			let half_size = (self.piece_widget_size - 8.0) / 2.0;
			let offset = *pos - Vector2::new(half_size, half_size);
			ui.send_message(WidgetMessage::desired_position(widget, MessageDirection::ToWidget, offset));
			// Don't return here - let other handlers process this event too
		}

		// Mouse enter board cell - update hover
		if let Some(WidgetMessage::MouseEnter) = message.data()
			&& let Some(ref mut drag) = self.dragging
		{
			if let Some(idx) = self.board_cells.iter().position(|&h| h == dest) {
				drag.hover_cell = Some(Coord::from_index(idx));
				self.refresh(ui);
			}
			return;
		}

		// Mouse leave board cell - clear hover
		if let Some(WidgetMessage::MouseLeave) = message.data()
			&& let Some(ref mut drag) = self.dragging
		{
			if let Some(idx) = self.board_cells.iter().position(|&h| h == dest)
				&& drag.hover_cell == Some(Coord::from_index(idx))
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

			let placed = if let Some(hover) = drag.hover_cell {
				state.place_shape(drag.shape, hover)
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
				// Rebuild tray to reset positions, then hide used pieces
				self.rebuild_piece_tray(ui);
				self.update_piece_visibility(ui);
			}

			self.refresh(ui);
		}
	}
}
