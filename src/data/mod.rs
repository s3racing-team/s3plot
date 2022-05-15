use std::f64::consts::PI;

use derive_more::{Deref, DerefMut};
use egui::plot::Value;

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

#[derive(Debug, Default, Deref, DerefMut)]
pub struct Data {
    entries: Vec<DataEntry>,
}

#[derive(Debug)]
pub struct DataEntry {
    ms: f32,

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
    ams_umin: i16,
    #[allow(unused)]
    ams_umin_true: i16,

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

#[derive(Debug, Default, Deref, DerefMut)]
pub struct Temp {
    pub entries: Vec<TempEntry>,
}

#[derive(Debug)]
pub struct TempEntry {
    ms: f32,

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
