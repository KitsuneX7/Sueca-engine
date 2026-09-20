use crate::gamestate::{GameState, Player, Suit};
use rand::prelude::*;
use rand::rngs::SmallRng;
use rayon::prelude::*;
use std::io::{self, Write};

#[derive(Copy, Clone, Debug)]
pub struct MCTSNode {
    pub card_idx: u8,
    pub player: Player,
    pub num_children: u8,
    pub first_child: u32,
    pub total_visits: u32,
    pub wins: f32,
}

impl MCTSNode {
    #[inline]
    pub fn new(c: u8, p: Player) -> Self {
        Self {
            card_idx: c,
            player: p,
            num_children: 0,
            first_child: u32::MAX,
            total_visits: 0,
            wins: 0.0,
        }
    }
}

#[derive(Debug)]
pub enum Agent {
    Human(HumanAgent),
    Random(RandomAgent),
    MCTS(Box<MCTSAgent>),
}

impl Agent {
    #[inline]
    pub fn select_card<const PRINT_ON: bool>(
        &mut self,
        state: &GameState,
        rng: &mut SmallRng,
    ) -> u8 {
        let legal_moves = state.legal_moves(state.current_player);
        match self {
            Agent::Human(agent) => agent.play_human::<PRINT_ON>(state, legal_moves),
            Agent::Random(agent) => agent.play_random(legal_moves, rng),
            Agent::MCTS(agent) => agent.play_mcts(state, legal_moves),
        }
    }
}

#[derive(Debug)]
pub struct HumanAgent {}

impl HumanAgent {
    #[inline]
    pub fn play_human<const PRINT_ON: bool>(&self, state: &GameState, legal_moves: u64) -> u8 {
        let mut buffer = String::new();
        loop {
            if PRINT_ON {
                state.print_table();
                state.print_hand(state.current_player);
                if state.lead_suit != Suit::NO_LEAD {
                    print!(
                        "Suit to follow: {} | ",
                        Suit::from_index(state.lead_suit as usize)
                    );
                }
                println!("Trump suit: {}", state.trump);
                println!("Input your desired move: ");
            }
            io::stdout().flush().unwrap();
            buffer.clear();
            io::stdin()
                .read_line(&mut buffer)
                .expect("Failed to read line");
            match buffer.trim().parse::<u8>() {
                Ok(choice) if choice < 40 && (legal_moves & (1u64 << choice)) != 0 => {
                    return choice;
                }
                Ok(choice) if choice < 40 => {
                    if PRINT_ON {
                        println!("Invalid move");
                        println!();
                    }
                }
                Ok(_) => {
                    if PRINT_ON {
                        println!("Invalid number");
                        println!();
                    }
                }
                Err(_) => {
                    if PRINT_ON {
                        println!("Invalid input");
                        println!();
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct RandomAgent {}

impl RandomAgent {
    #[inline]
    pub fn play_random(&self, legal_moves: u64, rng: &mut SmallRng) -> u8 {
        let count = legal_moves.count_ones();
        let target_idx = rng.random_range(0..count);
        let mut moves = legal_moves;
        for _ in 0..target_idx {
            moves &= moves - 1;
        }
        moves.trailing_zeros() as u8
    }
}

#[derive(Debug)]
pub struct MCTSAgent {
    pub arena: Vec<MCTSNode>,
    pub path: [u32; 40],
    pub path_len: usize,
    pub rng: SmallRng,
    pub c: f32,
    pub max_iter: u32,
    pub max_sims: u8,
    pub player: Player,
}

impl MCTSAgent {
    pub fn new(con: f32, i: u32, s: u8, p: Player) -> Self {
        Self {
            arena: Vec::with_capacity(0xFFFF),
            path: [u32::MAX; 40],
            path_len: 0,
            rng: SmallRng::from_os_rng(),
            c: con,
            max_iter: i,
            max_sims: s,
            player: p,
        }
    }

    #[inline]
    pub fn setup(&mut self, player: Player) {
        self.arena.push(MCTSNode::new(0xFF, player));
    }

    #[inline]
    fn apply_move_step(&self, state: &mut GameState, card: u8) {
        state.make_move(card);
        if state.is_trick_complete() {
            state.resolve_trick();
        } else {
            state.current_player = state.current_player.next();
        }
    }

    #[inline]
    fn calculate_ucb(&self, child_idx: usize, ln_parent_visits: f32) -> f32 {
        let child = &self.arena[child_idx];
        if child.total_visits == 0 {
            return f32::INFINITY;
        }
        let exploitation = child.wins / child.total_visits as f32;
        let exploration = self.c * (ln_parent_visits / child.total_visits as f32).sqrt();
        exploitation + exploration
    }

    #[inline]
    pub fn select_best_child_ucb(&self, parent_idx: usize) -> usize {
        let parent = &self.arena[parent_idx];
        let start = parent.first_child as usize;
        let end = start + parent.num_children as usize;
        let ln_parent_visits = (parent.total_visits as f32).ln();
        (start..end)
            .max_by(|&a_idx, &b_idx| {
                let ucb_a = self.calculate_ucb(a_idx, ln_parent_visits);
                let ucb_b = self.calculate_ucb(b_idx, ln_parent_visits);
                ucb_a.total_cmp(&ucb_b)
            })
            .expect("parent has at least one child")
    }

    pub fn selection(&mut self, state: &mut GameState) -> usize {
        self.path_len = 0;
        let mut node_idx = 0;
        self.path[self.path_len] = node_idx as u32;
        self.path_len += 1;
        while self.arena[node_idx].num_children > 0 {
            node_idx = self.select_best_child_ucb(node_idx);
            self.path[self.path_len] = node_idx as u32;
            self.path_len += 1;
            self.apply_move_step(state, self.arena[node_idx].card_idx);
        }
        node_idx
    }

    pub fn expansion(&mut self, parent_idx: usize, state: &GameState) {
        if state.number_of_cards_in_hands() == 0 {
            return;
        }
        let acting_player = state.current_player;
        let mut legal_moves = state.legal_moves(acting_player);
        let first_child = self.arena.len() as u32;
        let mut count = 0u8;
        while legal_moves > 0 {
            let card_idx = (63 - legal_moves.leading_zeros()) as u8;
            legal_moves ^= 1u64 << card_idx;
            self.arena.push(MCTSNode::new(card_idx, acting_player));
            count += 1;
        }
        let parent = &mut self.arena[parent_idx];
        parent.first_child = first_child;
        parent.num_children = count;
    }

    pub fn simulation(&mut self, state: &mut GameState) -> (bool, u8) {
        let (winning_team, points) = state.rollout_random(&mut self.rng);
        ((winning_team & 1) == (self.player as u8 & 1), points)
    }

    pub fn backpropagation(&mut self, win: bool, score: u8) {
        for &node_idx in &self.path[..self.path_len] {
            let node = &mut self.arena[node_idx as usize];
            node.total_visits += 1;
            node.wins += 0.5;
            if !(((node.player as u8 & 1) == (self.player as u8 & 1)) ^ win) {
                node.wins += score as f32 / 8.0;
            } else {
                node.wins -= score as f32 / 8.0;
            }
        }
    }

    pub fn solve_single_world(&mut self, state: &GameState) -> [u32; 40] {
        self.arena.clear();
        self.setup(self.player);
        for _ in 0..self.max_iter {
            let mut scratch_state = *state;
            let leaf_idx = self.selection(&mut scratch_state);
            if self.arena[leaf_idx].first_child == u32::MAX {
                self.expansion(leaf_idx, &scratch_state);
            }
            if self.arena[leaf_idx].num_children > 0 {
                let child_idx = self.arena[leaf_idx].first_child as usize;
                self.apply_move_step(&mut scratch_state, self.arena[child_idx].card_idx);
                self.path[self.path_len] = child_idx as u32;
                self.path_len += 1;
            }
            let (win, score) = self.simulation(&mut scratch_state);
            self.backpropagation(win, score);
        }
        let mut world_visits = [0u32; 40];
        let root = &self.arena[0];
        let start = root.first_child as usize;
        let end = start + root.num_children as usize;
        for idx in start..end {
            let child = &self.arena[idx];
            world_visits[child.card_idx as usize] = child.total_visits;
        }
        world_visits
    }

    pub fn play_mcts(&mut self, state: &GameState, legal_moves: u64) -> u8 {
        if legal_moves.count_ones() == 1 {
            return legal_moves.trailing_zeros() as u8;
        }
        let total_consensus: [u32; 40] = (0..self.max_sims)
            .into_par_iter()
            .map_init(
                || {
                    (
                        MCTSAgent::new(self.c, self.max_iter, 1, self.player),
                        SmallRng::from_os_rng(),
                    )
                },
                |(worker, rng), _| {
                    let world_state = state.generate_world(worker.player, rng);
                    worker.solve_single_world(&world_state)
                },
            )
            .reduce(
                || [0u32; 40],
                |mut acc, world_votes| {
                    for i in 0..40 {
                        acc[i] += world_votes[i];
                    }
                    acc
                },
            );
        let mut best_card = legal_moves.trailing_zeros() as u8;
        let mut max_visits = 0;
        let mut moves = legal_moves;
        while moves > 0 {
            let card_idx = (63 - moves.leading_zeros()) as u8;
            moves ^= 1u64 << card_idx;
            let visits = total_consensus[card_idx as usize];
            if visits >= max_visits {
                max_visits = visits;
                best_card = card_idx;
            }
        }
        best_card
    }
    
}
