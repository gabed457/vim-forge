use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use vimforge::app::Mode;
use vimforge::game::session::GameSession;
use vimforge::ui;

const FRAME_DURATION: Duration = Duration::from_millis(33); // ~30 FPS

fn main() -> io::Result<()> {
    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let result = run_app(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {e}");
    }
    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let (term_w, term_h) = crossterm::terminal::size().unwrap_or((80, 24));
    let mut session = GameSession::new(term_w as usize, term_h as usize);

    let mut last_frame = Instant::now();

    loop {
        // Render
        terminal.draw(|frame| {
            render_frame(frame, &session);
        })?;

        // Poll for events
        let timeout = FRAME_DURATION.saturating_sub(last_frame.elapsed());
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => {
                    if session.handle_key(key) {
                        break; // quit requested
                    }
                }
                Event::Resize(w, h) => {
                    session.set_term_size(w as usize, h as usize);
                }
                _ => {}
            }
        }

        // Simulation tick (time-gated inside)
        session.update_realtime();

        // Update animations
        if last_frame.elapsed() >= FRAME_DURATION {
            session.app.animations.tick();
            last_frame = Instant::now();
        }
    }

    Ok(())
}

fn render_frame(frame: &mut ratatui::Frame, session: &GameSession) {
    let size = frame.area();
    let app = &session.app;

    // Check if terminal is too small
    if ui::layout::is_terminal_too_small(size) {
        let area = ui::layout::too_small_area(size);
        let msg = ratatui::widgets::Paragraph::new("Terminal too small (need 80x24)")
            .style(ratatui::style::Style::default().fg(ratatui::style::Color::Red));
        frame.render_widget(msg, area);
        return;
    }

    if app.mode == Mode::Menu {
        ui::menu::render_menu(frame, size, app);
        return;
    }

    // Fill root terminal background — no pure black
    frame.render_widget(
        ratatui::widgets::Block::default()
            .style(ratatui::style::Style::default().bg(ratatui::style::Color::Rgb(10, 12, 16))),
        size,
    );

    let areas = ui::layout::compute_layout(size, app.show_sidebar, app.show_tutorial);

    // Tutorial hint bar
    if let Some(area) = areas.tutorial_bar {
        if let Some(ref tut) = session.tutorial {
            ui::tutorial_bar::render_tutorial_bar(frame, area, app, tut);
        }
    }

    // Game grid
    ui::grid_render::render_grid(frame, areas.game_grid, app, &session.viewport);

    // Sidebar
    if let Some(area) = areas.sidebar {
        ui::sidebar::render_sidebar(frame, area, app);
    }

    // Status bar
    ui::statusbar::render_statusbar(frame, areas.status_bar, app);

    // Popup overlay
    if app.popup.is_some() {
        ui::popup::render_popup(frame, size, app);
    }
}
