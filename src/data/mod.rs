use std::f64::consts::PI;
use std::{fmt, io};

use egui::plot::Value;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

use crate::app::{CustomValues, PlotData, WheelValues};
use crate::eval;
use crate::plot::CustomPlot;

pub use read::{read_extend_data, read_extend_temp};

mod read;

impl<'a, T, E: 'a> MapOverTime<'a, E> for T
where
    T: Iterator<Item = &'a E> + Sized,
    E: TimeStamped,
{
}
pub trait MapOverTime<'a, T: TimeStamped + 'a>: Iterator<Item = &'a T> + Sized {
    fn map_over_time(self, f: impl Fn(&T) -> f64) -> Vec<Value> {
        self.map(|e| Value::new(e.time(), f(e))).collect()
    }
}

pub trait TimeStamped {
    fn time(&self) -> f64;
}

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    SanityCheck(&'static str),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SanityCheck(message) => write!(
                f,
                "Sanity check failed: {message}. Maybe try selecting another version and reopening"
            ),
            Self::IO(error) => write!(f, "Error reading files: {}", error),
        }
    }
}

impl From<io::Error> for Error {
    fn from(inner: io::Error) -> Self {
        Self::IO(inner)
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, EnumIter, Display)]
pub enum Version {
    #[strum(serialize = "s3 21e")]
    S321e,
    #[default]
    #[strum(serialize = "s3 22e")]
    S322e,
}

#[derive(Clone, Debug)]
pub struct DataEntry {
    pub ms: f32,

    #[allow(unused)]
    power: f32,

    #[allow(unused)]
    driven: f32,
    #[allow(unused)]
    energy_to_finish_factor: f32,
    #[allow(unused)]
    energy_total: f32,

    #[allow(unused)]
    gas: f32,

    #[allow(unused)]
    ams_u_min: i16,
    #[allow(unused)]
    ams_u_min_true: i16,
    #[allow(unused)]
    ams_u_avg: i16,

    #[allow(unused)]
    l_uzk: f32,
    speed_rl: f32,
    torque_set_rl: f32,
    speed_rr: f32,
    torque_set_rr: f32,
    speed_fl: f32,
    torque_set_fl: f32,
    speed_fr: f32,
    torque_set_fr: f32,

    #[allow(unused)]
    accel_x: i16,
    #[allow(unused)]
    accel_y: i16,
    #[allow(unused)]
    accel_z: i16,

    #[allow(unused)]
    gyro_x: i16,
    #[allow(unused)]
    gyro_y: i16,
    #[allow(unused)]
    gyro_z: i16,

    #[allow(unused)]
    steering: i16,
    #[allow(unused)]
    break_front: f32,
    #[allow(unused)]
    break_rear: f32,
    #[allow(unused)]
    break_pedal: f32,

    #[allow(unused)]
    current: i32,
    #[allow(unused)]
    power_reduce: f32,

    torque_real_rl: f32,
    torque_real_rr: f32,
    torque_real_fl: f32,
    torque_real_fr: f32,

    #[allow(unused)]
    spring_fr: f32,
    #[allow(unused)]
    spring_fl: f32,
    #[allow(unused)]
    spring_rl: f32,
    #[allow(unused)]
    spring_rr: f32,
}

impl TimeStamped for DataEntry {
    fn time(&self) -> f64 {
        self.ms as f64 / 1000.0
    }
}

const POWER_FACTOR: f64 = 2.0 * PI / 60.0 * 0.0197;
const VELOCITY_FACTOR: f64 = 0.01155;
impl DataEntry {
    pub fn power_fl(&self) -> f64 {
        self.speed_fl as f64 * self.torque_set_fl as f64 * POWER_FACTOR
    }

    pub fn power_fr(&self) -> f64 {
        self.speed_fr as f64 * self.torque_set_fr as f64 * POWER_FACTOR
    }

    pub fn power_rl(&self) -> f64 {
        self.speed_rl as f64 * self.torque_set_rl as f64 * POWER_FACTOR
    }

    pub fn power_rr(&self) -> f64 {
        self.speed_rr as f64 * self.torque_set_rr as f64 * POWER_FACTOR
    }

    pub fn velocity_fl(&self) -> f64 {
        self.speed_fl as f64 * VELOCITY_FACTOR
    }

    pub fn velocity_fr(&self) -> f64 {
        self.speed_fr as f64 * VELOCITY_FACTOR
    }

    pub fn velocity_rl(&self) -> f64 {
        self.speed_rl as f64 * VELOCITY_FACTOR
    }

    pub fn velocity_rr(&self) -> f64 {
        self.speed_rr as f64 * VELOCITY_FACTOR
    }

    pub fn torque_set_fl(&self) -> f64 {
        self.torque_set_fl as f64
    }

    pub fn torque_set_fr(&self) -> f64 {
        self.torque_set_fr as f64
    }

    pub fn torque_set_rl(&self) -> f64 {
        self.torque_set_rl as f64
    }

    pub fn torque_set_rr(&self) -> f64 {
        self.torque_set_rr as f64
    }

    pub fn torque_real_fl(&self) -> f64 {
        self.torque_real_fl as f64
    }

    pub fn torque_real_fr(&self) -> f64 {
        self.torque_real_fr as f64
    }

    pub fn torque_real_rl(&self) -> f64 {
        self.torque_real_rl as f64
    }

    pub fn torque_real_rr(&self) -> f64 {
        self.torque_real_rr as f64
    }
}

#[derive(Clone, Debug)]
pub struct TempEntry {
    pub ms: f32,

    ams_temp_max: i16,

    water_temp_converter: i16,
    water_temp_motor: i16,

    temp_rl: f32,
    temp_rr: f32,
    temp_fl: f32,
    temp_fr: f32,

    room_temp_rl: i16,
    room_temp_rr: i16,
    room_temp_fl: i16,
    room_temp_fr: i16,

    heatsink_temp_rl: i16,
    heatsink_temp_rr: i16,
    heatsink_temp_fl: i16,
    heatsink_temp_fr: i16,
}

impl TimeStamped for TempEntry {
    fn time(&self) -> f64 {
        self.ms as f64 / 1000.0
    }
}

const TEMP_FACTOR: f64 = 0.1;
impl TempEntry {
    pub fn ams_temp_max(&self) -> f64 {
        self.ams_temp_max as f64 * TEMP_FACTOR
    }

    pub fn water_temp_converter(&self) -> f64 {
        self.water_temp_converter as f64 * TEMP_FACTOR
    }

    pub fn water_temp_motor(&self) -> f64 {
        self.water_temp_motor as f64 * TEMP_FACTOR
    }

    pub fn temp_rl(&self) -> f64 {
        self.temp_rl as f64 * TEMP_FACTOR
    }
    pub fn temp_rr(&self) -> f64 {
        self.temp_rr as f64 * TEMP_FACTOR
    }
    pub fn temp_fl(&self) -> f64 {
        self.temp_fl as f64 * TEMP_FACTOR
    }
    pub fn temp_fr(&self) -> f64 {
        self.temp_fr as f64 * TEMP_FACTOR
    }

    pub fn room_temp_rl(&self) -> f64 {
        self.room_temp_rl as f64 * TEMP_FACTOR
    }
    pub fn room_temp_rr(&self) -> f64 {
        self.room_temp_rr as f64 * TEMP_FACTOR
    }
    pub fn room_temp_fl(&self) -> f64 {
        self.room_temp_fl as f64 * TEMP_FACTOR
    }
    pub fn room_temp_fr(&self) -> f64 {
        self.room_temp_fr as f64 * TEMP_FACTOR
    }

    pub fn heatsink_temp_rl(&self) -> f64 {
        self.heatsink_temp_rl as f64 * TEMP_FACTOR
    }
    pub fn heatsink_temp_rr(&self) -> f64 {
        self.heatsink_temp_rr as f64 * TEMP_FACTOR
    }
    pub fn heatsink_temp_fl(&self) -> f64 {
        self.heatsink_temp_fl as f64 * TEMP_FACTOR
    }
    pub fn heatsink_temp_fr(&self) -> f64 {
        self.heatsink_temp_fr as f64 * TEMP_FACTOR
    }
}

pub fn process_data(d: Vec<DataEntry>, t: Vec<TempEntry>, custom_plots: &[CustomPlot]) -> PlotData {
    let power = WheelValues {
        fl: d.iter().map_over_time(DataEntry::power_fl),
        fr: d.iter().map_over_time(DataEntry::power_fr),
        rl: d.iter().map_over_time(DataEntry::power_rl),
        rr: d.iter().map_over_time(DataEntry::power_rr),
    };
    let velocity = WheelValues {
        fl: d.iter().map_over_time(DataEntry::velocity_fl),
        fr: d.iter().map_over_time(DataEntry::velocity_fr),
        rl: d.iter().map_over_time(DataEntry::velocity_rl),
        rr: d.iter().map_over_time(DataEntry::velocity_rr),
    };
    let torque_set = WheelValues {
        fl: d.iter().map_over_time(DataEntry::torque_set_fl),
        fr: d.iter().map_over_time(DataEntry::torque_set_fr),
        rl: d.iter().map_over_time(DataEntry::torque_set_rl),
        rr: d.iter().map_over_time(DataEntry::torque_set_rr),
    };
    let torque_real = WheelValues {
        fl: d.iter().map_over_time(DataEntry::torque_real_fl),
        fr: d.iter().map_over_time(DataEntry::torque_real_fr),
        rl: d.iter().map_over_time(DataEntry::torque_real_rl),
        rr: d.iter().map_over_time(DataEntry::torque_real_rr),
    };
    let temp = WheelValues {
        fl: t.iter().map_over_time(TempEntry::temp_fl),
        fr: t.iter().map_over_time(TempEntry::temp_fr),
        rl: t.iter().map_over_time(TempEntry::temp_rl),
        rr: t.iter().map_over_time(TempEntry::temp_rr),
    };
    let room_temp = WheelValues {
        fl: t.iter().map_over_time(TempEntry::room_temp_fl),
        fr: t.iter().map_over_time(TempEntry::room_temp_fr),
        rl: t.iter().map_over_time(TempEntry::room_temp_rl),
        rr: t.iter().map_over_time(TempEntry::room_temp_rr),
    };
    let heatsink_temp = WheelValues {
        fl: t.iter().map_over_time(TempEntry::heatsink_temp_fl),
        fr: t.iter().map_over_time(TempEntry::heatsink_temp_fr),
        rl: t.iter().map_over_time(TempEntry::heatsink_temp_rl),
        rr: t.iter().map_over_time(TempEntry::heatsink_temp_rr),
    };
    let ams_temp_max = t.iter().map_over_time(TempEntry::ams_temp_max);
    let water_temp_converter = t.iter().map_over_time(TempEntry::water_temp_converter);
    let water_temp_motor = t.iter().map_over_time(TempEntry::water_temp_motor);
    let custom = custom_plots
        .iter()
        .map(|p| {
            let r = eval::eval(&p.expr, &d, &t);
            CustomValues::from_result(r)
        })
        .collect();

    PlotData {
        raw_data: d,
        raw_temp: t,
        power,
        velocity,
        torque_set,
        torque_real,
        temp,
        room_temp,
        heatsink_temp,
        ams_temp_max,
        water_temp_converter,
        water_temp_motor,
        custom,
    }
}
