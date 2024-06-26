use crate::chess::*;
use crate::engine::{transtable::*, *};
use crate::types;

impl Engine {
    pub fn iterative_deepening(&mut self, pos: &Position) -> types::SearchInfo {
        let mut result = types::SearchInfo {
            depth: 0,
            score: types::UciScore::Centipawns(0),
            nodes: 0,
            duration: Duration::ZERO,
            main_line: Vec::new(),
        };
        {
            let mut legal_moves = MoveList::new();
            movegen::generate_legal_moves(pos, &mut legal_moves);
            if legal_moves.size >= 1 {
                let rnd_move = legal_moves.moves[0].mv;
                result.main_line.push(rnd_move);
            }
            if legal_moves.size <= 1 {
                return result;
            }
        }

        for h in 0..=2 {
            self.stack[h].killer1 = Move::EMPTY;
            self.stack[h].killer2 = Move::EMPTY;
        }

        for depth in 1..MAX_HEIGHT {
            if self.time_manager.check_timeout() {
                break;
            }
            self.root_depth = depth;
            let score = self.alphabeta(
                pos,
                -VALUE_INFINITY,
                VALUE_INFINITY,
                depth as isize,
                0,
                None,
                NODETYPE_NORMAL,
            );
            match score {
                Some(score) => {
                    result = types::SearchInfo {
                        depth: depth,
                        score: make_uci_score(score),
                        nodes: self.nodes,
                        duration: self.time_manager.elapsed(),
                        main_line: self.stack[0].pv[..self.stack[0].pv_size].to_vec(),
                    };
                    self.time_manager.iteration_complete(&result);
                }
                None => {
                    result.nodes = self.nodes;
                    result.duration = self.time_manager.elapsed();
                    break;
                }
            }
        }
        return result;
    }

    fn alphabeta(
        &mut self,
        pos: &Position,
        mut alpha: isize,
        mut beta: isize,
        mut depth: isize,
        height: usize,
        skip_move: Option<Move>,
        node_type: i8,
    ) -> Option<isize> {
        if depth <= 0 {
            return self.qs(pos, alpha, beta, height);
        }
        self.clear_pv(height);
        let root_node = height == 0;
        let pv_node = beta != alpha + 1;
        let in_check = pos.is_check();

        if !root_node {
            if height >= MAX_HEIGHT {
                return Some(self.evalaute(pos));
            }
            if is_draw(pos) {
                return Some(VALUE_DRAW);
            }
            if self.is_repeat(height) {
                return Some(VALUE_DRAW);
            }
            alpha = alpha.max(loss_in(height));
            beta = beta.min(win_in(height + 1));
            if alpha >= beta {
                return Some(alpha);
            }
        }

        let mut tt_depth: isize = 0;
        let mut tt_value: isize = 0;
        let mut tt_bound: usize = 0;
        let mut tt_move: Move = Move::EMPTY;
        let mut tt_hit: bool = false;

        if skip_move.is_none() {
            (tt_depth, tt_value, tt_bound, tt_move, tt_hit) = self.trans_table.read(pos.key);
        }
        if tt_hit {
            let tt_value = value_from_tt(tt_value, height);
            if tt_depth >= depth && !pv_node && !root_node {
                if tt_value >= beta && tt_bound & BOUND_LOWER != 0 {
                    return Some(tt_value);
                }
                if tt_value <= alpha && tt_bound & BOUND_UPPER != 0 {
                    return Some(tt_value);
                }
            }
        }

        let static_eval = self.evalaute(pos);
        self.stack[height].static_eval = static_eval;
        let improving = height < 2 || static_eval > self.stack[height - 2].static_eval;

        let mut history = chess::History::new();
        let mut tt_move_is_singular = false;

        if height + 2 <= MAX_HEIGHT {
            self.stack[height + 2].killer1 = Move::EMPTY;
            self.stack[height + 2].killer2 = Move::EMPTY;
        }

        if !root_node && skip_move.is_none() {
            if !pv_node && !in_check && beta > VALUE_LOSS && beta < VALUE_WIN {
                if depth <= 8 {
                    let score = static_eval - PAWN_VALUE * depth;
                    if score >= beta {
                        return Some(static_eval);
                    }
                }

                // null-move pruning
                if depth >= 2
                    && self.stack[height - 1].current_mv != Move::EMPTY
                    && !(tt_hit && tt_value < beta && (tt_bound & BOUND_UPPER) != 0)
                    && allow_nmp(pos)
                    && static_eval >= beta
                {
                    let reduction = 4 + depth / 6 + ((static_eval - beta) / 200).min(2);

                    //make move
                    let mut child = pos.clone();
                    child.make_move(Move::EMPTY, &mut history);
                    self.evaluator.make_move(&history);
                    self.stack[height].current_mv = Move::EMPTY;
                    self.stack[height + 1].key = child.key;
                    if self.check_timeout() {
                        return None;
                    }

                    let score = -(self.alphabeta(
                        &child,
                        -beta,
                        1 - beta,
                        depth - reduction,
                        height + 1,
                        None,
                        -node_type,
                    )?);
                    self.evaluator.unmake_move();
                    if score >= beta {
                        if score >= VALUE_WIN {
                            return Some(beta);
                        }
                        return Some(score);
                    }
                }

                let probcut_beta = (beta + 150).min(VALUE_WIN - 1);
                if depth >= 5 && !(tt_hit && tt_value < probcut_beta && tt_bound & BOUND_UPPER != 0)
                {
                    let mut ml = chess::MoveList::new();
                    chess::movegen::generate_noisy_moves(pos, &mut ml);
                    moveiter::eval_noisy(&mut ml.moves[..ml.size], pos, tt_move);
                    let mut mi = moveiter::MovePicker::new(&mut ml.moves[..ml.size]);

                    while let Some(mv) = mi.next() {
                        if !see::see_ge(pos, mv, 0) {
                            continue;
                        }

                        //make move
                        let mut child = pos.clone();
                        if !child.make_move(mv, &mut history) {
                            continue;
                        }
                        self.evaluator.make_move(&history);
                        self.stack[height].current_mv = mv;
                        self.stack[height + 1].key = child.key;
                        if self.check_timeout() {
                            return None;
                        }

                        let mut score =
                            -(self.qs(&child, -probcut_beta, -(probcut_beta - 1), height + 1)?);
                        if score >= probcut_beta {
                            score = -(self.alphabeta(
                                &child,
                                -probcut_beta,
                                -(probcut_beta - 1),
                                depth - 4,
                                height + 1,
                                None,
                                -node_type,
                            )?);
                        }
                        self.evaluator.unmake_move();
                        if score >= probcut_beta {
                            if !(tt_hit && tt_depth >= depth - 3) {
                                self.trans_table.update(
                                    pos.key,
                                    depth - 3,
                                    value_to_tt(score, height),
                                    BOUND_LOWER,
                                    mv,
                                );
                            }
                            return Some(score);
                        }
                    }
                }
            }

            if tt_move == Move::EMPTY && depth >= 5 && node_type != NODETYPE_ALL {
                depth -= 1;
            }

            // singular extension
            if depth >= 8
                && height < 2 * self.root_depth
                && tt_hit
                && tt_move != Move::EMPTY
                && (tt_bound & BOUND_LOWER) != 0
                && tt_depth >= depth - 3
                && tt_value > VALUE_LOSS
                && tt_value < VALUE_WIN
            {
                let singular_beta = (tt_value - depth).max(-VALUE_INFINITY);
                let score = self.alphabeta(
                    pos,
                    singular_beta - 1,
                    singular_beta,
                    depth / 2,
                    height,
                    Some(tt_move),
                    node_type,
                )?;
                tt_move_is_singular = score < singular_beta
            }
        }

        let killer1 = self.stack[height].killer1;
        let killer2 = self.stack[height].killer2;

        let counter_move = if height >= 1 {
            Some(self.stack[height - 1].current_mv)
        } else {
            None
        };

        let follow_move = if height >= 2 {
            Some(self.stack[height - 2].current_mv)
        } else {
            None
        };

        let mut ml = chess::MoveList::new();
        chess::movegen::generate_moves(pos, &mut ml);
        moveiter::eval_moves(
            &mut ml.moves[..ml.size],
            pos,
            tt_move,
            killer1,
            killer2,
            &self.history,
            counter_move,
            follow_move,
        );
        let mut mi = moveiter::MovePicker::new(&mut ml.moves[..ml.size]);
        let mut has_legal_move = false;
        let mut moves_searched = 0;
        let mut quiets_seen = 0;

        let mut quiets: [Move; MAX_MOVES] =
            unsafe { std::mem::MaybeUninit::uninit().assume_init() };
        let mut quiet_count = 0;

        let mut lmp = 3 + depth * depth;
        if !improving {
            lmp /= 2;
        }
        let old_alpha = alpha;
        let mut best_move = tt_move;
        let mut best = loss_in(height);

        while let Some(mv) = mi.next() {
            if Some(mv) == skip_move {
                continue;
            }

            let is_noisy = is_cap_or_prom(mv);
            if !is_noisy {
                quiets_seen += 1;
            }

            if depth <= 8 && best > VALUE_LOSS && has_legal_move && !in_check && !root_node {
                if !(is_noisy || mv == killer1 || mv == killer2) {
                    if quiets_seen > lmp {
                        mi.skip_queits();
                        continue;
                    }

                    if static_eval + 100 + 100 * depth <= alpha
                        && moves_searched >= 2
                        && !is_pawn_advance(mv, pos.side_to_move)
                    {
                        best = best.max(static_eval);
                        continue;
                    }
                }

                let see_margin = if is_noisy { depth } else { depth / 2 };
                if !see::see_ge(pos, mv, -see_margin) {
                    continue;
                }
            }

            //make move
            let mut child = pos.clone();
            if !child.make_move(mv, &mut history) {
                continue;
            }
            self.evaluator.make_move(&history);
            self.stack[height].current_mv = mv;
            self.stack[height + 1].key = child.key;
            if self.check_timeout() {
                return None;
            }
            has_legal_move = true;
            moves_searched += 1;

            let gives_check = child.is_check();

            let mut extension = 0;
            if mv == tt_move && tt_move_is_singular {
                extension = 1;
            }
            if in_check && !root_node && height < 2 * self.root_depth {
                extension = 1;
            }

            let new_depth = depth - 1 + extension;
            let mut reduction = 0;
            if new_depth >= 2 && moves_searched > 1 && !is_noisy {
                reduction =
                    self.reductions[depth.min(63) as usize][moves_searched.min(63)] as isize;

                if mv == killer1 || mv == killer2 {
                    reduction -= 1;
                }
                let history = self
                    .history
                    .read(pos.side_to_move, mv, counter_move, follow_move);
                reduction -= (history / 8_000).max(-2).min(2);
                if !pv_node {
                    reduction += 1;
                }
                if !improving {
                    reduction += 1;
                }
                if node_type == NODETYPE_CUT {
                    reduction += 1;
                }
                if gives_check {
                    reduction -= 1;
                }
                reduction = reduction.min(new_depth - 1).max(0);
            }

            // best_move может не попасть в quiets, если ограничить длину quiets
            if !is_noisy && quiet_count < quiets.len() {
                quiets[quiet_count] = mv;
                quiet_count += 1;
            }

            let mut score = alpha + 1;
            if moves_searched == 1 || new_depth <= 0 {
                score = -(self.alphabeta(
                    &child,
                    -beta,
                    -alpha,
                    new_depth,
                    height + 1,
                    None,
                    -node_type,
                )?);
            } else {
                score = -(self.alphabeta(
                    &child,
                    -(alpha + 1),
                    -alpha,
                    new_depth - reduction,
                    height + 1,
                    None,
                    NODETYPE_CUT,
                )?);
                if reduction > 0 && score > alpha {
                    score = -(self.alphabeta(
                        &child,
                        -(alpha + 1),
                        -alpha,
                        new_depth,
                        height + 1,
                        None,
                        -node_type,
                    )?);
                }
                if pv_node && score > alpha {
                    score = -(self.alphabeta(
                        &child,
                        -beta,
                        -alpha,
                        new_depth,
                        height + 1,
                        None,
                        -node_type,
                    )?);
                }
            }
            self.evaluator.unmake_move();

            best = best.max(score);
            if score > alpha {
                alpha = score;
                best_move = mv;
                self.assign_pv(height, mv);
                if alpha >= beta {
                    if !is_noisy {
                        if self.stack[height].killer1 != mv {
                            self.stack[height].killer2 = self.stack[height].killer1;
                            self.stack[height].killer1 = mv;
                        }

                        self.history.update(
                            pos.side_to_move,
                            &quiets[..quiet_count],
                            best_move,
                            depth,
                            counter_move,
                            follow_move,
                        );
                    }
                    break;
                }
            }
        }

        if !has_legal_move {
            if !in_check && skip_move.is_none() {
                return Some(VALUE_DRAW);
            }
            return Some(loss_in(height));
        }

        if skip_move.is_none() {
            let bound = if alpha >= beta {
                BOUND_LOWER
            } else if alpha > old_alpha {
                BOUND_EXACT
            } else {
                BOUND_UPPER
            };
            self.trans_table
                .update(pos.key, depth, value_to_tt(best, height), bound, best_move);
        }

        return Some(best);
    }

    fn qs(
        &mut self,
        pos: &Position,
        mut alpha: isize,
        beta: isize,
        height: usize,
    ) -> Option<isize> {
        self.clear_pv(height);
        if is_draw(pos) {
            return Some(VALUE_DRAW);
        }
        if height >= MAX_HEIGHT {
            return Some(self.evalaute(pos));
        }
        if self.is_repeat(height) {
            return Some(VALUE_DRAW);
        }

        let (_, tt_value, tt_bound, tt_move, tt_hit) = self.trans_table.read(pos.key);
        if tt_hit {
            let tt_value = value_from_tt(tt_value, height);
            if tt_value >= beta && tt_bound & BOUND_LOWER != 0 {
                return Some(tt_value);
            }
            if tt_value <= alpha && tt_bound & BOUND_UPPER != 0 {
                return Some(tt_value);
            }
        }

        let in_check = pos.is_check();
        let mut ml = chess::MoveList::new();
        let mut best = loss_in(height);
        if in_check {
            chess::movegen::generate_moves(pos, &mut ml);
            moveiter::eval_moves(
                &mut ml.moves[..ml.size],
                pos,
                tt_move,
                Move::EMPTY,
                Move::EMPTY,
                &self.history,
                None,
                None,
            );
        } else {
            let static_eval = self.evalaute(pos);
            best = best.max(static_eval);
            if static_eval >= alpha {
                alpha = static_eval;
                if alpha >= beta {
                    return Some(alpha);
                }
            }
            chess::movegen::generate_noisy_moves(pos, &mut ml);
            moveiter::eval_noisy(&mut ml.moves[..ml.size], pos, tt_move);
        }
        let mut mi = moveiter::MovePicker::new(&mut ml.moves[..ml.size]);
        let mut history = chess::History::new();
        let mut has_legal_move = false;
        while let Some(m) = mi.next() {
            if best > VALUE_LOSS && !in_check && !see::see_ge(pos, m, 0) {
                continue;
            }

            //make move
            let mut child = pos.clone();
            if !child.make_move(m, &mut history) {
                continue;
            }
            self.evaluator.make_move(&history);
            self.stack[height].current_mv = m;
            self.stack[height + 1].key = child.key;
            if self.check_timeout() {
                return None;
            }
            has_legal_move = true;

            let score = -(self.qs(&child, -beta, -alpha, height + 1)?);
            self.evaluator.unmake_move();

            best = best.max(score);
            if score > alpha {
                alpha = score;
                if alpha >= beta {
                    self.trans_table
                        .update(pos.key, 0, value_to_tt(alpha, height), BOUND_LOWER, m);
                    //return Some(alpha);
                    break;
                }
            }
        }
        if in_check && !has_legal_move {
            return Some(loss_in(height));
        }
        return Some(best);
    }

    fn clear_pv(&mut self, height: usize) {
        self.stack[height].pv_size = 0;
    }

    fn assign_pv(&mut self, height: usize, m: Move) {
        let (parent, child) = get_pair_mut(&mut self.stack, height);

        let child_size = child.pv_size;
        parent.pv_size = 1 + child_size;
        parent.pv[0] = m;
        parent.pv[1..1 + child_size].copy_from_slice(&child.pv[..child_size]);
    }

    fn evalaute(&mut self, pos: &Position) -> isize {
        const MAX_STATIC_EVAL: isize = 15_000;
        let mut static_eval = self
            .evaluator
            .quik_evaluate(pos)
            .max(-MAX_STATIC_EVAL)
            .min(MAX_STATIC_EVAL);

        static_eval = static_eval * (200 - pos.rule50) / 200;
        if pos.side_to_move == chess::SIDE_BLACK {
            static_eval = -static_eval;
        }
        const TEMPO: isize = 10;
        return static_eval + TEMPO;
    }

    fn check_timeout(&mut self) -> bool {
        self.nodes += 1;
        if self.nodes & 255 == 0 {
            //e.time_manager.on_nodes_changed(e.nodes);
            return self.time_manager.check_timeout();
        }
        return false;
    }

    fn is_repeat(&self, height: usize) -> bool {
        let key = self.stack[height].key;
        for item in self.stack[..height].iter().rev() {
            let mv = item.current_mv;
            if mv == Move::EMPTY
                || mv.moving_piece() == PIECE_PAWN
                || mv.captured_piece() != PIECE_EMPTY
            {
                return false;
            }
            if key == item.key {
                return true;
            }
        }
        if self.repeats.contains(&key) {
            return true;
        }
        return false;
    }
}

const PAWN_VALUE: isize = 100;

const NODETYPE_NORMAL: i8 = 0;
const NODETYPE_CUT: i8 = 1;
const NODETYPE_ALL: i8 = -NODETYPE_CUT;

fn get_pair_mut(stack: &mut [SearchStack], height: usize) -> (&mut SearchStack, &mut SearchStack) {
    let (a, b) = stack.split_at_mut(height + 1);
    return (&mut a[height], &mut b[0]);
}
