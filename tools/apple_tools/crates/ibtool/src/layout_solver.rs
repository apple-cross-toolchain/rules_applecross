//! A linear constraint solver for laying AppKit views out as Auto Layout
//! does (appkit.rs `auto_layout`): required constraints hold; then, from
//! the highest priority down, each priority's constraints are violated as
//! little as they can be (the sum of their violations least) without
//! giving up what higher ones got. A small dense simplex, for the handful
//! of views one superview holds.

/// How a constraint relates its expression to zero.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Relation {
    Equal,
    AtMost,
    AtLeast,
}

/// `coefficients · variables + constant (relation) 0`, at a priority
/// (1000 required).
#[derive(Clone, Debug)]
pub struct Constraint {
    pub coefficients: Vec<(usize, f64)>,
    pub constant: f64,
    pub relation: Relation,
    pub priority: f64,
}

const EPSILON: f64 = 1e-9;

/// The variables' values, `None` when the required constraints conflict.
pub fn solve(variables: usize, constraints: &[Constraint]) -> Option<Vec<f64>> {
    let mut priorities: Vec<f64> = constraints.iter().filter(|c| c.priority < 1000.0).map(|c| c.priority).collect();
    priorities.sort_by(|a, b| b.total_cmp(a));
    priorities.dedup();
    // Each free variable is the difference of two non-negative columns;
    // each soft constraint gets violation columns, below or above.
    let mut settled: Vec<(f64, f64)> = Vec::new();
    let mut last = None;
    for level in std::iter::once(None).chain(priorities.iter().copied().map(Some)) {
        let (values, objective) = lp(variables, constraints, level, &settled)?;
        if let Some(p) = level {
            settled.push((p, objective));
        }
        last = Some(values);
    }
    last
}

/// One stage: the required constraints hard, the soft ones of priorities
/// already settled no worse than they were (`settled`: priority, its
/// violations' least sum), and `level`'s violations least.
fn lp(variables: usize, constraints: &[Constraint], level: Option<f64>, settled: &[(f64, f64)]) -> Option<(Vec<f64>, f64)> {
    // Columns: 2 per variable, then 2 per soft constraint (violations),
    // then slack columns for inequalities.
    let soft: Vec<usize> = (0..constraints.len()).filter(|i| constraints[*i].priority < 1000.0).collect();
    let soft_column = |k: usize| 2 * variables + 2 * k;
    let mut rows: Vec<(Vec<(usize, f64)>, f64, Relation)> = Vec::new();
    for (i, c) in constraints.iter().enumerate() {
        let mut row: Vec<(usize, f64)> = Vec::new();
        for &(v, a) in &c.coefficients {
            row.push((2 * v, a));
            row.push((2 * v + 1, -a));
        }
        if let Some(k) = soft.iter().position(|s| *s == i) {
            // expression + below - above (relation) 0: an equality may be
            // missed either way, an inequality only the wrong way.
            match c.relation {
                Relation::Equal => {
                    row.push((soft_column(k), 1.0));
                    row.push((soft_column(k) + 1, -1.0));
                }
                Relation::AtMost => row.push((soft_column(k) + 1, -1.0)),
                Relation::AtLeast => row.push((soft_column(k), 1.0)),
            }
        }
        rows.push((row, -c.constant, c.relation));
    }
    // Settled priorities: their violations' sum no more than settled.
    for &(p, best) in settled {
        let row: Vec<(usize, f64)> = soft.iter().enumerate().filter(|(_, i)| constraints[**i].priority == p).flat_map(|(k, _)| [(soft_column(k), 1.0), (soft_column(k) + 1, 1.0)]).collect();
        rows.push((row, best + 1e-7, Relation::AtMost));
    }
    let mut objective = vec![0.0; 2 * variables + 2 * soft.len()];
    if let Some(p) = level {
        for (k, i) in soft.iter().enumerate() {
            if constraints[*i].priority == p {
                objective[soft_column(k)] = 1.0;
                objective[soft_column(k) + 1] = 1.0;
            }
        }
    }
    let columns = objective.len();
    let (x, value) = simplex(columns, &rows, &objective)?;
    let values = (0..variables).map(|v| x[2 * v] - x[2 * v + 1]).collect();
    Some((values, value))
}

/// Minimizes `objective · x` for `x ≥ 0` subject to `rows` (sparse
/// coefficients, right-hand side, relation): two phases, Bland's rule.
fn simplex(columns: usize, rows: &[(Vec<(usize, f64)>, f64, Relation)], objective: &[f64]) -> Option<(Vec<f64>, f64)> {
    let m = rows.len();
    // Slacks for inequalities, then an artificial per row.
    let slacks: Vec<Option<usize>> = {
        let mut next = columns;
        rows.iter()
            .map(|r| match r.2 {
                Relation::Equal => None,
                _ => {
                    next += 1;
                    Some(next - 1)
                }
            })
            .collect()
    };
    let total_slack = slacks.iter().flatten().count();
    let artificial0 = columns + total_slack;
    let width = artificial0 + m;
    let mut t = vec![vec![0.0; width + 1]; m];
    for (i, (row, b, relation)) in rows.iter().enumerate() {
        for &(j, a) in row {
            t[i][j] += a;
        }
        if let Some(s) = slacks[i] {
            t[i][s] = if *relation == Relation::AtMost { 1.0 } else { -1.0 };
        }
        t[i][width] = *b;
        if t[i][width] < 0.0 {
            for v in t[i].iter_mut() {
                *v = -*v;
            }
        }
        t[i][artificial0 + i] = 1.0;
    }
    let mut basis: Vec<usize> = (0..m).map(|i| artificial0 + i).collect();
    // Phase one: drive the artificials out.
    let mut phase1 = vec![0.0; width];
    for c in phase1.iter_mut().skip(artificial0) {
        *c = 1.0;
    }
    run(&mut t, &mut basis, &phase1, width)?;
    let infeasibility: f64 = basis.iter().enumerate().filter(|(_, b)| **b >= artificial0).map(|(i, _)| t[i][width]).sum();
    if infeasibility > 1e-6 {
        return None;
    }
    // Artificials left in the basis at zero: pivot them out where a real
    // column can take their place.
    for i in 0..m {
        if basis[i] >= artificial0 {
            if let Some(j) = (0..artificial0).find(|j| t[i][*j].abs() > EPSILON) {
                pivot(&mut t, &mut basis, i, j, width);
            }
        }
    }
    // Phase two, artificials out of play.
    for row in t.iter_mut() {
        for v in row.iter_mut().take(width).skip(artificial0) {
            *v = 0.0;
        }
    }
    let mut cost = vec![0.0; width];
    cost[..objective.len()].copy_from_slice(objective);
    run(&mut t, &mut basis, &cost, artificial0)?;
    let mut x = vec![0.0; columns];
    for (i, b) in basis.iter().enumerate() {
        if *b < columns {
            x[*b] = t[i][width];
        }
    }
    let value = objective.iter().zip(&x).map(|(c, v)| c * v).sum();
    Some((x, value))
}

/// Simplex iterations over the columns before `limit`, minimizing `cost`.
fn run(t: &mut [Vec<f64>], basis: &mut [usize], cost: &[f64], limit: usize) -> Option<()> {
    let width = t[0].len() - 1;
    for _ in 0..10_000 {
        // Reduced costs; Bland: the first column that improves.
        let entering = (0..limit).find(|&j| {
            if basis.contains(&j) {
                return false;
            }
            let reduced = cost[j] - basis.iter().enumerate().map(|(i, b)| cost[*b] * t[i][j]).sum::<f64>();
            reduced < -EPSILON
        });
        let Some(j) = entering else { return Some(()) };
        let mut leaving: Option<(usize, f64)> = None;
        for i in 0..t.len() {
            if t[i][j] > EPSILON {
                let ratio = t[i][width] / t[i][j];
                match leaving {
                    Some((l, r)) if ratio > r + EPSILON || (ratio > r - EPSILON && basis[i] > basis[l]) => {}
                    _ => leaving = Some((i, ratio)),
                }
            }
        }
        let (i, _) = leaving?;
        pivot(t, basis, i, j, width);
    }
    None
}

fn pivot(t: &mut [Vec<f64>], basis: &mut [usize], i: usize, j: usize, width: usize) {
    let p = t[i][j];
    for v in t[i].iter_mut() {
        *v /= p;
    }
    for r in 0..t.len() {
        if r != i && t[r][j].abs() > EPSILON {
            let f = t[r][j];
            for c in 0..=width {
                t[r][c] -= f * t[i][c];
            }
        }
    }
    basis[i] = j;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(coefficients: &[(usize, f64)], constant: f64, relation: Relation, priority: f64) -> Constraint {
        Constraint { coefficients: coefficients.to_vec(), constant, relation, priority }
    }

    #[test]
    fn compression_beats_hugging() {
        // Two equal widths, each hugging its content (250) and resisting
        // compression (750): both the wider content's.
        let cs = [
            c(&[(0, 1.0), (1, -1.0)], 0.0, Relation::Equal, 1000.0),
            c(&[(0, 1.0)], -40.0, Relation::AtLeast, 750.0),
            c(&[(0, 1.0)], -40.0, Relation::AtMost, 250.0),
            c(&[(1, 1.0)], -66.0, Relation::AtLeast, 750.0),
            c(&[(1, 1.0)], -66.0, Relation::AtMost, 250.0),
        ];
        let v = solve(2, &cs).unwrap();
        assert!((v[0] - 66.0).abs() < 1e-6 && (v[1] - 66.0).abs() < 1e-6, "{v:?}");
    }

    #[test]
    fn required_conflict() {
        let cs = [c(&[(0, 1.0)], -1.0, Relation::Equal, 1000.0), c(&[(0, 1.0)], -2.0, Relation::Equal, 1000.0)];
        assert!(solve(1, &cs).is_none());
    }
}
