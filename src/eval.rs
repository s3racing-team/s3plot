use std::str::FromStr;

use cods::{Ast, Context, Val, VarId};
use egui::plot::Value;
use serde::Deserialize;
use serde::Serialize;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::data::Data;
use crate::data::SAMPLE_RATE;

const TIME: &str = "t";
const POWER_FL: &str = "P_fl";
const POWER_FR: &str = "P_fr";
const POWER_RL: &str = "P_rl";
const POWER_RR: &str = "P_rr";
const VELOCITY_FL: &str = "v_fl";
const VELOCITY_FR: &str = "v_fr";
const VELOCITY_RL: &str = "v_rl";
const VELOCITY_RR: &str = "v_rr";
const TORQUE_SET_FL: &str = "M_set_fl";
const TORQUE_SET_FR: &str = "M_set_fr";
const TORQUE_SET_RL: &str = "M_set_rl";
const TORQUE_SET_RR: &str = "M_set_rr";
const TORQUE_REAL_FL: &str = "M_real_fl";
const TORQUE_REAL_FR: &str = "M_real_fr";
const TORQUE_REAL_RL: &str = "M_real_rl";
const TORQUE_REAL_RR: &str = "M_real_rr";

fn get_value(data: &Data, index: usize, var: Var) -> Val {
    let i = index;
    let val = match var {
        Var::Time => index as f64 * SAMPLE_RATE,
        Var::PowerFl => data.power_fl().nth(i).unwrap() as f64,
        Var::PowerFr => data.power_fr().nth(i).unwrap() as f64,
        Var::PowerRl => data.power_rl().nth(i).unwrap() as f64,
        Var::PowerRr => data.power_rr().nth(i).unwrap() as f64,
        Var::VelocityFl => data.velocity_fl().nth(i).unwrap() as f64,
        Var::VelocityFr => data.velocity_fr().nth(i).unwrap() as f64,
        Var::VelocityRl => data.velocity_rl().nth(i).unwrap() as f64,
        Var::VelocityRr => data.velocity_rr().nth(i).unwrap() as f64,
        Var::TorqueSetFl => data.torque_set_fl().nth(i).unwrap() as f64,
        Var::TorqueSetFr => data.torque_set_fr().nth(i).unwrap() as f64,
        Var::TorqueSetRl => data.torque_set_rl().nth(i).unwrap() as f64,
        Var::TorqueSetRr => data.torque_set_rr().nth(i).unwrap() as f64,
        Var::TorqueRealFl => data.torque_real_fl().nth(i).unwrap() as f64,
        Var::TorqueRealFr => data.torque_real_fr().nth(i).unwrap() as f64,
        Var::TorqueRealRl => data.torque_real_rl().nth(i).unwrap() as f64,
        Var::TorqueRealRr => data.torque_real_rr().nth(i).unwrap() as f64,
    };

    Val::Float(val)
}

#[derive(Clone, Copy, Debug, PartialEq, EnumIter)]
pub enum Var {
    Time,
    PowerFl,
    PowerFr,
    PowerRl,
    PowerRr,
    VelocityFl,
    VelocityFr,
    VelocityRl,
    VelocityRr,
    TorqueSetFl,
    TorqueSetFr,
    TorqueSetRl,
    TorqueSetRr,
    TorqueRealFl,
    TorqueRealFr,
    TorqueRealRl,
    TorqueRealRr,
}

impl FromStr for Var {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            TIME => Ok(Self::Time),
            POWER_FL => Ok(Self::PowerFl),
            POWER_FR => Ok(Self::PowerFr),
            POWER_RL => Ok(Self::PowerRl),
            POWER_RR => Ok(Self::PowerRr),
            VELOCITY_FL => Ok(Self::VelocityFl),
            VELOCITY_FR => Ok(Self::VelocityFr),
            VELOCITY_RL => Ok(Self::VelocityRl),
            VELOCITY_RR => Ok(Self::VelocityRr),
            TORQUE_SET_FL => Ok(Self::TorqueSetFl),
            TORQUE_SET_FR => Ok(Self::TorqueSetFr),
            TORQUE_SET_RL => Ok(Self::TorqueSetRl),
            TORQUE_SET_RR => Ok(Self::TorqueSetRr),
            TORQUE_REAL_FL => Ok(Self::TorqueRealFl),
            TORQUE_REAL_FR => Ok(Self::TorqueRealFr),
            TORQUE_REAL_RL => Ok(Self::TorqueRealRl),
            TORQUE_REAL_RR => Ok(Self::TorqueRealRr),
            _ => Err(()),
        }
    }
}

impl Var {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Time => TIME,
            Self::PowerFl => POWER_FL,
            Self::PowerFr => POWER_FR,
            Self::PowerRl => POWER_RL,
            Self::PowerRr => POWER_RR,
            Self::VelocityFl => VELOCITY_FL,
            Self::VelocityFr => VELOCITY_FR,
            Self::VelocityRl => VELOCITY_RL,
            Self::VelocityRr => VELOCITY_RR,
            Self::TorqueSetFl => TORQUE_SET_FL,
            Self::TorqueSetFr => TORQUE_SET_FR,
            Self::TorqueSetRl => TORQUE_SET_RL,
            Self::TorqueSetRr => TORQUE_SET_RR,
            Self::TorqueRealFl => TORQUE_REAL_FL,
            Self::TorqueRealFr => TORQUE_REAL_FR,
            Self::TorqueRealRl => TORQUE_REAL_RL,
            Self::TorqueRealRr => TORQUE_REAL_RR,
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct Expr {
    pub x: String,
    pub y: String,
}

pub fn eval(expr: &Expr, data: &Data) -> anyhow::Result<Vec<Value>> {
    let mut ctx = Context::default();
    for v in Var::iter() {
        ctx.push_var(v.name());
    }

    let calc_x = parse(&mut ctx, &expr.x)?;
    let calc_y = parse(&mut ctx, &expr.y)?;

    let var_count = Var::iter().count();
    let mut values = Vec::with_capacity(data.len);
    for i in 0..data.len {
        ctx.clear_errors();
        ctx.vars.shrink_to(var_count);
        for (id, v) in Var::iter().enumerate() {
            let val = get_value(data, i, v);
            ctx.set_var(VarId(id), Some(val));
        }

        let x = ctx.eval_all(&calc_x);
        let y = ctx.eval_all(&calc_y);
        if let (Ok(Some(x)), Ok(Some(y))) = (x, y) {
            if let (Some(x), Some(y)) = (x.to_f64(), y.to_f64()) {
                values.push(Value::new(x, y));
            }
        };
    }

    Ok(values)
}

fn parse(ctx: &mut Context, input: &str) -> anyhow::Result<Vec<Ast>> {
    let ast = ctx.parse_str(input)?;
    if !ctx.errors.is_empty() {
        Err(ctx.errors.remove(0))?;
    }
    Ok(ast)
}
