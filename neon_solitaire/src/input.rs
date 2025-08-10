use crate::card::Card;
use crate::game::{GameState, PileType};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind, EnableMouseCapture, DisableMouseCapture},
    terminal,
    execute,
};
use std::time::Duration;
use std::io::stdout;

#[derive(Copy, Clone, Debug)]
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
        let _ = terminal::enable_raw_mode();
        let _ = execute!(stdout(), EnableMouseCapture);
        
        InputHandler {
            mouse_enabled: true,
            drag_start: None,
            dragging: false,
        }
    }

    pub fn poll_input(&mut self) -> InputAction {
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
            KeyCode::Char('1') => InputAction::SelectColumn(0),
            KeyCode::Char('2') => InputAction::SelectColumn(1),
            KeyCode::Char('3') => InputAction::SelectColumn(2),
            KeyCode::Char('4') => InputAction::SelectColumn(3),
            KeyCode::Char('5') => InputAction::SelectColumn(4),
            KeyCode::Char('6') => InputAction::SelectColumn(5),
            KeyCode::Char('7') => InputAction::SelectColumn(6),
            
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
                InputAction::MouseClick(mouse.column, mouse.row)
            }
            _ => InputAction::None,
        }
    }

    pub fn cleanup(&self) {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(stdout(), DisableMouseCapture);
    }
}

pub fn convert_mouse_to_game_position(x: u16, y: u16, game: &GameState) -> Option<(PileType, usize, usize)> {
    // Stock area
    if y == 6 && x >= 9 && x <= 14 {
        return Some((PileType::Stock, 0, 0));
    }
    
    // Waste area
    if y == 6 && x >= 16 && x <= 35 {
        if !game.waste.is_empty() {
            return Some((PileType::Waste, 0, game.waste.len() - 1));
        }
    }
    
    // Foundation area
    if y == 6 && x >= 53 && x <= 70 {
        let foundation_idx = ((x - 53) / 5) as usize;
        if foundation_idx < 4 {
            return Some((PileType::Foundation, foundation_idx, 0));
        }
    }
    
    // Tableau area - FIXED: properly handle clicking on columns
    if y >= 10 && x >= 2 && x <= 44 {
        let col = ((x - 2) / 6) as usize;
        if col < 7 {
            let row = (y - 10) as usize;
            // If clicking on an empty column or beyond the cards, return the column with row 0
            if game.tableau[col].is_empty() || row >= game.tableau[col].len() {
                return Some((PileType::Tableau, col, game.tableau[col].len()));
            } else {
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
                if let Some((pile_type, from_col, from_row)) = game.selected_card {
                    // We have a selected card, try to move it to this column
                    match pile_type {
                        PileType::Tableau => {
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
                                    
                                    if let Some(new_top) = game.tableau[from_col].last_mut() {
                                        if !new_top.face_up {
                                            new_top.face_up = true;
                                            game.score += 5;
                                        }
                                    }
                                    
                                    game.move_count += 1;
                                    game.score += 5;
                                } else {
                                    for card in cards_to_move {
                                        game.tableau[from_col].push(card);
                                    }
                                }
                            }
                        }
                        PileType::Waste => {
                            if let Some(&card) = game.waste.last() {
                                if game.is_valid_tableau_move(&card, col) {
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
                    // No card selected, select one from this column
                    if !game.tableau[col].is_empty() {
                        // Find the first face-up card
                        for i in 0..game.tableau[col].len() {
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
                if game.selected_card == Some((PileType::Waste, 0, game.waste.len() - 1)) {
                    game.selected_card = None;
                } else {
                    game.selected_card = Some((PileType::Waste, 0, game.waste.len() - 1));
                }
            }
        }
        InputAction::DrawFromStock => {
            game.draw_from_stock();
            game.selected_card = None;
        }
        InputAction::AutoMove => {
            // Try auto-move to foundation first
            if !game.auto_move_to_foundation() {
                // If no foundation moves, try the hint move
                if let Some(hint) = game.get_hint() {
                    // Parse hint to execute it
                    if hint.contains("from column") && hint.contains("to column") {
                        // Extract column numbers from hint
                        let parts: Vec<&str> = hint.split_whitespace().collect();
                        if let Some(from_pos) = parts.iter().position(|&x| x == "column") {
                            if let Some(to_pos) = parts.iter().rposition(|&x| x == "column") {
                                if from_pos < parts.len() - 1 && to_pos < parts.len() - 1 {
                                    if let (Ok(from_col), Ok(to_col)) = (
                                        parts[from_pos + 1].parse::<usize>(),
                                        parts[to_pos + 1].parse::<usize>()
                                    ) {
                                        // Execute the hinted move
                                        let from_col = from_col - 1;  // Convert to 0-based
                                        let to_col = to_col - 1;
                                        
                                        if from_col < 7 && to_col < 7 && !game.tableau[from_col].is_empty() {
                                            // Find first face-up card
                                            for i in 0..game.tableau[from_col].len() {
                                                if game.tableau[from_col][i].face_up {
                                                    let cards_to_move: Vec<Card> = game.tableau[from_col]
                                                        .drain(i..)
                                                        .collect();
                                                    
                                                    if !cards_to_move.is_empty() && 
                                                       game.is_valid_tableau_move(&cards_to_move[0], to_col) {
                                                        game.save_undo_state();
                                                        for card in cards_to_move {
                                                            game.tableau[to_col].push(card);
                                                        }
                                                        
                                                        if let Some(new_top) = game.tableau[from_col].last_mut() {
                                                            if !new_top.face_up {
                                                                new_top.face_up = true;
                                                                game.score += 5;
                                                            }
                                                        }
                                                        
                                                        game.move_count += 1;
                                                        game.score += 5;
                                                    } else {
                                                        for card in cards_to_move {
                                                            game.tableau[from_col].push(card);
                                                        }
                                                    }
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        InputAction::Undo => {
            game.undo();
        }
        InputAction::ToggleDrawCount => {
            game.draw_count = if game.draw_count == 1 { 3 } else { 1 };
        }
        InputAction::MouseClick(x, y) => {
            if let Some(position) = convert_mouse_to_game_position(x, y, game) {
                match position.0 {
                    PileType::Stock => {
                        game.draw_from_stock();
                        game.selected_card = None;
                    }
                    PileType::Waste => {
                        if game.selected_card == Some((PileType::Waste, 0, game.waste.len() - 1)) {
                            game.selected_card = None;
                        } else {
                            game.selected_card = Some((PileType::Waste, 0, game.waste.len() - 1));
                        }
                    }
                    PileType::Tableau => {
                        let (_, col, clicked_row) = position;
                        
                        if let Some((from_pile, from_col, from_row)) = game.selected_card {
                            // We have a selected card, try to move it here
                            match from_pile {
                                PileType::Waste => {
                                    if let Some(&card) = game.waste.last() {
                                        if game.is_valid_tableau_move(&card, col) {
                                            game.save_undo_state();
                                            game.waste.pop();
                                            game.tableau[col].push(card);
                                            game.move_count += 1;
                                            game.score += 5;
                                        }
                                    }
                                }
                                PileType::Tableau if from_col != col => {
                                    // Move from one tableau column to another
                                    let cards_to_move: Vec<Card> = game.tableau[from_col]
                                        .drain(from_row..)
                                        .collect();
                                    
                                    if !cards_to_move.is_empty() && 
                                       game.is_valid_tableau_move(&cards_to_move[0], col) {
                                        game.save_undo_state();
                                        for card in cards_to_move {
                                            game.tableau[col].push(card);
                                        }
                                        
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
                                PileType::Tableau if from_col == col => {
                                    // Clicking on same column, just deselect
                                    game.selected_card = None;
                                }
                                _ => {}
                            }
                            game.selected_card = None;
                        } else {
                            // No card selected, select one if clicking on a face-up card
                            if clicked_row < game.tableau[col].len() && game.tableau[col][clicked_row].face_up {
                                // Find the first face-up card from this row upward
                                for i in 0..=clicked_row {
                                    if game.tableau[col][i].face_up {
                                        game.selected_card = Some((PileType::Tableau, col, i));
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    PileType::Foundation => {
                        let f_idx = position.1;
                        if let Some((from_pile, from_col, _)) = game.selected_card {
                            match from_pile {
                                PileType::Waste => {
                                    if let Some(&card) = game.waste.last() {
                                        if game.is_valid_foundation_move(&card, f_idx) {
                                            game.save_undo_state();
                                            let card = game.waste.pop().unwrap();
                                            game.foundations[f_idx].push(card);
                                            game.score += 10;
                                            game.move_count += 1;
                                        }
                                    }
                                }
                                PileType::Tableau => {
                                    if let Some(&card) = game.tableau[from_col].last() {
                                        if game.is_valid_foundation_move(&card, f_idx) {
                                            game.save_undo_state();
                                            let card = game.tableau[from_col].pop().unwrap();
                                            game.foundations[f_idx].push(card);
                                            
                                            if let Some(new_top) = game.tableau[from_col].last_mut() {
                                                if !new_top.face_up {
                                                    new_top.face_up = true;
                                                    game.score += 5;
                                                }
                                            }
                                            
                                            game.score += 10;
                                            game.move_count += 1;
                                        }
                                    }
                                }
                                _ => {}
                            }
                            game.selected_card = None;
                        }
                    }
                }
            } else {
                // Clicked outside, deselect
                game.selected_card = None;
            }
        }
        InputAction::Quit => {
            return true;
        }
        _ => {}
    }
    
    false
}