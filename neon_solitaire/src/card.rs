use std::fmt;
use crossterm::style::{Color, Stylize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rank {
    Ace = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
    Ten = 10,
    Jack = 11,
    Queen = 12,
    King = 13,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
    pub face_up: bool,
}

impl Card {
    pub fn new(suit: Suit, rank: Rank) -> Self {
        Card {
            suit,
            rank,
            face_up: false,
        }
    }

    pub fn flip(&mut self) {
        self.face_up = !self.face_up;
    }

    pub fn is_red(&self) -> bool {
        matches!(self.suit, Suit::Hearts | Suit::Diamonds)
    }

    pub fn is_black(&self) -> bool {
        !self.is_red()
    }

    pub fn can_stack_on(&self, other: &Card) -> bool {
        // In Solitaire, you can stack a card on another if:
        // 1. The colors are different (red on black or black on red)
        // 2. This card's rank is one less than the other card
        self.is_red() != other.is_red() && self.rank as u8 == other.rank as u8 - 1
    }

    pub fn get_color(&self) -> Color {
        match self.suit {
            Suit::Hearts => Color::Rgb { r: 255, g: 50, b: 100 },   // Neon Pink
            Suit::Diamonds => Color::Rgb { r: 100, g: 200, b: 255 }, // Neon Blue
            Suit::Clubs => Color::Rgb { r: 150, g: 255, b: 150 },    // Neon Green
            Suit::Spades => Color::Rgb { r: 255, g: 255, b: 100 },   // Neon Yellow
        }
    }

    pub fn to_string_colored(&self) -> String {
        if !self.face_up {
            return format!("{}", "╭─────╮\n│ ??? │\n╰─────╯".with(Color::Rgb { r: 100, g: 100, b: 150 }));
        }

        let rank_str = match self.rank {
            Rank::Ace => "A ",
            Rank::Two => "2 ",
            Rank::Three => "3 ",
            Rank::Four => "4 ",
            Rank::Five => "5 ",
            Rank::Six => "6 ",
            Rank::Seven => "7 ",
            Rank::Eight => "8 ",
            Rank::Nine => "9 ",
            Rank::Ten => "10",
            Rank::Jack => "J ",
            Rank::Queen => "Q ",
            Rank::King => "K ",
        };

        let suit_char = match self.suit {
            Suit::Hearts => "♥",
            Suit::Diamonds => "♦",
            Suit::Clubs => "♣",
            Suit::Spades => "♠",
        };

        let color = self.get_color();
        
        format!(
            "{}\n{} {}\n{}",
            "╭─────╮".with(color),
            format!("│ {}", rank_str).with(color),
            format!("{}│", suit_char).with(color),
            "╰─────╯".with(color)
        )
    }

    pub fn to_string_compact(&self) -> String {
        if !self.face_up {
            return "[??]".to_string();
        }

        let rank_str = match self.rank {
            Rank::Ace => "A",
            Rank::Two => "2",
            Rank::Three => "3",
            Rank::Four => "4",
            Rank::Five => "5",
            Rank::Six => "6",
            Rank::Seven => "7",
            Rank::Eight => "8",
            Rank::Nine => "9",
            Rank::Ten => "10",
            Rank::Jack => "J",
            Rank::Queen => "Q",
            Rank::King => "K",
        };

        let suit_char = match self.suit {
            Suit::Hearts => "♥",
            Suit::Diamonds => "♦",
            Suit::Clubs => "♣",
            Suit::Spades => "♠",
        };

        format!("[{}{}]", rank_str, suit_char)
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_compact())
    }
}

pub fn create_standard_deck() -> Vec<Card> {
    let mut deck = Vec::with_capacity(52);
    
    for &suit in &[Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades] {
        for rank_val in 1..=13 {
            let rank = match rank_val {
                1 => Rank::Ace,
                2 => Rank::Two,
                3 => Rank::Three,
                4 => Rank::Four,
                5 => Rank::Five,
                6 => Rank::Six,
                7 => Rank::Seven,
                8 => Rank::Eight,
                9 => Rank::Nine,
                10 => Rank::Ten,
                11 => Rank::Jack,
                12 => Rank::Queen,
                13 => Rank::King,
                _ => unreachable!(),
            };
            deck.push(Card::new(suit, rank));
        }
    }
    
    deck
}