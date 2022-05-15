use cods::{Context, Cst, Ident, IdentSpan, Scopes, Span, Stack, Val};
use egui::plot::Value;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString, IntoEnumIterator, IntoStaticStr};

use crate::data::{Data, DataEntry, TimeStamped};

fn get_value(e: &DataEntry, var: Var) -> Val {
    let val = match var {
        Var::Time => e.time(),
        Var::PowerFl => e.power_fl(),
        Var::PowerFr => e.power_fr(),
        Var::PowerRl => e.power_rl(),
        Var::PowerRr => e.power_rr(),
        Var::VelocityFl => e.velocity_fl(),
        Var::VelocityFr => e.velocity_fr(),
        Var::VelocityRl => e.velocity_rl(),
        Var::VelocityRr => e.velocity_rr(),
        Var::TorqueSetFl => e.torque_set_fl(),
        Var::TorqueSetFr => e.torque_set_fr(),
        Var::TorqueSetRl => e.torque_set_rl(),
        Var::TorqueSetRr => e.torque_set_rr(),
        Var::TorqueRealFl => e.torque_real_fl(),
        Var::TorqueRealFr => e.torque_real_fr(),
        Var::TorqueRealRl => e.torque_real_rl(),
        Var::TorqueRealRr => e.torque_real_rr(),
    };

    Val::Float(val)
}

#[derive(Clone, Copy, Debug, PartialEq, EnumIter, EnumString, IntoStaticStr, Display)]
pub enum Var {
    #[strum(serialize = "t")]
    Time,
    #[strum(serialize = "P_fl")]
    PowerFl,
    #[strum(serialize = "P_fr")]
    PowerFr,
    #[strum(serialize = "P_rl")]
    PowerRl,
    #[strum(serialize = "P_rr")]
    PowerRr,
    #[strum(serialize = "v_fl")]
    VelocityFl,
    #[strum(serialize = "v_fr")]
    VelocityFr,
    #[strum(serialize = "v_rl")]
    VelocityRl,
    #[strum(serialize = "v_rr")]
    VelocityRr,
    #[strum(serialize = "M_set_fl")]
    TorqueSetFl,
    #[strum(serialize = "M_set_fr")]
    TorqueSetFr,
    #[strum(serialize = "M_set_rl")]
    TorqueSetRl,
    #[strum(serialize = "M_set_rr")]
    TorqueSetRr,
    #[strum(serialize = "M_real_fl")]
    TorqueRealFl,
    #[strum(serialize = "M_real_fr")]
    TorqueRealFr,
    #[strum(serialize = "M_real_rl")]
    TorqueRealRl,
    #[strum(serialize = "M_real_rr")]
    TorqueRealRr,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Expr {
    pub x: String,
    pub y: String,
}

pub fn eval(expr: &Expr, data: &Data) -> anyhow::Result<Vec<Value>> {
    let mut ctx_x = Context::default();
    let mut ctx_y = Context::default();
    for v in Var::iter() {
        ctx_x.idents.push(v.into());
        ctx_y.idents.push(v.into());
    }

    let csts_x = parse(&mut ctx_x, &expr.x)?;
    let csts_y = parse(&mut ctx_y, &expr.y)?;

    let var_count = Var::iter().count();
    let mut vars = Vec::with_capacity(var_count);
    let mut scopes_x = Scopes::default();
    let mut scopes_y = Scopes::default();
    for (id, v) in Var::iter().enumerate() {
        let ident = IdentSpan::new(Ident(id), Span::pos(0));
        let inner = ctx_x.def_var(&mut scopes_x, ident, cods::DataType::Float, true, false);
        ctx_y.def_var(&mut scopes_y, ident, cods::DataType::Float, true, false);
        vars.push((inner, v));
    }

    let asts_x = ctx_x.check_with(&mut scopes_x, csts_x)?;
    let asts_y = ctx_y.check_with(&mut scopes_y, csts_y)?;

    let mut values = Vec::with_capacity(data.len());
    let mut stack_x = Stack::default();
    let mut stack_y = Stack::default();
    stack_x.extend_to(vars.len());
    stack_y.extend_to(vars.len());
    for e in data.iter() {
        for (var, val) in vars.iter() {
            let val = get_value(e, *val);
            stack_x.set(var, val.clone());
            stack_y.set(var, val);
        }

        let x = cods::eval_with(&mut stack_x, &asts_x);
        let y = cods::eval_with(&mut stack_y, &asts_y);

        if let (Ok(x), Ok(y)) = (x, y) {
            if let (Some(x), Some(y)) = (cast_float(x), cast_float(y)) {
                values.push(Value::new(x, y));
            }
        };
    }

    Ok(values)
}

fn parse(ctx: &mut Context, input: &str) -> anyhow::Result<Vec<Cst>> {
    let tokens = ctx.lex(input)?;
    let items = ctx.group(tokens)?;
    let csts = ctx.parse(items)?;
    if !ctx.errors.is_empty() {
        return Err(ctx.errors.remove(0).into());
    }
    Ok(csts)
}

fn cast_float(val: Val) -> Option<f64> {
    match val {
        Val::Int(i) => Some(i as f64),
        Val::Float(f) => Some(f),
        _ => None,
    }
}
