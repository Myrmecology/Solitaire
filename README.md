╝SOLITAIRE 

## 🚀 Quick Start

Work in progress

For a video demo, please check out: https://www.youtube.com/watch?v=eomSZTt5JJU

### Prerequisites

- **Rust** (1.70.0 or later)
- **Cargo** (comes with Rust)
- A terminal that supports:
  - Unicode characters
  - 256 colors or true color
  - Mouse input (optional but recommended)

### Installation & Running

```bash
# Clone the repository
git clone https://github.com/Myrmecology/Solitaire.git
cd neon_solitaire

# Build the project
cargo build --release

# Run the game
cargo run --release
For development mode with faster compilation:
bashcargo run
🎮 How to Play Solitaire
Objective
Move all 52 cards to the four foundation piles, organized by suit from Ace to King.
Game Layout
═══════════════════════════════════════
      N E O N   S O L I T A I R E     
═══════════════════════════════════════

Score: 0  Moves: 0  Draw: 3 cards

Stock: [24]  Waste: [??]    Foundations: [♥] [♦] [♣] [♠]

  1     2     3     4     5     6     7   <- Column Numbers
[??]  [??]  [??]  [??]  [??]  [??]  [??]   <- Face-down cards
      [5♦]  [J♣]  [2♠]  [K♥]  [8♣]  [A♠]   <- Face-up cards
            [10♥]  [A♦]        [7♦]
                   [K♣]
Game Areas

Stock Pile (Top Left)

Face-down cards you haven't drawn yet
Click or press Space to draw cards


Waste Pile (Next to Stock)

Cards drawn from the stock
Only the top card(s) can be played


Foundations (Top Right - ♥♦♣♠)

Four piles where you build suits from Ace to King
Start with Aces, then 2, 3, 4... up to King
Cards must be the same suit


Tableau (Seven Columns)

Main playing area with 7 columns
Build down in alternating colors (Red on Black, Black on Red)
Example: Black 10 → Red 9 → Black 8 → Red 7



Basic Rules
Building in the Tableau

Cards must be in descending order (King → Queen → Jack → 10 → 9...)
Cards must alternate colors:

Red cards (♥♦) can only go on Black cards (♣♠)
Black cards (♣♠) can only go on Red cards (♥♦)


You can move sequences of properly arranged cards together
Only Kings can be placed in empty columns

Building Foundations

Start with Aces
Build up by suit: A → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9 → 10 → J → Q → K
Once placed in foundation, cards cannot be moved back

Drawing from Stock

Click the stock pile or press Space to draw cards
Draw 1 or 3 cards at a time (press D to toggle)
When stock is empty, click it to flip the waste pile back

🎯 Controls
Keyboard Controls
KeyAction1-7Select/move to tableau columns 1-7SpaceDraw cards from stockWSelect the waste pileSDraw from stock (same as Space)AAuto-move (finds obvious moves to foundations)FForce move to foundationZUndo last moveHShow hint (suggests a valid move)DToggle draw count (1 or 3 cards)Q / EscQuit game
Mouse Controls

Click a card - Selects it (shows purple highlight)
Click destination - Moves selected card there (if valid)
Click stock pile - Draw new cards
Click foundation - Move selected card to foundation
Click outside - Deselect current card

🎨 Visual Features
Neon Card Colors

♥ Hearts - Neon Pink
♦ Diamonds - Neon Blue
♣ Clubs - Neon Green
♠ Spades - Neon Yellow

Card Display

[??] - Face-down card
[A♠] - Face-up Ace of Spades
[K♥] - Face-up King of Hearts
Purple Background - Currently selected card

💡 Strategy Tips
For Beginners

Always move Aces to foundations immediately (press A)
Uncover face-down cards as your first priority
Empty columns are valuable - save them for Kings
Use Undo (Z) liberally to try different approaches
Check the hint (H) when you're stuck

Advanced Strategy

Plan several moves ahead before committing
Don't auto-move everything - sometimes you need cards in tableau
Build long sequences to move multiple cards at once
Manage empty columns carefully - they're your flexibility
Consider the draw count - 1-card draw is easier than 3-card draw

🏆 Winning the Game
You win when:

All 52 cards are moved to the foundations
Each foundation has a complete suit (Ace through King)
The win animation plays automatically

Scoring System

+5 points - Moving cards within tableau
+5 points - Uncovering a face-down card
+10 points - Moving card to foundation
-20 points - Recycling the stock pile

🛠️ Technical Details
Built With

Rust 🦀 - Systems programming language
crossterm - Terminal manipulation and mouse support
rand - Card shuffling
serde - Game state serialization (for future save/load features)

Project Structure
neon_solitaire/
├── Cargo.toml          # Project dependencies
├── README.md           # This file
├── .gitignore          # Git ignore rules
└── src/
    ├── main.rs         # Game loop and initialization
    ├── card.rs         # Card structures and logic
    ├── game.rs         # Game state and rules
    ├── display.rs      # Terminal rendering
    ├── input.rs        # Keyboard and mouse handling
    └── moves.rs        # Move validation and execution
System Requirements

OS: Windows, macOS, Linux
Terminal: Any modern terminal with UTF-8 support
RAM: < 50 MB
Disk: < 10 MB

🐛 Troubleshooting
Cards not displaying correctly?

Ensure your terminal supports Unicode
Try a different terminal (Windows Terminal, iTerm2, Alacritty)

Colors look wrong?

Enable true color support in your terminal
Some terminals need COLORTERM=truecolor environment variable

Mouse not working?

Not all terminals support mouse input
Try Windows Terminal, iTerm2, or most Linux terminals
Keyboard controls always work as fallback

Game flickering?

Try running in release mode: cargo run --release
Ensure your terminal GPU acceleration is enabled

📝 License
This project is open source and available under the MIT License.
🤝 Contributing
Contributions are welcome! Feel free to:

Report bugs
Suggest new features
Submit pull requests
Improve documentation


Made with ❤️ and 🦀 by a Rust enthusiast.
Happy coding