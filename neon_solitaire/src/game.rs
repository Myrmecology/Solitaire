use crate::card::{Card, Rank, create_standard_deck};
use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Debug, Clone)]
pub struct GameState {
    pub tableau: Vec<Vec<Card>>,  // 7 columns of cards
    pub stock: Vec<Card>,          // Draw pile (face down)
    pub waste: Vec<Card>,          // Cards drawn from stock (face up)
    pub foundations: Vec<Vec<Card>>, // 4 piles for each suit (Ace to King)
    pub selected_card: Option<(PileType, usize, usize)>, // What's currently selected
    pub move_count: u32,
    pub score: i32,
    pub undo_stack: Vec<GameState>,
    pub draw_count: usize,        // How many cards to draw (1 or 3)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PileType {
    Tableau,
    Stock,
    Waste,
    Foundation,
}

impl GameState {
    pub fn new() -> Self {
        let mut deck = create_standard_deck();
        deck.shuffle(&mut thread_rng());
        
        let mut game = GameState {
            tableau: vec![Vec::new(); 7],
            stock: Vec::new(),
            waste: Vec::new(),
            foundations: vec![Vec::new(); 4],
            selected_card: None,
            move_count: 0,
            score: 0,
            undo_stack: Vec::new(),
            draw_count: 3, // Default to draw 3
        };
        
        // Deal cards to tableau
        let mut deck_index = 0;
        for col in 0..7 {
            for row in 0..=col {
                let mut card = deck[deck_index];
                if row == col {
                    card.face_up = true; // Top card of each column is face up
                }
                game.tableau[col].push(card);
                deck_index += 1;
            }
        }
        
        // Remaining cards go to stock
        for i in deck_index..52 {
            game.stock.push(deck[i]);
        }
        
        game
    }
    
    pub fn draw_from_stock(&mut self) {
        self.save_undo_state();
        
        if self.stock.is_empty() {
            // Flip waste back to stock
            while let Some(mut card) = self.waste.pop() {
                card.face_up = false;
                self.stock.push(card);
            }
            self.score = (self.score - 20).max(0); // Penalty for recycling
        } else {
            // Draw cards from stock to waste
            let cards_to_draw = self.draw_count.min(self.stock.len());
            for _ in 0..cards_to_draw {
                if let Some(mut card) = self.stock.pop() {
                    card.face_up = true;
                    self.waste.push(card);
                }
            }
        }
        
        self.move_count += 1;
    }
    
    pub fn is_valid_tableau_move(&self, card: &Card, target_col: usize) -> bool {
        if self.tableau[target_col].is_empty() {
            // Only Kings can go on empty columns
            card.rank == Rank::King
        } else {
            let target_card = self.tableau[target_col].last().unwrap();
            card.can_stack_on(target_card)
        }
    }
    
    pub fn is_valid_foundation_move(&self, card: &Card, foundation_idx: usize) -> bool {
        if self.foundations[foundation_idx].is_empty() {
            // Only Aces can start a foundation
            card.rank == Rank::Ace
        } else {
            let top_card = self.foundations[foundation_idx].last().unwrap();
            // Must be same suit and one rank higher
            card.suit == top_card.suit && card.rank as u8 == top_card.rank as u8 + 1
        }
    }
    
    pub fn auto_move_to_foundation(&mut self) -> bool {
        let mut moved = false;
        
        // Check waste pile
        if let Some(card) = self.waste.last() {
            for f in 0..4 {
                if self.is_valid_foundation_move(card, f) {
                    self.save_undo_state();
                    let card = self.waste.pop().unwrap();
                    self.foundations[f].push(card);
                    self.score += 10;
                    moved = true;
                    break;
                }
            }
        }
        
        // Check tableau columns
        if !moved {
            for col in 0..7 {
                if !self.tableau[col].is_empty() {
                    if let Some(card) = self.tableau[col].last() {
                        if card.face_up {
                            for f in 0..4 {
                                if self.is_valid_foundation_move(card, f) {
                                    self.save_undo_state();
                                    let card = self.tableau[col].pop().unwrap();
                                    self.foundations[f].push(card);
                                    
                                    // Flip the new top card if needed
                                    if let Some(new_top) = self.tableau[col].last_mut() {
                                        if !new_top.face_up {
                                            new_top.face_up = true;
                                            self.score += 5;
                                        }
                                    }
                                    
                                    self.score += 10;
                                    moved = true;
                                    break;
                                }
                            }
                        }
                    }
                    if moved { break; }
                }
            }
        }
        
        self.move_count += if moved { 1 } else { 0 };
        moved
    }
    
    pub fn is_won(&self) -> bool {
        self.foundations.iter().all(|f| f.len() == 13)
    }
    
    pub fn save_undo_state(&mut self) {
        // Keep only last 100 states to avoid memory issues
        if self.undo_stack.len() >= 100 {
            self.undo_stack.remove(0);
        }
        
        let mut state_copy = self.clone();
        state_copy.undo_stack.clear(); // Don't store undo stack in undo stack
        self.undo_stack.push(state_copy);
    }
    
    pub fn undo(&mut self) -> bool {
        if let Some(previous_state) = self.undo_stack.pop() {
            let undo_stack = self.undo_stack.clone();
            *self = previous_state;
            self.undo_stack = undo_stack;
            true
        } else {
            false
        }
    }
    
    pub fn get_hint(&self) -> Option<String> {
        // Check for moves to foundation
        for col in 0..7 {
            if !self.tableau[col].is_empty() {
                if let Some(card) = self.tableau[col].last() {
                    if card.face_up {
                        for f in 0..4 {
                            if self.is_valid_foundation_move(card, f) {
                                return Some(format!("Move {} from column {} to foundation", card, col + 1));
                            }
                        }
                    }
                }
            }
        }
        
        // Check for tableau to tableau moves
        for from_col in 0..7 {
            if !self.tableau[from_col].is_empty() {
                // Find the lowest face-up card
                let mut from_idx = 0;
                for (i, card) in self.tableau[from_col].iter().enumerate() {
                    if card.face_up {
                        from_idx = i;
                        break;
                    }
                }
                
                let card = &self.tableau[from_col][from_idx];
                
                for to_col in 0..7 {
                    if from_col != to_col && self.is_valid_tableau_move(card, to_col) {
                        return Some(format!("Move {} from column {} to column {}", 
                                          card, from_col + 1, to_col + 1));
                    }
                }
            }
        }
        
        // Check waste pile
        if let Some(card) = self.waste.last() {
            for col in 0..7 {
                if self.is_valid_tableau_move(card, col) {
                    return Some(format!("Move {} from waste to column {}", card, col + 1));
                }
            }
        }
        
        if !self.stock.is_empty() || !self.waste.is_empty() {
            return Some("Draw from stock".to_string());
        }
        
        None
    }
}