mod card;
mod game;
mod display;
mod input;
mod moves;

use game::GameState;
use display::Display;
use input::{InputHandler, InputAction, handle_game_action};
use moves::auto_complete;
use crossterm::{
    execute,
    terminal::{self, Clear, ClearType},
    cursor::{MoveTo, Show, Hide},
    style::{Color, Print, SetForegroundColor, ResetColor},
    event::{self, Event, KeyCode},
};
use std::io::stdout;
use std::time::{Duration, Instant};
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize terminal and display
    let display = Display::new();
    let mut input_handler = InputHandler::new();
    
    // Set up panic handler to clean up terminal on crash
    std::panic::set_hook(Box::new(|_| {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(stdout(), Show, ResetColor, Clear(ClearType::All));
    }));
    
    // Initialize terminal
    terminal::enable_raw_mode()?;
    display.init_terminal()?;
    
    // Show welcome screen and WAIT for key press
    show_welcome_screen()?;
    wait_for_keypress()?;
    
    // Create new game
    let mut game = GameState::new();
    let mut last_draw = Instant::now();
    let mut auto_completing = false;
    let mut force_redraw = true;
    
    // Initial draw
    display.draw_game(&game)?;
    
    // Main game loop
    loop {
        // Check for win
        if game.is_won() && !auto_completing {
            display.draw_win_animation()?;
            thread::sleep(Duration::from_secs(3));
            break;
        }
        
        // Auto-complete mode
        if auto_completing {
            if last_draw.elapsed() > Duration::from_millis(200) {
                if !auto_complete(&mut game) {
                    auto_completing = false;
                }
                display.draw_game(&game)?;
                last_draw = Instant::now();
            }
        }
        
        // Handle input
        let action = input_handler.poll_input();
        
        // Store state before action
        let old_selected = game.selected_card;
        let old_moves = game.move_count;
        let old_score = game.score;
        let old_waste_len = game.waste.len();
        let old_stock_len = game.stock.len();
        
        let should_quit = match action {
            InputAction::None => false,
            InputAction::Quit => {
                if confirm_quit()? {
                    break;
                }
                force_redraw = true;
                false
            }
            InputAction::AutoMove => {
                if !game.auto_move_to_foundation() {
                    auto_completing = true;
                }
                force_redraw = true;
                false
            }
            _ => handle_game_action(&mut game, action)
        };
        
        if should_quit {
            break;
        }
        
        // Only redraw if something changed
        if force_redraw || 
           old_selected != game.selected_card ||
           old_moves != game.move_count ||
           old_score != game.score ||
           old_waste_len != game.waste.len() ||
           old_stock_len != game.stock.len() {
            display.draw_game(&game)?;
            force_redraw = false;
        }
        
        // Small delay to prevent CPU spinning
        thread::sleep(Duration::from_millis(10));
    }
    
    // Cleanup
    display.cleanup_terminal()?;
    input_handler.cleanup();
    terminal::disable_raw_mode()?;
    
    // Show final stats
    show_final_stats(&game);
    
    Ok(())
}

fn show_welcome_screen() -> Result<(), Box<dyn std::error::Error>> {
    execute!(
        stdout(),
        Clear(ClearType::All),
        Hide,
        MoveTo(0, 0)
    )?;
    
    let lines = vec![
        "",
        "     ███╗   ██╗███████╗ ██████╗ ███╗   ██╗",
        "     ████╗  ██║██╔════╝██╔═══██╗████╗  ██║",
        "     ██╔██╗ ██║█████╗  ██║   ██║██╔██╗ ██║",
        "     ██║╚██╗██║██╔══╝  ██║   ██║██║╚██╗██║",
        "     ██║ ╚████║███████╗╚██████╔╝██║ ╚████║",
        "     ╚═╝  ╚═══╝╚══════╝ ╚═════╝ ╚═╝  ╚═══╝",
        "",
        "         ███████╗ ██████╗ ██╗     ██╗████████╗",
        "         ██╔════╝██╔═══██╗██║     ██║╚══██╔══╝",
        "         ███████╗██║   ██║██║     ██║   ██║",
        "         ╚════██║██║   ██║██║     ██║   ██║",
        "         ███████║╚██████╔╝███████╗██║   ██║",
        "         ╚══════╝ ╚═════╝ ╚══════╝╚═╝   ╚═╝",
        "",
        "                  ♠ ♥ ♦ ♣",
        "",
        "              === HOW TO PLAY ===",
        "",
        "   • Build foundations from Ace to King by suit",
        "   • Stack tableau cards in descending order",
        "   • Alternate colors (red on black, black on red)",
        "   • Click cards to select, click again to move",
        "   • Press SPACE to draw cards",
        "   • Press A for auto-move",
        "",
        "              Press any key to start...",
    ];
    
    for (i, line) in lines.iter().enumerate() {
        execute!(
            stdout(),
            MoveTo(10, 3 + i as u16),
            SetForegroundColor(Color::Rgb { r: 255, g: 50, b: 255 }),
            Print(line),
            ResetColor
        )?;
    }
    
    Ok(())
}

fn wait_for_keypress() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(_) = event::read()? {
                execute!(
                    stdout(),
                    Clear(ClearType::All),
                    MoveTo(0, 0)
                )?;
                break;
            }
        }
    }
    Ok(())
}

fn confirm_quit() -> Result<bool, Box<dyn std::error::Error>> {
    execute!(
        stdout(),
        Clear(ClearType::All),
        MoveTo(20, 10),
        SetForegroundColor(Color::Rgb { r: 255, g: 200, b: 100 }),
        Print("Are you sure you want to quit?"),
        MoveTo(20, 12),
        Print("[Y] Yes, quit the game"),
        MoveTo(20, 13),
        Print("[N] No, keep playing"),
        ResetColor
    )?;
    
    loop {
        if let Ok(event) = event::read() {
            if let Event::Key(key) = event {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => return Ok(true),
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => return Ok(false),
                    _ => {}
                }
            }
        }
    }
}

fn show_final_stats(game: &GameState) {
    println!("\n════════════════════════════════════════");
    println!("         GAME STATISTICS");
    println!("════════════════════════════════════════");
    println!(" Final Score: {}", game.score);
    println!(" Total Moves: {}", game.move_count);
    println!(" Status: {}", if game.is_won() { "🏆 VICTORY!" } else { "Game Ended" });
    println!("════════════════════════════════════════");
    println!("\nThanks for playing Neon Solitaire!");
    
    if !game.is_won() {
        println!("\n💡 Tips for next time:");
        println!("  • Try to uncover face-down cards early");
        println!("  • Empty columns are valuable - save them for Kings");
        println!("  • Use Undo (Z) to try different strategies");
        println!("  • Press H for hints when stuck");
    }
}