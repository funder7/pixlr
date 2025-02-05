use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{
    execute,
    terminal::{Clear, ClearType},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::Paragraph,
    Frame, Terminal,
};
use std::io;

struct DrawingWidget {
    grid: Vec<Vec<Color>>,
    cursor: (usize, usize),
}

impl DrawingWidget {
    fn new(width: usize, height: usize) -> DrawingWidget {
        DrawingWidget {
            grid: vec![vec![Color::Reset; width]; height],
            cursor: (0, 0),
        }
    }

    fn move_cursor(&mut self, dx: isize, dy: isize) {
        let new_x = self.cursor.0 as isize + dx;
        let new_y = self.cursor.1 as isize + dy;

        if new_x >= 0 && new_x < self.grid[0].len() as isize {
            self.cursor.0 = new_x as usize;
        }
        if new_y >= 0 && new_y < self.grid.len() as isize {
            self.cursor.1 = new_y as usize;
        }
    }

    fn set_cell_color(&mut self, x: usize, y: usize, color: Color) {
        if y < self.grid.len() && x < self.grid[y].len() {
            self.grid[y][x] = color;
        }
    }

    fn render(&self, area: Rect, f: &mut Frame) {
        let cell_width = area.width / self.grid[0].len() as u16;
        let cell_height = area.height / self.grid.len() as u16;

        for (y, row) in self.grid.iter().enumerate() {
            for (x, &color) in row.iter().enumerate() {
                let x_pos = area.x + x as u16 * cell_width;
                let y_pos = area.y + y as u16 * cell_height;
                let cell = Paragraph::new("░").style(Style::default().bg(color));
                f.render_widget(cell, Rect::new(x_pos, y_pos, cell_width, cell_height));
            }
        }

        // Highlight the cursor
        let (cursor_x, cursor_y) = self.cursor;
        let cursor_x_pos = area.x + cursor_x as u16 * cell_width;
        let cursor_y_pos = area.y + cursor_y as u16 * cell_height;
        let cursor_cell = Paragraph::new("█").style(Style::default().bg(Color::DarkGray));
        f.render_widget(
            cursor_cell,
            Rect::new(cursor_x_pos, cursor_y_pos, cell_width, cell_height),
        );
    }
}

struct ToolsWidget {
    selected_tool: Tool,
}

enum Tool {
    Pencil,
    Eraser,
    ColorPicker,
}

impl ToolsWidget {
    fn new() -> ToolsWidget {
        ToolsWidget {
            selected_tool: Tool::Pencil,
        }
    }

    fn render(&self, area: Rect, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                ]
                .as_ref(),
            )
            .split(area);

        let pencil_button =
            Paragraph::new("1 Pencil").style(if matches!(self.selected_tool, Tool::Pencil) {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });
        f.render_widget(pencil_button, chunks[0]);

        let eraser_button =
            Paragraph::new("2 Eraser").style(if matches!(self.selected_tool, Tool::Eraser) {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });
        f.render_widget(eraser_button, chunks[1]);

        let color_picker_button = Paragraph::new("3 Color Picker").style(
            if matches!(self.selected_tool, Tool::ColorPicker) {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            },
        );
        f.render_widget(color_picker_button, chunks[2]);
    }
}

struct ColorPickerOverlay {
    colors: Vec<Color>,
    selected_color: usize,
}

impl ColorPickerOverlay {
    fn new() -> ColorPickerOverlay {
        ColorPickerOverlay {
            colors: vec![
                Color::Black,
                Color::Red,
                Color::Green,
                Color::Yellow,
                Color::Blue,
                Color::Magenta,
                Color::Cyan,
                Color::White,
            ],
            selected_color: 0,
        }
    }

    fn handle_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Up => {
                if self.selected_color >= 4 {
                    self.selected_color -= 4;
                }
            }
            KeyCode::Down => {
                if self.selected_color + 4 < self.colors.len() {
                    self.selected_color += 4;
                }
            }
            KeyCode::Left => {
                if self.selected_color > 0 {
                    self.selected_color -= 1;
                }
            }
            KeyCode::Right => {
                if self.selected_color + 1 < self.colors.len() {
                    self.selected_color += 1;
                }
            }
            KeyCode::Enter => {
                // Close the overlay and return the selected color
                // You can implement this logic in the `App` struct
            }
            _ => {}
        }
    }

    fn render(&self, area: Rect, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
            .split(area);

        // Render color grid
        let num_colors = self.colors.len();
        let num_columns = 4; // Number of columns in the grid
        let num_rows = (num_colors + num_columns - 1) / num_columns; // Calculate number of rows

        let color_grid = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(1); num_rows])
            .split(chunks[0]);

        for (i, &color) in self.colors.iter().enumerate() {
            let row = i / num_columns;
            let col = i % num_columns;

            let cell_width = color_grid[0].width / num_columns as u16;
            let cell_height = 1; // Each row is 1 unit tall

            let x = color_grid[row].x + col as u16 * cell_width;
            let y = color_grid[row].y;
            let cell = Paragraph::new(" ").style(Style::default().bg(color));
            f.render_widget(cell, Rect::new(x, y, cell_width, cell_height));
        }

        // Render selected color preview
        let selected_color_preview = Paragraph::new("Selected Color")
            .style(Style::default().bg(self.colors[self.selected_color]));
        f.render_widget(selected_color_preview, chunks[1]);
    }
}

struct App {
    drawing_widget: DrawingWidget,
    tools_widget: ToolsWidget,
    color_picker_overlay: Option<ColorPickerOverlay>,
    selected_tool: Tool,
    selected_color: Color,
}

impl App {
    fn new() -> App {
        App {
            drawing_widget: DrawingWidget::new(16, 16), // Example: 16x16 grid
            tools_widget: ToolsWidget::new(),
            color_picker_overlay: None,
            selected_tool: Tool::Pencil,
            selected_color: Color::White,
        }
    }

    fn handle_input(&mut self, key: KeyCode) {
        if let Some(overlay) = &mut self.color_picker_overlay {
            overlay.handle_input(key);
            if let KeyCode::Enter = key {
                self.selected_color = overlay.colors[overlay.selected_color];
                self.color_picker_overlay = None; // Close the overlay
            }
        } else {
            match key {
                KeyCode::Char('1') => self.selected_tool = Tool::Pencil,
                KeyCode::Char('2') => self.selected_tool = Tool::Eraser,
                KeyCode::Char('3') => {
                    self.color_picker_overlay = Some(ColorPickerOverlay::new());
                }
                KeyCode::Up => self.drawing_widget.move_cursor(0, -1),
                KeyCode::Down => self.drawing_widget.move_cursor(0, 1),
                KeyCode::Left => self.drawing_widget.move_cursor(-1, 0),
                KeyCode::Right => self.drawing_widget.move_cursor(1, 0),
                KeyCode::Char(' ') => {
                    let (x, y) = self.drawing_widget.cursor;
                    match self.selected_tool {
                        Tool::Pencil => self.drawing_widget.set_cell_color(x, y, self.selected_color),
                        Tool::Eraser => self.drawing_widget.set_cell_color(x, y, Color::Reset),
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}

fn main() -> io::Result<()> {
    // Enter raw mode and alternate screen
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;

    // Clear the terminal
    execute!(io::stdout(), Clear(ClearType::All))?;

    // Set up terminal
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create application
    let mut app = App::new();

    // Main loop
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
                .split(f.area());

            app.drawing_widget.render(chunks[0], f);
            app.tools_widget.render(chunks[1], f);

            if let Some(overlay) = &app.color_picker_overlay {
                overlay.render(f.area(), f);
            }
        })?;

        // Handle input
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                _ => app.handle_input(key.code),
            }
        }
    }

    // Clean up and restore terminal state
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}
