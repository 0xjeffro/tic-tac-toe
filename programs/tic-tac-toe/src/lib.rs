use anchor_lang::prelude::*;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

declare_id!("AkBh5XtEmD5nQaZkTtNoPc1xhC6miVZHE4jK4in2WSbs");

// #[program]
// pub mod tic_tac_toe {
//     use super::*;

//     pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
//         msg!("Greetings from: {:?}", ctx.program_id);
//         Ok(())
//     }
// }

#[account]
pub struct Game {
    players: [Pubkey; 2],
    turn: u8,
    board: [[Option<Sign>; 3]; 3],
    state: GameState,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum GameState {
    Active,
    Tie,
    Won { winner: Pubkey },
}


#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, FromPrimitive)]
pub enum Sign {
    X = 0,
    O = 1,
}

pub struct Tile {
    row: u8,
    col: u8,
}

#[error_code]
pub enum TTTError {
    TileOutOfBounds,
    TileAlreadySet,
    GameAlreadyOver,
    NotPlayersTurn,
    GameAlreadyStarted,
}

impl Game {
    pub const MAXIMUM_SIZE: usize = (32 * 2) + 1 + (9 * (1 + 1)) + (32 + 1);

    pub fn start(&mut self, players: [Pubkey; 2]) -> Result<()> {
        require_eq!(self.turn, 0, TTTError:: GameAlreadyStarted);
        self.players = players;
        self.turn = 1;
        Ok(())
    }

    pub fn is_active(&self) -> bool {
        self.state == GameState::Active
    }

    fn current_player_index(&self) -> usize {
        ((self.turn - 1) % 2) as usize
    }
    
    pub fn current_player(&self) -> Pubkey {
        self.players[self.current_player_index()]
    }

    pub fn play(&mut self, tile: &Tile) -> Result<()> {
        require!(self.is_active(), TTTError::GameAlreadyOver);
        
        match tile {
            tile @ Tile { 
                row: 0..=2, 
                col: 0..=2,
            } => match self.board[tile.row as usize][tile.col as usize] {
                Some(_) => return Err(TTTError::TileAlreadySet.into()),
                None => {
                    self.board[tile.row as usize][tile.col as usize] = 
                    Some(
                        Sign::from_usize(self.current_player_index()).unwrap()
                    );
                }
            },
            _ => return Err(TTTError::TileOutOfBounds.into()),
        }

        self.update_state();
        if GameState::Active == self.state {
            self.turn += 1;
        }
        Ok(())
    }

    fn is_winning_trio(&self, trio:[(usize, usize); 3]) -> bool {
        let [a, b, c] = trio;
        self.board[a.0][a.1].is_some() &&
        self.board[a.0][a.1] == self.board[b.0][b.1] &&
        self.board[a.0][a.1] == self.board[c.0][c.1]
    }

    fn update_state(&mut self) {
        for i in 0..=2 {
            if self.is_winning_trio([(i, 0), (i, 1), (i, 2)]) {
                self.state = GameState::Won {
                    winner: self.current_player(),
                };
                return;
            }
            if self.is_winning_trio([(0, i), (1, i), (2, i)]) {
                self.state = GameState::Won {
                    winner: self.current_player(),
                };
                return;
            }
        }

        if self.is_winning_trio([(0, 0), (1, 1), (2, 2)]) ||
        self.is_winning_trio([(0, 2), (1, 1), (2, 0)]) {
            self.state = GameState::Won {
                winner: self.current_player(),
            };
            return;
        }

        for row in 0..=2 {
            for col in 0..=2 {
                if self.board[row][col].is_none() {
                    return;
                }
            }
        }
        self.state = GameState::Tie;
    }
}