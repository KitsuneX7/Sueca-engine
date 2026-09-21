use crate::agent::Agent;
use crate::gamestate::{Card, GameState, Suit};
use rand::SeedableRng;
use rand::prelude::*;
use rand::rngs::SmallRng;
use std::io::{self, Write};
use std::thread::sleep;
use std::time::{Duration, Instant};

pub struct GamePlay {
    pub round: u8,
    pub to_win: u8,
    pub wins: [u8; 2],
    pub gamestate: GameState,
    pub rng: SmallRng,
    pub agents: [Agent; 4],
}

impl GamePlay {

    const BASE_DECK: [u8; 40] = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39,
    ];

    pub fn new(to_win: u8, agents: [Agent; 4]) -> Self {
        Self { to_win, round: 0, wins: [0, 0], gamestate: GameState::new(), agents, rng: SmallRng::from_os_rng() }
    }

    #[inline]
    fn clear_screen() {
        print!("\x1B[2J\x1B[1;1H");
        let _ = io::stdout().flush();
    }

    #[inline]
    fn pause(msg: &str) {
        print!("{msg}");
        let _ = io::stdout().flush();
        let mut dummy = String::new();
        let _ = io::stdin().read_line(&mut dummy);
    }

    #[inline]
    pub fn new_deck(&mut self) -> [u8; 40] {
        let mut deck = Self::BASE_DECK;
        deck.shuffle(&mut self.rng);
        deck
    }

    #[inline]
    pub fn setup(&mut self) -> Card {
        let deck = self.new_deck();
        let trump_card = if self.round == 1 { self.rng.random_range(0..40) } else {
            let prev_floor = self.gamestate.dealer.next() as u8 * 10;
            self.rng.random_range(prev_floor..prev_floor + 10)
        };
        self.gamestate.setup(deck, trump_card);
        Card::from_index(deck[trump_card as usize] as usize)
    }

    #[inline]
    pub fn update_scores<const PRINT_ON: bool>(&mut self) {
        let (team_who_won, score) = self.gamestate.round_score();
        self.wins[team_who_won as usize] += score;
        if PRINT_ON {
            println!("Team wins so far:");
            println!("Team North/South: {}", self.wins[0]);
            println!("Team East/West: {}", self.wins[1]);
        }
    }

    #[inline]
    pub fn play_trick<const PRINT_ON: bool>(&mut self) {
        if PRINT_ON {
            Self::clear_screen();
            println!("----- Trick {} -----", self.gamestate.trick());
            println!("{} will play first.", self.gamestate.current_player);
            println!();
        }
        for _ in 0..4 {
            let mut start = None;
            if PRINT_ON {
                println!("--- {}'s turn ---", self.gamestate.current_player);
                start = Some(Instant::now());
            }
            let choice = self.agents[self.gamestate.current_player as usize]
                .select_card::<PRINT_ON>(&self.gamestate, &mut self.rng);
            self.gamestate.make_move(choice);
            if PRINT_ON {
                let min_delay = Duration::from_millis(100);
                let elapsed = start.unwrap().elapsed();
                if elapsed < min_delay {
                    sleep(min_delay - elapsed);
                }
                println!(
                    "{} plays {}{}",
                    self.gamestate.current_player,
                    Card::from_index(choice as usize),
                    Suit::from_index(choice as usize)
                );
                println!();
            }
            self.gamestate.current_player = self.gamestate.current_player.next();
        }
        if PRINT_ON {
            println!("--- Trick {} results ---", self.gamestate.trick() - 1);
            self.gamestate.print_table();
        }
        self.gamestate.resolve_trick();
        if PRINT_ON {
            println!("{} wins this trick", self.gamestate.current_player);
            self.gamestate.print_scores();
            Self::pause("Press Enter to continue...");
        }
    }

    #[inline]
    pub fn play_round<const PRINT_ON: bool>(&mut self) {
        if PRINT_ON { Self::clear_screen(); }
        let card = self.setup();
        if PRINT_ON {
            println!("------- Round {} start -------", self.round);
            println!(
                "{} reveals trump card: {}{} | lead suit is {}",
                self.gamestate.current_player.prev(),
                card,
                self.gamestate.trump,
                self.gamestate.trump
            );
            Self::pause("Press Enter to start playing...");
        }
        for _ in 0..10 { self.play_trick::<PRINT_ON>(); }
        if PRINT_ON {
            Self::clear_screen();
            println!("Round {} over", self.round);
            println!();
            self.gamestate.print_scores();
        }
        let (team_who_won, score) = self.gamestate.round_score();
        self.wins[team_who_won as usize] += score;
        if PRINT_ON {
            let team_name = if team_who_won & 1 == 0 { "North/South" } else { "East/West" };
            let win_or_wins = if score == 1 { "win" } else { "wins" };
            println!("Team {team_name} gets {score} {win_or_wins}");
            println!("Team wins so far:");
            println!("Team North/South: {}", self.wins[0]);
            println!("Team East/West: {}", self.wins[1]);
            println!();
            Self::pause("Press Enter to continue to next round...");
        }
    }

    pub fn play_game<const PRINT_ON: bool>(&mut self) {
        if PRINT_ON {
            println!("--------- Game start ---------");
            println!();
        }
        while self.wins[0] < self.to_win && self.wins[1] < self.to_win {
            self.round += 1;
            self.play_round::<PRINT_ON>();
        }
        if PRINT_ON {
            let winner_team = if self.wins[0] >= self.to_win {
                "North/South"
            } else {
                "East/West"
            };
            println!("Game over");
            println!("Team {winner_team} won the match!");
        }
        self.gamestate.clear();
        self.round = 0;
        self.to_win = 0;
        self.wins[0] = 0;
        self.wins[1] = 0;
    }
}
