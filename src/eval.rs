use std::rc::Rc;

use cods::{Context, Cst, Ident, IdentSpan, Scopes, Span, Val};
use egui::plot::Value;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString, IntoEnumIterator, IntoStaticStr};

use crate::data::{Data, DataEntry};

fn get_value(e: &DataEntry, var: Var) -> Val {
    let val = match var {
        Var::Time => e.time() as f64,
        Var::PowerFl => e.power_fl() as f64,
        Var::PowerFr => e.power_fr() as f64,
        Var::PowerRl => e.power_rl() as f64,
        Var::PowerRr => e.power_rr() as f64,
        Var::VelocityFl => e.velocity_fl() as f64,
        Var::VelocityFr => e.velocity_fr() as f64,
        Var::VelocityRl => e.velocity_rl() as f64,
        Var::VelocityRr => e.velocity_rr() as f64,
        Var::TorqueSetFl => e.torque_set_fl as f64,
        Var::TorqueSetFr => e.torque_set_fr as f64,
        Var::TorqueSetRl => e.torque_set_rl as f64,
        Var::TorqueSetRr => e.torque_set_rr as f64,
        Var::TorqueRealFl => e.torque_real_fl as f64,
        Var::TorqueRealFr => e.torque_real_fr as f64,
        Var::TorqueRealRl => e.torque_real_rl as f64,
        Var::TorqueRealRr => e.torque_real_rr as f64,
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
    let mut ctx = Context::default();
    for v in Var::iter() {
        ctx.idents.push(v.into());
    }

    let csts_x = parse(&mut ctx, &expr.x)?;
    let csts_y = parse(&mut ctx, &expr.y)?;

    let var_count = Var::iter().count();
    let mut vars = Vec::with_capacity(var_count);
    let mut scopes = Scopes::default();
    for (id, _) in Var::iter().enumerate() {
        let ident = IdentSpan::new(Ident(id), Span::pos(0));
        let inner = Rc::new(cods::ast::Var::new(None));
        let var = cods::Var::new(ident, cods::DataType::Float, true, false, Rc::clone(&inner));
        ctx.def_var(&mut scopes, var);
        vars.push(inner);
    }
    let asts_x = ctx.check_with(csts_x, &mut scopes)?;
    let asts_y = ctx.check_with(csts_y, &mut scopes)?;

    let mut values = Vec::with_capacity(data.len());
    for e in data.iter() {
        for (id, v) in Var::iter().enumerate() {
            let val = get_value(e, v);
            vars[id].set(val)
        }

        let x = cods::eval_all(&asts_x);
        let y = cods::eval_all(&asts_y);

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
