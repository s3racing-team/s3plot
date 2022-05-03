use std::f32::consts::PI;
use std::io::{self, Read, Seek, SeekFrom};

use egui::plot::Value;

pub const SAMPLE_RATE: f64 = 0.02;

const DATA_SAMPLE_SIZE: usize = 132;
const TEMP_SAMPLE_SIZE: usize = 132;

impl<T: Iterator<Item = f32>> MapOverTime for T {}
pub trait MapOverTime: Iterator<Item = f32> + Sized {
    fn map_over_time(self) -> Vec<Value> {
        self.enumerate()
            .map(|(i, v)| Value::new(i as f64 * SAMPLE_RATE, v as f64))
            .collect()
    }
}

#[derive(Debug, Default)]
pub struct Data {
    pub len: usize,
    pub time: Vec<f32>, // 20ms steps

    pub power: Vec<f32>,

    pub driven: Vec<f32>,
    pub energy_to_finish_factor: Vec<f32>,
    pub energy_total: Vec<f32>,

    pub gas: Vec<f32>,

    pub ams_umin: Vec<i16>,
    pub ams_umin_true: Vec<i16>,

    pub l_uzk: Vec<f32>,
    pub speed_rl: Vec<f32>,
    pub torque_rl: Vec<f32>,
    pub speed_rr: Vec<f32>,
    pub torque_rr: Vec<f32>,
    pub speed_fl: Vec<f32>,
    pub torque_fl: Vec<f32>,
    pub speed_fr: Vec<f32>,
    pub torque_fr: Vec<f32>,

    pub accel_x: Vec<i16>,
    pub accel_y: Vec<i16>,
    pub accel_z: Vec<i16>,

    pub gyro_x: Vec<i16>,
    pub gyro_y: Vec<i16>,
    pub gyro_z: Vec<i16>,

    pub steering: Vec<i16>,
    pub break_fron: Vec<f32>,
    pub break_rear: Vec<f32>,
    pub break_pedal: Vec<f32>,

    pub current: Vec<i32>,
    pub power_reduce: Vec<f32>,

    pub torque_out_rl: Vec<f32>,
    pub torque_out_rr: Vec<f32>,
    pub torque_out_fl: Vec<f32>,
    pub torque_out_fr: Vec<f32>,

    pub spring_fr: Vec<f32>,
    pub spring_fl: Vec<f32>,
    pub spring_rl: Vec<f32>,
    pub spring_rr: Vec<f32>,
}

impl Data {
    fn extend_capacity(&mut self, cap: usize) {
        self.len += cap;
        self.time.reserve(cap);

        self.power.reserve(cap);

        self.driven.reserve(cap);
        self.energy_to_finish_factor.reserve(cap);
        self.energy_total.reserve(cap);

        self.gas.reserve(cap);

        self.ams_umin.reserve(cap);
        self.ams_umin_true.reserve(cap);

        self.l_uzk.reserve(cap);
        self.speed_rl.reserve(cap);
        self.torque_rl.reserve(cap);
        self.speed_rr.reserve(cap);
        self.torque_rr.reserve(cap);
        self.speed_fl.reserve(cap);
        self.torque_fl.reserve(cap);
        self.speed_fr.reserve(cap);
        self.torque_fr.reserve(cap);

        self.accel_x.reserve(cap);
        self.accel_y.reserve(cap);
        self.accel_z.reserve(cap);

        self.gyro_x.reserve(cap);
        self.gyro_y.reserve(cap);
        self.gyro_z.reserve(cap);

        self.steering.reserve(cap);
        self.break_fron.reserve(cap);
        self.break_rear.reserve(cap);
        self.break_pedal.reserve(cap);

        self.current.reserve(cap);
        self.power_reduce.reserve(cap);

        self.torque_out_rl.reserve(cap);
        self.torque_out_rr.reserve(cap);
        self.torque_out_fl.reserve(cap);
        self.torque_out_fr.reserve(cap);

        self.spring_fr.reserve(cap);
        self.spring_fl.reserve(cap);
        self.spring_rl.reserve(cap);
        self.spring_rr.reserve(cap);
    }

    pub fn read_extend(&mut self, reader: &mut (impl Read + Seek)) -> anyhow::Result<()> {
        let len = reader.len()?;
        let samples = len as usize / DATA_SAMPLE_SIZE;
        self.extend_capacity(samples);

        for _ in 0..samples {
            self.time.push(reader.read_f32()?);

            self.power.push(reader.read_f32()?);

            self.driven.push(reader.read_f32()?);
            self.energy_to_finish_factor.push(reader.read_f32()?);
            self.energy_total.push(reader.read_f32()?);

            self.gas.push(reader.read_f32()?);

            self.ams_umin.push(reader.read_i16()?);
            self.ams_umin_true.push(reader.read_i16()?);

            self.l_uzk.push(reader.read_f32()?);
            self.speed_rl.push(reader.read_f32()?);
            self.torque_rl.push(reader.read_f32()?);
            self.speed_rr.push(reader.read_f32()?);
            self.torque_rr.push(-reader.read_f32()?);
            self.speed_fl.push(reader.read_f32()?);
            self.torque_fl.push(reader.read_f32()?);
            self.speed_fr.push(reader.read_f32()?);
            self.torque_fr.push(-reader.read_f32()?);

            self.accel_x.push(reader.read_i16()?);
            self.accel_y.push(reader.read_i16()?);
            self.accel_z.push(reader.read_i16()?);

            self.gyro_x.push(reader.read_i16()?);
            self.gyro_y.push(reader.read_i16()?);
            self.gyro_z.push(reader.read_i16()?);

            self.steering.push(reader.read_i16()?);
            self.break_fron.push(reader.read_f32()?);
            self.break_rear.push(reader.read_f32()?);
            self.break_pedal.push(reader.read_f32()?);

            self.current.push(reader.read_i32()? / 1000);
            self.power_reduce.push(reader.read_f32()?);

            self.torque_out_rl.push(reader.read_f32()?);
            self.torque_out_rr.push(reader.read_f32()?);
            self.torque_out_fl.push(reader.read_f32()?);
            self.torque_out_fr.push(reader.read_f32()?);

            self.spring_fr.push(reader.read_f32()? - 1630.0 - 420.0);
            self.spring_fl.push(reader.read_f32()? - 4750.0 + 400.0);
            self.spring_rl.push(reader.read_f32()? - 3125.0 + 115.0);
            self.spring_rr.push(reader.read_f32()? - 4005.0 - 200.0);
        }

        Ok(())
    }

    pub fn power_fl(&self) -> impl Iterator<Item = f32> + '_ {
        self.torque_fl
            .iter()
            .zip(self.speed_fl.iter())
            .map(|(&torque, &speed)| 2.0 * PI / 60.0 * torque * 0.0197 * speed)
    }

    pub fn power_fr(&self) -> impl Iterator<Item = f32> + '_ {
        self.torque_fr
            .iter()
            .zip(self.speed_fr.iter())
            .map(|(&torque, &speed)| 2.0 * PI / 60.0 * torque * 0.0197 * speed)
    }

    pub fn power_rl(&self) -> impl Iterator<Item = f32> + '_ {
        self.torque_rl
            .iter()
            .zip(self.speed_rl.iter())
            .map(|(&torque, &speed)| 2.0 * PI / 60.0 * torque * 0.0197 * speed)
    }

    pub fn power_rr(&self) -> impl Iterator<Item = f32> + '_ {
        self.torque_rr
            .iter()
            .zip(self.speed_rr.iter())
            .map(|(&torque, &speed)| 2.0 * PI / 60.0 * torque * 0.0197 * speed)
    }

    const VELOCITY_FACTOR: f32 = 0.01155;
    pub fn velocity_fl(&self) -> impl Iterator<Item = f32> + '_ {
        self.speed_fl.iter().map(|v| *v * Self::VELOCITY_FACTOR)
    }

    pub fn velocity_fr(&self) -> impl Iterator<Item = f32> + '_ {
        self.speed_fr.iter().map(|v| *v * Self::VELOCITY_FACTOR)
    }

    pub fn velocity_rl(&self) -> impl Iterator<Item = f32> + '_ {
        self.speed_rl.iter().map(|v| *v * Self::VELOCITY_FACTOR)
    }

    pub fn velocity_rr(&self) -> impl Iterator<Item = f32> + '_ {
        self.speed_rr.iter().map(|v| *v * Self::VELOCITY_FACTOR)
    }

    pub fn torque_set_fl(&self) -> impl Iterator<Item = f32> + '_ {
        self.torque_fl.iter().copied()
    }

    pub fn torque_set_fr(&self) -> impl Iterator<Item = f32> + '_ {
        self.torque_fr.iter().copied()
    }

    pub fn torque_set_rl(&self) -> impl Iterator<Item = f32> + '_ {
        self.torque_rl.iter().copied()
    }

    pub fn torque_set_rr(&self) -> impl Iterator<Item = f32> + '_ {
        self.torque_rr.iter().copied()
    }

    pub fn torque_real_fl(&self) -> impl Iterator<Item = f32> + '_ {
        self.torque_out_fl.iter().copied()
    }

    pub fn torque_real_fr(&self) -> impl Iterator<Item = f32> + '_ {
        self.torque_out_fr.iter().copied()
    }

    pub fn torque_real_rl(&self) -> impl Iterator<Item = f32> + '_ {
        self.torque_out_rl.iter().copied()
    }

    pub fn torque_real_rr(&self) -> impl Iterator<Item = f32> + '_ {
        self.torque_out_rr.iter().copied()
    }
}

#[derive(Debug, Default)]
pub struct Temp {
    pub len: usize,
    pub time: Vec<f32>,

    pub ams_temp_max: Vec<i16>,

    pub water_temp_converter: Vec<i16>,
    pub water_temp_motor: Vec<i16>,

    pub temp_rl: Vec<f32>,
    pub temp_rr: Vec<f32>,
    pub temp_fl: Vec<f32>,
    pub temp_fr: Vec<f32>,

    pub room_temp_rl: Vec<i16>,
    pub room_temp_rr: Vec<i16>,
    pub room_temp_fl: Vec<i16>,
    pub room_temp_fr: Vec<i16>,

    pub kk_temp_rl: Vec<i16>,
    pub kk_temp_rr: Vec<i16>,
    pub kk_temp_fl: Vec<i16>,
    pub kk_temp_fr: Vec<i16>,
}

impl Temp {
    fn extend_capacity(&mut self, cap: usize) {
        self.len += cap;
        self.time.reserve(cap);
        self.time.reserve(cap);

        self.ams_temp_max.reserve(cap);

        self.water_temp_converter.reserve(cap);
        self.water_temp_motor.reserve(cap);

        self.temp_rl.reserve(cap);
        self.temp_rr.reserve(cap);
        self.temp_fl.reserve(cap);
        self.temp_fr.reserve(cap);

        self.room_temp_rl.reserve(cap);
        self.room_temp_rr.reserve(cap);
        self.room_temp_fl.reserve(cap);
        self.room_temp_fr.reserve(cap);

        self.kk_temp_rl.reserve(cap);
        self.kk_temp_rr.reserve(cap);
        self.kk_temp_fl.reserve(cap);
        self.kk_temp_fr.reserve(cap);
    }

    pub fn read_extend(&mut self, reader: &mut (impl Read + Seek)) -> anyhow::Result<()> {
        let len = reader.len()?;
        let samples = len as usize / TEMP_SAMPLE_SIZE;
        self.extend_capacity(samples);

        for _ in 0..samples {
            self.time.push(reader.read_f32()?);

            self.ams_temp_max.push(reader.read_i16()?);

            self.water_temp_converter.push(reader.read_i16()?);
            self.water_temp_motor.push(reader.read_i16()?);

            self.temp_rl.push(reader.read_f32()?);
            self.temp_rr.push(reader.read_f32()?);
            self.temp_fl.push(reader.read_f32()?);
            self.temp_fr.push(reader.read_f32()?);

            self.room_temp_rl.push(reader.read_i16()?);
            self.room_temp_rr.push(reader.read_i16()?);
            self.room_temp_fl.push(reader.read_i16()?);
            self.room_temp_fr.push(reader.read_i16()?);

            self.kk_temp_rl.push(reader.read_i16()?);
            self.kk_temp_rr.push(reader.read_i16()?);
            self.kk_temp_fl.push(reader.read_i16()?);
            self.kk_temp_fr.push(reader.read_i16()?);
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
