use std::f32::consts::PI;
use std::io::{self, Read, Seek, SeekFrom};
use std::mem::size_of;

use derive_more::{Deref, DerefMut};
use egui::plot::Value;

pub const SAMPLE_RATE: f64 = 0.02;

const DATA_SAMPLE_SIZE: usize = size_of::<DataEntry>();
const TEMP_SAMPLE_SIZE: usize = size_of::<TempEntry>();

impl<T: Iterator<Item = f32>> MapOverTime for T {}
pub trait MapOverTime: Iterator<Item = f32> + Sized {
    fn map_over_time(self) -> Vec<Value> {
        self.enumerate()
            .map(|(i, v)| Value::new(i as f64 * SAMPLE_RATE, v as f64))
            .collect()
    }
}

#[derive(Debug, Default, Deref, DerefMut)]
pub struct Data {
    entries: Vec<DataEntry>,
}

#[derive(Debug)]
pub struct DataEntry {
    pub time: f32, // 20ms steps

    pub power: f32,

    pub driven: f32,
    pub energy_to_finish_factor: f32,
    pub energy_total: f32,

    pub gas: f32,

    pub ams_umin: i16,
    pub ams_umin_true: i16,

    pub l_uzk: f32,
    pub speed_rl: f32,
    pub torque_rl: f32,
    pub speed_rr: f32,
    pub torque_rr: f32,
    pub speed_fl: f32,
    pub torque_fl: f32,
    pub speed_fr: f32,
    pub torque_fr: f32,

    pub accel_x: i16,
    pub accel_y: i16,
    pub accel_z: i16,

    pub gyro_x: i16,
    pub gyro_y: i16,
    pub gyro_z: i16,

    pub steering: i16,
    pub break_fron: f32,
    pub break_rear: f32,
    pub break_pedal: f32,

    pub current: i32,
    pub power_reduce: f32,

    pub torque_out_rl: f32,
    pub torque_out_rr: f32,
    pub torque_out_fl: f32,
    pub torque_out_fr: f32,

    pub spring_fr: f32,
    pub spring_fl: f32,
    pub spring_rl: f32,
    pub spring_rr: f32,
}

impl Data {
    fn extend_capacity(&mut self, cap: usize) {
        self.entries.reserve(cap);
    }

    pub fn read_extend(&mut self, reader: &mut (impl Read + Seek)) -> anyhow::Result<()> {
        let len = reader.len()?;
        let samples = len as usize / DATA_SAMPLE_SIZE;
        self.extend_capacity(samples);

        for _ in 0..samples {
            self.entries.push(DataEntry {
                time: reader.read_f32()?,

                power: reader.read_f32()?,

                driven: reader.read_f32()?,
                energy_to_finish_factor: reader.read_f32()?,
                energy_total: reader.read_f32()?,

                gas: reader.read_f32()?,

                ams_umin: reader.read_i16()?,
                ams_umin_true: reader.read_i16()?,

                l_uzk: reader.read_f32()?,
                speed_rl: reader.read_f32()?,
                torque_rl: reader.read_f32()?,
                speed_rr: reader.read_f32()?,
                torque_rr: -reader.read_f32()?,
                speed_fl: reader.read_f32()?,
                torque_fl: reader.read_f32()?,
                speed_fr: reader.read_f32()?,
                torque_fr: -reader.read_f32()?,

                accel_x: reader.read_i16()?,
                accel_y: reader.read_i16()?,
                accel_z: reader.read_i16()?,

                gyro_x: reader.read_i16()?,
                gyro_y: reader.read_i16()?,
                gyro_z: reader.read_i16()?,

                steering: reader.read_i16()?,
                break_fron: reader.read_f32()?,
                break_rear: reader.read_f32()?,
                break_pedal: reader.read_f32()?,

                current: reader.read_i32()? / 1000,
                power_reduce: reader.read_f32()?,

                torque_out_rl: reader.read_f32()?,
                torque_out_rr: reader.read_f32()?,
                torque_out_fl: reader.read_f32()?,
                torque_out_fr: reader.read_f32()?,

                spring_fr: reader.read_f32()? - 1630.0 - 420.0,
                spring_fl: reader.read_f32()? - 4750.0 + 400.0,
                spring_rl: reader.read_f32()? - 3125.0 + 115.0,
                spring_rr: reader.read_f32()? - 4005.0 - 200.0,
            });
        }

        Ok(())
    }
}

pub fn power_fl(e: &DataEntry) -> f32 {
    2.0 * PI / 60.0 * e.torque_fl * 0.0197 * e.speed_fl
}

pub fn power_fr(e: &DataEntry) -> f32 {
    2.0 * PI / 60.0 * e.torque_fr * 0.0197 * e.speed_fr
}

pub fn power_rl(e: &DataEntry) -> f32 {
    2.0 * PI / 60.0 * e.torque_rl * 0.0197 * e.speed_rl
}

pub fn power_rr(e: &DataEntry) -> f32 {
    2.0 * PI / 60.0 * e.torque_rr * 0.0197 * e.speed_rr
}

const VELOCITY_FACTOR: f32 = 0.01155;
pub fn velocity_fl(e: &DataEntry) -> f32 {
    e.speed_fl * VELOCITY_FACTOR
}

pub fn velocity_fr(e: &DataEntry) -> f32 {
    e.speed_fr * VELOCITY_FACTOR
}

pub fn velocity_rl(e: &DataEntry) -> f32 {
    e.speed_rl * VELOCITY_FACTOR
}

pub fn velocity_rr(e: &DataEntry) -> f32 {
    e.speed_rr * VELOCITY_FACTOR
}

pub fn torque_set_fl(e: &DataEntry) -> f32 {
    e.torque_fl
}

pub fn torque_set_fr(e: &DataEntry) -> f32 {
    e.torque_fr
}

pub fn torque_set_rl(e: &DataEntry) -> f32 {
    e.torque_rl
}

pub fn torque_set_rr(e: &DataEntry) -> f32 {
    e.torque_rr
}

pub fn torque_real_fl(e: &DataEntry) -> f32 {
    e.torque_out_fl
}

pub fn torque_real_fr(e: &DataEntry) -> f32 {
    e.torque_out_fr
}

pub fn torque_real_rl(e: &DataEntry) -> f32 {
    e.torque_out_rl
}

pub fn torque_real_rr(e: &DataEntry) -> f32 {
    e.torque_out_rr
}

#[derive(Debug, Default, Deref, DerefMut)]
pub struct Temp {
    pub entries: Vec<TempEntry>,
}

#[derive(Debug)]
pub struct TempEntry {
    pub time: f32,

    pub ams_temp_max: i16,

    pub water_temp_converter: i16,
    pub water_temp_motor: i16,

    pub temp_rl: f32,
    pub temp_rr: f32,
    pub temp_fl: f32,
    pub temp_fr: f32,

    pub room_temp_rl: i16,
    pub room_temp_rr: i16,
    pub room_temp_fl: i16,
    pub room_temp_fr: i16,

    pub kk_temp_rl: i16,
    pub kk_temp_rr: i16,
    pub kk_temp_fl: i16,
    pub kk_temp_fr: i16,
}

impl Temp {
    fn extend_capacity(&mut self, cap: usize) {
        self.entries.reserve(cap);
    }

    pub fn read_extend(&mut self, reader: &mut (impl Read + Seek)) -> anyhow::Result<()> {
        let len = reader.len()?;
        let samples = len as usize / TEMP_SAMPLE_SIZE;
        self.extend_capacity(samples);

        for _ in 0..samples {
            self.entries.push(TempEntry {
                time: reader.read_f32()?,

                ams_temp_max: reader.read_i16()?,

                water_temp_converter: reader.read_i16()?,
                water_temp_motor: reader.read_i16()?,

                temp_rl: reader.read_f32()?,
                temp_rr: reader.read_f32()?,
                temp_fl: reader.read_f32()?,
                temp_fr: reader.read_f32()?,

                room_temp_rl: reader.read_i16()?,
                room_temp_rr: reader.read_i16()?,
                room_temp_fl: reader.read_i16()?,
                room_temp_fr: reader.read_i16()?,

                kk_temp_rl: reader.read_i16()?,
                kk_temp_rr: reader.read_i16()?,
                kk_temp_fl: reader.read_i16()?,
                kk_temp_fr: reader.read_i16()?,
            })
        }

        Ok(())
    }
}

impl<T: Read> ReadUtils for T {}
pub trait ReadUtils: Read {
    fn read_i16(&mut self) -> io::Result<i16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(i16::from_be_bytes(buf))
    }

    fn read_i32(&mut self) -> io::Result<i32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(i32::from_be_bytes(buf))
    }

    fn read_f32(&mut self) -> io::Result<f32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(f32::from_be_bytes(buf))
    }
}

impl<T: Seek> SeekUtils for T {}
pub trait SeekUtils: Seek {
    fn len(&mut self) -> io::Result<u64> {
        let pos = self.seek(SeekFrom::Current(0))?;
        let len = self.seek(SeekFrom::End(0))?;
        self.seek(SeekFrom::Start(pos))?;
        Ok(len)
    }
}
