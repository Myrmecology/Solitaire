use crate::card::Card;
use crate::game::{GameState, PileType};

#[derive(Debug, Clone)]
pub struct Move {
    pub from: MoveLocation,
    pub to: MoveLocation,
    pub cards: Vec<Card>,
    pub score_change: i32,
    pub flipped_card: Option<(usize, Card)>,  // Column index and card that was flipped
}

#[derive(Debug, Clone, PartialEq)]
pub struct MoveLocation {
    pub pile_type: PileType,
    pub pile_index: usize,
    pub card_index: usize,
}

impl Move {
    pub fn new(from: MoveLocation, to: MoveLocation, cards: Vec<Card>) -> Self {
        Move {
            from,
            to,
            cards,
            score_change: 0,
            flipped_card: None,
        }
    }

    pub fn execute(&mut self, game: &mut GameState) -> bool {
        // Validate the move first
        if !self.is_valid(game) {
            return false;
        }

        game.save_undo_state();

        // Remove cards from source
        let cards_to_move = match self.from.pile_type {
            PileType::Tableau => {
                let col = self.from.pile_index;
                let from_idx = self.from.card_index;
                
                // Store if we need to flip a card
                if from_idx > 0 {
                    let card_below = game.tableau[col][from_idx - 1];
                    if !card_below.face_up {
                        self.flipped_card = Some((col, card_below));
                    }
                }
                
                game.tableau[col].drain(from_idx..).collect()
            }
            PileType::Waste => {
                vec![game.waste.pop().unwrap()]
            }
            PileType::Foundation => {
                vec![game.foundations[self.from.pile_index].pop().unwrap()]
            }
            _ => return false,
        };

        // Add cards to destination
        match self.to.pile_type {
            PileType::Tableau => {
                for card in cards_to_move {
                    game.tableau[self.to.pile_index].push(card);
                }
                self.score_change = 5;
            }
            PileType::Foundation => {
                for card in cards_to_move {
                    game.foundations[self.to.pile_index].push(card);
                }
                self.score_change = 10;
            }
            _ => return false,
        }

        // Flip card if needed
        if let Some((col, _)) = self.flipped_card {
            if let Some(card) = game.tableau[col].last_mut() {
                card.face_up = true;
                self.score_change += 5;
            }
        }

        game.score += self.score_change;
        game.move_count += 1;

        true
    }

    pub fn is_valid(&self, game: &GameState) -> bool {
        // Check source has cards
        let source_cards = match self.from.pile_type {
            PileType::Tableau => {
                let col = self.from.pile_index;
                if col >= 7 || self.from.card_index >= game.tableau[col].len() {
                    return false;
                }
                &game.tableau[col][self.from.card_index..]
            }
            PileType::Waste => {
                if game.waste.is_empty() {
                    return false;
                }
                std::slice::from_ref(game.waste.last().unwrap())
            }
            PileType::Foundation => {
                if self.from.pile_index >= 4 || game.foundations[self.from.pile_index].is_empty() {
                    return false;
                }
                std::slice::from_ref(game.foundations[self.from.pile_index].last().unwrap())
            }
            _ => return false,
        };

        if source_cards.is_empty() {
            return false;
        }

        // Check if all source cards are face up
        if !source_cards.iter().all(|c| c.face_up) {
            return false;
        }

        // Check destination validity
        match self.to.pile_type {
            PileType::Tableau => {
                let col = self.to.pile_index;
                if col >= 7 {
                    return false;
                }
                game.is_valid_tableau_move(&source_cards[0], col)
            }
            PileType::Foundation => {
                if source_cards.len() != 1 {
                    return false;  // Can only move one card to foundation
                }
                let foundation_idx = self.to.pile_index;
                if foundation_idx >= 4 {
                    return false;
                }
                game.is_valid_foundation_move(&source_cards[0], foundation_idx)
            }
            _ => false,
        }
    }
}

pub fn find_valid_moves(game: &GameState) -> Vec<Move> {
    let mut moves = Vec::new();

    // Waste to tableau/foundation
    if !game.waste.is_empty() {
        let from = MoveLocation {
            pile_type: PileType::Waste,
            pile_index: 0,
            card_index: game.waste.len() - 1,
        };

        // Try each tableau column
        for col in 0..7 {
            let to = MoveLocation {
                pile_type: PileType::Tableau,
                pile_index: col,
                card_index: game.tableau[col].len(),
            };
            let mv = Move::new(from.clone(), to, vec![*game.waste.last().unwrap()]);
            if mv.is_valid(game) {
                moves.push(mv);
            }
        }

        // Try each foundation
        for f in 0..4 {
            let to = MoveLocation {
                pile_type: PileType::Foundation,
                pile_index: f,
                card_index: game.foundations[f].len(),
            };
            let mv = Move::new(from.clone(), to, vec![*game.waste.last().unwrap()]);
            if mv.is_valid(game) {
                moves.push(mv);
            }
        }
    }

    // Tableau to tableau/foundation
    for from_col in 0..7 {
        if game.tableau[from_col].is_empty() {
            continue;
        }

        // Find all face-up sequences
        for from_idx in 0..game.tableau[from_col].len() {
            if !game.tableau[from_col][from_idx].face_up {
                continue;
            }

            let cards: Vec<Card> = game.tableau[from_col][from_idx..].to_vec();
            let from = MoveLocation {
                pile_type: PileType::Tableau,
                pile_index: from_col,
                card_index: from_idx,
            };

            // Try moving to other tableau columns
            for to_col in 0..7 {
                if from_col == to_col {
                    continue;
                }

                let to = MoveLocation {
                    pile_type: PileType::Tableau,
                    pile_index: to_col,
                    card_index: game.tableau[to_col].len(),
                };
                let mv = Move::new(from.clone(), to, cards.clone());
                if mv.is_valid(game) {
                    moves.push(mv);
                }
            }

            // Try moving single cards to foundations
            if cards.len() == 1 {
                for f in 0..4 {
                    let to = MoveLocation {
                        pile_type: PileType::Foundation,
                        pile_index: f,
                        card_index: game.foundations[f].len(),
                    };
                    let mv = Move::new(from.clone(), to, cards.clone());
                    if mv.is_valid(game) {
                        moves.push(mv);
                    }
                }
            }
        }
    }

    moves
}

pub fn find_best_move(game: &GameState) -> Option<Move> {
    let moves = find_valid_moves(game);
    
    // Prioritize moves to foundation
    for mv in &moves {
        if mv.to.pile_type == PileType::Foundation {
            return Some(mv.clone());
        }
    }
    
    // Then moves that reveal cards
    for mv in &moves {
        if mv.from.pile_type == PileType::Tableau {
            let col = mv.from.pile_index;
            if mv.from.card_index > 0 && !game.tableau[col][mv.from.card_index - 1].face_up {
                return Some(mv.clone());
            }
        }
    }
    
    // Then any tableau to tableau move
    for mv in &moves {
        if mv.from.pile_type == PileType::Tableau && mv.to.pile_type == PileType::Tableau {
            return Some(mv.clone());
        }
    }
    
    // Finally, waste to tableau
    for mv in &moves {
        if mv.from.pile_type == PileType::Waste {
            return Some(mv.clone());
        }
    }
    
    None
}

pub fn auto_complete(game: &mut GameState) -> bool {
    let mut moves_made = false;
    let mut attempts = 0;
    const MAX_ATTEMPTS: u32 = 100;
    
    while attempts < MAX_ATTEMPTS {
        let mut made_move = false;
        
        // Try to move any card to foundation
        for col in 0..7 {
            if !game.tableau[col].is_empty() {
                if let Some(card) = game.tableau[col].last() {
                    if card.face_up {
                        for f in 0..4 {
                            if game.is_valid_foundation_move(card, f) {
                                game.save_undo_state();
                                let card = game.tableau[col].pop().unwrap();
                                game.foundations[f].push(card);
                                
                                // Flip new top card if needed
                                if let Some(new_top) = game.tableau[col].last_mut() {
                                    if !new_top.face_up {
                                        new_top.face_up = true;
                                    }
                                }
                                
                                game.score += 10;
                                game.move_count += 1;
                                made_move = true;
                                moves_made = true;
                                break;
                            }
                        }
                    }
                }
                if made_move { break; }
            }
        }
        
        // Try waste pile
        if !made_move && !game.waste.is_empty() {
            if let Some(card) = game.waste.last() {
                for f in 0..4 {
                    if game.is_valid_foundation_move(card, f) {
                        game.save_undo_state();
                        let card = game.waste.pop().unwrap();
                        game.foundations[f].push(card);
                        game.score += 10;
                        game.move_count += 1;
                        made_move = true;
                        moves_made = true;
                        break;
                    }
                }
            }
        }
        
        if !made_move {
            break;
        }
        
        attempts += 1;
    }
    
    moves_made
}