use std::f32::consts::PI;

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
    pub ms: f32,

    pub power: f32,

    pub driven: f32,
    pub energy_to_finish_factor: f32,
    pub energy_total: f32,

    pub gas: f32,

    pub ams_umin: i16,
    pub ams_umin_true: i16,

    pub l_uzk: f32,
    pub speed_rl: f32,
    pub torque_set_rl: f32,
    pub speed_rr: f32,
    pub torque_set_rr: f32,
    pub speed_fl: f32,
    pub torque_set_fl: f32,
    pub speed_fr: f32,
    pub torque_set_fr: f32,

    pub accel_x: i16,
    pub accel_y: i16,
    pub accel_z: i16,

    pub gyro_x: i16,
    pub gyro_y: i16,
    pub gyro_z: i16,

    pub steering: i16,
    pub break_front: f32,
    pub break_rear: f32,
    pub break_pedal: f32,

    pub current: i32,
    pub power_reduce: f32,

    pub torque_real_rl: f32,
    pub torque_real_rr: f32,
    pub torque_real_fl: f32,
    pub torque_real_fr: f32,

    pub spring_fr: f32,
    pub spring_fl: f32,
    pub spring_rl: f32,
    pub spring_rr: f32,
}

const VELOCITY_FACTOR: f32 = 0.01155;
impl DataEntry {
    pub fn timed(&self, y: f32) -> Value {
        Value::new(self.time(), y as f64)
    }

    pub fn time(&self) -> f32 {
        self.ms / 1000.0
    }

    pub fn power_fl(&self) -> f32 {
        2.0 * PI / 60.0 * self.torque_set_fl * 0.0197 * self.speed_fl
    }

    pub fn power_fr(&self) -> f32 {
        2.0 * PI / 60.0 * self.torque_set_fr * 0.0197 * self.speed_fr
    }

    pub fn power_rl(&self) -> f32 {
        2.0 * PI / 60.0 * self.torque_set_rl * 0.0197 * self.speed_rl
    }

    pub fn power_rr(&self) -> f32 {
        2.0 * PI / 60.0 * self.torque_set_rr * 0.0197 * self.speed_rr
    }

    pub fn velocity_fl(&self) -> f32 {
        self.speed_fl * VELOCITY_FACTOR
    }

    pub fn velocity_fr(&self) -> f32 {
        self.speed_fr * VELOCITY_FACTOR
    }

    pub fn velocity_rl(&self) -> f32 {
        self.speed_rl * VELOCITY_FACTOR
    }

    pub fn velocity_rr(&self) -> f32 {
        self.speed_rr * VELOCITY_FACTOR
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

impl TempEntry {
    pub fn ams_temp_max(&self) -> f64 {
        self.ams_temp_max as f64 / 10.0
    }

    pub fn water_temp_converter(&self) -> f64 {
        self.water_temp_converter as f64 / 10.0
    }

    pub fn water_temp_motor(&self) -> f64 {
        self.water_temp_motor as f64 / 10.0
    }

    pub fn temp_rl(&self) -> f64 {
        self.temp_rl as f64 / 10.0
    }
    pub fn temp_rr(&self) -> f64 {
        self.temp_rr as f64 / 10.0
    }
    pub fn temp_fl(&self) -> f64 {
        self.temp_fl as f64 / 10.0
    }
    pub fn temp_fr(&self) -> f64 {
        self.temp_fr as f64 / 10.0
    }

    pub fn room_temp_rl(&self) -> f64 {
        self.room_temp_rl as f64 / 10.0
    }
    pub fn room_temp_rr(&self) -> f64 {
        self.room_temp_rr as f64 / 10.0
    }
    pub fn room_temp_fl(&self) -> f64 {
        self.room_temp_fl as f64 / 10.0
    }
    pub fn room_temp_fr(&self) -> f64 {
        self.room_temp_fr as f64 / 10.0
    }

    pub fn heatsink_temp_rl(&self) -> f64 {
        self.heatsink_temp_rl as f64 / 10.0
    }
    pub fn heatsink_temp_rr(&self) -> f64 {
        self.heatsink_temp_rr as f64 / 10.0
    }
    pub fn heatsink_temp_fl(&self) -> f64 {
        self.heatsink_temp_fl as f64 / 10.0
    }
    pub fn heatsink_temp_fr(&self) -> f64 {
        self.heatsink_temp_fr as f64 / 10.0
    }
}
