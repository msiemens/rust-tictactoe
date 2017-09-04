#![feature(vec_remove_item)]

#[macro_use]
extern crate enum_display_derive;
extern crate rand;

use std::{char, thread, time};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;

mod game;
mod mcts;

const PLAYER_USER: game::Player = game::Player::X;
const PLAYER_AI: game::Player = game::Player::O;
const START_PLAYER: game::Player = PLAYER_USER;


fn main() {
    let mut board = game::Board::new(START_PLAYER);
    let ai_action = Arc::new(Mutex::new(None));
    let (tx, rx) = channel();

    let _ai_action = ai_action.clone();

    thread::spawn(move || {
        let mut ai = mcts::MCTS::new(PLAYER_AI, PLAYER_AI == START_PLAYER);

        loop {
            // Process player action
            if let Ok(action) = rx.try_recv() {
                ai.perform_action(action);
            }

            // Perform MCTS
            ai.run();

            // Update action
            let mut action = _ai_action.lock().unwrap();
            *action = ai.get_action();
        }
    });

    while !board.is_ended() {
        board.print();
        let next_player = board.next_player();

        println!("Turn: {}", next_player);
        let mut action;

        if next_player == PLAYER_USER {
            print!("Action [e.g. a1]: ");
            io::stdout().flush().unwrap();

            loop {
                // Read next player's action
                let mut player_action = String::new();
                io::stdin().read_line(&mut player_action).unwrap();

                let player_action = player_action.trim();

                if player_action.is_empty() {
                    print!("> ");
                    io::stdout().flush().unwrap();
                    continue;
                }

                // Validate input
                if player_action.len() != 2 {
                    println!("Invalid action");
                    print!("> ");
                    io::stdout().flush().unwrap();
                    continue;
                };

                let mut chars = player_action.chars();

                let row = chars.next().unwrap() as i32 - '1' as i32;
                let col = chars.next().unwrap() as i32 - 'a' as i32;

                action = (row, col);

                if !board.is_legal_action(action) {
                    println!("Illegal action");
                    print!("> ");
                    io::stdout().flush().unwrap();
                    continue;
                };

                break;
            }
        } else {
            thread::sleep(time::Duration::from_millis(100));

            action = ai_action.lock().unwrap().unwrap();
            println!(
                "AI action: {}{}",
                char::from_u32(action.0 as u32 + '1' as u32).unwrap(),
                char::from_u32(action.1 as u32 + 'a' as u32).unwrap()
            )
        }

        tx.send(action).unwrap();
        board.perform_action(action);

        println!();
    }

    if let Some(player) = board.get_winner() {
        println!("Winner is Player {}", player);
    } else {
        println!("Game ended with a draw");
    }
}