use cods::{Asts, Context, Ident, IdentSpan, Scopes, Span, Stack, Val, VarRef};
use egui::plot::Value;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString, IntoEnumIterator, IntoStaticStr};

use crate::data::{Data, DataEntry, Temp, TempEntry, TimeStamped};

fn lerp(d: &DataEntry, t: &[TempEntry], f: impl Fn(&TempEntry) -> f64) -> f64 {
    match t {
        [a] => f(a),
        [a, b] => {
            let range = b.time() - a.time();
            let pos = d.time() - a.time();
            let factor = pos / range;
            f(a) + factor * (f(b) - f(a))
        }
        _ => f64::NAN,
    }
}

fn get_value(var: Var, d: &DataEntry, t: &[TempEntry]) -> Val {
    let val = match var {
        Var::Time => d.time(),
        // data
        Var::PowerFl => d.power_fl(),
        Var::PowerFr => d.power_fr(),
        Var::PowerRl => d.power_rl(),
        Var::PowerRr => d.power_rr(),
        Var::VelocityFl => d.velocity_fl(),
        Var::VelocityFr => d.velocity_fr(),
        Var::VelocityRl => d.velocity_rl(),
        Var::VelocityRr => d.velocity_rr(),
        Var::TorqueSetFl => d.torque_set_fl(),
        Var::TorqueSetFr => d.torque_set_fr(),
        Var::TorqueSetRl => d.torque_set_rl(),
        Var::TorqueSetRr => d.torque_set_rr(),
        Var::TorqueRealFl => d.torque_real_fl(),
        Var::TorqueRealFr => d.torque_real_fr(),
        Var::TorqueRealRl => d.torque_real_rl(),
        Var::TorqueRealRr => d.torque_real_rr(),
        // temp
        Var::AmsTempMax => lerp(d, t, TempEntry::ams_temp_max),
        Var::WaterTempConverter => lerp(d, t, TempEntry::water_temp_converter),
        Var::WaterTempMotor => lerp(d, t, TempEntry::water_temp_motor),
        Var::TempFl => lerp(d, t, TempEntry::temp_fl),
        Var::TempFr => lerp(d, t, TempEntry::temp_fr),
        Var::TempRl => lerp(d, t, TempEntry::temp_rl),
        Var::TempRr => lerp(d, t, TempEntry::temp_rr),
        Var::RoomTempFl => lerp(d, t, TempEntry::room_temp_fl),
        Var::RoomTempFr => lerp(d, t, TempEntry::room_temp_fr),
        Var::RoomTempRl => lerp(d, t, TempEntry::room_temp_rl),
        Var::RoomTempRr => lerp(d, t, TempEntry::room_temp_rr),
        Var::HeatsinkTempFl => lerp(d, t, TempEntry::heatsink_temp_fl),
        Var::HeatsinkTempFr => lerp(d, t, TempEntry::heatsink_temp_fr),
        Var::HeatsinkTempRl => lerp(d, t, TempEntry::heatsink_temp_rl),
        Var::HeatsinkTempRr => lerp(d, t, TempEntry::heatsink_temp_rr),
    };

    Val::Float(val)
}

#[derive(Clone, Copy, Debug, PartialEq, EnumIter, EnumString, IntoStaticStr, Display)]
pub enum Var {
    #[strum(serialize = "t")]
    Time,
    // data
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
    // temp
    #[strum(serialize = "T_ams_max")]
    AmsTempMax,
    #[strum(serialize = "T_water_converter")]
    WaterTempConverter,
    #[strum(serialize = "T_water_motor")]
    WaterTempMotor,
    #[strum(serialize = "T_fl")]
    TempFl,
    #[strum(serialize = "T_fr")]
    TempFr,
    #[strum(serialize = "T_rl")]
    TempRl,
    #[strum(serialize = "T_rr")]
    TempRr,
    #[strum(serialize = "T_room_fl")]
    RoomTempFl,
    #[strum(serialize = "T_room_fr")]
    RoomTempFr,
    #[strum(serialize = "T_room_rl")]
    RoomTempRl,
    #[strum(serialize = "T_room_rr")]
    RoomTempRr,
    #[strum(serialize = "T_heatsink_fl")]
    HeatsinkTempFl,
    #[strum(serialize = "T_heatsink_fr")]
    HeatsinkTempFr,
    #[strum(serialize = "T_heatsink_rl")]
    HeatsinkTempRl,
    #[strum(serialize = "T_heatsink_rr")]
    HeatsinkTempRr,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Expr {
    pub x: String,
    pub y: String,
}

#[derive(Default)]
pub struct ExprError {
    pub x: Option<cods::Error>,
    pub y: Option<cods::Error>,
}

pub fn eval(expr: &Expr, data: &Data, temp: &Temp) -> Result<Vec<Value>, ExprError> {
    let mut ctx_x = Context::default();
    let mut ctx_y = Context::default();
    for v in Var::iter() {
        ctx_x.idents.push(v.into());
        ctx_y.idents.push(v.into());
    }

    let var_count = Var::iter().count();
    let mut vars_x = Vec::with_capacity(var_count);
    let mut vars_y = Vec::with_capacity(var_count);
    let asts_x = parse(&mut ctx_x, &mut vars_x, &expr.x);
    let asts_y = parse(&mut ctx_y, &mut vars_y, &expr.y);

    let (asts_x, asts_y) = match (asts_x, asts_y) {
        (Ok(x), Ok(y)) => (x, y),
        (x, y) => {
            return Err(ExprError {
                x: x.err(),
                y: y.err(),
            })
        }
    };

    let mut values = Vec::with_capacity(data.len());
    let mut stack_x = Stack::default();
    let mut stack_y = Stack::default();
    stack_x.extend_to(vars_x.len());
    stack_y.extend_to(vars_y.len());

    let mut temp_index = 0;
    let mut temp_entries: &[TempEntry] = &[];
    for d in data.iter() {
        while let Some(t) = temp.get(temp_index) {
            if t.ms == d.ms || t.ms > d.ms && temp_index == 0 {
                temp_entries = std::slice::from_ref(t);
            } else if t.ms > d.ms {
                temp_entries = &temp[temp_index - 1..temp_index + 1];
            } else if temp_index + 1 == temp.len() {
                temp_entries = std::slice::from_ref(t);
            } else {
                temp_index += 1;
                continue;
            }
            break;
        }

        for (var, id) in vars_x.iter() {
            let val = get_value(*id, d, temp_entries);
            stack_x.set(var, val);
        }
        for (var, id) in vars_y.iter() {
            let val = get_value(*id, d, temp_entries);
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

fn parse(ctx: &mut Context, vars: &mut Vec<(VarRef, Var)>, input: &str) -> cods::Result<Asts> {
    let tokens = ctx.lex(input)?;
    let items = ctx.group(tokens)?;
    let csts = ctx.parse(items)?;

    let mut scopes = Scopes::default();
    for (id, v) in Var::iter().enumerate() {
        let ident = IdentSpan::new(Ident(id), Span::pos(0, 0));
        let inner = ctx.def_var(&mut scopes, ident, cods::DataType::Float, true, false);
        vars.push((inner, v));
    }

    let asts = ctx.check_with(&mut scopes, csts)?;
    if !ctx.errors.is_empty() {
        return Err(ctx.errors.remove(0));
    }

    Ok(asts)
}

fn cast_float(val: Val) -> Option<f64> {
    match val {
        Val::Int(i) => Some(i as f64),
        Val::Float(f) => Some(f),
        _ => None,
    }
}
