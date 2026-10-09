use clap::{Parser, Subcommand};
use sueca::{Agent, GamePlay, HumanAgent, MCTSAgent, Player, RandomAgent};

#[derive(Parser)]
#[command(name = "sueca", version, about = "CLI Sueca Card Game")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start playing a game of Sueca
    Play {
        /// Choose player types with four numbers in a row (0: Human, 1: Random, 2: MCTS)
        #[arg(short, long, default_value_t = 222)]
        players: u16,
        /// Round wins required to win the game (default 5, max 255)
        #[arg(short = 't', long, default_value_t = 5)]
        points: u8,
        /// Exploration constant for MCTS (default 1.41)
        #[arg(short = 'c', long, default_value_t = 1.41)]
        mcts_const: f32,
        /// Iterations done per determinized world for MCTS (default 10000)
        #[arg(short = 'i', long, default_value_t = 10000)]
        mcts_iter: u32,
        /// Number of determinized worlds for MCTS (default 100, max 255)
        #[arg(short = 'w', long, default_value_t = 100)]
        mcts_worlds: u8,
        /// Run game silently without printing tricks and scores
        #[arg(short, long)]
        quiet: bool,
    },
}

fn create_agent(agent_type: u8, player: Player, c: f32, iter: u32, worlds: u8) -> Agent {
    match agent_type {
        0 => Agent::Human(HumanAgent {}),
        1 => Agent::Random(RandomAgent {}),
        2 => Agent::MCTS(Box::new(MCTSAgent::new(c, iter, worlds, player))),
        _ => Agent::MCTS(Box::new(MCTSAgent::new(c, iter, worlds, player)))
    }
}

fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Play { players, points, mcts_const, mcts_iter, mcts_worlds, quiet, } => {
            let seats: [u8; 4] = [((players / 1000) % 10) as u8, ((players / 100) % 10) as u8, 
            ((players / 10) % 10) as u8, (players % 10) as u8];
            let agents = [
                create_agent(seats[0], Player::North, mcts_const, mcts_iter, mcts_worlds),
                create_agent(seats[1], Player::East, mcts_const, mcts_iter, mcts_worlds),
                create_agent(seats[2], Player::South, mcts_const, mcts_iter, mcts_worlds),
                create_agent(seats[3], Player::West, mcts_const, mcts_iter, mcts_worlds)
            ];
            let mut game = GamePlay::new(points, agents);
            if quiet {
                game.play_game::<false>();
            } else {
                game.play_game::<true>();
            }
        }
    }
}