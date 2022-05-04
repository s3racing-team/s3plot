use std::rc::Rc;

use cods::{Context, Cst, Ident, IdentSpan, Scopes, Span, Val};
use egui::plot::Value;
use serde::Deserialize;
use serde::Serialize;
use strum::{Display, EnumIter, EnumString, IntoEnumIterator, IntoStaticStr};

use crate::data::SAMPLE_RATE;
use crate::data::{self, Data};

fn get_value(data: &Data, index: usize, var: Var) -> Val {
    let i = index;
    let val = match var {
        Var::Time => index as f64 * SAMPLE_RATE,
        Var::PowerFl => data::power_fl(&data[i]) as f64,
        Var::PowerFr => data::power_fr(&data[i]) as f64,
        Var::PowerRl => data::power_rl(&data[i]) as f64,
        Var::PowerRr => data::power_rr(&data[i]) as f64,
        Var::VelocityFl => data::velocity_fl(&data[i]) as f64,
        Var::VelocityFr => data::velocity_fr(&data[i]) as f64,
        Var::VelocityRl => data::velocity_rl(&data[i]) as f64,
        Var::VelocityRr => data::velocity_rr(&data[i]) as f64,
        Var::TorqueSetFl => data::torque_set_fl(&data[i]) as f64,
        Var::TorqueSetFr => data::torque_set_fr(&data[i]) as f64,
        Var::TorqueSetRl => data::torque_set_rl(&data[i]) as f64,
        Var::TorqueSetRr => data::torque_set_rr(&data[i]) as f64,
        Var::TorqueRealFl => data::torque_real_fl(&data[i]) as f64,
        Var::TorqueRealFr => data::torque_real_fr(&data[i]) as f64,
        Var::TorqueRealRl => data::torque_real_rl(&data[i]) as f64,
        Var::TorqueRealRr => data::torque_real_rr(&data[i]) as f64,
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
    for i in 0..data.len() {
        for (id, v) in Var::iter().enumerate() {
            let val = get_value(data, i, v);
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
