use std::fmt;
use std::str::FromStr;

use cods::Provider;
use eframe::egui::plot::Value;
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

struct Plotter<'a> {
    index: usize,
    data: &'a Data,
}

impl Provider<Var> for Plotter<'_> {
    fn var_to_f64(&self, var: Var) -> f64 {
        let i = self.index;
        match var {
            Var::Time => self.index as f64 * SAMPLE_RATE,
            Var::PowerFl => self.data.power_fl().nth(i).unwrap() as f64,
            Var::PowerFr => self.data.power_fr().nth(i).unwrap() as f64,
            Var::PowerRl => self.data.power_rl().nth(i).unwrap() as f64,
            Var::PowerRr => self.data.power_rr().nth(i).unwrap() as f64,
            Var::VelocityFl => self.data.velocity_fl().nth(i).unwrap() as f64,
            Var::VelocityFr => self.data.velocity_fr().nth(i).unwrap() as f64,
            Var::VelocityRl => self.data.velocity_rl().nth(i).unwrap() as f64,
            Var::VelocityRr => self.data.velocity_rr().nth(i).unwrap() as f64,
            Var::TorqueSetFl => self.data.torque_set_fl().nth(i).unwrap() as f64,
            Var::TorqueSetFr => self.data.torque_set_fr().nth(i).unwrap() as f64,
            Var::TorqueSetRl => self.data.torque_set_rl().nth(i).unwrap() as f64,
            Var::TorqueSetRr => self.data.torque_set_rr().nth(i).unwrap() as f64,
            Var::TorqueRealFl => self.data.torque_real_fl().nth(i).unwrap() as f64,
            Var::TorqueRealFr => self.data.torque_real_fr().nth(i).unwrap() as f64,
            Var::TorqueRealRl => self.data.torque_real_rl().nth(i).unwrap() as f64,
            Var::TorqueRealRr => self.data.torque_real_rr().nth(i).unwrap() as f64,
        }
    }
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

impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Time => f.write_str(TIME),
            Self::PowerFl => f.write_str(POWER_FL),
            Self::PowerFr => f.write_str(POWER_FR),
            Self::PowerRl => f.write_str(POWER_RL),
            Self::PowerRr => f.write_str(POWER_RR),
            Self::VelocityFl => f.write_str(VELOCITY_FL),
            Self::VelocityFr => f.write_str(VELOCITY_FR),
            Self::VelocityRl => f.write_str(VELOCITY_RL),
            Self::VelocityRr => f.write_str(VELOCITY_RR),
            Self::TorqueSetFl => f.write_str(TORQUE_SET_FL),
            Self::TorqueSetFr => f.write_str(TORQUE_SET_FR),
            Self::TorqueSetRl => f.write_str(TORQUE_SET_RL),
            Self::TorqueSetRr => f.write_str(TORQUE_SET_RR),
            Self::TorqueRealFl => f.write_str(TORQUE_REAL_FL),
            Self::TorqueRealFr => f.write_str(TORQUE_REAL_FR),
            Self::TorqueRealRl => f.write_str(TORQUE_REAL_RL),
            Self::TorqueRealRr => f.write_str(TORQUE_REAL_RR),
        }
    }
}

impl cods::Var for Var {}

pub fn eval(input: &str, data: &Data) -> anyhow::Result<Vec<Value>, ()> {
    let (calc, ctx) = cods::parse::<Var>(input);
    let calc = calc?;
    if !ctx.errors.is_empty() {
        return Err(());
    }

    let mut plotter = Plotter { index: 0, data };
    let mut values = Vec::with_capacity(data.len);
    for i in 0..data.len {
        plotter.index = i;
        let y = match calc.eval(&plotter) {
            Ok(v) => plotter.val_to_f64(v),
            Err(_) => 0.0, //TODO: use last value
        };

        values.push(Value::new(i as f64 * SAMPLE_RATE, y));
    }

    Ok(values)
}
