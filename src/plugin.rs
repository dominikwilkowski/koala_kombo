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

use crate::koala_kombo::{CELL_PX, GAP_PX, GRID_SIZE, KoalaKombo, Shape};

/// Active drag-and-drop state for a piece being dragged
#[derive(Debug)]
struct ActiveDrag {
	piece_index: usize,
	hover_cell: Option<usize>,
	drag_widget: Handle<UiNode>,
}

#[derive(Default, Visit, Reflect, Debug)]
pub struct GamePlugin {
	ui_root: Handle<UiNode>,
	board_cells: Vec<Handle<UiNode>>,
	piece_grid: Handle<UiNode>,
	piece_buttons: Vec<Handle<UiNode>>,
	score_text: Handle<UiNode>,

	#[visit(skip)]
	#[reflect(hidden)]
	state: Option<KoalaKombo>,

	#[visit(skip)]
	#[reflect(hidden)]
	active_drag: Option<ActiveDrag>,
}

impl GamePlugin {
	fn build_ui(&mut self, ctx: &mut BuildContext) -> Handle<UiNode> {
		if self.state.is_none() {
			self.state = Some(KoalaKombo::new());
		}

		self.board_cells.clear();

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

		// Board grid
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

		// Build piece tray
		let piece_children = self.build_piece_tray(ctx);
		self.piece_grid = GridBuilder::new(WidgetBuilder::new().with_children(piece_children))
			.add_rows(vec![Row::strict(170.0)])
			.add_columns(vec![Column::strict(170.0), Column::strict(170.0), Column::strict(170.0)])
			.build(ctx);

		let piece_border =
			BorderBuilder::new(WidgetBuilder::new().with_margin(Thickness::uniform(12.0)).with_child(self.piece_grid))
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

	fn build_piece_tray(&mut self, ctx: &mut BuildContext) -> Vec<Handle<UiNode>> {
		let state = self.state.as_ref().unwrap();
		self.piece_buttons.clear();
		let mut piece_children = Vec::with_capacity(3);

		for i in 0..3 {
			let shape = &state.available_pieces[i];

			let shape_grid = if shape.used {
				Handle::NONE
			} else {
				Self::build_shape_preview(ctx, shape)
			};

			let piece_widget = BorderBuilder::new(
				WidgetBuilder::new()
					.with_margin(Thickness::uniform(8.0))
					.with_width(150.0)
					.with_height(150.0)
					.on_column(i)
					.with_child(shape_grid)
					.with_background(Brush::Solid(Color::from_rgba(30, 30, 30, 255)).into()),
			)
			.with_stroke_thickness(Thickness::uniform(2.0).into())
			.build(ctx);

			self.piece_buttons.push(piece_widget);
			piece_children.push(piece_widget);
		}

		piece_children
	}

	fn rebuild_piece_tray(&mut self, ui: &mut UserInterface) {
		// Remove old piece buttons
		for &button in &self.piece_buttons {
			ui.send_message(WidgetMessage::remove(button, MessageDirection::ToWidget));
		}

		// Build new ones
		let new_pieces = {
			let mut build_ctx = ui.build_ctx();
			self.build_piece_tray(&mut build_ctx)
		};

		// Add them to the piece grid
		for piece in new_pieces {
			ui.send_message(WidgetMessage::link(piece, MessageDirection::ToWidget, self.piece_grid));
		}
	}

	fn paint_board_cell(
		ui: &mut UserInterface,
		handle: Handle<UiNode>,
		filled: bool,
		preview: bool,
		preview_valid: bool,
	) {
		let brush = if preview {
			if preview_valid {
				Brush::Solid(Color::from_rgba(100, 200, 100, 180))
			} else {
				Brush::Solid(Color::from_rgba(200, 100, 100, 180))
			}
		} else if filled {
			Brush::Solid(Color::from_rgba(100, 150, 255, 255))
		} else {
			Brush::Solid(Color::from_rgba(40, 40, 40, 255))
		};

		ui.send_message(WidgetMessage::background(handle, MessageDirection::ToWidget, brush.into()));
	}

	fn build_shape_preview(ctx: &mut BuildContext, shape: &Shape) -> Handle<UiNode> {
		let (min_x, max_x, min_y, max_y) =
			shape.blocks.iter().fold((i32::MAX, i32::MIN, i32::MAX, i32::MIN), |(min_x, max_x, min_y, max_y), &(x, y)| {
				(min_x.min(x), max_x.max(x), min_y.min(y), max_y.max(y))
			});

		let width = (max_x - min_x + 1) as usize;
		let height = (max_y - min_y + 1) as usize;

		let cell_size = 30.0;
		let gap = 2.0;

		let rows = (0..height).map(|_| Row::strict(cell_size + gap)).collect::<Vec<_>>();
		let cols = (0..width).map(|_| Column::strict(cell_size + gap)).collect::<Vec<_>>();

		let mut children = Vec::new();
		for (block_x, block_y) in shape.blocks {
			let grid_x = (block_x - min_x) as usize;
			let grid_y = (block_y - min_y) as usize;

			let cell = BorderBuilder::new(
				WidgetBuilder::new()
					.on_row(grid_y)
					.on_column(grid_x)
					.with_margin(Thickness::uniform(gap * 0.5))
					.with_background(Brush::Solid(Color::from_rgba(100, 150, 255, 255)).into()),
			)
			.with_stroke_thickness(Thickness::uniform(1.0).into())
			.build(ctx);

			children.push(cell);
		}

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

	fn refresh_ui(&self, ui: &mut UserInterface) {
		let state = self.state.as_ref().unwrap();

		// Calculate which cells should show preview
		let mut preview_cells = Vec::new();
		let mut preview_valid = false;

		if let Some(ref drag) = self.active_drag
			&& let Some(hover_cell) = drag.hover_cell
		{
			let hover_x = hover_cell % GRID_SIZE;
			let hover_y = hover_cell / GRID_SIZE;
			let shape = &state.available_pieces[drag.piece_index];

			preview_valid = state.can_place(shape, hover_x, hover_y);

			// Calculate all cells that would be occupied by the shape
			for (dx, dy) in shape.blocks {
				let x = hover_x as i32 + dx;
				let y = hover_y as i32 + dy;
				if x >= 0 && y >= 0 && (x as usize) < GRID_SIZE && (y as usize) < GRID_SIZE {
					let idx = KoalaKombo::idx(x as usize, y as usize);
					preview_cells.push(idx);
				}
			}
		}

		// Paint board
		for y in 0..GRID_SIZE {
			for x in 0..GRID_SIZE {
				let idx = KoalaKombo::idx(x, y);
				let handle = self.board_cells[idx];
				let filled = state.board[idx].filled;
				let is_preview = preview_cells.contains(&idx);
				Self::paint_board_cell(ui, handle, filled, is_preview, preview_valid);
			}
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

		// Only process FROM widget messages (user interactions)
		if message.direction() != MessageDirection::FromWidget {
			return;
		}

		// Handle mouse down on piece widgets - start drag
		if let Some(widget_msg) = message.data::<WidgetMessage>()
			&& let WidgetMessage::MouseDown { button, pos, .. } = widget_msg
			&& *button == MouseButton::Left
			&& let Some(piece_idx) = self.piece_buttons.iter().position(|h| *h == dest)
			&& !state.available_pieces[piece_idx].used
		{
			let shape = &state.available_pieces[piece_idx];
			let ui_root = ui.root();
			let drag_widget = {
				let mut build_ctx = ui.build_ctx();
				let shape_preview = Self::build_shape_preview(&mut build_ctx, shape);
				let widget = BorderBuilder::new(
					WidgetBuilder::new()
						.with_width(100.0)
						.with_height(100.0)
						.with_child(shape_preview)
						.with_hit_test_visibility(false),
				)
				.build(&mut build_ctx);
				build_ctx.link(widget, ui_root);
				widget
			};

			ui.send_message(WidgetMessage::desired_position(drag_widget, MessageDirection::ToWidget, *pos));
			ui.send_message(WidgetMessage::topmost(drag_widget, MessageDirection::ToWidget));

			self.active_drag = Some(ActiveDrag {
				piece_index: piece_idx,
				hover_cell: None,
				drag_widget,
			});
			self.refresh_ui(ui);
			return;
		}

		// Handle mouse move anywhere - update drag widget position
		if let Some(ref drag) = self.active_drag
			&& let Some(widget_msg) = message.data::<WidgetMessage>()
			&& let WidgetMessage::MouseMove { pos, .. } = widget_msg
		{
			ui.send_message(WidgetMessage::desired_position(drag.drag_widget, MessageDirection::ToWidget, *pos));
		}

		// Handle mouse move over board cells - update preview
		if let Some(ref mut drag) = self.active_drag
			&& let Some(widget_msg) = message.data::<WidgetMessage>()
			&& matches!(widget_msg, WidgetMessage::MouseEnter)
			&& let Some(cell_idx) = self.board_cells.iter().position(|h| *h == dest)
		{
			drag.hover_cell = Some(cell_idx);
			self.refresh_ui(ui);
			return;
		}

		// Handle mouse leaving board cells - clear preview
		if let Some(ref mut drag) = self.active_drag
			&& let Some(widget_msg) = message.data::<WidgetMessage>()
			&& matches!(widget_msg, WidgetMessage::MouseLeave)
			&& let Some(cell_idx) = self.board_cells.iter().position(|h| *h == dest)
			&& drag.hover_cell == Some(cell_idx)
		{
			drag.hover_cell = None;
			self.refresh_ui(ui);
			return;
		}

		// Handle mouse up - attempt placement and end drag
		if let Some(widget_msg) = message.data::<WidgetMessage>()
			&& let WidgetMessage::MouseUp { button, .. } = widget_msg
			&& *button == MouseButton::Left
			&& let Some(drag) = self.active_drag.take()
		{
			ui.send_message(WidgetMessage::remove(drag.drag_widget, MessageDirection::ToWidget));

			if let Some(hover_cell) = drag.hover_cell {
				let x = hover_cell % GRID_SIZE;
				let y = hover_cell / GRID_SIZE;

				let shape_blocks = state.available_pieces[drag.piece_index].blocks;
				let shape = Shape {
					blocks: shape_blocks,
					used: false,
				};

				if state.can_place(&shape, x, y) {
					state.place(&shape, x, y);
					state.mark_piece_used(drag.piece_index);

					let line_score = state.clear_complete_lines();
					state.score += line_score;

					if state.all_pieces_used() {
						state.generate_new_pieces();
						self.rebuild_piece_tray(ui);
					} else {
						self.rebuild_piece_tray(ui);
					}
				}
			}

			self.refresh_ui(ui);
		}
	}
}
