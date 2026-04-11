use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use vimforge::app::{AppState, Mode, PopupKind};
use vimforge::commands::Command;
use vimforge::input::handler::InputState;
use vimforge::levels::config;
use vimforge::map::save;
use vimforge::render::viewport::Viewport;
use vimforge::tutorial::engine::TutorialState;
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
    let mut app = AppState::new(80, 50);
    let mut input = InputState::new();
    let mut tutorial: Option<TutorialState> = None;
    let mut viewport = Viewport::new(40, 24);

    // Check for existing save
    app.has_save = save::default_save_path().exists();

    // Start at the main menu
    app.mode = Mode::Menu;

    let mut last_frame = Instant::now();

    loop {
        // Render
        terminal.draw(|frame| {
            render_frame(frame, &app, &input, &tutorial, &viewport);
        })?;

        // Poll for events
        let timeout = FRAME_DURATION.saturating_sub(last_frame.elapsed());
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => {
                    if handle_key(
                        key,
                        &mut app,
                        &mut input,
                        &mut tutorial,
                        &mut viewport,
                    ) {
                        break; // quit requested
                    }
                }
                Event::Resize(w, h) => {
                    let cols = (w as usize).saturating_sub(if app.show_sidebar { 16 } else { 0 })
                        / 2;
                    let rows =
                        (h as usize).saturating_sub(1 + if app.show_tutorial { 3 } else { 0 });
                    viewport.width = cols;
                    viewport.height = rows;
                }
                _ => {}
            }
        }

        // Simulation tick
        if app.mode != Mode::Menu {
            let mut level_completed = false;
            if app.simulation.update(&mut app.world, &mut app.map) {
                // Check tutorial completion after each tick
                if let Some(ref mut tut) = tutorial {
                    level_completed = check_tutorial_completion(&app, tut);
                }

                // Day/night cycle
                app.day_tick = (app.day_tick + 1) % 600;

                // Particle & trail updates
                app.particles.tick();
                app.trails.tick();

                // Economic cycle (every 60 ticks)
                if app.simulation.tick_count % 60 == 0 {
                    app.economy.advance_cycle();
                    app.market.update(app.simulation.tick_count);
                    app.contract_board.check_deadlines(app.simulation.tick_count);
                    let _reward = app.contract_board.check_completions(app.simulation.tick_count);
                }
            }
            // Load the next level if the current one was just completed
            if level_completed {
                let next = tutorial.as_ref().map(|t| t.current_level);
                if let Some(level) = next {
                    if config::get_level(level).is_some() {
                        app.status_message =
                            format!("Level {} complete! Starting next level...", level - 1);
                        start_level(&mut app, &mut input, &mut tutorial, level);
                    } else {
                        // All tutorial levels finished
                        app.freeplay_unlocked = true;
                        app.status_message =
                            "All levels complete! Freeplay unlocked! Type :freeplay".to_string();
                    }
                }
            }
        }

        // Update animations
        if last_frame.elapsed() >= FRAME_DURATION {
            app.animations.tick();
            last_frame = Instant::now();
        }
    }

    Ok(())
}

fn handle_key(
    key: KeyEvent,
    app: &mut AppState,
    input: &mut InputState,
    tutorial: &mut Option<TutorialState>,
    viewport: &mut Viewport,
) -> bool {
    // Global quit: Ctrl-C
    if key.modifiers.contains(KeyModifiers::CONTROL)
        && key.code == KeyCode::Char('c')
    {
        return true;
    }

    // Handle popup dismiss
    if app.popup.is_some() {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                app.popup = None;
                app.popup_scroll = 0;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                app.popup_scroll += 1;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.popup_scroll = app.popup_scroll.saturating_sub(1);
            }
            _ => {}
        }
        return false;
    }

    // Handle main menu
    if app.mode == Mode::Menu {
        match key.code {
            KeyCode::Char('1') => {
                // Start tutorial from level 1
                start_level(app, input, tutorial, 1);
            }
            KeyCode::Char('2') => {
                if app.freeplay_unlocked {
                    start_freeplay(app, input, tutorial);
                }
            }
            KeyCode::Char('3') => {
                if app.has_save {
                    // Load save
                    if let Ok(data) = save::load_game(&save::default_save_path()) {
                        load_save_data(app, &data);
                    }
                }
            }
            KeyCode::Char('4') | KeyCode::Char('q') => {
                return true;
            }
            _ => {}
        }
        return false;
    }

    // Feed key to input handler
    let commands = input.handle_key(key, &mut app.map, &mut app.world, &mut app.undo_stack, &mut app.inventory);

    // Process commands that affect app-level state
    for cmd in &commands {
        match cmd {
            Command::ToggleSidebar => {
                app.show_sidebar = !app.show_sidebar;
            }
            Command::CmdSave(path) => {
                let save_path = path
                    .as_ref()
                    .map(|p| std::path::PathBuf::from(p))
                    .unwrap_or_else(save::default_save_path);
                let data = build_save_data(app, input, tutorial);
                match save::save_game(&data, &save_path) {
                    Ok(()) => {
                        app.status_message = "Saved".to_string();
                        app.has_save = true;
                    }
                    Err(e) => {
                        app.status_message = e;
                        app.status_error = true;
                    }
                }
            }
            Command::CmdQuit(force) => {
                if *force {
                    return true;
                } else {
                    // TODO: check unsaved changes
                    return true;
                }
            }
            Command::CmdSaveQuit | Command::SaveAndQuit => {
                let data = build_save_data(app, input, tutorial);
                let _ = save::save_game(&data, &save::default_save_path());
                return true;
            }
            Command::QuitNoSave => {
                return true;
            }
            Command::CmdSetSpeed(s) => {
                app.simulation.set_speed(*s);
            }
            Command::CmdPause => {
                app.simulation.pause();
            }
            Command::CmdResume => {
                app.simulation.resume();
            }
            Command::CmdStep => {
                app.simulation.step(&mut app.world, &mut app.map);
            }
            Command::CmdStats => {
                app.popup = Some(PopupKind::Stats);
            }
            Command::CmdRegisters => {
                app.popup = Some(PopupKind::Registers);
            }
            Command::CmdMarks => {
                app.popup = Some(PopupKind::Marks);
            }
            Command::CmdHelp(topic) => {
                app.popup = Some(PopupKind::Help(topic.clone()));
            }
            Command::CmdContracts => {
                app.popup = Some(PopupKind::Contracts);
            }
            Command::CmdMarket => {
                app.popup = Some(PopupKind::Market);
            }
            Command::CmdFinance => {
                app.popup = Some(PopupKind::Finance);
            }
            Command::CmdResearch => {
                app.popup = Some(PopupKind::Research);
            }
            Command::CmdLevel(Some(n)) => {
                start_level(app, input, tutorial, *n);
            }
            Command::CmdRestart => {
                if let Some(ref tut) = tutorial {
                    let level = tut.current_level;
                    start_level(app, input, tutorial, level);
                }
            }
            Command::CmdFreeplay => {
                if app.freeplay_unlocked {
                    start_freeplay(app, input, tutorial);
                } else {
                    app.status_message = "Complete level 6 to unlock Freeplay".to_string();
                    app.status_error = true;
                }
            }
            Command::CmdMenu => {
                app.mode = Mode::Menu;
                *tutorial = None;
            }
            Command::CmdNoHighlight => {
                input.search.clear();
                app.search.clear();
            }
            _ => {}
        }
    }

    // Sync cursor between input state and app state
    app.cursor_x = input.cursor_x;
    app.cursor_y = input.cursor_y;

    // Track cursor position for tutorial navigation objectives and auto-advance hints
    if let Some(ref mut tut) = tutorial {
        tut.visit_position(app.cursor_x, app.cursor_y);
        tut.auto_advance_hint();
    }
    app.pending_keys = input.parser.command_buffer.clone();
    app.command_buffer = input.parser.command_line.clone();
    app.insert_facing = input.parser.insert_facing;
    app.recording_macro = input.parser.recording_macro;
    app.show_sidebar = input.sidebar_visible;

    // Sync mode
    app.mode = match input.parser.mode {
        vimforge::vim::parser::Mode::Normal => Mode::Normal,
        vimforge::vim::parser::Mode::Insert => Mode::Insert,
        vimforge::vim::parser::Mode::Visual => Mode::Visual,
        vimforge::vim::parser::Mode::VisualLine => Mode::VisualLine,
        vimforge::vim::parser::Mode::VisualBlock => Mode::VisualBlock,
        vimforge::vim::parser::Mode::Command => Mode::Command,
        vimforge::vim::parser::Mode::Search => Mode::Search,
    };

    // Update viewport to follow cursor
    viewport.follow_cursor(input.cursor_x, input.cursor_y, app.map.width, app.map.height);
    input.viewport_top = viewport.offset_y;
    input.viewport_height = viewport.height;

    // Sync search state for rendering
    app.search = input.search.clone();

    // Visual anchor sync
    if let Some((ax, ay)) = input.visual_anchor {
        app.visual_anchor_x = ax;
        app.visual_anchor_y = ay;
    }

    // Status message sync
    if !input.status_message.is_empty() {
        app.status_message = input.status_message.clone();
        input.status_message.clear();
    }

    false
}

fn render_frame(
    frame: &mut ratatui::Frame,
    app: &AppState,
    _input: &InputState,
    tutorial: &Option<TutorialState>,
    viewport: &Viewport,
) {
    let size = frame.area();

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

    let areas = ui::layout::compute_layout(size, app.show_sidebar, app.show_tutorial);

    // Tutorial hint bar
    if let Some(area) = areas.tutorial_bar {
        if let Some(ref tut) = tutorial {
            ui::tutorial_bar::render_tutorial_bar(frame, area, app, tut);
        }
    }

    // Game grid
    ui::grid_render::render_grid(frame, areas.game_grid, app, viewport);

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

fn start_level(
    app: &mut AppState,
    input: &mut InputState,
    tutorial: &mut Option<TutorialState>,
    level: usize,
) {
    if let Some(cfg) = config::get_level(level) {
        // Reset game state
        app.world = hecs::World::new();
        app.map = vimforge::map::grid::Map::new(cfg.map_width, cfg.map_height);
        app.cursor_x = 0;
        app.cursor_y = 0;
        app.simulation = vimforge::game::simulation::Simulation::new();
        app.undo_stack = vimforge::game::undo::UndoStack::new();
        app.inventory = vimforge::game::inventory::Inventory::new();

        // Place level entities
        for le in &cfg.entities {
            let _ = app.map.place_entity_on_map(
                &mut app.world,
                le.x,
                le.y,
                le.entity_type,
                le.facing,
                le.player_placed,
            );
        }

        // Initialize tutorial state
        let mut tut = if let Some(ref existing) = tutorial {
            TutorialState::new_with_progress(level, existing.levels_completed.clone())
        } else {
            TutorialState::new(level)
        };
        tut.current_level = level;
        *tutorial = Some(tut);

        app.mode = Mode::Normal;
        app.show_tutorial = true;
        app.current_level = Some(level);

        // Reset input state
        input.parser = vimforge::vim::parser::VimParser::new();
        input.cursor_x = 0;
        input.cursor_y = 0;
    }
}

fn start_freeplay(
    app: &mut AppState,
    input: &mut InputState,
    tutorial: &mut Option<TutorialState>,
) {
    if let Some(cfg) = config::get_level(14) {
        app.world = hecs::World::new();
        app.map = vimforge::map::grid::Map::new(cfg.map_width, cfg.map_height);

        for le in &cfg.entities {
            let _ = app.map.place_entity_on_map(
                &mut app.world,
                le.x,
                le.y,
                le.entity_type,
                le.facing,
                le.player_placed,
            );
        }

        *tutorial = None;
        app.mode = Mode::Normal;
        app.show_tutorial = false;
        app.current_level = None;
        app.inventory = vimforge::game::inventory::Inventory::new();

        input.parser = vimforge::vim::parser::VimParser::new();
        input.cursor_x = 0;
        input.cursor_y = 0;
    }
}

/// Check if the current tutorial level is complete.
/// Returns true if the level was just completed (so the caller can load the next level).
fn check_tutorial_completion(app: &AppState, tut: &mut TutorialState) -> bool {
    // Gather output bin counts
    let mut total_widgets: u64 = 0;
    let mut total_ingots: u64 = 0;
    let mut total_ore: u64 = 0;

    for (_e, counter) in app
        .world
        .query::<&vimforge::ecs::components::OutputCounter>()
        .iter()
    {
        total_widgets += counter.widget_count();
        total_ingots += counter.ingot_count();
        total_ore += counter.ore_count();
    }

    if tut.check_completion(total_ore, total_ingots, total_widgets, (0, 0)) {
        tut.complete_level();
        return true;
    }
    false
}

fn build_save_data(
    app: &AppState,
    input: &InputState,
    tutorial: &Option<TutorialState>,
) -> save::SaveData {
    let mut entities = Vec::new();
    let mut resources = Vec::new();

    use vimforge::ecs::components::*;

    // Collect entities
    for (entity, (pos, kind)) in app.world.query::<(&Position, &EntityKind)>().iter() {
        let facing = app
            .world
            .get::<&FacingComponent>(entity)
            .ok()
            .map(|f| f.facing)
            .unwrap_or(vimforge::resources::Facing::Right);

        let proc = app.world.get::<&Processing>(entity).ok();
        let emitter = app.world.get::<&OreEmitter>(entity).ok();
        let counter = app.world.get::<&OutputCounter>(entity).ok();
        let splitter = app.world.get::<&SplitterState>(entity).ok();
        let merger = app.world.get::<&MergerState>(entity).ok();
        let player = app.world.get::<&PlayerPlaced>(entity).is_ok();

        entities.push(save::SavedEntity {
            x: pos.x,
            y: pos.y,
            entity_type: kind.kind,
            facing,
            processing_ticks: proc.as_ref().map(|p| p.ticks_remaining),
            input_a: proc.as_ref().and_then(|p| p.input_a),
            input_b: proc.as_ref().and_then(|p| p.input_b),
            output: proc.as_ref().and_then(|p| p.output),
            ore_emit_counter: emitter.as_ref().map(|e| e.ticks_since_emit),
            output_counts: counter
                .as_ref()
                .map(|c| (c.ore_count(), c.ingot_count(), c.widget_count())),
            output_counts_ext: counter
                .as_ref()
                .map(|c| c.counts.clone()),
            splitter_state: splitter.as_ref().map(|s| s.next_output),
            merger_state: merger.as_ref().map(|s| s.priority),
            player_placed: player,
        });
    }

    // Collect resources on tiles
    for y in 0..app.map.height {
        for x in 0..app.map.width {
            if let Some(resource) = app.map.resource_at(x, y) {
                resources.push(save::SavedResource { x, y, resource });
            }
        }
    }

    // Get total counts
    let mut total_widgets = 0u64;
    let mut total_ingots = 0u64;
    let mut total_ore = 0u64;
    for (_e, c) in app.world.query::<&OutputCounter>().iter() {
        total_widgets += c.widget_count();
        total_ingots += c.ingot_count();
        total_ore += c.ore_count();
    }

    save::SaveData {
        version: 2,
        map_width: app.map.width,
        map_height: app.map.height,
        entities,
        resources,
        registers: std::collections::HashMap::new(),
        marks: input.marks.list().into_iter().map(|(c, x, y)| (c, (x, y))).collect(),
        score: save::ScoreData {
            total_widgets,
            total_ingots,
            total_ore,
        },
        simulation_speed: app.simulation.speed,
        tick_count: app.simulation.tick_count,
        inventory: app.inventory.clone(),
        tutorial_state: tutorial.as_ref().map(|t| save::TutorialSaveState {
            current_level: t.current_level,
            levels_completed: t.levels_completed.clone(),
            commands_learned: Vec::new(),
        }),
        economy_cash: app.economy.cash,
        economy_difficulty: Some(app.economy.difficulty.name().to_string()),
        scaling_level: app.scaling.level,
        day_tick: app.day_tick,
        research_completed: app.research.completed.iter().map(|t| format!("{:?}", t)).collect(),
        game_mode: Some(match app.game_mode {
            vimforge::app::GameMode::Tutorial => "Tutorial",
            vimforge::app::GameMode::Campaign => "Campaign",
            vimforge::app::GameMode::Freeplay => "Freeplay",
        }.to_string()),
    }
}

fn load_save_data(app: &mut AppState, data: &save::SaveData) {
    app.world = hecs::World::new();
    app.map = vimforge::map::grid::Map::new(data.map_width, data.map_height);

    for se in &data.entities {
        let _ = app.map.place_entity_on_map(
            &mut app.world,
            se.x,
            se.y,
            se.entity_type,
            se.facing,
            se.player_placed,
        );
    }

    for sr in &data.resources {
        app.map.set_resource(sr.x, sr.y, sr.resource);
    }

    app.inventory = data.inventory.clone();
    app.simulation.set_speed(data.simulation_speed);
    app.simulation.tick_count = data.tick_count;
    app.mode = Mode::Normal;

    // Restore v2 fields (defaults used for v1 saves)
    app.economy.cash = data.economy_cash;
    app.scaling.level = data.scaling_level;
    app.day_tick = data.day_tick;
    app.game_mode = match data.game_mode.as_deref() {
        Some("Campaign") => vimforge::app::GameMode::Campaign,
        Some("Freeplay") => vimforge::app::GameMode::Freeplay,
        _ => vimforge::app::GameMode::Tutorial,
    };
}
