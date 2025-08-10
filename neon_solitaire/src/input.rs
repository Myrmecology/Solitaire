use crate::game::{GameState, PileType};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind},
    terminal,
};
use std::time::Duration;

pub enum InputAction {
    SelectColumn(usize),
    SelectWaste,
    DrawFromStock,
    SelectFoundation(usize),
    AutoMove,
    Undo,
    Hint,
    Quit,
    ToggleDrawCount,
    MouseClick(u16, u16),
    MouseDrag(u16, u16),
    None,
}

pub struct InputHandler {
    pub mouse_enabled: bool,
    pub drag_start: Option<(u16, u16)>,
    pub dragging: bool,
}

impl InputHandler {
    pub fn new() -> Self {
        // Enable mouse support
        let _ = terminal::enable_raw_mode();
        
        InputHandler {
            mouse_enabled: true,
            drag_start: None,
            dragging: false,
        }
    }

    pub fn poll_input(&mut self) -> InputAction {
        // Poll for events with a small timeout
        if event::poll(Duration::from_millis(50)).unwrap_or(false) {
            if let Ok(event) = event::read() {
                return self.handle_event(event);
            }
        }
        InputAction::None
    }

    fn handle_event(&mut self, event: Event) -> InputAction {
        match event {
            Event::Key(key_event) => self.handle_key(key_event),
            Event::Mouse(mouse_event) => self.handle_mouse(mouse_event),
            _ => InputAction::None,
        }
    }

    fn handle_key(&self, key: KeyEvent) -> InputAction {
        match key.code {
            // Number keys for columns
            KeyCode::Char('1') => InputAction::SelectColumn(0),
            KeyCode::Char('2') => InputAction::SelectColumn(1),
            KeyCode::Char('3') => InputAction::SelectColumn(2),
            KeyCode::Char('4') => InputAction::SelectColumn(3),
            KeyCode::Char('5') => InputAction::SelectColumn(4),
            KeyCode::Char('6') => InputAction::SelectColumn(5),
            KeyCode::Char('7') => InputAction::SelectColumn(6),
            
            // Other controls
            KeyCode::Char('w') | KeyCode::Char('W') => InputAction::SelectWaste,
            KeyCode::Char('s') | KeyCode::Char('S') => InputAction::DrawFromStock,
            KeyCode::Char(' ') => InputAction::DrawFromStock,
            KeyCode::Char('f') | KeyCode::Char('F') => InputAction::AutoMove,
            KeyCode::Char('a') | KeyCode::Char('A') => InputAction::AutoMove,
            KeyCode::Char('z') | KeyCode::Char('Z') => InputAction::Undo,
            KeyCode::Char('h') | KeyCode::Char('H') => InputAction::Hint,
            KeyCode::Char('d') | KeyCode::Char('D') => InputAction::ToggleDrawCount,
            KeyCode::Char('q') | KeyCode::Char('Q') => InputAction::Quit,
            KeyCode::Esc => InputAction::Quit,
            
            _ => InputAction::None,
        }
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) -> InputAction {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                self.drag_start = Some((mouse.column, mouse.row));
                self.dragging = true;
                InputAction::MouseClick(mouse.column, mouse.row)
            }
            MouseEventKind::Up(MouseButton::Left) => {
                if self.dragging {
                    self.dragging = false;
                    if let Some(start) = self.drag_start {
                        // If we didn't move much, treat it as a click
                        if (start.0 as i16 - mouse.column as i16).abs() < 3 &&
                           (start.1 as i16 - mouse.row as i16).abs() < 2 {
                            InputAction::MouseClick(mouse.column, mouse.row)
                        } else {
                            InputAction::MouseDrag(mouse.column, mouse.row)
                        }
                    } else {
                        InputAction::MouseClick(mouse.column, mouse.row)
                    }
                } else {
                    InputAction::MouseClick(mouse.column, mouse.row)
                }
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                if self.dragging {
                    InputAction::MouseDrag(mouse.column, mouse.row)
                } else {
                    InputAction::None
                }
            }
            _ => InputAction::None,
        }
    }

    pub fn cleanup(&self) {
        let _ = terminal::disable_raw_mode();
    }
}

pub fn convert_mouse_to_game_position(x: u16, y: u16, game: &GameState) -> Option<(PileType, usize, usize)> {
    // Stock area (roughly x: 2-10, y: 6)
    if y == 6 && x >= 2 && x <= 10 {
        return Some((PileType::Stock, 0, 0));
    }
    
    // Waste area (roughly x: 12-30, y: 6)
    if y == 6 && x >= 12 && x <= 30 {
        if !game.waste.is_empty() {
            return Some((PileType::Waste, 0, game.waste.len() - 1));
        }
    }
    
    // Foundation area (roughly x: 40-60, y: 6)
    if y == 6 && x >= 40 && x <= 60 {
        let foundation_idx = ((x - 40) / 5) as usize;
        if foundation_idx < 4 {
            return Some((PileType::Foundation, foundation_idx, 0));
        }
    }
    
    // Tableau area (columns start at x: 2, each column is 6 chars wide)
    if y >= 10 && x >= 2 {
        let col = ((x - 2) / 6) as usize;
        if col < 7 {
            let row = (y - 10) as usize;
            if row < game.tableau[col].len() {
                return Some((PileType::Tableau, col, row));
            }
        }
    }
    
    None
}

pub fn handle_game_action(game: &mut GameState, action: InputAction) -> bool {
    match action {
        InputAction::SelectColumn(col) => {
            if col < 7 {
                // If we have a selected card, try to move it here
                if let Some((pile_type, from_col, from_row)) = game.selected_card {
                    match pile_type {
                        PileType::Tableau => {
                            // Move cards from another tableau column
                            if from_col != col {
                                let cards_to_move: Vec<Card> = game.tableau[from_col]
                                    .drain(from_row..)
                                    .collect();
                                
                                if !cards_to_move.is_empty() && 
                                   game.is_valid_tableau_move(&cards_to_move[0], col) {
                                    game.save_undo_state();
                                    for card in cards_to_move {
                                        game.tableau[col].push(card);
                                    }
                                    
                                    // Flip new top card if needed
                                    if let Some(new_top) = game.tableau[from_col].last_mut() {
                                        if !new_top.face_up {
                                            new_top.face_up = true;
                                            game.score += 5;
                                        }
                                    }
                                    
                                    game.move_count += 1;
                                    game.score += 5;
                                } else {
                                    // Invalid move, put cards back
                                    for card in cards_to_move {
                                        game.tableau[from_col].push(card);
                                    }
                                }
                            }
                        }
                        PileType::Waste => {
                            // Move from waste to tableau
                            if let Some(card) = game.waste.last() {
                                if game.is_valid_tableau_move(card, col) {
                                    game.save_undo_state();
                                    let card = game.waste.pop().unwrap();
                                    game.tableau[col].push(card);
                                    game.move_count += 1;
                                    game.score += 5;
                                }
                            }
                        }
                        _ => {}
                    }
                    game.selected_card = None;
                } else {
                    // Select the top face-up card(s) in this column
                    if !game.tableau[col].is_empty() {
                        for i in (0..game.tableau[col].len()).rev() {
                            if game.tableau[col][i].face_up {
                                game.selected_card = Some((PileType::Tableau, col, i));
                                break;
                            }
                        }
                    }
                }
            }
        }
        InputAction::SelectWaste => {
            if !game.waste.is_empty() {
                game.selected_card = Some((PileType::Waste, 0, game.waste.len() - 1));
            }
        }
        InputAction::DrawFromStock => {
            game.draw_from_stock();
            game.selected_card = None;
        }
        InputAction::AutoMove => {
            game.auto_move_to_foundation();
        }
        InputAction::Undo => {
            game.undo();
        }
        InputAction::ToggleDrawCount => {
            game.draw_count = if game.draw_count == 1 { 3 } else { 1 };
        }
        InputAction::MouseClick(x, y) => {
            if let Some(position) = convert_mouse_to_game_position(x, y, game) {
                match position {
                    (PileType::Stock, _, _) => {
                        game.draw_from_stock();
                        game.selected_card = None;
                    }
                    _ => {
                        // Handle like column selection
                        if let Some(selected) = game.selected_card {
                            // Try to move selected card to clicked position
                            // This is simplified - you'd need more logic here
                            game.selected_card = None;
                        } else {
                            game.selected_card = Some(position);
                        }
                    }
                }
            }
        }
        InputAction::Quit => {
            return true;  // Signal to quit
        }
        _ => {}
    }
    
    false  // Don't quit
}