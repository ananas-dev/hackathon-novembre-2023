use std::{io, collections::VecDeque};

use serde::{Deserialize, Serialize};

use crate::{net::FeedForwardNetwork, pos::Position, search};

#[derive(Deserialize)]
#[serde(tag = "type")]
enum In {
    Load {
        input_nodes: Vec<i64>,
        output_nodes: Vec<i64>,
        node_evals: Vec<(i64, f64, f64, Vec<(i64, f64)>)>,
    },
    Pos {
        score: i64,
        current_piece: usize,
        next_piece: usize,
        lines: usize,
        board: Vec<Vec<usize>>,
    },
    Peek,
    PlayGame,
    Ready,
    Go,
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum Out {
    Move {
        col: usize,
        row: usize,
    },
    Pos {
        score: i64,
        current_piece: usize,
        next_piece: usize,
        lines: usize,
        board: Vec<Vec<usize>>,
    },
    GameResult {
        score: i64,
    },
    Ok,
    Ko,
}

fn send(msg: &Out) -> io::Result<()> {
    println!("{}", serde_json::to_string(msg)?);
    Ok(())
}

pub fn start() -> io::Result<()> {
    let mut buffer = String::new();
    let stdin = io::stdin(); // We get `Stdin` here.
    let mut pos: Position = Position::default();
    let mut net: Option<FeedForwardNetwork> = None;
    let mut total_moves = 0;

    loop {
        buffer.clear();
        stdin.read_line(&mut buffer)?;
        let msg_in: In = serde_json::from_str(&buffer).expect("");

        match msg_in {
            In::Load {
                input_nodes,
                output_nodes,
                node_evals,
            } => {
                net = Some(FeedForwardNetwork::new(
                    input_nodes,
                    output_nodes,
                    node_evals,
                ));
            }
            In::Pos {
                score,
                current_piece,
                next_piece,
                lines,
                board,
            } => {
                pos = Position::new(current_piece, next_piece, lines, score, board);
                total_moves = 0;
            }
            In::Go => {
                if let Some(nn) = &mut net {
                    let best = search::find_best_move(nn, &pos);
                    pos.lines = 0;
                    pos = pos.apply_move(best.0, best.1).unwrap();
                }
            }
            In::Peek => {
                println!(
                    "{}",
                    serde_json::to_string(&Out::Pos {
                        score: pos.score,
                        current_piece: pos.current_piece,
                        next_piece: pos.next_piece,
                        lines: pos.lines,
                        board: pos.board.clone(),
                    })?
                )
            }
            In::PlayGame => {
                if total_moves <= 500 {
                    if let Some(nn) = &mut net {
                        let mut best = search::find_best_move(nn, &pos);
                        while let Some(new_pos) = pos.apply_move(best.0, best.1) {
                            pos = new_pos;
                            pos.lines = 0;
                            best = search::find_best_move(nn, &pos);
                        }

                        send(&Out::GameResult { score: pos.score })?;
                        pos = Position::default();
                        total_moves += 1;
                    }
                };
            },
            In::Ready => {
                match net {
                    Some(_) => send(&Out::Ok)?,
                    None => send(&Out::Ko)?,
                }
            },
        }
    }
}
