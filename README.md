# ♠️ Sueca Engine

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#license)

A high-performance Portuguese Sueca game engine and AI written in pure Rust.

Designed around 64-bit bitboard state representations, exact bipartite max-flow constraint solving for hand determinization, and an arena-backed Perfect Information Monte Carlo (PIMC) tree search with zero allocations on the hot path.

---

## ⚡ Key Architectural Features

- **Bitboard Core Representation:** The entire 40-card state, trick progress, player hands, and legal moves fit inside compact `u64` bitboards, allowing card evaluation and validation to resolve via bitwise operations and CPU registers.
- **Zero-Allocation Hot Path:** MCTS tree nodes live inside a pre-allocated vector arena (`Vec<Node>`), eliminating dynamic heap thrashing (`malloc`/`jemalloc`) during tree traversals and rollouts.
- **Exact Hand Determinization via Max-Flow:** Solves non-public card assignments subject to void constraints (*renúncia*) using an Edmonds-Karp bipartite max-flow algorithm, paired with Monte Carlo swap diffusion to generate unbiased, valid hidden hand states.
- **Domain-Specific Rollout Heuristics:** Rollout policies prevent non-tactical blunders (such as gifting loose Aces and 7s) while maintaining high simulation throughput.
- **Multi-Threaded Scaling:** Parallel world sampling and tree rollouts via [Rayon](https://github.com/rayon-rs/rayon), allowing thousands of determinized games to resolve in fractions of a second.

---

## 🏗️ Architecture & Pipeline

┌─────────────────────────────────────────────────────────────┐
│                       GameState                             │
│  - Bitboards: Player hands, played cards, legal masks (u64) │
│  - Trick state, current lead, trump suit, scoring LUTs      │
└──────────────────────────────┬──────────────────────────────┘
                               │
            ┌──────────────────┴──────────────────┐
            ▼                                     ▼
┌──────────────────────────────┐     ┌────────────────────────┐
│     Bipartite Max-Flow       │     │    Arena-Backed PIMC   │
│  - Edmonds-Karp capacity     │───▶│  - Rayon parallelism   │
│    matching across voids     │     │  - Zero hot-path alloc │
│  - Monte Carlo diffusion     │     │  - Fast UCB exploration│
└──────────────────────────────┘     └────────────────────────┘

---

## 🃏 Sueca Card Values & Rules

Sueca is a 4-player, point-trick game played with a 40-card Portuguese deck. A total of **120 points** are distributed across the deck:

| Card | Portuguese Name | Rank Value | Trick Hierarchy |
| :--- | :--- | :--- | :--- |
| **Ace** | Às | **11** | Highest in suit |
| **Seven** | Bisca | **10** | 2nd |
| **King** | Rei | **4** | 3rd |
| **Jack** | Valete | **3** | 4th |
| **Queen** | Dama | **2** | 5th |
| **6, 5, 4, 3, 2** | Cartas Baixas | **0** | Lowest |

Following the lead suit is mandatory if a player holds cards of that suit (*não renunciar*). If void, players may play a trump card or discard off-suit cards.

---

## 🚀 Getting Started

### Prerequisites

Ensure you have a recent stable version of [Rust](https://www.rust-lang.org/tools/install) installed.

### Installation & Run

Clone the repository and run the engine in release mode:

```bash
git clone [https://github.com/KitsuneX7/Sueca-engine.git](https://github.com/KitsuneX7/Sueca-engine.git)
cd Sueca-engine
cargo run --release
```

## 🎮 Interactive CLI Play

The default binary launches an interactive CLI session where a human player can challenge the MCTS engine:

```text
----- Trick 1 -----
West will play first.

--- West's turn ---
West plays 7♦

--- South's turn ---
South plays K♦

--- East's turn ---
East plays J♦

--- North's turn ---
     -- 
 7♦      J♦ 
     K♦ 
North's hand:
4♣ (8) | K♣ (28) | A♣ (36) | J♠ (25) | K♠ (29) | 4♦ (10) | 5♦ (14) | 4♥ (11) | 6♥ (19) | 7♥ (35)
Suit to follow: ♦ | Trump suit: ♣
Input your desired move:
```

---

## 🗺️ Roadmap

- [x] 64-bit bitboard state & legal move generation
- [x] Edmonds-Karp bipartite solver for void-constrained card sampling
- [x] Monte Carlo swap diffusion
- [x] Pre-allocated arena MCTS with Rayon parallelism
- [x] Interactive ASCII terminal interface
- [ ] Offline batch self-play generator for training datasets
- [ ] Deep Reinforcement Learning integration via `candle` (Policy & Value heads)

---

## 📄 License

- MIT license ([LICENSE-MIT](LICENSE-MIT))