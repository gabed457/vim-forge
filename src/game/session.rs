//! GameSession: the complete game driver, shared by the terminal binary and
//! by headless playthrough tests.
//!
//! Everything that used to live in `main.rs` (key dispatch, simulation
//! post-tick bookkeeping, level loading/advancement, save/load) lives here so
//! that automated tests exercise EXACTLY the same code path a player does.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{AppState, GameMode, MenuScreen, Mode, OverlayCard, PopupKind, TitleAction};
use crate::commands::Command;
use crate::ecs::components::{EntityKind, FacingComponent, OutputCounter};
use crate::input::handler::InputState;
use crate::levels::config;
use crate::map::save;
use crate::render::viewport::Viewport;
use crate::resources::{EntityType, Facing};
use crate::tutorial::engine::TutorialState;

pub struct GameSession {
    pub app: AppState,
    pub input: InputState,
    pub tutorial: Option<TutorialState>,
    pub viewport: Viewport,
    /// Terminal dimensions used for viewport sizing (cells, not tiles).
    pub term_width: usize,
    pub term_height: usize,
    /// Set to true when the player quits.
    pub quit: bool,
    /// Write campaign progress to the default save on level completion.
    /// Off by default so headless tests never touch the player's save;
    /// main.rs enables it for real play.
    pub autosave_enabled: bool,
    /// Status-message expiry tracking (post_tick clears stale messages).
    last_status: String,
    status_age: u32,
}

impl GameSession {
    pub fn new(term_width: usize, term_height: usize) -> Self {
        let mut app = AppState::new(80, 50);
        app.mode = Mode::Menu;
        app.has_save = save::default_save_path().exists();
        // Read campaign progress from the save (menu N/30 readout and
        // level-select locks). Corrupt/missing saves are simply ignored.
        if app.has_save {
            if let Ok(data) = save::load_game(&save::default_save_path()) {
                if let Some(ts) = data.tutorial_state {
                    app.campaign_completed = ts.levels_completed;
                    app.saved_level = Some(ts.current_level);
                }
            }
        }
        let initial_cols = term_width / 2;
        let initial_rows = term_height.saturating_sub(1);
        GameSession {
            app,
            input: InputState::new(),
            tutorial: None,
            viewport: Viewport::new(initial_cols, initial_rows),
            term_width,
            term_height,
            quit: false,
            autosave_enabled: false,
            last_status: String::new(),
            status_age: 0,
        }
    }

    /// Update stored terminal size and recompute the viewport.
    pub fn set_term_size(&mut self, w: usize, h: usize) {
        self.term_width = w;
        self.term_height = h;
        self.resize_viewport();
    }

    pub fn resize_viewport(&mut self) {
        // Horizontal: match the real layout — the Command Dock is 30 columns
        // and only shows on terminals >= 140 wide (ui::layout), so the grid
        // rect the renderer receives is exactly term_width - dock. Keeping
        // these in sync makes centering pads and the fitted zoom scale exact
        // (a stale 20-col sidebar here used to shift the map off-center and
        // clip its right edge at wide sizes).
        let sidebar = if self.app.show_sidebar
            && self.term_width >= crate::ui::layout::MIN_WIDTH_FOR_DOCK as usize
        {
            crate::ui::layout::DOCK_WIDTH as usize
        } else {
            0
        };
        // Vertical: legacy numbers (1 status row + 3 tutorial rows). The real
        // top bar is 2 rows, but page-scroll sizes (Ctrl-d/f = half/full page)
        // derive from this height and are part of the game's muscle-memory
        // contract, so it stays put; the renderer derives row coverage from
        // the actual rect, so the 1-row difference only affects centering by
        // at most one row.
        let tutorial = if self.app.show_tutorial { 3 } else { 0 };
        let cols = self.term_width.saturating_sub(sidebar) / 2;
        let rows = self.term_height.saturating_sub(1 + tutorial);
        self.viewport.resize(cols, rows);
        // Re-fit the tile scale to the new rect (no-op while the player has
        // zoomed manually with zi/zo).
        self.viewport
            .auto_fit(self.app.map.width, self.app.map.height);
    }

    /// Load a level (tutorial/campaign) and reset per-level state.
    pub fn start_level(&mut self, level: usize) {
        if let Some(cfg) = config::get_level(level) {
            self.app.world = hecs::World::new();
            self.app.map = crate::map::grid::Map::new(cfg.map_width, cfg.map_height);
            self.app.cursor_x = 0;
            self.app.cursor_y = 0;
            self.app.simulation = crate::game::simulation::Simulation::new();
            self.app.undo_stack = crate::game::undo::UndoStack::new();
            self.app.inventory = crate::game::inventory::Inventory::new();

            for le in &cfg.entities {
                let _ = self.app.map.place_entity_on_map(
                    &mut self.app.world,
                    le.x,
                    le.y,
                    le.entity_type,
                    le.facing,
                    le.player_placed,
                );
            }

            let mut tut = if let Some(ref existing) = self.tutorial {
                TutorialState::new_with_progress(level, existing.levels_completed.clone())
            } else {
                TutorialState::new(level)
            };
            tut.current_level = level;
            self.tutorial = Some(tut);

            self.app.mode = Mode::Normal;
            self.app.show_tutorial = true;
            self.app.current_level = Some(level);
            self.app.game_mode = GameMode::Tutorial;

            // Campaign levels have no money/research pressure: Tutorial
            // difficulty makes credit/deduct no-ops and zeroes expenses,
            // and all recipes stay unlocked.
            self.app.economy =
                crate::economy::ledger::Economy::new(crate::economy::ledger::Difficulty::Tutorial);
            self.app.simulation.config.unlocked_recipes = None;
            self.app.simulation.config.freeplay_power = false;
            self.app.delivered_snapshot.clear();
            self.app.delivered_lifetime.clear();
            self.app.income_this_cycle = 0.0;
            self.app.income_last_cycle = 0.0;

            self.input.parser = crate::vim::parser::VimParser::new();
            self.input.cursor_x = 0;
            self.input.cursor_y = 0;
            self.resize_viewport();
            // New level: clear any manual zoom and fit the whole map.
            self.viewport
                .reset_zoom_for_level(cfg.map_width, cfg.map_height);
            // Level intro card (fall-through: any key dismisses AND executes).
            self.app.overlay = Some(OverlayCard::intro(level));
        }
    }

    /// Load the freeplay sandbox.
    pub fn start_freeplay(&mut self) {
        if let Some(cfg) = config::get_level(config::FREEPLAY_LEVEL) {
            self.app.world = hecs::World::new();
            self.app.map = crate::map::grid::Map::new(cfg.map_width, cfg.map_height);

            for le in &cfg.entities {
                let _ = self.app.map.place_entity_on_map(
                    &mut self.app.world,
                    le.x,
                    le.y,
                    le.entity_type,
                    le.facing,
                    le.player_placed,
                );
            }

            self.tutorial = None;
            self.app.mode = Mode::Normal;
            self.app.show_tutorial = false;
            self.app.current_level = None;
            self.app.game_mode = GameMode::Freeplay;
            self.app.inventory = crate::game::inventory::Inventory::new();
            self.app.simulation = crate::game::simulation::Simulation::new();
            self.app.undo_stack = crate::game::undo::UndoStack::new();

            // Freeplay: live economy, research, contracts, market, power.
            self.app.economy =
                crate::economy::ledger::Economy::new(crate::economy::ledger::Difficulty::Normal);
            self.app.loans = crate::economy::loans::LoanManager::new(
                crate::economy::ledger::Difficulty::Normal,
            );
            self.app.research = crate::research::tree::ResearchState::new();
            self.app.contract_board = crate::contracts::board::ContractBoard::new();
            self.app.market = crate::market::prices::MarketState::new();
            self.app.pollution = crate::waste::pollution::PollutionState::new();
            self.app.scaling = crate::scaling::difficulty::ScalingState::new();
            self.app.delivered_snapshot.clear();
            self.app.delivered_lifetime.clear();
            self.app.income_this_cycle = 0.0;
            self.app.income_last_cycle = 0.0;
            self.app.last_expense_report = crate::economy::expenses::ExpenseReport::default();
            self.app.simulation.config.freeplay_power = true;
            self.refresh_unlocked_recipes();
            self.auto_select_research();
            self.app.status_message = format!(
                "Freeplay: ${} starting cash. Deliveries auto-sell; :research :market :contracts",
                self.app.economy.cash
            );

            self.input.parser = crate::vim::parser::VimParser::new();
            self.input.cursor_x = 0;
            self.input.cursor_y = 0;
            self.resize_viewport();
            self.viewport
                .reset_zoom_for_level(cfg.map_width, cfg.map_height);
            self.app.overlay = None;
        }
    }

    /// Recompute the unlocked-recipe set from completed research.
    /// Recipes with numeric_id 0 are always available; the rest need the
    /// technology that lists them. Applied only in Freeplay — campaign
    /// levels run with everything unlocked (None).
    fn refresh_unlocked_recipes(&mut self) {
        if self.app.game_mode != GameMode::Freeplay {
            self.app.simulation.config.unlocked_recipes = None;
            return;
        }
        let mut set = std::collections::HashSet::new();
        for tech in crate::research::tree::get_all_techs() {
            if self.app.research.completed.contains(&tech.id) {
                for rid in &tech.unlocks_recipes {
                    set.insert(*rid);
                }
            }
        }
        self.app.simulation.config.unlocked_recipes = Some(set);
    }

    /// When no research is active, automatically start the cheapest
    /// available technology (popups cannot take selection input, so the
    /// queue is driven for the player; see the :research popup).
    /// Available (unlocked, not completed, non-infinite) techs in
    /// get_all_techs order — the order the research popup lists them.
    pub fn available_research(&self) -> Vec<crate::research::tree::TechId> {
        let completed = &self.app.research.completed;
        crate::research::tree::get_all_techs()
            .into_iter()
            .filter(|t| {
                !t.is_infinite
                    && !completed.contains(&t.id)
                    && crate::research::tree::is_available(t.id, completed)
            })
            .map(|t| t.id)
            .collect()
    }

    fn auto_select_research(&mut self) {
        if self.app.research.current.is_some() {
            return;
        }
        let completed = self.app.research.completed.clone();
        let mut best: Option<(u64, crate::research::tree::TechId)> = None;
        for tech in crate::research::tree::get_all_techs() {
            if tech.is_infinite || completed.contains(&tech.id) {
                continue;
            }
            if !crate::research::tree::is_available(tech.id, &completed) {
                continue;
            }
            let cost: u64 = tech.science_cost.iter().map(|(_, n)| *n).sum();
            if best.map(|(c, _)| cost < c).unwrap_or(true) {
                best = Some((cost, tech.id));
            }
        }
        if let Some((_, id)) = best {
            self.app.research.start_research(id);
        }
    }

    /// Handle a single key event. Returns true when the player has quit.
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        // Global quit: Ctrl-C
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.quit = true;
            return true;
        }

        // Overlay cards (level intro / complete / finale) use FALL-THROUGH
        // semantics: any key dismisses the card AND still executes normally,
        // so muscle memory and headless tests are never interrupted.
        if let Some(mut card) = self.app.overlay.take() {
            self.app.overlay = card.followup.take().map(|b| *b);
        }

        // Popup dismiss / scroll / interactive selection
        if let Some(kind) = self.app.popup.clone() {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.app.popup = None;
                    self.app.popup_scroll = 0;
                    self.app.popup_cursor = 0;
                }
                KeyCode::Char('j') | KeyCode::Down => match kind {
                    PopupKind::Research => {
                        let n = self.available_research().len();
                        if n > 0 && self.app.popup_cursor + 1 < n {
                            self.app.popup_cursor += 1;
                        }
                        self.app.popup_scroll += 1;
                    }
                    PopupKind::Contracts => {
                        let n = self.app.contract_board.available.len();
                        if n > 0 && self.app.popup_cursor + 1 < n {
                            self.app.popup_cursor += 1;
                        }
                        self.app.popup_scroll += 1;
                    }
                    _ => self.app.popup_scroll += 1,
                },
                KeyCode::Char('k') | KeyCode::Up => {
                    self.app.popup_cursor = self.app.popup_cursor.saturating_sub(1);
                    self.app.popup_scroll = self.app.popup_scroll.saturating_sub(1);
                }
                KeyCode::Enter => match kind {
                    PopupKind::Research => {
                        let avail = self.available_research();
                        if let Some(&id) = avail.get(self.app.popup_cursor) {
                            self.app.research.start_research(id);
                            let name = crate::research::tree::get_tech(id).name;
                            self.app.status_message =
                                format!("Researching: {name}");
                        }
                    }
                    PopupKind::Contracts => {
                        let id = self
                            .app
                            .contract_board
                            .available
                            .get(self.app.popup_cursor)
                            .map(|c| c.id);
                        if let Some(id) = id {
                            let tick = self.app.simulation.tick_count;
                            if self.app.contract_board.accept(id, tick) {
                                self.app.status_message =
                                    "Contract accepted".to_string();
                                self.app.popup_cursor = 0;
                            } else {
                                self.app.status_message =
                                    "Contract slots full".to_string();
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
            return false;
        }

        // Main menu (Title / LevelSelect / Help screens)
        if self.app.mode == Mode::Menu {
            return self.handle_menu_key(key);
        }

        // Feed key to the input handler / vim parser
        let commands = self.input.handle_key(
            key,
            &mut self.app.map,
            &mut self.app.world,
            &mut self.app.undo_stack,
            &mut self.app.inventory,
        );

        // Track edits & commands for tutorial objectives
        if let Some(ref mut tut) = self.tutorial {
            for cmd in &commands {
                if cmd.is_edit() {
                    tut.record_edit();
                }
                if let Some(name) = cmd.tutorial_name() {
                    tut.use_command(name);
                }
            }
        }

        // Juice: flashes and particles for edits (rendering reads these;
        // simulation state is untouched).
        for cmd in &commands {
            match cmd {
                Command::PlaceEntity(et) => {
                    self.app
                        .animations
                        .flash_placement(self.input.cursor_x, self.input.cursor_y);
                    crate::render::particles::spawn_machine_particles(
                        &mut self.app.particles,
                        *et,
                        self.input.cursor_x,
                        self.input.cursor_y,
                    );
                }
                Command::Demolish(_)
                | Command::DemolishLine(_)
                | Command::DeleteUnderCursor(_) => {
                    self.app
                        .animations
                        .flash_demolition(self.input.cursor_x, self.input.cursor_y);
                }
                _ => {}
            }
        }
        // Failed placements set an error message in the input handler —
        // flash the status bar red so the failure is impossible to miss.
        if self.input.status_message.starts_with("Can't place") {
            self.app.animations.flash_status_failure();
            self.app
                .animations
                .flash_error(self.input.cursor_x, self.input.cursor_y);
        }

        let status_before_commands = self.app.status_message.clone();
        for cmd in &commands {
            match cmd {
                Command::ToggleSidebar => {
                    self.app.show_sidebar = !self.app.show_sidebar;
                    self.resize_viewport();
                }
                Command::CmdSave(path) => {
                    let save_path = path
                        .as_ref()
                        .map(std::path::PathBuf::from)
                        .unwrap_or_else(save::default_save_path);
                    let data = self.build_save_data();
                    match save::save_game(&data, &save_path) {
                        Ok(()) => {
                            self.app.status_message = "Saved".to_string();
                            self.app.has_save = true;
                        }
                        Err(e) => {
                            self.app.status_message = e.clone();
                            self.app.status_error = true;
                        }
                    }
                }
                Command::CmdLoad(path) => {
                    let p = std::path::PathBuf::from(path);
                    match save::load_game(&p) {
                        Ok(data) => {
                            self.load_save_data(&data);
                            self.app.status_message = "Loaded".to_string();
                        }
                        Err(e) => {
                            self.app.status_message = e.clone();
                            self.app.status_error = true;
                        }
                    }
                }
                Command::CmdQuit(_force) => {
                    self.quit = true;
                    return true;
                }
                Command::CmdSaveQuit | Command::SaveAndQuit => {
                    let data = self.build_save_data();
                    let _ = save::save_game(&data, &save::default_save_path());
                    self.quit = true;
                    return true;
                }
                Command::QuitNoSave => {
                    self.quit = true;
                    return true;
                }
                Command::CmdSetSpeed(s) => {
                    self.app.simulation.set_speed(*s);
                }
                Command::CmdPause => self.app.simulation.pause(),
                Command::CmdResume => self.app.simulation.resume(),
                Command::CmdStep => {
                    self.app
                        .simulation
                        .step(&mut self.app.world, &mut self.app.map);
                    self.post_tick();
                }
                Command::CmdStats => self.app.popup = Some(PopupKind::Stats),
                Command::CmdRegisters => self.app.popup = Some(PopupKind::Registers),
                Command::CmdMarks => self.app.popup = Some(PopupKind::Marks),
                Command::CmdHelp(topic) => {
                    self.app.popup = Some(PopupKind::Help(topic.clone()))
                }
                Command::CmdContracts => self.app.popup = Some(PopupKind::Contracts),
                Command::CmdMarket => self.app.popup = Some(PopupKind::Market),
                Command::CmdFinance => self.app.popup = Some(PopupKind::Finance),
                Command::CmdResearch => self.app.popup = Some(PopupKind::Research),
                Command::CmdPrestige => self.app.popup = Some(PopupKind::Prestige),
                Command::CmdRecipe(_) => self.app.popup = Some(PopupKind::Recipes),
                Command::CmdSell => {
                    self.app.status_message =
                        "Everything delivered to output bins is sold automatically at market price"
                            .to_string();
                }
                Command::CmdLoan => {
                    let rate = self.app.loans.offered_rate(&self.app.economy);
                    let amount = 5_000;
                    if self
                        .app
                        .loans
                        .take_loan(amount, rate, 20, &mut self.app.economy)
                    {
                        self.app.status_message = format!(
                            "Loan: +${} at {:.0}% per cycle over 20 cycles",
                            amount,
                            rate * 100.0
                        );
                    } else {
                        self.app.status_message =
                            "Loan denied: not enough collateral/credit".to_string();
                        self.app.status_error = true;
                    }
                }
                Command::CmdLevel(Some(n)) => {
                    self.start_level(*n);
                }
                Command::CmdRestart => {
                    if let Some(ref tut) = self.tutorial {
                        let level = tut.current_level;
                        self.start_level(level);
                    }
                }
                Command::CmdFreeplay => {
                    // Sandbox is always available — no campaign gate.
                    self.start_freeplay();
                }
                Command::CmdMenu => {
                    // Fold live campaign progress into the menu's view of the
                    // world before dropping the tutorial state.
                    if let Some(ref tut) = self.tutorial {
                        for l in &tut.levels_completed {
                            if !self.app.campaign_completed.contains(l) {
                                self.app.campaign_completed.push(*l);
                            }
                        }
                        self.app.campaign_completed.sort_unstable();
                    }
                    self.app.mode = Mode::Menu;
                    self.app.menu_screen = MenuScreen::Title;
                    self.app.menu_cursor = 0;
                    self.app.overlay = None;
                    self.tutorial = None;
                }
                Command::CmdNoHighlight => {
                    self.input.search.clear();
                    self.app.search.clear();
                }
                // --- Viewport zoom (zi/zo/zf) ---
                Command::ZoomIn => {
                    self.viewport
                        .zoom_in(self.app.map.width, self.app.map.height);
                    self.app.status_message = format!("zoom: {}x", self.viewport.scale);
                }
                Command::ZoomOut => {
                    self.viewport
                        .zoom_out(self.app.map.width, self.app.map.height);
                    self.app.status_message = format!("zoom: {}x", self.viewport.scale);
                }
                Command::ZoomFit => {
                    self.viewport
                        .zoom_fit(self.app.map.width, self.app.map.height);
                    self.app.status_message =
                        format!("zoom: fit ({}x)", self.viewport.scale);
                }
                // --- Viewport scroll commands (zz/zt/zb, Ctrl-d/u/f/b) ---
                // The input handler already moved the cursor / its mirrored
                // viewport_top; here the real Viewport is adjusted.
                Command::ScrollCenterCursor => {
                    self.viewport.center_on(
                        self.input.cursor_x,
                        self.input.cursor_y,
                        self.app.map.width,
                        self.app.map.height,
                    );
                }
                Command::ScrollCursorTop => {
                    self.viewport.cursor_to_top(
                        self.input.cursor_y,
                        self.app.map.width,
                        self.app.map.height,
                    );
                }
                Command::ScrollCursorBottom => {
                    self.viewport.cursor_to_bottom(
                        self.input.cursor_y,
                        self.app.map.width,
                        self.app.map.height,
                    );
                }
                Command::ScrollHalfPageDown(n) => {
                    let lines = (self.viewport.height / 2).max(1) * n;
                    self.viewport.scroll_down(lines, self.app.map.height);
                }
                Command::ScrollHalfPageUp(n) => {
                    let lines = (self.viewport.height / 2).max(1) * n;
                    self.viewport.scroll_up(lines);
                }
                Command::ScrollFullPageDown(n) => {
                    let lines = self.viewport.height.max(1) * n;
                    self.viewport.scroll_down(lines, self.app.map.height);
                }
                Command::ScrollFullPageUp(n) => {
                    let lines = self.viewport.height.max(1) * n;
                    self.viewport.scroll_up(lines);
                }
                _ => {}
            }
        }

        let session_set_message = self.app.status_message != status_before_commands;
        self.sync_after_key(session_set_message);
        false
    }

    // -------------------------------------------------------------------
    // Menu screens (Title / LevelSelect / Help)
    // -------------------------------------------------------------------

    /// Handle a key while in the menu. Returns true when the player quit.
    fn handle_menu_key(&mut self, key: KeyEvent) -> bool {
        match self.app.menu_screen {
            MenuScreen::Title => self.handle_title_key(key),
            MenuScreen::LevelSelect => {
                self.handle_level_select_key(key);
                false
            }
            MenuScreen::Help => {
                // Any of the usual "back" keys returns to the title.
                if matches!(
                    key.code,
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('h') | KeyCode::Enter
                ) {
                    self.app.menu_screen = MenuScreen::Title;
                    self.app.menu_cursor = 0;
                }
                false
            }
        }
    }

    fn handle_title_key(&mut self, key: KeyEvent) -> bool {
        let entries = self.app.title_entries();
        let n = entries.len();
        match key.code {
            // Quick keys: number/letter shortcuts fire immediately.
            KeyCode::Char('1') => {
                // Fresh campaign run from level 1 (Enter on Campaign opens
                // the level-select grid instead).
                self.start_level(1);
                false
            }
            KeyCode::Char('2') => {
                // Sandbox is ALWAYS unlocked.
                self.start_freeplay();
                false
            }
            KeyCode::Char('3') => {
                self.menu_continue();
                false
            }
            KeyCode::Char('4') | KeyCode::Char('q') => {
                self.quit = true;
                true
            }
            KeyCode::Char('h') => {
                self.app.menu_screen = MenuScreen::Help;
                false
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.app.menu_cursor = (self.app.menu_cursor + 1) % n.max(1);
                false
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.app.menu_cursor = self.app.menu_cursor.checked_sub(1).unwrap_or(n - 1);
                false
            }
            KeyCode::Enter => {
                let action = entries
                    .get(self.app.menu_cursor.min(n.saturating_sub(1)))
                    .map(|e| e.action);
                match action {
                    Some(TitleAction::Campaign) => {
                        // Open the level-select grid with the cursor on the
                        // next level to play (furthest completed + 1).
                        self.app.menu_screen = MenuScreen::LevelSelect;
                        self.app.menu_cursor =
                            self.app.furthest_campaign_level().min(29);
                    }
                    Some(TitleAction::Sandbox) => self.start_freeplay(),
                    Some(TitleAction::Continue) => self.menu_continue(),
                    Some(TitleAction::Help) => self.app.menu_screen = MenuScreen::Help,
                    Some(TitleAction::Quit) => {
                        self.quit = true;
                        return true;
                    }
                    None => {}
                }
                false
            }
            _ => false,
        }
    }

    fn handle_level_select_key(&mut self, key: KeyEvent) {
        // 5 columns x 6 act rows; menu_cursor is 0..29 (level - 1).
        let cur = self.app.menu_cursor.min(29);
        let (col, row) = (cur % 5, cur / 5);
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.app.menu_screen = MenuScreen::Title;
                self.app.menu_cursor = 0;
            }
            KeyCode::Char('h') | KeyCode::Left => {
                self.app.menu_cursor = row * 5 + col.saturating_sub(1);
            }
            KeyCode::Char('l') | KeyCode::Right => {
                self.app.menu_cursor = row * 5 + (col + 1).min(4);
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.app.menu_cursor = (row + 1).min(5) * 5 + col;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.app.menu_cursor = row.saturating_sub(1) * 5 + col;
            }
            KeyCode::Char('g') => self.app.menu_cursor = 0,
            KeyCode::Char('G') => self.app.menu_cursor = 29,
            KeyCode::Enter => {
                let level = cur + 1;
                if self.app.is_level_unlocked(level) {
                    // Carry saved campaign progress into the new run so
                    // completing this level doesn't reset furthest-level.
                    let completed = self.app.campaign_completed.clone();
                    self.tutorial = Some(TutorialState::new_with_progress(level, completed));
                    self.start_level(level);
                }
            }
            _ => {}
        }
    }

    /// Continue: load the default save, if present.
    fn menu_continue(&mut self) {
        if self.app.has_save {
            if let Ok(data) = save::load_game(&save::default_save_path()) {
                self.load_save_data(&data);
                // Re-show the intro card for the level being resumed.
                if let Some(level) = self.app.current_level {
                    if config::get_level(level).is_some() && level != config::FREEPLAY_LEVEL {
                        self.app.overlay = Some(OverlayCard::intro(level));
                    }
                }
            }
        }
    }

    /// Silently write campaign progress to the default save (called on every
    /// level completion). Merges with any progress already on disk so a
    /// replay of an early level can never lose furthest-level progress.
    fn autosave_campaign(&mut self) {
        if !self.autosave_enabled {
            return;
        }
        let mut merged = self.app.campaign_completed.clone();
        if let Some(ref tut) = self.tutorial {
            for l in &tut.levels_completed {
                if !merged.contains(l) {
                    merged.push(*l);
                }
            }
        }
        merged.sort_unstable();
        merged.dedup();
        self.app.campaign_completed = merged.clone();

        let mut data = self.build_save_data();
        if let Some(ref mut ts) = data.tutorial_state {
            ts.levels_completed = merged;
        }
        if save::save_game(&data, &save::default_save_path()).is_ok() {
            self.app.has_save = true;
            self.app.saved_level = self.tutorial.as_ref().map(|t| t.current_level);
        }
    }

    /// Synchronize AppState mirrors of InputState after key handling.
    /// `session_set_message`: the command loop just set an app-level status
    /// message; don't clobber it with the input handler's generic echo.
    fn sync_after_key(&mut self, session_set_message: bool) {
        self.app.cursor_x = self.input.cursor_x;
        self.app.cursor_y = self.input.cursor_y;

        if let Some(ref mut tut) = self.tutorial {
            tut.visit_position(self.app.cursor_x, self.app.cursor_y);
            // Standing on ANY tile of a multi-tile building counts as
            // visiting its anchor (NavigateToAll targets are anchors).
            if let Some(entity) = self.app.map.entity_at(self.app.cursor_x, self.app.cursor_y) {
                let anchor = self
                    .app
                    .world
                    .get::<&crate::ecs::components::PartOfBuilding>(entity)
                    .map(|p| p.anchor)
                    .unwrap_or(entity);
                if let Ok(pos) = self.app.world.get::<&crate::ecs::components::Position>(anchor) {
                    tut.visit_position(pos.x, pos.y);
                }
            }
            tut.auto_advance_hint();
        }
        self.app.pending_keys = self.input.parser.command_buffer.clone();
        self.app.command_buffer = self.input.parser.command_line.clone();
        self.app.insert_facing = self.input.parser.insert_facing;
        self.app.insert_category = self.input.parser.insert_category_name();
        self.app.recording_macro = self.input.parser.recording_macro;
        self.app.show_sidebar = self.input.sidebar_visible;

        self.app.mode = match self.input.parser.mode {
            crate::vim::parser::Mode::Normal => Mode::Normal,
            crate::vim::parser::Mode::Insert => {
                if self.input.parser.replace_mode {
                    Mode::Replace
                } else {
                    Mode::Insert
                }
            }
            crate::vim::parser::Mode::Visual => Mode::Visual,
            crate::vim::parser::Mode::VisualLine => Mode::VisualLine,
            crate::vim::parser::Mode::VisualBlock => Mode::VisualBlock,
            crate::vim::parser::Mode::Command => Mode::Command,
            crate::vim::parser::Mode::Search => Mode::Search,
        };

        self.viewport.follow_cursor(
            self.input.cursor_x,
            self.input.cursor_y,
            self.app.map.width,
            self.app.map.height,
        );
        self.input.viewport_top = self.viewport.offset_y;
        self.input.viewport_height = self.viewport.height;

        self.app.search = self.input.search.clone();

        if let Some((ax, ay)) = self.input.visual_anchor {
            self.app.visual_anchor_x = ax;
            self.app.visual_anchor_y = ay;
        }

        if !self.input.status_message.is_empty() {
            if !session_set_message {
                self.app.status_message = self.input.status_message.clone();
            }
            self.input.status_message.clear();
        }
    }

    /// Real-time update from the main loop: tick when enough time elapsed.
    pub fn update_realtime(&mut self) {
        if self.app.mode == Mode::Menu {
            return;
        }
        if self
            .app
            .simulation
            .update(&mut self.app.world, &mut self.app.map)
        {
            self.post_tick();
        }
    }

    /// Force `n` simulation ticks (headless testing / :step).
    pub fn tick(&mut self, n: usize) {
        for _ in 0..n {
            if self.app.mode == Mode::Menu {
                return;
            }
            self.app
                .simulation
                .step(&mut self.app.world, &mut self.app.map);
            self.post_tick();
        }
    }

    /// Per-tick bookkeeping: tutorial completion, day/night, economy cycle,
    /// and campaign level advancement.
    fn post_tick(&mut self) {
        // Snapshot per-level stats BEFORE the completion check (which resets
        // them) so the level-complete card can show them.
        let edits_this_level = self.tutorial.as_ref().map(|t| t.edit_count).unwrap_or(0);

        let mut level_completed = false;
        if let Some(ref mut tut) = self.tutorial {
            level_completed = Self::check_tutorial_completion(&self.app, tut);
        }

        // Overlay cards fade on their own after ~300 ticks (any key also
        // dismisses them, falling through to normal handling).
        if let Some(ref mut card) = self.app.overlay {
            if card.ticks_left == 0 {
                self.app.overlay = card.followup.take().map(|b| *b);
            } else {
                card.ticks_left -= 1;
            }
        }

        // Status messages expire instead of lingering forever.
        if self.app.status_message == self.last_status {
            self.status_age += 1;
            if self.status_age > 240 {
                self.app.status_message.clear();
                self.last_status.clear();
                self.status_age = 0;
            }
        } else {
            self.last_status = self.app.status_message.clone();
            self.status_age = 0;
        }

        // A stuck player still gets the next hint eventually.
        if self.app.simulation.tick_count % 150 == 0 {
            if let Some(ref mut tut) = self.tutorial {
                tut.advance_hint();
            }
        }

        self.app.day_tick = (self.app.day_tick + 1) % 600;
        self.app.particles.tick();
        self.app.trails.tick();

        self.deliveries_and_sales();
        self.research_tick();
        self.pollution_tick();

        if self.app.simulation.tick_count % 60 == 0 {
            self.economy_cycle();
        }

        if level_completed {
            let output_this_level = self.output_totals().3;
            let next = self.tutorial.as_ref().map(|t| t.current_level);
            if let Some(level) = next {
                if config::get_level(level).is_some() && level != config::FREEPLAY_LEVEL {
                    let done = level - 1;
                    let name = config::get_level(done)
                        .map(|c| c.name)
                        .unwrap_or("");
                    self.app.status_message = format!(
                        "* LEVEL {done} COMPLETE — {name}! * Next: level {level} of 30"
                    );
                    self.app.animations.flash_status_success();
                    self.start_level(level);
                    // start_level queued the next level's intro card; show
                    // the LEVEL COMPLETE card first, then that intro.
                    let intro = self.app.overlay.take();
                    self.app.overlay = Some(OverlayCard::complete(
                        done,
                        edits_this_level,
                        output_this_level,
                        intro,
                    ));
                } else {
                    self.app.freeplay_unlocked = true;
                    self.app.animations.flash_status_success();
                    self.app.status_message =
                        "*** CAMPAIGN COMPLETE — you know vim! *** Sandbox awaits: type :freeplay"
                            .to_string();
                    self.app.overlay =
                        Some(OverlayCard::finale(edits_this_level, output_this_level));
                }
                // Campaign progress autosaves silently on every completion.
                self.autosave_campaign();
            }
        }
    }

    // -------------------------------------------------------------------
    // Live-loop wiring: sales, contracts, research, pollution, economy
    // -------------------------------------------------------------------

    /// Research progress granted per science-pack set consumed by a lab.
    const PACK_PROGRESS: u32 = 20;

    /// Detect new output-bin deliveries since the last tick; auto-sell them
    /// at market price (output = income) and count them toward active
    /// contracts and lifetime production.
    fn deliveries_and_sales(&mut self) {
        use crate::resources::Resource;
        let mut current: std::collections::HashMap<Resource, u64> =
            std::collections::HashMap::new();
        for (_e, counter) in self.app.world.query::<&OutputCounter>().iter() {
            for (r, n) in &counter.counts {
                *current.entry(*r).or_insert(0) += n;
            }
        }

        for (r, total) in &current {
            let prev = self.app.delivered_snapshot.get(r).copied().unwrap_or(0);
            if *total <= prev {
                continue;
            }
            let delta = *total - prev;
            *self.app.delivered_lifetime.entry(*r).or_insert(0) += delta;

            // Count toward active contracts (first come, first served).
            let mut remaining = delta;
            for contract in &mut self.app.contract_board.active {
                if remaining == 0 {
                    break;
                }
                remaining -= contract.deliver(*r, remaining);
            }

            // Auto-sell at the trade-hub price. In campaign levels the
            // economy runs at Tutorial difficulty, so credit() is a no-op.
            let price = self.app.market.sell_price(*r);
            if price > 0.0 {
                let revenue = price * delta as f64;
                self.app.economy.credit(revenue.round() as i64);
                self.app.market.record_sale(*r, delta);
                self.app.income_this_cycle += revenue;
            }
        }

        self.app.delivered_snapshot = current;
    }

    /// Labs consume stocked science packs and advance the active research.
    /// A lab contributes when it accepts (and has) every pack type the tech
    /// requires; each contribution consumes one pack of each type and grants
    /// `PACK_PROGRESS * lab speed` progress.
    fn research_tick(&mut self) {
        use crate::ecs::components::LabStock;

        self.auto_select_research();
        let current = match self.app.research.current {
            Some(id) => id,
            None => return,
        };
        let tech = crate::research::tree::get_tech(current);

        let mut progress: u32 = 0;
        for (_e, (kind, stock)) in self.app.world.query_mut::<(&EntityKind, &mut LabStock)>() {
            let spec = match crate::research::labs::get_lab_spec(kind.kind) {
                Some(s) => s,
                None => continue,
            };
            let accepts_all = tech
                .science_cost
                .iter()
                .all(|(r, _)| spec.accepted_packs.contains(r));
            let has_all = tech.science_cost.iter().all(|(r, _)| stock.get(*r) > 0);
            if !accepts_all || !has_all {
                continue;
            }
            for (r, _) in &tech.science_cost {
                stock.take(*r);
            }
            progress += (Self::PACK_PROGRESS as f64 * spec.speed_multiplier) as u32;
        }

        if progress > 0 {
            if let Some(done) = self.app.research.tick(progress) {
                let done_tech = crate::research::tree::get_tech(done);
                self.app.economy.credit(done_tech.cash_grant as i64);
                self.app.status_message = format!(
                    "Research complete: {} (+${})",
                    done_tech.name, done_tech.cash_grant
                );
                self.refresh_unlocked_recipes();
                self.auto_select_research();
            }
        }
    }

    /// Feed this tick's emissions (recipe waste + generator smoke) into the
    /// global pollution state; scrubber units reduce at the source.
    fn pollution_tick(&mut self) {
        let source = self.app.simulation.last_report.pollution;
        let mut scrubbers = 0u32;
        for (_e, kind) in self.app.world.query::<&EntityKind>().iter() {
            if kind.kind == EntityType::ScrubberUnit {
                scrubbers += 1;
            }
        }
        crate::waste::pollution::update_pollution(&mut self.app.pollution, source, 0, scrubbers);
    }

    /// Snapshot of the factory for cycle-expense computation.
    fn factory_snapshot(&self) -> crate::economy::expenses::FactorySnapshot {
        use crate::ecs::components::{Position, Processing};
        let built_tiles = self.app.world.query::<&Position>().iter().count() as u64;
        let operating_machines = self.app.world.query::<&Processing>().iter().count() as u64;
        crate::economy::expenses::FactorySnapshot {
            mw_consumed: self.app.simulation.last_report.power_demand,
            waste_disposed: Vec::new(),
            pollution_level: self.app.pollution.level,
            pollution_threshold: 200.0,
            fine_rate: 1.0,
            built_tiles,
            operating_machines,
            co2_vented: 0,
            active_trains: 0,
            active_trucks: 0,
            active_drones: 0,
            active_planes: 0,
        }
    }

    /// Every 60 ticks: market drift, contract lifecycle, upkeep expenses,
    /// scaling, and (freeplay) contract generation with auto-accept.
    fn economy_cycle(&mut self) {
        let tick = self.app.simulation.tick_count;
        self.app.economy.advance_cycle();
        self.app.market.update(tick);

        // Contract lifecycle.
        self.app.contract_board.check_deadlines(tick);
        let reward = self.app.contract_board.check_completions(tick);
        if reward > 0 {
            self.app.economy.credit(reward);
            self.app.status_message = format!("Contract complete! +${}", reward);
        }

        // Upkeep expenses (zero at Tutorial difficulty, i.e. campaign).
        let (interest, _missed) = self.app.loans.update_cycle(&mut self.app.economy);
        let snapshot = self.factory_snapshot();
        let report = crate::economy::expenses::compute_cycle_expenses(
            &snapshot,
            &self.app.economy,
            &self.app.regulations,
            interest,
        );
        let total = report.total.round() as i64;
        if total > 0 {
            self.app.economy.deduct(total);
        }
        self.app.last_expense_report = report;
        self.app.income_last_cycle = self.app.income_this_cycle;
        self.app.income_this_cycle = 0.0;

        // No bankruptcy game-over: clamp at zero with a warning instead.
        if self.app.economy.cash < 0 {
            self.app.economy.cash = 0;
            self.app.economy.bankruptcy_counter = 0;
            self.app.status_message =
                "WARNING: expenses exceeded cash — treasury drained to $0".to_string();
            self.app.status_error = true;
        }

        // Scaling ratchets with completed contracts / net worth / time.
        let completed = self.app.contract_board.completed_count as u32;
        self.app
            .scaling
            .check_scaling_increase(&self.app.economy, completed);

        // Contract generation (freeplay only). Contracts are accepted by
        // the player in the :contracts popup (j/k + Enter); as a safety net
        // the first eligible contract auto-accepts only while the player has
        // never accepted one manually, so the loop demos itself once.
        if self.app.game_mode == GameMode::Freeplay
            && self.app.contract_board.refresh_available(tick)
        {
            let tier = crate::contracts::generation::tier_for_scaling(self.app.scaling.level);
            let produced: Vec<crate::resources::Resource> = self
                .app
                .delivered_lifetime
                .keys()
                .copied()
                .filter(|r| !r.is_waste())
                .collect();
            for _ in 0..3 {
                if self.app.contract_board.available.len()
                    >= crate::contracts::board::MAX_AVAILABLE
                {
                    break;
                }
                let id = self.app.contract_board.next_id();
                let contract = crate::contracts::generation::generate_contract(
                    tier,
                    self.app.scaling.level,
                    &produced,
                    tick.wrapping_add(id),
                    id,
                );
                self.app.contract_board.add_available(contract);
            }
            let never_engaged = self.app.contract_board.active.is_empty()
                && self.app.contract_board.completed_count == 0
                && self.app.contract_board.failed_count == 0;
            if never_engaged {
                let auto_id: Option<u64> = self
                    .app
                    .contract_board
                    .available
                    .iter()
                    .find(|c| {
                        c.requirements.iter().all(|req| {
                            self.app.delivered_lifetime.contains_key(&req.resource)
                        })
                    })
                    .map(|c| c.id);
                if let Some(id) = auto_id {
                    self.app.contract_board.accept(id, tick);
                    self.app.status_message =
                        "First contract accepted — :contracts to manage them".to_string();
                }
            }
        }
    }

    /// Check whether the current tutorial level is complete (including custom
    /// conditions that need world access). Marks it complete when it is.
    fn check_tutorial_completion(app: &AppState, tut: &mut TutorialState) -> bool {
        let mut total_widgets: u64 = 0;
        let mut total_ingots: u64 = 0;
        let mut total_ore: u64 = 0;

        for (_e, counter) in app.world.query::<&OutputCounter>().iter() {
            total_widgets += counter.widget_count();
            total_ingots += counter.ingot_count();
            total_ore += counter.ore_count();
        }

        let done = tut.check_completion(total_ore, total_ingots, total_widgets, (0, 0))
            || Self::check_custom_completion(app, tut);

        if done {
            tut.complete_level();
            return true;
        }
        false
    }

    /// Evaluate Custom completion conditions that need world/map access.
    fn check_custom_completion(app: &AppState, tut: &TutorialState) -> bool {
        let cfg = match config::get_level(tut.current_level) {
            Some(c) => c,
            None => return false,
        };
        let name = match &cfg.completion {
            config::CompletionCondition::Custom(n) => n.as_str(),
            _ => return false,
        };
        match name {
            // Level 11: every belt on the map faces Right.
            "all_conveyors_facing_right" => {
                let mut any = false;
                for (_e, (kind, facing)) in app
                    .world
                    .query::<(&EntityKind, &FacingComponent)>()
                    .iter()
                {
                    if kind.kind == EntityType::BasicBelt {
                        any = true;
                        if facing.facing != Facing::Right {
                            return false;
                        }
                    }
                }
                any
            }
            // Level 9: every output bin on the map has received at least one item.
            "all_5_clusters_producing" => {
                let mut bins = 0usize;
                let mut producing = 0usize;
                for (_e, counter) in app.world.query::<&OutputCounter>().iter() {
                    bins += 1;
                    if counter.total() >= 1 {
                        producing += 1;
                    }
                }
                bins > 0 && bins == producing
            }
            _ => false,
        }
    }

    // -------------------------------------------------------------------
    // Save / load
    // -------------------------------------------------------------------

    pub fn build_save_data(&self) -> save::SaveData {
        use crate::ecs::components::*;
        let app = &self.app;
        let mut entities = Vec::new();
        let mut resources = Vec::new();

        for (entity, (pos, kind)) in app.world.query::<(&Position, &EntityKind)>().iter() {
            let facing = app
                .world
                .get::<&FacingComponent>(entity)
                .ok()
                .map(|f| f.facing)
                .unwrap_or(Facing::Right);

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
                output_counts_ext: counter.as_ref().map(|c| c.counts.clone()),
                splitter_state: splitter.as_ref().map(|s| s.next_output),
                merger_state: merger.as_ref().map(|s| s.priority),
                player_placed: player,
            });
        }

        for y in 0..app.map.height {
            for x in 0..app.map.width {
                if let Some(resource) = app.map.resource_at(x, y) {
                    resources.push(save::SavedResource { x, y, resource });
                }
            }
        }

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
            marks: self
                .input
                .marks
                .list()
                .into_iter()
                .map(|(c, x, y)| (c, (x, y)))
                .collect(),
            score: save::ScoreData {
                total_widgets,
                total_ingots,
                total_ore,
            },
            simulation_speed: app.simulation.speed,
            tick_count: app.simulation.tick_count,
            inventory: app.inventory.clone(),
            tutorial_state: self.tutorial.as_ref().map(|t| save::TutorialSaveState {
                current_level: t.current_level,
                levels_completed: t.levels_completed.clone(),
                commands_learned: Vec::new(),
            }),
            economy_cash: app.economy.cash,
            economy_difficulty: Some(app.economy.difficulty.name().to_string()),
            scaling_level: app.scaling.level,
            day_tick: app.day_tick,
            research_completed: app
                .research
                .completed
                .iter()
                .map(|t| format!("{:?}", t))
                .collect(),
            game_mode: Some(
                match app.game_mode {
                    GameMode::Tutorial => "Tutorial",
                    GameMode::Campaign => "Campaign",
                    GameMode::Freeplay => "Freeplay",
                }
                .to_string(),
            ),
        }
    }

    pub fn load_save_data(&mut self, data: &save::SaveData) {
        let app = &mut self.app;
        app.world = hecs::World::new();
        app.map = crate::map::grid::Map::new(data.map_width, data.map_height);

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

        app.economy.cash = data.economy_cash;

        // Restore completed research (saved as TechId Debug strings) and
        // recompute the unlocked-recipe set from it.
        app.research = crate::research::tree::ResearchState::new();
        for tech in crate::research::tree::get_all_techs() {
            if data
                .research_completed
                .iter()
                .any(|name| name == &format!("{:?}", tech.id))
            {
                app.research.completed.insert(tech.id);
            }
        }
        app.scaling.level = data.scaling_level;
        app.day_tick = data.day_tick;
        app.game_mode = match data.game_mode.as_deref() {
            Some("Campaign") => GameMode::Campaign,
            Some("Freeplay") => GameMode::Freeplay,
            _ => GameMode::Tutorial,
        };
        app.simulation.config.freeplay_power = app.game_mode == GameMode::Freeplay;

        app.overlay = None;

        // Restore tutorial progress
        if let Some(ref ts) = data.tutorial_state {
            let mut tut =
                TutorialState::new_with_progress(ts.current_level, ts.levels_completed.clone());
            tut.current_level = ts.current_level;
            self.tutorial = Some(tut);
            self.app.current_level = Some(ts.current_level);
            self.app.saved_level = Some(ts.current_level);
            for l in &ts.levels_completed {
                if !self.app.campaign_completed.contains(l) {
                    self.app.campaign_completed.push(*l);
                }
            }
            self.app.campaign_completed.sort_unstable();
            self.app.show_tutorial = true;
            self.app.freeplay_unlocked = self
                .tutorial
                .as_ref()
                .map(|t| t.is_freeplay_unlocked())
                .unwrap_or(false);
        } else {
            self.tutorial = None;
            self.app.show_tutorial = false;
        }
        self.refresh_unlocked_recipes();
        self.resize_viewport();
    }

    // -------------------------------------------------------------------
    // Headless testing helpers
    // -------------------------------------------------------------------

    /// Feed a string of keys in vim notation. Special keys use angle
    /// brackets: `<Esc>`, `<CR>`, `<Tab>`, `<BS>`, `<Space>`,
    /// `<Up>/<Down>/<Left>/<Right>`, `<Home>/<End>`, `<C-x>` (ctrl),
    /// `<S-x>` (shift). Uppercase letters automatically get SHIFT.
    pub fn feed_keys(&mut self, keys: &str) {
        for key in parse_key_notation(keys) {
            if self.handle_key(key) {
                return;
            }
        }
    }

    /// Sum of all output-bin counters as (ore, ingots, widgets, total).
    pub fn output_totals(&self) -> (u64, u64, u64, u64) {
        let mut ore = 0;
        let mut ingots = 0;
        let mut widgets = 0;
        let mut total = 0;
        for (_e, c) in self.app.world.query::<&OutputCounter>().iter() {
            ore += c.ore_count();
            ingots += c.ingot_count();
            widgets += c.widget_count();
            total += c.total();
        }
        (ore, ingots, widgets, total)
    }

    /// Entity type at a map position, if any.
    pub fn entity_type_at(&self, x: usize, y: usize) -> Option<EntityType> {
        self.app.map.entity_type_at(&self.app.world, x, y)
    }

    /// Current campaign level number (if in a level).
    pub fn current_level(&self) -> Option<usize> {
        self.tutorial.as_ref().map(|t| t.current_level)
    }

    pub fn cursor(&self) -> (usize, usize) {
        (self.input.cursor_x, self.input.cursor_y)
    }

    pub fn mode(&self) -> Mode {
        self.app.mode.clone()
    }
}

/// Parse a vim-style key notation string into crossterm KeyEvents.
pub fn parse_key_notation(s: &str) -> Vec<KeyEvent> {
    let mut out = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '<' {
            // Find closing '>'
            if let Some(close) = chars[i..].iter().position(|&c| c == '>') {
                let token: String = chars[i + 1..i + close].iter().collect();
                if let Some(key) = parse_special_token(&token) {
                    out.push(key);
                    i += close + 1;
                    continue;
                }
            }
            // Literal '<'
            out.push(KeyEvent::new(KeyCode::Char('<'), KeyModifiers::NONE));
            i += 1;
        } else {
            out.push(char_to_key(chars[i]));
            i += 1;
        }
    }
    out
}

fn parse_special_token(token: &str) -> Option<KeyEvent> {
    let lower = token.to_ascii_lowercase();
    match lower.as_str() {
        "esc" => Some(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
        "cr" | "enter" | "return" => Some(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        "tab" => Some(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)),
        "bs" | "backspace" => Some(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)),
        "space" => Some(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE)),
        "up" => Some(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)),
        "down" => Some(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
        "left" => Some(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)),
        "right" => Some(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE)),
        "home" => Some(KeyEvent::new(KeyCode::Home, KeyModifiers::NONE)),
        "end" => Some(KeyEvent::new(KeyCode::End, KeyModifiers::NONE)),
        _ => {
            // <C-x> / <S-x> forms (case of x preserved from original token)
            let orig: Vec<char> = token.chars().collect();
            if orig.len() == 3 && orig[1] == '-' {
                let target = orig[2];
                match orig[0] {
                    'C' | 'c' => Some(KeyEvent::new(
                        KeyCode::Char(target.to_ascii_lowercase()),
                        KeyModifiers::CONTROL,
                    )),
                    'S' | 's' => Some(KeyEvent::new(
                        KeyCode::Char(target.to_ascii_uppercase()),
                        KeyModifiers::SHIFT,
                    )),
                    'A' | 'a' | 'M' | 'm' => Some(KeyEvent::new(
                        KeyCode::Char(target),
                        KeyModifiers::ALT,
                    )),
                    _ => None,
                }
            } else {
                None
            }
        }
    }
}

fn char_to_key(c: char) -> KeyEvent {
    if c.is_ascii_uppercase() {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::SHIFT)
    } else {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
    }
}
