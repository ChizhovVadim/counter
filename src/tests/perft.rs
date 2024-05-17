use crate::chess;
use std::time::Instant;

struct PerftIfno {
    pub fen: &'static str,
    pub depth: usize,
    pub nodes: usize,
}

//https://www.chessprogramming.org/Perft_Results
pub fn perft_handler() {
    let data = [
        PerftIfno {
            fen: chess::INITIAL_POSITION_FEN,
            depth: 6,
            nodes: 119060324,
        },
        /*PerftIfno{fen: "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -", depth: 5, nodes: 193690690},
        PerftIfno{fen: "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -", depth: 7, nodes: 178633661},
        PerftIfno{fen: "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1", depth: 5, nodes: 15833292},
        PerftIfno{fen: "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", depth: 5, nodes: 89941194},
        PerftIfno{fen: "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10", depth: 5, nodes: 164075551},*/
    ];

    let start = Instant::now();
    for item in data {
        let mut pos = chess::Position::from_fen(item.fen).unwrap();
        let n = perft(&mut pos, item.depth as isize);
        if n != item.nodes as isize {
            println!("BUG {} {} {}", item.fen, item.nodes, n)
        }
    }

    let duration = start.elapsed();
    println!("Time elapsed in expensive_function() is: {:?}", duration);
}

fn perft(p: &chess::Position, depth: isize) -> isize {
    let mut result: isize = 0;
    let mut ml = chess::MoveList::new();
    chess::generate_moves(p, &mut ml);
    let mut history: chess::History = chess::History::new();
    for i in 0..ml.size {
        let m = ml.moves[i].mv;
        let mut child = *p;
        if !child.make_move(m, &mut history) {
            continue;
        }
        if depth > 1 {
            result += perft(&child, depth - 1)
        } else {
            result += 1;
        }
    }
    return result;
}
