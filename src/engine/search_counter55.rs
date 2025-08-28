use super::{Engine, SearchStack, moveorder, see, transtable, utils};
use crate::chess::{Move, MoveList, Piece, Position, Side};
use crate::domain::{IEvaluator, SearchInfo};

// Клон поиска из Counter 5.5 (на golang)
// https://github.com/ChizhovVadim/CounterGo/blob/a46cf2d76571b28ab093e626acead47815dd4c25/pkg/engine/search.go
pub fn iterative_deepening(e: &mut Engine) -> SearchInfo {
    e.reductions = utils::Reductions::new(lmr_counter55);

    let mut result = SearchInfo::default();

    let mut legal_moves = MoveList::new();
    legal_moves.gen_legal_moves(&e.stack[0].position);
    if legal_moves.size == 0 {
        return result;
    }
    let rnd_move = legal_moves.moves[0].mv;
    result.main_line = vec![rnd_move];

    for height in 0..=2 {
        e.stack[height].killer1 = Move::NONE;
        e.stack[height].killer2 = Move::NONE;
    }

    let mut prev_score = 0;
    for depth in 1..utils::MAX_HEIGHT {
        if e.time_manager.check_timeout(e.nodes) {
            break;
        }
        //self.root_depth = depth;
        let score = aspiration_window(e, depth as isize, prev_score);
        match score {
            Some(score) => {
                assert!(e.stack[0].pv_size > 0);
                prev_score = score;
                result = SearchInfo {
                    depth: depth,
                    score: utils::make_uci_score(score),
                    nodes: e.nodes,
                    duration: e.time_manager.elapsed(),
                    main_line: e.stack[0].pv[..e.stack[0].pv_size].to_vec(),
                };
                (e.progress)(&result);
                e.time_manager.iteration_complete(&result);
            }
            None => {
                result.nodes = e.nodes;
                result.duration = e.time_manager.elapsed();
                break;
            }
        }
    }
    return result;
}

fn aspiration_window(e: &mut Engine, depth: isize, prev_score: isize) -> Option<isize> {
    if depth >= 5 && prev_score > utils::VALUE_LOSS && prev_score < utils::VALUE_WIN {
        const WINDOW: isize = 25;
        let mut alpha = (prev_score - WINDOW).max(-utils::VALUE_INFINITY);
        let mut beta = (prev_score + WINDOW).min(utils::VALUE_INFINITY);
        let score = search(e, alpha, beta, depth, 0, Move::NONE)?;
        if score > alpha && score < beta {
            return Some(score);
        }
        if score >= beta {
            beta = utils::VALUE_INFINITY
        }
        if score <= alpha {
            alpha = -utils::VALUE_INFINITY;
        }
        let score = search(e, alpha, beta, depth, 0, Move::NONE)?;
        if score > alpha && score < beta {
            return Some(score);
        }
    }
    return search(
        e,
        -utils::VALUE_INFINITY,
        utils::VALUE_INFINITY,
        depth,
        0,
        Move::NONE,
    );
}

#[allow(invalid_value)]
fn search(
    e: &mut Engine,
    mut alpha: isize,
    beta: isize,
    depth: isize,
    height: usize,
    skip_move: Move,
) -> Option<isize> {
    if depth <= 0 {
        return qs(e, alpha, beta, height);
    }

    e.stack[height].pv_size = 0; // TODO clear_pv(e, height)
    let root_node = height == 0;
    let pv_node = beta != alpha + 1;

    if !root_node {
        if height >= utils::MAX_HEIGHT {
            return Some(evaluate2(e, height));
        }
        if utils::is_draw(&e.stack[height].position) {
            return Some(utils::VALUE_DRAW);
        }
        if is_repeat(e, height) {
            return Some(utils::VALUE_DRAW);
        }
        let ralpha = alpha.max(utils::loss_in(height));
        let rbeta = beta.min(utils::win_in(height + 1));
        if ralpha >= rbeta {
            return Some(ralpha);
        }
    }

    let (tt_depth, mut tt_value, tt_bound, tt_move, tt_hit) = if skip_move == Move::NONE {
        e.trans_table.read(e.stack[height].position.key)
    } else {
        (0, 0, 0, Move::NONE, false)
    };
    if tt_hit {
        tt_value = utils::value_from_tt(tt_value, height);
        if tt_depth >= depth
            && !root_node
            && !pv_node
            && !(e.stack[height - 1].current_mv.is_null())
        {
            if tt_value >= beta && tt_bound & transtable::BOUND_LOWER != 0 {
                if tt_move != Move::NONE && !utils::is_capture_or_promotion(tt_move) {
                    update_killer(e, height, tt_move);
                    //e.history.update(e.stack[height].position.side_to_move, &[], tt_move, depth);
                }
                return Some(tt_value);
            }
            if tt_value <= alpha && tt_bound & transtable::BOUND_UPPER != 0 {
                return Some(tt_value);
            }
        }
    }

    if height + 2 <= utils::MAX_HEIGHT {
        e.stack[height + 2].killer1 = Move::NONE;
        e.stack[height + 2].killer2 = Move::NONE;
    }

    let in_check = e.stack[height].position.is_check();
    let static_eval = evaluate2(e, height);
    e.stack[height].static_eval = static_eval;
    let improving = height < 2 || static_eval > e.stack[height - 2].static_eval;

    let mut tt_move_is_singular = false;

    if !root_node && skip_move == Move::NONE {
        // reverse futility pruning
        if depth <= 8 && !pv_node && !in_check {
            let score = static_eval - 100 * depth;
            if score >= beta {
                return Some(static_eval);
            }
        }

        // null-move pruning
        if depth >= 2
            && !pv_node
            && !in_check
            && beta < utils::VALUE_WIN
            && static_eval >= beta
            && !(tt_hit && tt_value < beta && (tt_bound & transtable::BOUND_UPPER) != 0)
            && !(e.stack[height - 1].current_mv.is_null())
            && !(height >= 2 && e.stack[height - 2].current_mv.is_null())
            && utils::allow_nmp(&e.stack[height].position)
        {
            let reduction = 4 + depth / 6 + ((static_eval - beta) / 200).min(2);
            make_move(e, Move::NULL, height);
            if is_timeout(e) {
                return None;
            }
            let score = -(search(
                e,
                -beta,
                1 - beta,
                depth - reduction,
                height + 1,
                Move::NONE,
            )?);
            unmake_move(e);
            if score >= beta {
                if score >= utils::VALUE_WIN {
                    return Some(beta);
                }
                return Some(score);
            }
        }

        let probcut_beta = (beta + 150).min(utils::VALUE_WIN - 1);
        if depth >= 5
            && !pv_node
            && !in_check
            && beta > utils::VALUE_LOSS
            && beta < utils::VALUE_WIN
            && !(tt_hit
                && tt_depth >= depth - 4
                && tt_value < probcut_beta
                && tt_bound & transtable::BOUND_UPPER != 0)
        {
            let mut ml = MoveList::new();
            ml.gen_captures(&e.stack[height].position);
            moveorder::evaluate_captures(&mut ml);
            ml.sort();

            for item in &ml.moves[..ml.size] {
                let mv = item.mv;
                if !see::see_ge(&e.stack[height].position, mv, 0) {
                    continue;
                }
                if !make_move(e, mv, height) {
                    continue;
                }
                if is_timeout(e) {
                    return None;
                }
                let mut score = -(qs(e, -probcut_beta, -(probcut_beta - 1), height + 1)?);
                if score >= probcut_beta {
                    score = -(search(
                        e,
                        -probcut_beta,
                        1 - probcut_beta,
                        depth - 4,
                        height + 1,
                        Move::NONE,
                    )?);
                }
                unmake_move(e);
                if score >= probcut_beta {
                    return Some(score);
                }
            }
        }

        // singular extension
        if depth >= 8
            && tt_hit
            && tt_move != Move::NONE
            && tt_bound & transtable::BOUND_LOWER != 0
            && tt_depth >= depth - 3
            && tt_value > utils::VALUE_LOSS
            && tt_value < utils::VALUE_WIN
        {
            let singular_beta = (tt_value - 1 * depth).max(-utils::VALUE_INFINITY);
            let score = search(
                e,
                singular_beta - 1,
                singular_beta,
                depth / 2,
                height,
                tt_move,
            )?;
            if score < singular_beta {
                tt_move_is_singular = true;
            }
        }
    }

    let moveorder_context = get_moveorder_context(e, height, tt_move);
    let pos = &e.stack[height].position;
    let mut ml = MoveList::new();
    ml.gen_moves(pos);
    moveorder_context.evaluate_moves_counter55(&mut ml, pos, &e.history);
    ml.sort();

    let mut has_legal_move = false;
    let mut best_move = Move::NONE;
    let mut moves_searched = 0;
    let mut quiets: [Move; 64] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
    let mut quiet_count = 0;
    let mut quiets_seen = 0;
    let old_alpha = alpha;
    let mut best = utils::loss_in(height);
    let lmp = {
        let mut res = 5 + (depth - 1) * depth;
        if !improving {
            res /= 2;
        }
        res
    };

    for item in &ml.moves[..ml.size] {
        let mv = item.mv;
        if mv == skip_move {
            continue;
        }
        let is_noisy = utils::is_capture_or_promotion(mv);
        if !is_noisy {
            quiets_seen += 1;
        }

        if depth <= 8 && best > utils::VALUE_LOSS && has_legal_move && !root_node && !in_check {
            if is_noisy {
                let see_margin = depth.max((static_eval - alpha + 100) / 100);
                if !see::see_ge(&e.stack[height].position, mv, -see_margin) {
                    continue;
                }
            } else {
                if quiets_seen > lmp
                    && !(mv == moveorder_context.killer1 || mv == moveorder_context.killer2)
                {
                    continue;
                }
                if static_eval + 100 + 100 * depth <= alpha
                    && !(mv == moveorder_context.killer1 || mv == moveorder_context.killer2)
                {
                    continue;
                }
                if !see::see_ge(&e.stack[height].position, mv, -depth / 2) {
                    continue;
                }
            }
        }

        if !make_move(e, mv, height) {
            continue;
        }
        if is_timeout(e) {
            return None;
        }
        has_legal_move = true;
        moves_searched += 1;
        let gives_check = e.stack[height + 1].position.is_check();

        let mut extension = 0;
        if mv == tt_move && tt_move_is_singular {
            extension = 1;
        }
        if gives_check && depth >= 3 {
            extension = 1;
        }

        let mut reduction = 0;
        if depth >= 3 && moves_searched > 1 && !is_noisy {
            reduction = e.reductions.get(depth, moves_searched);
            if mv == moveorder_context.killer1 || mv == moveorder_context.killer2 {
                reduction -= 1;
            }
            if in_check || gives_check {
                reduction -= 1;
            }
            if pv_node {
                reduction -= 2;
            }
            if !in_check {
                reduction -= ((e.history.read_total(&moveorder_context, mv)) / 5_000).clamp(-2, 2);
                if !improving {
                    reduction += 1;
                }
            }
            reduction = reduction.max(0) + extension;
            reduction = reduction.min(depth - 2).max(0);
        }

        let new_depth = depth - 1 + extension;

        // best_move может не попасть в quiets, если ограничить длину quiets
        if !is_noisy && quiet_count < quiets.len() {
            quiets[quiet_count] = mv;
            quiet_count += 1;
        }

        let mut score = alpha + 1;
        if reduction > 0 {
            score = -(search(
                e,
                -(alpha + 1),
                -alpha,
                new_depth - reduction,
                height + 1,
                Move::NONE,
            )?);
        }
        if score > alpha && beta != alpha + 1 && moves_searched > 1 && new_depth > 0 {
            score = -(search(e, -(alpha + 1), -alpha, new_depth, height + 1, Move::NONE)?);
        }
        if score > alpha {
            score = -(search(e, -beta, -alpha, new_depth, height + 1, Move::NONE)?);
        }

        unmake_move(e);
        if score > best {
            best = score;
            best_move = mv;
        }
        if score > alpha {
            alpha = score;
            assign_pv(e, height, mv);
            if alpha >= beta {
                break;
            }
        }
    }

    if !has_legal_move {
        if !in_check && skip_move == Move::NONE {
            return Some(utils::VALUE_DRAW);
        }
        return Some(utils::loss_in(height));
    }

    if alpha > old_alpha && best_move != Move::NONE && !utils::is_capture_or_promotion(best_move) {
        update_killer(e, height, best_move);

        e.history
            .update(&moveorder_context, &quiets[..quiet_count], best_move, depth);
    }

    if skip_move == Move::NONE {
        let bound = if best >= beta {
            transtable::BOUND_LOWER
        } else if best > old_alpha {
            transtable::BOUND_EXACT
        } else {
            transtable::BOUND_UPPER
        };
        if !(root_node && bound == transtable::BOUND_UPPER) {
            e.trans_table.update_old_policy(
                e.stack[height].position.key,
                depth,
                utils::value_to_tt(best, height),
                bound,
                best_move,
            );
        }
    }

    return Some(best);
}

fn qs(e: &mut Engine, mut alpha: isize, beta: isize, height: usize) -> Option<isize> {
    e.stack[height].pv_size = 0;

    if is_repeat(e, height) {
        return Some(utils::VALUE_DRAW);
    }
    let stack = &e.stack[height];
    if height >= utils::MAX_HEIGHT {
        return Some(evaluate2(e, height));
    }
    if utils::is_draw(&stack.position) {
        return Some(utils::VALUE_DRAW);
    }

    let (_, mut tt_value, tt_bound, tt_move, tt_hit) =
        e.trans_table.read(e.stack[height].position.key);
    if tt_hit {
        tt_value = utils::value_from_tt(tt_value, height);
        if tt_bound == transtable::BOUND_EXACT
            || tt_bound == transtable::BOUND_LOWER && tt_value >= beta
            || tt_bound == transtable::BOUND_UPPER && tt_value <= alpha
        {
            return Some(tt_value);
        }
    }

    let in_check = stack.position.is_check();
    let mut ml = MoveList::new();
    let mut best = utils::loss_in(height);
    if best > alpha {
        alpha = best;
        if alpha >= beta {
            return Some(alpha);
        }
    }
    if in_check {
        let moveorder_context = get_moveorder_context(e, height, tt_move);
        let pos = &stack.position;
        ml.gen_moves(pos);
        moveorder_context.evaluate_moves_counter55(&mut ml, pos, &e.history);
    } else {
        let static_eval = evaluate(e.evaluator.as_mut(), &stack.position);
        best = best.max(static_eval);
        if static_eval > alpha {
            alpha = static_eval;
            if alpha >= beta {
                return Some(alpha);
            }
        }
        ml.gen_captures(&stack.position);
        moveorder::evaluate_captures(&mut ml);
    }
    ml.sort();

    for item in &ml.moves[..ml.size] {
        let mv = item.mv;
        if !in_check && !see::see_ge(&e.stack[height].position, mv, 0) {
            continue;
        }
        if !make_move(e, mv, height) {
            continue;
        }
        if is_timeout(e) {
            return None;
        }
        let score = -(qs(e, -beta, -alpha, height + 1)?);
        unmake_move(e);
        best = best.max(score);
        if score > alpha {
            alpha = score;
            //assign_pv(e, height, mv);
            if alpha >= beta {
                break;
            }
        }
    }
    return Some(best);
}

fn update_killer(e: &mut Engine, height: usize, mv: Move) {
    let stack = &mut e.stack[height];
    if stack.killer1 != mv {
        stack.killer2 = stack.killer1;
        stack.killer1 = mv;
    }
}

fn is_repeat(e: &Engine, height: usize) -> bool {
    let key = e.stack[height].position.key;
    for item in e.stack[..height].iter().rev() {
        let mv = item.current_mv;
        if mv.is_null() || mv.moving_piece() == Piece::PAWN || mv.captured_piece() != Piece::NONE {
            return false;
        }
        if item.position.key == key {
            return true;
        }
    }
    return e.repeats.contains(&key);
}

fn evaluate(evaluator: &mut dyn IEvaluator, pos: &Position) -> isize {
    let mut res = evaluator.quik_evaluate(pos).clamp(-20_000, 20_000);
    if pos.side_to_move == Side::BLACK {
        res = -res;
    }
    return res;
}

fn evaluate2(e: &mut Engine, height: usize) -> isize {
    let pos = &e.stack[height].position;
    let mut res = e.evaluator.quik_evaluate(pos).clamp(-20_000, 20_000);
    if pos.side_to_move == Side::BLACK {
        res = -res;
    }
    return res;
}

fn is_timeout(e: &mut Engine) -> bool {
    e.nodes += 1;
    const CHECK_NODES_MASK: u64 = (1_u64 << 11) - 1;
    if e.nodes & CHECK_NODES_MASK == 0 {
        return e.time_manager.check_timeout(e.nodes);
    }
    return false;
}

fn make_move(e: &mut Engine, mv: Move, height: usize) -> bool {
    let (parent, child) = get_pair_mut(&mut e.stack, height);
    if mv.is_null() {
        parent.position.make_null_move(&mut child.position);
    } else {
        if !parent.position.make_move(mv, &mut child.position) {
            return false;
        }
    }
    e.evaluator.make_move(&parent.position, mv);
    parent.current_mv = mv;
    return true;
}

fn unmake_move(e: &mut Engine) {
    e.evaluator.unmake_move();
}

fn assign_pv(e: &mut Engine, height: usize, m: Move) {
    let (parent, child) = get_pair_mut(&mut e.stack, height);
    let child_size = child.pv_size;
    parent.pv_size = 1 + child_size;
    parent.pv[0] = m;
    parent.pv[1..1 + child_size].copy_from_slice(&child.pv[..child_size]);
}

fn get_pair_mut(stack: &mut [SearchStack], height: usize) -> (&mut SearchStack, &mut SearchStack) {
    let (head, tail) = stack.split_at_mut(height + 1);
    return (&mut head[height], &mut tail[0]);
}

fn lirp(x: f64, x1: f64, x2: f64, y1: f64, y2: f64) -> f64 {
    return y1 + (y2 - y1) * (x - x1) / (x2 - x1);
}

fn lmr_counter55(d: f64, m: f64) -> f64 {
    lirp(
        d.ln() * m.ln(),
        5_f64.ln() * 22_f64.ln(),
        63_f64.ln() * 63_f64.ln(),
        3_f64,
        8_f64,
    )
}

fn get_moveorder_context(
    e: &Engine,
    height: usize,
    trans_move: Move,
) -> moveorder::MoveOrderContext {
    let side = e.stack[height].position.side_to_move;
    let killer1 = e.stack[height].killer1;
    let killer2 = e.stack[height].killer2;
    let counter_move = if height >= 1 {
        e.stack[height - 1].current_mv
    } else {
        Move::NONE
    };
    let follow_move = if height >= 2 {
        e.stack[height - 2].current_mv
    } else {
        Move::NONE
    };
    return moveorder::MoveOrderContext::new(
        side,
        trans_move,
        killer1,
        killer2,
        counter_move,
        follow_move,
    );
}
