use rand::prelude::*;
use rand::rngs::SmallRng;
use std::fmt;

#[derive(Copy, Clone, Debug)]
pub struct UndoToken {
    pub card: u8,
    pub player: Player,
    pub lead_suit: u8,
    pub previous_trick: [u8; 4],
    pub previous_score: u8,
    pub previous_voids: u16,
    pub trick_completed: bool,
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Card {
    Two = 0,
    Three = 1,
    Four = 2,
    Five = 3,
    Six = 4,
    Queen = 5,
    Jack = 6,
    King = 7,
    Seven = 8,
    Ace = 9,
}

impl Card {
    pub const CARDS: [Card; 10] = [
        Card::Two,
        Card::Three,
        Card::Four,
        Card::Five,
        Card::Six,
        Card::Queen,
        Card::Jack,
        Card::King,
        Card::Seven,
        Card::Ace,
    ];

    #[inline(always)]
    pub const fn from_index(idx: usize) -> Self {
        Self::CARDS[((idx >> 2) % 10) & 0xF]
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            Self::Two => '2',
            Self::Three => '3',
            Self::Four => '4',
            Self::Five => '5',
            Self::Six => '6',
            Self::Queen => 'Q',
            Self::Jack => 'J',
            Self::King => 'K',
            Self::Seven => '7',
            Self::Ace => 'A',
        };
        write!(f, "{symbol}")
    }
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Player {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

impl Player {
    pub const PLAYERS: [Player; 4] = [Player::North, Player::East, Player::South, Player::West];

    #[inline(always)]
    pub const fn from_index(idx: usize) -> Self {
        Self::PLAYERS[idx & 3]
    }

    #[inline]
    pub const fn next(self) -> Self {
        Self::from_index((self as usize + 3) & 3)
    }

    #[inline]
    pub const fn partner(self) -> Self {
        Self::from_index((self as usize + 2) & 3)
    }

    #[inline]
    pub const fn prev(self) -> Self {
        Self::from_index((self as usize + 1) & 3)
    }

    /// Team 0: North/South (even), Team 1: East/West (odd)
    #[inline]
    pub const fn team(self) -> bool {
        (self as u8) & 1 != 0
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::North => "North",
            Self::East => "East",
            Self::South => "South",
            Self::West => "West",
        };
        write!(f, "{name}")
    }
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Suit {
    Clubs = 0,
    Spades = 1,
    Diamonds = 2,
    Hearts = 3,
}

impl Suit {
    pub const SUITS: [Suit; 4] = [Suit::Clubs, Suit::Spades, Suit::Diamonds, Suit::Hearts];
    pub const NO_LEAD: u8 = 4;

    #[inline(always)]
    pub const fn from_index(idx: usize) -> Self {
        Self::SUITS[idx & 3]
    }
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            Self::Clubs => "♣",
            Self::Spades => "♠",
            Self::Diamonds => "♦",
            Self::Hearts => "♥",
        };
        write!(f, "{symbol}")
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct GameState {
    pub current_player: Player,
    pub four: u8,
    pub score: u8,
    pub trump: Suit,
    pub current_trick: [u8; 4],
    pub player_hands: [u64; 4],
    pub voids: u16,
    pub lead_suit: u8,
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameState {
    pub const DECK_MASK: u64 = 0x0000_00FF_FFFF_FFFF; // 40 cards total (bits 0..40)
    // 10 cards per suit, spaced every 4 bits: 0b0001 repeated 10 times
    pub const SUIT_STRIDE: u64 = 0x0000_0011_1111_1111;
    pub const POINT_VALUES: [u8; 5] = [2, 3, 4, 10, 11];
    pub const RANK_MASKS: [u64; 5] = [
        0xF << (5 << 2), // Queens (Rank 5)
        0xF << (6 << 2), // Jacks  (Rank 6)
        0xF << (7 << 2), // Kings  (Rank 7)
        0xF << (8 << 2), // 7s     (Rank 8)
        0xF << (9 << 2), // Aces   (Rank 9)
    ];
    const SCORE_LUT: [(u8, u8); 121] = {
        let mut table = [(0, 0); 121];
        let mut s = 0;
        while s <= 120 {
            table[s] = if s == 0 {
                (1, 4)
            } else if s <= 30 {
                (1, 2)
            } else if s < 60 {
                (1, 1)
            } else if s == 60 {
                (0, 0)
            } else if s < 90 {
                (0, 1)
            } else if s < 120 {
                (0, 2)
            } else {
                (0, 4)
            };
            s += 1;
        }
        table
    };

    pub fn new() -> Self {
        GameState {
            current_player: Player::North,
            current_trick: [0xFF, 0xFF, 0xFF, 0xFF],
            four: 0,
            lead_suit: Suit::NO_LEAD,
            player_hands: [0, 0, 0, 0],
            score: 0,
            trump: Suit::Clubs,
            voids: 0,
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.current_player = Player::North;
        for i in self.current_trick.iter_mut() { *i = 0xFF; }
        self.lead_suit = Suit::NO_LEAD;
        for i in self.player_hands.iter_mut() { *i = 0; }
        self.trump = Suit::Clubs;
        self.score = 0;
    }

    #[inline]
    pub fn setup(&mut self, deck: [u8; 40], trump_card: u8) {
        self.distribute_hands(deck);
        for i in self.current_trick.iter_mut() { *i = 0xFF; }
        self.lead_suit = Suit::NO_LEAD;
        self.score = 0;
        self.voids = 0;
        self.current_player = Player::from_index((trump_card / 10) as usize).next();
        self.trump = Suit::from_index(deck[trump_card as usize] as usize);
    }

    #[inline]
    pub fn score(&self, team: bool) -> u8 {
        if team { self.score } else { self.points_played() - self.score }
    }

    #[inline]
    pub fn trick(&self) -> u8 {
        1 + (self.number_of_cards_played() >> 2)
    }

    #[inline]
    pub fn distribute_hands(&mut self, deck: [u8; 40]) {
        for i in 0..4 {
            for &card in &deck[i * 10..(i + 1) * 10] {
                self.player_hands[i] |= 1 << card;
            }
        }
    }

    #[inline]
    pub fn is_void(&self, player: Player, suit: Suit) -> bool {
        (self.voids >> (((player as u8) << 2) + suit as u8)) & 1 == 1
    }

    // --- Getting Legal Moves ---

    #[inline]
    pub fn legal_moves(&self, player: Player) -> u64 {
        let player_hand = self.player_hands[player as usize];
        if self.lead_suit != Suit::NO_LEAD {
            let matching = self.suits_in_hand(player, Suit::from_index(self.lead_suit as usize));
            if matching != 0 { matching } else { player_hand }
        } else {
            player_hand
        }
    }

    // --- Global Hands Queries ---

    #[inline]
    pub fn cards_in_hands(&self) -> u64 {
        self.player_hands.iter().fold(0, |acc, &hand| acc | hand)
    }

    #[inline]
    pub fn suits_in_hands(&self, suit: Suit) -> u64 {
        self.cards_in_hands() & (Self::SUIT_STRIDE << (suit as u8))
    }

    #[inline]
    pub fn trumps_in_hands(&self) -> u64 {
        self.suits_in_hands(self.trump)
    }

    #[inline]
    pub fn number_of_cards_in_hands(&self) -> u8 {
        self.cards_in_hands().count_ones() as u8
    }

    #[inline]
    pub fn number_of_suits_in_hands(&self, suit: Suit) -> u8 {
        self.suits_in_hands(suit).count_ones() as u8
    }

    #[inline]
    pub fn number_of_trumps_in_hands(&self) -> u8 {
        self.number_of_suits_in_hands(self.trump)
    }

    // --- Single-Player Hand Queries ---

    #[inline]
    pub fn cards_in_hand(&self, player: Player) -> u64 {
        self.player_hands[player as usize]
    }

    #[inline]
    pub fn suits_in_hand(&self, player: Player, suit: Suit) -> u64 {
        self.cards_in_hand(player) & (Self::SUIT_STRIDE << (suit as u8))
    }

    #[inline]
    pub fn trumps_in_hand(&self, player: Player) -> u64 {
        self.suits_in_hand(player, self.trump)
    }

    #[inline]
    pub fn number_of_cards_in_hand(&self, player: Player) -> u8 {
        self.cards_in_hand(player).count_ones() as u8
    }

    #[inline]
    pub fn number_of_suits_in_hand(&self, player: Player, suit: Suit) -> u8 {
        self.suits_in_hand(player, suit).count_ones() as u8
    }

    #[inline]
    pub fn number_of_trumps_in_hand(&self, player: Player) -> u8 {
        self.trumps_in_hand(player).count_ones() as u8
    }

    // --- Played Cards Queries ---

    #[inline]
    pub fn cards_played(&self) -> u64 {
        !self.cards_in_hands() & Self::DECK_MASK
    }

    #[inline]
    pub fn suits_played(&self, suit: Suit) -> u64 {
        !self.suits_in_hands(suit) & (Self::SUIT_STRIDE << (suit as u8))
    }

    #[inline]
    pub fn trumps_played(&self) -> u64 {
        self.suits_played(self.trump)
    }

    #[inline]
    pub fn number_of_cards_played(&self) -> u8 {
        40 - self.number_of_cards_in_hands()
    }

    #[inline]
    pub fn number_of_suits_played(&self, suit: Suit) -> u8 {
        10 - self.number_of_suits_in_hands(suit)
    }

    #[inline]
    pub fn number_of_trumps_played(&self) -> u8 {
        self.number_of_suits_played(self.trump)
    }

    // --- Point Counting And Wins ---

    #[inline]
    pub fn evaluate_points(cards: u64) -> u8 {
        Self::RANK_MASKS
            .iter()
            .zip(Self::POINT_VALUES)
            .map(|(&mask, pts)| (cards & mask).count_ones() as u8 * pts)
            .sum()
    }

    #[inline]
    pub fn points_in_hands(&self) -> u8 {
        Self::evaluate_points(self.cards_in_hands())
    }

    #[inline]
    pub fn points_played(&self) -> u8 {
        120 - self.points_in_hands()
    }

    #[inline]
    pub fn points_in_hand(&self, player: Player) -> u8 {
        Self::evaluate_points(self.cards_in_hand(player))
    }

    #[inline]
    pub fn points_in_trick(&self) -> u8 {
        let trick_mask = self
            .current_trick
            .iter()
            .filter(|&&card| card != 0xFF)
            .fold(0u64, |acc, &card| acc | (1u64 << card));
        Self::evaluate_points(trick_mask)
    }

    #[inline]
    pub fn get_winner(&self, trump_played: bool) -> Player {
        let mut player = self.current_player;
        let mut best_idx = player as usize;
        let mut best_val = 0xFF;
        for _ in 0..4 {
            if ((trump_played
                && Suit::from_index(self.current_trick[player as usize] as usize) == self.trump)
                || (!trump_played
                    && Suit::from_index(self.current_trick[player as usize] as usize) as u8
                        == self.lead_suit))
                && (best_val == 0xFF || self.current_trick[player as usize] >= best_val)
            {
                best_val = self.current_trick[player as usize];
                best_idx = player as usize;
            }
            player = player.next();
        }
        Player::from_index(best_idx)
    }

    // --- Move management ---

    pub fn make_move(&mut self, card: u8) {
        let player = self.current_player;
        let p_idx = player as usize;
        let suit = Suit::from_index(card as usize);
        if self.lead_suit != Suit::NO_LEAD {
            if self.lead_suit != suit as u8 {
                self.voids |= 1 << ((p_idx << 2) + (self.lead_suit as usize));
            }
        } else {
            self.lead_suit = suit as u8;
        }
        self.player_hands[p_idx] &= !(1u64 << card);
        self.current_trick[p_idx] = card;
    }

    pub fn resolve_trick(&mut self) {
        let trump_played = self
            .current_trick
            .iter()
            .any(|&c| Suit::from_index(c as usize) == self.trump);
        let winner = self.get_winner(trump_played);
        let points = self.points_in_trick();
        if winner == Player::North || winner == Player::South {
            self.score += points;
        }
        self.current_player = winner;
        self.lead_suit = Suit::NO_LEAD;
        self.current_trick = [0xFF, 0xFF, 0xFF, 0xFF];
    }

    #[inline]
    pub fn round_score(&self) -> (u8, u8) {
        let (team, mut pts) = Self::SCORE_LUT[self.score as usize];
        if pts == 4
            && ((self.score == 120 && self.four != 1) || (self.score == 0 && self.four != 2))
        {
            pts = 2;
        }
        (team, pts)
    }

    // --- MCTS world generation ---

    #[inline]
    fn setup_residual(&self, player: Player) -> [[u8; 9]; 9] {
        let suit_counts = [
            self.number_of_suits_in_hands(Suit::Clubs)
                - self.number_of_suits_in_hand(player, Suit::Clubs),
            self.number_of_suits_in_hands(Suit::Spades)
                - self.number_of_suits_in_hand(player, Suit::Spades),
            self.number_of_suits_in_hands(Suit::Diamonds)
                - self.number_of_suits_in_hand(player, Suit::Diamonds),
            self.number_of_suits_in_hands(Suit::Hearts)
                - self.number_of_suits_in_hand(player, Suit::Hearts),
        ];
        let mut residual = [[0u8; 9]; 9];
        residual[0][1..5].copy_from_slice(&suit_counts);
        for i in 0..4 {
            let mut temp_player = player;
            let temp_suit = Suit::from_index(i);
            for j in 0..3 {
                temp_player = temp_player.next();
                if !self.is_void(temp_player, temp_suit) {
                    residual[i + 1][j + 5] = 0xFF;
                }
            }
        }
        let mut temp_player = player;
        for row in residual.iter_mut().take(8).skip(5) {
            temp_player = temp_player.next();
            row[8] = self.number_of_cards_in_hand(temp_player);
        }
        residual
    }

    #[inline]
    fn max_flow_bfs(residual: &[[u8; 9]; 9], parent: &mut [u8; 9]) -> bool {
        parent.fill(0xFF);
        let mut queue = [0u8; 9];
        let mut head = 0;
        let mut tail = 0;
        queue[tail] = 0;
        tail += 1;
        while head < tail {
            let u = queue[head] as usize;
            head += 1;
            if u == 8 {
                return true;
            }
            for v in 1..9 {
                if parent[v] == 0xFF && residual[u][v] > 0 {
                    parent[v] = u as u8;
                    queue[tail] = v as u8;
                    tail += 1;
                }
            }
        }
        false
    }

    #[inline]
    fn max_flow(residual: &mut [[u8; 9]; 9]) -> [[u8; 3]; 4] {
        let mut parent = [0xFF; 9];
        while Self::max_flow_bfs(residual, &mut parent) {
            let mut bottleneck = 0xFF;
            let mut curr = 8;
            while parent[curr] != 0xFF {
                let prev = parent[curr] as usize;
                bottleneck = bottleneck.min(residual[prev][curr]);
                curr = prev;
            }
            curr = 8;
            while parent[curr] != 0xFF {
                let prev = parent[curr] as usize;
                residual[prev][curr] -= bottleneck;
                residual[curr][prev] += bottleneck;
                curr = prev;
            }
        }
        let mut distribution = [[0u8; 3]; 4];
        for s in 0..4 {
            for p in 0..3 {
                distribution[s][p] = residual[p + 5][s + 1];
            }
        }
        distribution
    }

    #[inline]
    fn diffuse_distribution(
        distribution: &mut [[u8; 3]; 4],
        residual: &[[u8; 9]; 9],
        rng: &mut SmallRng,
        steps: u8,
    ) {
        for _ in 0..steps {
            let s1 = rng.random_range(0..4usize);
            let s2 = rng.random_range(0..4usize);
            if s1 == s2 {
                continue;
            }
            let p1 = rng.random_range(0..3usize);
            let p2 = rng.random_range(0..3usize);
            if p1 == p2 {
                continue;
            }
            if distribution[s1][p1] > 0
                && distribution[s2][p2] > 0
                && residual[s1 + 1][p2 + 5] > 0
                && residual[s2 + 1][p1 + 5] > 0
            {
                distribution[s1][p1] -= 1;
                distribution[s1][p2] += 1;
                distribution[s2][p1] += 1;
                distribution[s2][p2] -= 1;
            }
        }
    }

    #[inline]
    pub fn generate_world(&self, player: Player, rng: &mut SmallRng) -> Self {
        let mut cards_available = self.cards_in_hand(player.next())
            | self.cards_in_hand(player.partner())
            | self.cards_in_hand(player.prev());
        let mut residual = self.setup_residual(player);
        let mut new_world = *self;
        new_world.player_hands[player.next() as usize] = 0;
        new_world.player_hands[player.partner() as usize] = 0;
        new_world.player_hands[player.prev() as usize] = 0;
        let mut distribution = GameState::max_flow(&mut residual);
        Self::diffuse_distribution(&mut distribution, &residual, rng, 24);
        for (i, row) in distribution.iter().enumerate() {
            let suit_mask = cards_available & (Self::SUIT_STRIDE << i);
            let mut bit_indices = [0u8; 10];
            let mut count = 0;
            let mut temp_mask = suit_mask;
            while temp_mask != 0 {
                let bit_pos = temp_mask.trailing_zeros() as u8;
                bit_indices[count] = bit_pos;
                count += 1;
                temp_mask &= temp_mask - 1;
            }
            bit_indices[..count].shuffle(rng);
            let mut offset = 0;
            let mut temp_player = player;
            for &num_cards_u8 in &row[..3] {
                temp_player = temp_player.next();
                let num_cards = num_cards_u8 as usize;
                if num_cards > 0 {
                    let mut assigned_mask = 0u64;
                    for &bit in &bit_indices[offset..offset + num_cards] {
                        assigned_mask |= 1u64 << bit;
                    }
                    new_world.player_hands[temp_player as usize] |= assigned_mask;
                    cards_available &= !assigned_mask;
                    offset += num_cards;
                }
            }
        }
        new_world
    }

    // --- MCTS trick ---

    #[inline]
    pub fn cards_in_current_trick(&self) -> u8 {
        self.current_trick.iter().filter(|&&c| c != 0xFF).count() as u8
    }

    #[inline]
    pub fn is_trick_complete(&self) -> bool {
        self.cards_in_current_trick() == 4
    }

    #[inline]
    fn card_beats_trick(&self, card: u8, best_card: u8) -> bool {
        if best_card == 0xFF {
            return true;
        }
        let card_suit = Suit::from_index((card & 3) as usize);
        let best_suit = Suit::from_index((best_card & 3) as usize);
        if card_suit == self.trump {
            if best_suit != self.trump {
                return true;
            }
            card > best_card
        } else if best_suit == self.trump {
            false
        } else if card_suit as u8 == self.lead_suit {
            card > best_card
        } else {
            false
        }
    }

    #[inline]
    pub fn finish_trick_random(&mut self, rng: &mut SmallRng) {
        let cards_needed = 4 - self.cards_in_current_trick();
        for _ in 0..cards_needed {
            let moves = self.legal_moves(self.current_player);
            let mut filtered = moves;
            if moves.count_ones() > 1 {
                let trump_played = self
                    .current_trick
                    .iter()
                    .any(|&c| c != 0xFF && Suit::from_index((c & 3) as usize) == self.trump);
                let mut best_card = 0xFF;
                let mut best_player = self.current_player;
                let mut p = self.current_player;
                for _ in 0..4 {
                    let c = self.current_trick[p as usize];
                    if c != 0xFF
                        && ((trump_played && Suit::from_index((c & 3) as usize) == self.trump)
                            || (!trump_played
                                && Suit::from_index((c & 3) as usize) as u8 == self.lead_suit))
                        && (best_card == 0xFF || c >= best_card)
                    {
                        best_card = c;
                        best_player = p;
                    }
                    p = p.next();
                }
                let partner_winning =
                    best_card != 0xFF && best_player == self.current_player.partner();
                let can_win = (0..moves.count_ones())
                    .fold((false, moves), |(won, m), _| {
                        let card = m.trailing_zeros() as u8;
                        (won || self.card_beats_trick(card, best_card), m & (m - 1))
                    })
                    .0;
                let trash = moves & !GameState::RANK_MASKS[3] & !GameState::RANK_MASKS[4];
                if best_card != 0xFF && !partner_winning && !can_win && trash != 0 {
                    filtered = trash;
                } else {
                    let played = self.cards_played();
                    let mut safe = moves;
                    let sevens = moves & GameState::RANK_MASKS[3];
                    let mut s = sevens;
                    while s != 0 {
                        let card = s.trailing_zeros() as u8;
                        s &= s - 1;
                        let suit = card & 3;
                        let ace = (9 << 2) | suit;
                        let ace_loose = (played & (1u64 << ace)) == 0;
                        let mut opp_trumps = false;
                        if Suit::from_index(suit as usize) != self.trump {
                            let mut nxt = self.current_player.next();
                            while self.current_trick[nxt as usize] == 0xFF
                                && nxt != self.current_player.partner()
                            {
                                if self.is_void(nxt, Suit::from_index(suit as usize))
                                    && !self.is_void(nxt, self.trump)
                                {
                                    opp_trumps = true;
                                    break;
                                }
                                nxt = nxt.next();
                            }
                        }
                        if ace_loose || opp_trumps {
                            safe &= !(1u64 << card);
                        }
                    }
                    if safe != 0 {
                        filtered = safe;
                    }
                }
            }
            let count = filtered.count_ones();
            let target_idx = rng.random_range(0..count);
            let mut m = filtered;
            for _ in 0..target_idx {
                m &= m - 1;
            }
            let card = m.trailing_zeros() as u8;
            self.make_move(card);
            self.current_player = self.current_player.next();
        }
        self.resolve_trick();
    }

    #[inline]
    pub fn rollout_random(&mut self, rng: &mut SmallRng) -> (u8, u8) {
        if self.cards_in_current_trick() > 0 {
            self.finish_trick_random(rng);
        }
        while self.number_of_cards_in_hands() > 0 {
            self.finish_trick_random(rng);
        }
        self.round_score()
    }

    // --- I/O Debugging ---

    #[inline]
    fn format_card(card: u8) -> String {
        if card == 0xFF {
            "--".to_string()
        } else {
            let c = Card::from_index(card as usize);
            let s = Suit::from_index((card & 3) as usize);
            format!("{c}{s}")
        }
    }

    pub fn print_trick(&self) {
        println!("Trick {} start", self.trick());
    }

    pub fn print_hand(&self, player: Player) {
        println!("{player}'s hand:");
        let mut first = true;
        for &suit in &Suit::SUITS {
            let mut suit_cards = self.suits_in_hand(player, suit);
            while suit_cards != 0 {
                let card_idx = suit_cards.trailing_zeros() as usize;
                suit_cards &= suit_cards - 1;
                if !first {
                    print!(" | ");
                }
                first = false;
                let card = Card::from_index(card_idx);
                print!("{card}{suit} ({card_idx})");
            }
        }
        println!();
    }

    pub fn print_table(&self) {
        let n = Self::format_card(self.current_trick[Player::North as usize]);
        let e = Self::format_card(self.current_trick[Player::East as usize]);
        let s = Self::format_card(self.current_trick[Player::South as usize]);
        let w = Self::format_card(self.current_trick[Player::West as usize]);
        println!("    {:^4}", n);
        println!("{:^4}    {:^4}", w, e);
        println!("    {:^4}", s);
    }

    pub fn print_scores(&self) {
        println!("Scores: ");
        println!("Team North/South: {}", self.score(true));
        println!("Team East/West: {}", self.score(false));
        println!();
    }
}
