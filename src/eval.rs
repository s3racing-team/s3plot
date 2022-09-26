use std::sync::Arc;

use cods::{Asts, Checker, Context, Funs, Ident, IdentSpan, Span, Stack, Val, VarRef};
use egui::plot::PlotPoint;
use serde::{Deserialize, Serialize};

use crate::data::LogStream;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Expr {
    pub x: String,
    pub y: String,
}

impl Expr {
    pub fn new(x: impl Into<String>, y: impl Into<String>) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
        }
    }
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

    let mut lerp_values = Vec::with_capacity(data.len() - 1);
    for d in data.iter().skip(1) {
        lerp_values.push((0, &d.time[0..1]));
    }
    for (i, &time) in data[0].time.iter().enumerate() {
        for (j, d) in data.iter().skip(1).enumerate() {
            let mut d_index = 0;
            while let Some(&t) = d.time.get(d_index) {
                if t == time || t > time && d_index == 0 {
                    lerp_values[j] = (d_index, &d.time[d_index..d_index + 1]);
                } else if t > time {
                    lerp_values[j] = (d_index - 1, &d.time[d_index - 1..d_index + 1]);
                } else if d_index + 1 == d.len() {
                    lerp_values[j] = (d_index, &d.time[d_index..d_index + 1]);
                } else {
                    d_index += 1;
                    continue;
                }
                break;
            }
        }

        for (var, id) in vars_x.iter() {
            let val = get_value(&data, *id, i, time, &lerp_values);
            stack_x.set(var, val);
        }
        for (var, id) in vars_y.iter() {
            let val = get_value(&data, *id, i, time, &lerp_values);
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

fn get_value(
    data: &[LogStream],
    id: (usize, usize),
    index: usize,
    time: u32,
    lerp_values: &[(usize, &[u32])],
) -> Val {
    if id.0 == 0 {
        Val::Float(data[id.0].entries[id.1].kind.get_f64(index))
    } else if id.0 < data.len() {
        match lerp_values[id.0 - 1] {
            (index, [_time]) => Val::Float(data[id.0].entries[id.1].kind.get_f64(index)),
            (index, [time0, time1]) => {
                let range = time1 - time0;
                let pos = time - time0;
                let factor = pos as f64 / range as f64;
                let val0 = data[id.0].entries[id.1].kind.get_f64(index);
                let val1 = data[id.0].entries[id.1].kind.get_f64(index + 1);
                Val::Float(val0 + factor * (val1 - val0))
            }
            _ => Val::Float(f64::NAN),
        }
    } else {
        Val::Float(time as f64 / 1000.0)
    }
}
