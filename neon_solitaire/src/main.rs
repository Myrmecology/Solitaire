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
    let mut needs_redraw = true;  // Only redraw when needed
    let mut last_move_count = 0;
    
    // Main game loop
    loop {
        // Only draw when something changed
        if needs_redraw || auto_completing {
            display.draw_game(&game)?;
            needs_redraw = false;
        }
        
        // Check for win
        if game.is_won() {
            display.draw_win_animation()?;
            thread::sleep(Duration::from_secs(3));
            break;
        }
        
        // Auto-complete mode
        if auto_completing {
            if last_draw.elapsed() > Duration::from_millis(200) {  // Slower animation
                if !auto_complete(&mut game) {
                    auto_completing = false;
                }
                last_draw = Instant::now();
                needs_redraw = true;
            }
        }
        
        // Handle input
        let action = input_handler.poll_input();
        
        match action {
            InputAction::None => {
                // No action, don't redraw
            }
            InputAction::MouseClick(x, y) | InputAction::MouseDrag(x, y) => {
                // Store state before handling action
                let old_selected = game.selected_card;
                let old_move_count = game.move_count;
                
                if handle_game_action(&mut game, InputAction::MouseClick(x, y)) {
                    break;  // Quit was confirmed
                }
                
                // Only redraw if something actually changed
                if old_selected != game.selected_card || old_move_count != game.move_count {
                    needs_redraw = true;
                }
            }
            InputAction::Quit => {
                if confirm_quit()? {
                    break;
                }
                needs_redraw = true;
            }
            InputAction::AutoMove => {
                // Try single auto-move first
                if !game.auto_move_to_foundation() {
                    // If no single move, try auto-complete
                    auto_completing = true;
                }
                needs_redraw = true;
            }
            InputAction::Hint => {
                needs_redraw = true;
            }
            _ => {
                // Store move count before action
                let old_move_count = game.move_count;
                
                if handle_game_action(&mut game, action) {
                    break;  // Quit was confirmed
                }
                
                // Only redraw if a move was made or selection changed
                if game.move_count != old_move_count {
                    needs_redraw = true;
                    last_move_count = game.move_count;
                } else if matches!(action, InputAction::SelectColumn(_) | 
                                          InputAction::SelectWaste | 
                                          InputAction::DrawFromStock |
                                          InputAction::Undo |
                                          InputAction::ToggleDrawCount) {
                    needs_redraw = true;
                }
            }
        }
        
        // Small delay to prevent CPU spinning
        thread::sleep(Duration::from_millis(20));
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
    
    let welcome = r#"
    ╔══════════════════════════════════════════════════════════╗
    ║                                                          ║
    ║     ███╗   ██╗███████╗ ██████╗ ███╗   ██╗              ║
    ║     ████╗  ██║██╔════╝██╔═══██╗████╗  ██║              ║
    ║     ██╔██╗ ██║█████╗  ██║   ██║██╔██╗ ██║              ║
    ║     ██║╚██╗██║██╔══╝  ██║   ██║██║╚██╗██║              ║
    ║     ██║ ╚████║███████╗╚██████╔╝██║ ╚████║              ║
    ║     ╚═╝  ╚═══╝╚══════╝ ╚═════╝ ╚═╝  ╚═══╝              ║
    ║                                                          ║
    ║         ███████╗ ██████╗ ██╗     ██╗████████╗          ║
    ║         ██╔════╝██╔═══██╗██║     ██║╚══██╔══╝          ║
    ║         ███████╗██║   ██║██║     ██║   ██║             ║
    ║         ╚════██║██║   ██║██║     ██║   ██║             ║
    ║         ███████║╚██████╔╝███████╗██║   ██║             ║
    ║         ╚══════╝ ╚═════╝ ╚══════╝╚═╝   ╚═╝             ║
    ║                                                          ║
    ║                  ♠ ♥ ♦ ♣                                ║
    ║                                                          ║
    ║              === HOW TO PLAY ===                        ║
    ║                                                          ║
    ║   • Build foundations from Ace to King by suit         ║
    ║   • Stack tableau cards in descending order            ║
    ║   • Alternate colors (red on black, black on red)      ║
    ║   • Press SPACE to draw cards                          ║
    ║   • Press 1-7 to select columns                        ║
    ║   • Press A for auto-move                              ║
    ║   • Click cards with mouse to move them                ║
    ║                                                          ║
    ║              Press any key to start...                  ║
    ║                                                          ║
    ╚══════════════════════════════════════════════════════════╝"#;
    
    execute!(
        stdout(),
        MoveTo(10, 3),
        SetForegroundColor(Color::Rgb { r: 255, g: 50, b: 255 }),
        Print(welcome),
        ResetColor
    )?;
    
    Ok(())
}

fn wait_for_keypress() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(_) = event::read()? {
                // Clear the screen completely before returning
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
    println!("\n╔════════════════════════════════════╗");
    println!("║         GAME STATISTICS            ║");
    println!("╠════════════════════════════════════╣");
    println!("║ Final Score: {:6}                ║", game.score);
    println!("║ Total Moves: {:6}                ║", game.move_count);
    println!("║ Status: {}            ║", if game.is_won() { "🏆 VICTORY!   " } else { "Game Ended    " });
    println!("╚════════════════════════════════════╝");
    println!("\nThanks for playing Neon Solitaire!");
    
    // Suggest improvements
    if !game.is_won() {
        println!("\n💡 Tips for next time:");
        println!("  • Try to uncover face-down cards early");
        println!("  • Empty columns are valuable - save them for Kings");
        println!("  • Use Undo (Z) to try different strategies");
        println!("  • Press H for hints when stuck");
    }
}