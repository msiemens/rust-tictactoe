use std::f64;
use rand::{self, Rng};
use game::Board;
use game::Player;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum NodeState {
    Leaf,
    FullyExpanded,
    Expandable,
}

#[derive(Debug, Clone)]
struct Node {
    us: Player,
    board: Board,
    children: Vec<Node>,
    runs: i32,
    wins: i32,
    action: Option<(i32, i32)>,
    state: NodeState,
}

impl Node {
    fn best_child(&mut self) -> Option<&mut Node> {
        let mut best_value = f64::NEG_INFINITY;
        let mut best_child = None;
        let n_total = self.runs as f64;

        for child in &mut self.children {
            let w = child.wins as f64;
            let n = child.runs as f64;
            let value = w / n + (2. * n_total.ln() / n).sqrt();

            if value > best_value {
                best_value = value;
                best_child = Some(child);
            }
        }

        best_child
    }

    /// Add child with previously unexplored action
    fn expand(&mut self) -> Option<&mut Node> {
        let mut actions = self.board.get_actions();

        if actions.is_empty() {
            self.state = NodeState::Leaf;
            return None;
        }

        // Remove already tried actions
        for child in &self.children {
            actions.remove_item(&child.action.expect("Child has no action"));
        }

        if actions.len() == 1 {
            self.state = NodeState::FullyExpanded;
        }

        // Perform action
        let action = *rand::thread_rng().choose(&actions).unwrap();
        let mut board = self.board.clone();
        board.perform_action(action);

        self.children.push(Node {
            us: self.us,
            board: board,
            children: Vec::new(),
            runs: 0,
            wins: 0,
            action: Some(action),
            state: NodeState::Expandable,
        });

        self.children.last_mut()
    }

    fn simulate(&mut self) -> i32 {
        assert!(self.runs == 0);
        assert!(self.wins == 0);

        let mut board = self.board.clone();

        loop {
            let actions = board.get_actions();

            if !actions.is_empty() {
                let action = *rand::thread_rng().choose(&actions).unwrap();
                board.perform_action(action);
            }

            if board.is_ended() {
                return board.get_reward(self.us);
            }
        }
    }

    fn perform_mcts(&mut self) -> i32 {
        let current_reward = self.board.get_reward(self.us);
        let reward = match self.state {
            NodeState::Leaf => return current_reward,
            NodeState::FullyExpanded => {
                // No actions are missing, explore the best child
                let child = self.best_child().unwrap();
                child.perform_mcts()
            }
            NodeState::Expandable => {
                // Expand this node -> add unexplored child and simulate it
                match self.expand() {
                    Some(child) => {
                        let reward = child.simulate();

                        assert!(child.runs == 0);
                        assert!(child.wins == 0);

                        child.runs = 1;
                        child.wins = reward;

                        reward
                    }
                    None => return current_reward,
                }
            }
        };

        self.runs += 1;
        self.wins += reward;

        reward
    }

    fn finished(&self) -> bool {
        (self.state == NodeState::FullyExpanded || self.state == NodeState::Leaf)
            && self.children.iter().all(|c| c.state == NodeState::FullyExpanded || c.state == NodeState::Leaf)
    }
}

#[derive(Debug)]
pub struct MCTS {
    root: Node,
    us: Player,
}

impl MCTS {
    pub fn new(player: Player, first_action: bool) -> MCTS {
        MCTS {
            root: Node {
                us: player,
                board: Board::new(if first_action {
                    player
                } else {
                    player.opponent()
                }),
                children: Vec::new(),
                runs: 0,
                wins: 0,
                action: None,
                state: NodeState::Expandable,
            },
            us: player,
        }
    }

    pub fn get_action(&mut self) -> Option<(i32, i32)> {
        self.root.best_child().map(|c| c.action.unwrap())
    }

    pub fn run(&mut self) {
        if !self.root.finished() {
            self.root.perform_mcts();
        }
    }

    pub fn perform_action(&mut self, action: (i32, i32)) {
        let idx = self.root
            .children
            .iter()
            .enumerate()
            .find(|&(_, c)| c.action.unwrap() == action)
            .map(|(i, _)| i)
            .unwrap();

        let node = self.root.children.remove(idx);

        self.root = node;
    }
}
