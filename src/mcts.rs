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

        // Remove already explored actions
        for child in &self.children {
            actions.remove_item(&child.action.expect("Child has no action"));
        }

        if actions.len() == 1 {
            // Only one action to explore, then this node will be fully expanded
            self.state = NodeState::FullyExpanded;
        }

        // Perform action
        let action = *rand::thread_rng().choose(&actions).expect("actions is empty");
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

    /// Simulate the current node's game until reaching an outcome
    fn simulate(&mut self) -> i32 {
        assert!(self.runs == 0);
        assert!(self.wins == 0);

        let mut board = self.board.clone();

        loop {
            let actions = board.get_actions();

            if !actions.is_empty() {
                let action = *rand::thread_rng().choose(&actions).expect("actions is empty");
                board.perform_action(action);
            }

            if let Some(reward) = board.get_reward(self.us) {
                self.runs = 1;
                self.wins = reward;

                reward
            }
        }
    }

    /// Perform Monte Carlo Tree Search (selection, expansion, simulation, backpropagation)
    fn perform_mcts(&mut self) -> i32 {
        let current_reward = self.board.get_reward(self.us);
        let reward = match self.state {
            NodeState::Leaf => return current_reward,
            NodeState::FullyExpanded => {
                // Current state's actions are fully explored, explore the best child (selection)
                let child = self.best_child().expect("Fully expanded node without children");
                child.perform_mcts()
            }
            NodeState::Expandable => {
                // Current state has unexplored actions -> expansion + simulation
                match self.expand() {
                    Some(child) => child.simulate(),
                    None => return current_reward,
                }
            }
        };

        // Backpropagation of simulation results
        self.runs += 1;
        self.wins += reward;

        reward
    }

    fn finished(&self) -> bool {
        self.state != NodeState::Expandable
            && self.children.iter().all(|c| c.state != NodeState::Expandable)
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
        self.root.best_child().map(|c| c.action.expect("Best child without action"))
    }

    pub fn run(&mut self) {
        if !self.root.finished() {
            self.root.perform_mcts();
        }
    }

    pub fn perform_action(&mut self, action: (i32, i32)) {
        // Find index of child node with the desired action
        // That way, we don't have to start over but re-use all previous calculations
        let idx = self.root
            .children
            .iter()
            .enumerate()
            .find(|&(_, c)| c.action.expect("Child without action") == action)
            .map(|(i, _)| i)
            .expect("No child with action to be performed");

        let node = self.root.children.remove(idx);

        self.root = node;
    }
}
