use std::sync::Arc;

use cods::{Asts, Checker, Context, Funs, Ident, IdentSpan, Span, Stack, Val, VarRef};
use egui::plot::PlotPoint;
use serde::{Deserialize, Serialize};

use crate::data::LogStream;

// fn lerp(time: f64, timed_values: &[(f64, f64)]) -> f64 {
//     match timed_values {
//         [(t, v)] => *v,
//         [(t1, v1), (t2, v2)] => {
//             let range = t2 - t1;
//             let pos = time - t1;
//             let factor = pos / range;
//             v1 + factor * (v2 - v1)
//         }
//         _ => f64::NAN,
//     }
// }

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Expr {
    pub x: String,
    pub y: String,
}

#[derive(Default)]
pub struct ExprError {
    pub x: Option<cods::Error>,
    pub y: Option<cods::Error>,
}

pub fn eval(expr: &Expr, data: Arc<[LogStream]>) -> Result<Vec<PlotPoint>, Box<ExprError>> {
    let mut ctx_x = Context::default();
    let mut ctx_y = Context::default();

    // number of all entries plus the always present time entry
    let num_vars = data.iter().map(|g| g.entries.len()).sum::<usize>() + 1;
    let mut vars_x = Vec::with_capacity(num_vars);
    let mut vars_y = Vec::with_capacity(num_vars);

    let asts_x = parse(&data, &mut ctx_x, &mut vars_x, &expr.x);
    let asts_y = parse(&data, &mut ctx_y, &mut vars_y, &expr.y);

    let ((funs_x, asts_x), (funs_y, asts_y)) = match (asts_x, asts_y) {
        (Ok(x), Ok(y)) => (x, y),
        (x, y) => {
            return Err(Box::new(ExprError {
                x: x.err(),
                y: y.err(),
            }));
        }
    };

    let mut values = Vec::with_capacity(data.len());
    let mut stack_x = Stack::default();
    let mut stack_y = Stack::default();
    stack_x.resize(vars_x.len());
    stack_y.resize(vars_y.len());

    for (i, &time) in data[0].time.iter().enumerate() {
        // TODO: figure out which stream has the highest frequency,
        //       use that as the time axis and lerp the other streams values.
        //
        // while let Some(t) = temp.get(temp_index) {
        //     if t.ms == d.ms || t.ms > d.ms && temp_index == 0 {
        //         temp_entries = std::slice::from_ref(t);
        //     } else if t.ms > d.ms {
        //         temp_entries = &temp[temp_index - 1..temp_index + 1];
        //     } else if temp_index + 1 == temp.len() {
        //         temp_entries = std::slice::from_ref(t);
        //     } else {
        //         temp_index += 1;
        //         continue;
        //     }
        //     break;
        // }

        for (var, id) in vars_x.iter() {
            let val = get_value(&data, *id, i, time);
            stack_x.set(var, val);
        }
        for (var, id) in vars_y.iter() {
            let val = get_value(&data, *id, i, time);
            stack_y.set(var, val);
        }

        let x = cods::eval_with(&mut stack_x, &funs_x, &asts_x);
        let y = cods::eval_with(&mut stack_y, &funs_y, &asts_y);

        if let (Ok(x), Ok(y)) = (x, y) {
            if let (Some(x), Some(y)) = (cast_float(x), cast_float(y)) {
                values.push(PlotPoint::new(x, y));
            }
        };
    }

    Ok(values)
}

fn parse(
    data: &[LogStream],
    ctx: &mut Context,
    vars: &mut Vec<(VarRef, (usize, usize))>,
    input: &str,
) -> cods::Result<(Funs, Asts)> {
    for v in data.iter().flat_map(|g| g.entries.iter()) {
        ctx.idents.push(&v.name);
    }
    ctx.idents.push("time");

    let tokens = ctx.lex(input)?;
    let items = ctx.group(tokens)?;
    let csts = ctx.parse(items)?;

    let mut checker = Checker::default();
    let mut id = 0;
    for (i, group) in data.iter().enumerate() {
        for j in 0..group.entries.len() {
            let ident = IdentSpan::new(Ident(id), Span::pos(0, 0));
            let inner = ctx.def_var(
                &mut checker.scopes,
                ident,
                cods::DataType::Float,
                true,
                false,
            );
            vars.push((inner, (i, j)));

            id += 1;
        }
    }
    let ident = IdentSpan::new(Ident(vars.len()), Span::pos(0, 0));
    let inner = ctx.def_var(
        &mut checker.scopes,
        ident,
        cods::DataType::Float,
        true,
        false,
    );
    vars.push((inner, (data.len(), 0)));

    let asts = ctx.check_with(&mut checker, csts)?;
    if !ctx.errors.is_empty() {
        return Err(ctx.errors.remove(0));
    }

    Ok((checker.funs, asts))
}

fn cast_float(val: Val) -> Option<f64> {
    match val {
        Val::Int(i) => Some(i as f64),
        Val::Float(f) => Some(f),
        _ => None,
    }
}

fn get_value(data: &[LogStream], id: (usize, usize), index: usize, time: u32) -> Val {
    if id.0 < data.len() {
        Val::Float(data[id.0].entries[id.1].kind.get_f64(index))
    } else {
        Val::Float(time as f64 / 1000.0)
    }
}
