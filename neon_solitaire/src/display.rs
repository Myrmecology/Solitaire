use crate::game::{GameState, PileType};
use crate::card::Card;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType, EnableLineWrap, DisableLineWrap},
};
use std::io::{stdout, Write};

pub struct Display {
    pub selected_position: (usize, usize),
    pub hover_pile: Option<(PileType, usize, usize)>,
}

impl Display {
    pub fn new() -> Self {
        Display {
            selected_position: (0, 0),
            hover_pile: None,
        }
    }

    pub fn init_terminal(&self) -> std::io::Result<()> {
        execute!(
            stdout(),
            Clear(ClearType::All),
            Hide,
            DisableLineWrap,
            MoveTo(0, 0)
        )?;
        Ok(())
    }

    pub fn cleanup_terminal(&self) -> std::io::Result<()> {
        execute!(
            stdout(),
            Show,
            EnableLineWrap,
            ResetColor,
            Clear(ClearType::All),
            MoveTo(0, 0)
        )?;
        Ok(())
    }

    pub fn draw_game(&self, game: &GameState) -> std::io::Result<()> {
        // Move to top-left instead of clearing entire screen
        execute!(stdout(), MoveTo(0, 0))?;
        
        // Draw title
        self.draw_title()?;
        
        // Draw score and stats
        self.draw_stats(game)?;
        
        // Draw stock and waste
        self.draw_stock_waste(game)?;
        
        // Draw foundations
        self.draw_foundations(game)?;
        
        // Draw tableau
        self.draw_tableau(game)?;
        
        // Draw controls hint
        self.draw_controls()?;
        
        // Draw hint if available
        execute!(stdout(), MoveTo(0, 23))?;
        if let Some(hint) = game.get_hint() {
            execute!(
                stdout(),
                SetForegroundColor(Color::Rgb { r: 100, g: 255, b: 100 }),
                Print(format!("💡 Hint: {}                                        ", hint)),
                ResetColor
            )?;
        } else {
            execute!(
                stdout(),
                Print("                                                                      ")
            )?;
        }
        
        stdout().flush()?;
        Ok(())
    }

    fn draw_title(&self) -> std::io::Result<()> {
        execute!(
            stdout(),
            MoveTo(20, 0),
            SetForegroundColor(Color::Rgb { r: 255, g: 0, b: 255 }),
            Print("═══════════════════════════════════════"),
            MoveTo(20, 1),
            Print("      N E O N   S O L I T A I R E     "),
            MoveTo(20, 2),
            Print("═══════════════════════════════════════"),
            ResetColor
        )?;
        Ok(())
    }

    fn draw_stats(&self, game: &GameState) -> std::io::Result<()> {
        execute!(
            stdout(),
            MoveTo(2, 4),
            SetForegroundColor(Color::Rgb { r: 100, g: 200, b: 255 }),
            Print(format!("Score: {:4} ", game.score)),
            SetForegroundColor(Color::Rgb { r: 255, g: 200, b: 100 }),
            Print(format!("Moves: {:4} ", game.move_count)),
            SetForegroundColor(Color::Rgb { r: 200, g: 100, b: 255 }),
            Print(format!("Draw: {}     ", if game.draw_count == 1 { "1 card " } else { "3 cards" })),
            ResetColor
        )?;
        Ok(())
    }

    fn draw_stock_waste(&self, game: &GameState) -> std::io::Result<()> {
        execute!(stdout(), MoveTo(2, 6))?;
        
        // Draw stock
        execute!(
            stdout(),
            SetForegroundColor(Color::Rgb { r: 150, g: 150, b: 200 }),
            Print("Stock: "),
            ResetColor
        )?;
        
        if game.stock.is_empty() {
            execute!(
                stdout(),
                SetForegroundColor(Color::Rgb { r: 100, g: 100, b: 100 }),
                Print("[♻]  "),
                ResetColor
            )?;
        } else {
            execute!(
                stdout(),
                SetForegroundColor(Color::Rgb { r: 100, g: 100, b: 200 }),
                Print(format!("[{:2}] ", game.stock.len())),
                ResetColor
            )?;
        }
        
        // Draw waste
        execute!(
            stdout(),
            SetForegroundColor(Color::Rgb { r: 150, g: 150, b: 200 }),
            Print("Waste: "),
            ResetColor
        )?;
        
        if game.waste.is_empty() {
            execute!(
                stdout(),
                SetForegroundColor(Color::Rgb { r: 100, g: 100, b: 100 }),
                Print("[ ]          "),
                ResetColor
            )?;
        } else {
            let start = if game.waste.len() > 3 { game.waste.len() - 3 } else { 0 };
            for (i, card) in game.waste[start..].iter().enumerate() {
                let is_selected = game.selected_card == Some((PileType::Waste, 0, start + i));
                self.draw_card_compact(card, is_selected)?;
                execute!(stdout(), Print(" "))?;
            }
            // Clear any remaining space
            execute!(stdout(), Print("          "))?;
        }
        
        Ok(())
    }

    fn draw_foundations(&self, game: &GameState) -> std::io::Result<()> {
        execute!(stdout(), MoveTo(40, 6))?;
        
        execute!(
            stdout(),
            SetForegroundColor(Color::Rgb { r: 255, g: 200, b: 100 }),
            Print("Foundations: "),
            ResetColor
        )?;
        
        let suits = ["♥", "♦", "♣", "♠"];
        let colors = [
            Color::Rgb { r: 255, g: 50, b: 100 },   // Hearts - Neon Pink
            Color::Rgb { r: 100, g: 200, b: 255 },  // Diamonds - Neon Blue
            Color::Rgb { r: 150, g: 255, b: 150 },  // Clubs - Neon Green
            Color::Rgb { r: 255, g: 255, b: 100 },  // Spades - Neon Yellow
        ];
        
        for (i, foundation) in game.foundations.iter().enumerate() {
            if foundation.is_empty() {
                execute!(
                    stdout(),
                    SetForegroundColor(colors[i]),
                    Print(format!("[{}] ", suits[i])),
                    ResetColor
                )?;
            } else {
                let card = foundation.last().unwrap();
                self.draw_card_compact(card, false)?;
                execute!(stdout(), Print(" "))?;
            }
        }
        
        Ok(())
    }

    fn draw_tableau(&self, game: &GameState) -> std::io::Result<()> {
        // Column headers
        execute!(stdout(), MoveTo(2, 9))?;
        for i in 1..=7 {
            execute!(
                stdout(),
                SetForegroundColor(Color::Rgb { r: 200, g: 200, b: 255 }),
                Print(format!("  {}   ", i)),
                ResetColor
            )?;
        }
        
        // Find max column height
        let max_height = game.tableau.iter().map(|col| col.len()).max().unwrap_or(0);
        
        // Draw cards - add padding to clear old cards
        for row in 0..(max_height + 5) {
            execute!(stdout(), MoveTo(2, 10 + row as u16))?;
            
            if row < max_height {
                for col in 0..7 {
                    if row < game.tableau[col].len() {
                        let card = &game.tableau[col][row];
                        let is_selected = game.selected_card == Some((PileType::Tableau, col, row));
                        self.draw_card_compact(card, is_selected)?;
                    } else {
                        execute!(stdout(), Print("      "))?;
                    }
                }
            } else {
                // Clear remaining rows
                execute!(stdout(), Print("                                                  "))?;
            }
        }
        
        Ok(())
    }

    fn draw_card_compact(&self, card: &Card, selected: bool) -> std::io::Result<()> {
        if !card.face_up {
            if selected {
                execute!(
                    stdout(),
                    SetBackgroundColor(Color::Rgb { r: 100, g: 0, b: 100 }),
                    SetForegroundColor(Color::Rgb { r: 150, g: 150, b: 200 }),
                    Print("[??]"),
                    ResetColor
                )?;
            } else {
                execute!(
                    stdout(),
                    SetForegroundColor(Color::Rgb { r: 100, g: 100, b: 150 }),
                    Print("[??]"),
                    ResetColor
                )?;
            }
        } else {
            let rank_str = match card.rank {
                crate::card::Rank::Ace => "A ",
                crate::card::Rank::Two => "2 ",
                crate::card::Rank::Three => "3 ",
                crate::card::Rank::Four => "4 ",
                crate::card::Rank::Five => "5 ",
                crate::card::Rank::Six => "6 ",
                crate::card::Rank::Seven => "7 ",
                crate::card::Rank::Eight => "8 ",
                crate::card::Rank::Nine => "9 ",
                crate::card::Rank::Ten => "10",
                crate::card::Rank::Jack => "J ",
                crate::card::Rank::Queen => "Q ",
                crate::card::Rank::King => "K ",
            };
            
            let suit_char = match card.suit {
                crate::card::Suit::Hearts => "♥",
                crate::card::Suit::Diamonds => "♦",
                crate::card::Suit::Clubs => "♣",
                crate::card::Suit::Spades => "♠",
            };
            
            if selected {
                execute!(
                    stdout(),
                    SetBackgroundColor(Color::Rgb { r: 100, g: 0, b: 100 }),
                    SetForegroundColor(card.get_color()),
                    Print(format!("[{}{}]", rank_str, suit_char)),
                    ResetColor
                )?;
            } else {
                execute!(
                    stdout(),
                    SetForegroundColor(card.get_color()),
                    Print(format!("[{}{}]", rank_str, suit_char)),
                    ResetColor
                )?;
            }
        }
        
        Ok(())
    }

    fn draw_controls(&self) -> std::io::Result<()> {
        execute!(
            stdout(),
            MoveTo(0, 25),
            SetForegroundColor(Color::Rgb { r: 150, g: 150, b: 200 }),
            Print("═══════════════════════════════════════════════════════════════"),
            MoveTo(0, 26),
            Print("[1-7] Select Column | [W] Waste | [S] Stock | [F] Foundation  "),
            MoveTo(0, 27),
            Print("[Space] Draw | [Z] Undo | [H] Hint | [A] Auto | [Q] Quit     "),
            MoveTo(0, 28),
            Print("═══════════════════════════════════════════════════════════════"),
            ResetColor
        )?;
        
        Ok(())
    }

    pub fn draw_win_animation(&self) -> std::io::Result<()> {
        execute!(
            stdout(),
            Clear(ClearType::All),
            MoveTo(20, 10),
            SetForegroundColor(Color::Rgb { r: 255, g: 50, b: 255 }),
            Print("════════════════════════════════════"),
            MoveTo(20, 11),
            Print("    🎉  Y O U   W I N !  🎉        "),
            MoveTo(20, 12),
            Print("    N E O N   V I C T O R Y        "),
            MoveTo(20, 13),
            Print("════════════════════════════════════"),
            ResetColor
        )?;
        
        Ok(())
    }
}