use sueca::{Agent, GamePlay, HumanAgent, MCTSAgent, Player};

fn main() {
    let agents = [
        Agent::Human(HumanAgent {}),
        Agent::MCTS(Box::new(MCTSAgent::new(1.41, 10000, 100, Player::East))),
        Agent::MCTS(Box::new(MCTSAgent::new(1.41, 10000, 100, Player::South))),
        Agent::MCTS(Box::new(MCTSAgent::new(1.41, 10000, 100, Player::West))),
    ];
    let mut game = GamePlay::new(4, agents);
    game.play_game::<true>();
}
