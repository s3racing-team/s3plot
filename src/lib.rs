use std::f32::consts::PI;
use std::io::{self, Read, Seek, SeekFrom};

use eframe::egui::plot::Value;

const SAMPLE_SIZE: usize = 132;

#[derive(Debug)]
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
    fn with_capacity(cap: usize) -> Self {
        Self {
            len: cap,
            time: Vec::with_capacity(cap),
            power: Vec::with_capacity(cap),

            driven: Vec::with_capacity(cap),
            energy_to_finish_factor: Vec::with_capacity(cap),
            energy_total: Vec::with_capacity(cap),

            gas: Vec::with_capacity(cap),

            ams_umin: Vec::with_capacity(cap),
            ams_umin_true: Vec::with_capacity(cap),

            l_uzk: Vec::with_capacity(cap),
            speed_rl: Vec::with_capacity(cap),
            torque_rl: Vec::with_capacity(cap),
            speed_rr: Vec::with_capacity(cap),
            torque_rr: Vec::with_capacity(cap),
            speed_fl: Vec::with_capacity(cap),
            torque_fl: Vec::with_capacity(cap),
            speed_fr: Vec::with_capacity(cap),
            torque_fr: Vec::with_capacity(cap),

            accel_x: Vec::with_capacity(cap),
            accel_y: Vec::with_capacity(cap),
            accel_z: Vec::with_capacity(cap),

            gyro_x: Vec::with_capacity(cap),
            gyro_y: Vec::with_capacity(cap),
            gyro_z: Vec::with_capacity(cap),

            steering: Vec::with_capacity(cap),
            break_fron: Vec::with_capacity(cap),
            break_rear: Vec::with_capacity(cap),
            break_pedal: Vec::with_capacity(cap),

            current: Vec::with_capacity(cap),
            power_reduce: Vec::with_capacity(cap),

            torque_out_rl: Vec::with_capacity(cap),
            torque_out_rr: Vec::with_capacity(cap),
            torque_out_fl: Vec::with_capacity(cap),
            torque_out_fr: Vec::with_capacity(cap),

            spring_fr: Vec::with_capacity(cap),
            spring_fl: Vec::with_capacity(cap),
            spring_rl: Vec::with_capacity(cap),
            spring_rr: Vec::with_capacity(cap),
        }
    }

    pub fn read(reader: &mut (impl Read + Seek)) -> anyhow::Result<Self> {
        let len = reader.len()?;
        let samples = len as usize / SAMPLE_SIZE;
        let mut data = Self::with_capacity(samples);
        for _ in 0..samples {
            data.time.push(reader.read_f32()?);
            data.power.push(reader.read_f32()?);

            data.driven.push(reader.read_f32()?);
            data.energy_to_finish_factor.push(reader.read_f32()?);
            data.energy_total.push(reader.read_f32()?);

            data.gas.push(reader.read_f32()?);

            data.ams_umin.push(reader.read_i16()?);
            data.ams_umin_true.push(reader.read_i16()?);

            data.l_uzk.push(reader.read_f32()?);
            data.speed_rl.push(reader.read_f32()?);
            data.torque_rl.push(reader.read_f32()?);
            data.speed_rr.push(reader.read_f32()?);
            data.torque_rr.push(-reader.read_f32()?);
            data.speed_fl.push(reader.read_f32()?);
            data.torque_fl.push(reader.read_f32()?);
            data.speed_fr.push(reader.read_f32()?);
            data.torque_fr.push(-reader.read_f32()?);

            data.accel_x.push(reader.read_i16()?);
            data.accel_y.push(reader.read_i16()?);
            data.accel_z.push(reader.read_i16()?);

            data.gyro_x.push(reader.read_i16()?);
            data.gyro_y.push(reader.read_i16()?);
            data.gyro_z.push(reader.read_i16()?);

            data.steering.push(reader.read_i16()?);
            data.break_fron.push(reader.read_f32()?);
            data.break_rear.push(reader.read_f32()?);
            data.break_pedal.push(reader.read_f32()?);

            data.current.push(reader.read_i32()? / 1000);
            data.power_reduce.push(reader.read_f32()?);

            data.torque_out_rl.push(reader.read_f32()?);
            data.torque_out_rr.push(reader.read_f32()?);
            data.torque_out_fl.push(reader.read_f32()?);
            data.torque_out_fr.push(reader.read_f32()?);

            data.spring_fr.push(reader.read_f32()? - 1630.0 - 420.0);
            data.spring_fl.push(reader.read_f32()? - 4750.0 + 400.0);
            data.spring_rl.push(reader.read_f32()? - 3125.0 + 115.0);
            data.spring_rr.push(reader.read_f32()? - 4005.0 - 200.0);
        }

        Ok(data)
    }

    pub fn power_fl(&self) -> impl Iterator<Item = Value> + '_ {
        self.torque_fl
            .iter()
            .zip(self.speed_fl.iter())
            .map(|(&torque, &speed)| 2.0 * PI / 60.0 * torque * (19.7 / 1000.0) * speed)
            .enumerate()
            .map(|(i, v)| Value::new(i as f64 * 0.02, v as f64))
    }

    pub fn power_fr(&self) -> impl Iterator<Item = Value> + '_ {
        self.torque_fr
            .iter()
            .zip(self.speed_fr.iter())
            .map(|(&torque, &speed)| 2.0 * PI / 60.0 * torque * (19.7 / 1000.0) * speed)
            .enumerate()
            .map(|(i, v)| Value::new(i as f64 * 0.02, v as f64))
    }

    pub fn power_rl(&self) -> impl Iterator<Item = Value> + '_ {
        self.torque_rl
            .iter()
            .zip(self.speed_rl.iter())
            .map(|(&torque, &speed)| 2.0 * PI / 60.0 * torque * (19.7 / 1000.0) * speed)
            .enumerate()
            .map(|(i, v)| Value::new(i as f64 * 0.02, v as f64))
    }

    pub fn power_rr(&self) -> impl Iterator<Item = Value> + '_ {
        self.torque_rr
            .iter()
            .zip(self.speed_rr.iter())
            .map(|(&torque, &speed)| 2.0 * PI / 60.0 * torque * (19.7 / 1000.0) * speed)
            .enumerate()
            .map(|(i, v)| Value::new(i as f64 * 0.02, v as f64))
    }

    pub fn speed_fl(&self) -> impl Iterator<Item = Value> + '_ {
        self.speed_fl
            .iter()
            .enumerate()
            .map(|(i, v)| Value::new(i as f64 * 0.02, *v as f64))
    }

    pub fn speed_fr(&self) -> impl Iterator<Item = Value> + '_ {
        self.speed_fr
            .iter()
            .enumerate()
            .map(|(i, v)| Value::new(i as f64 * 0.02, *v as f64))
    }

    pub fn speed_rl(&self) -> impl Iterator<Item = Value> + '_ {
        self.speed_rl
            .iter()
            .enumerate()
            .map(|(i, v)| Value::new(i as f64 * 0.02, *v as f64))
    }

    pub fn speed_rr(&self) -> impl Iterator<Item = Value> + '_ {
        self.speed_rr
            .iter()
            .enumerate()
            .map(|(i, v)| Value::new(i as f64 * 0.02, *v as f64))
    }

    pub fn torque_set_fl(&self) -> impl Iterator<Item = Value> + '_ {
        self.torque_fl
            .iter()
            .enumerate()
            .map(|(i, v)| Value::new(i as f64 * 0.02, *v as f64))
    }

    pub fn torque_set_fr(&self) -> impl Iterator<Item = Value> + '_ {
        self.torque_fr
            .iter()
            .enumerate()
            .map(|(i, v)| Value::new(i as f64 * 0.02, *v as f64))
    }

    pub fn torque_set_rl(&self) -> impl Iterator<Item = Value> + '_ {
        self.torque_rl
            .iter()
            .enumerate()
            .map(|(i, v)| Value::new(i as f64 * 0.02, *v as f64))
    }

    pub fn torque_set_rr(&self) -> impl Iterator<Item = Value> + '_ {
        self.torque_rr
            .iter()
            .enumerate()
            .map(|(i, v)| Value::new(i as f64 * 0.02, *v as f64))
    }

    pub fn torque_real_fl(&self) -> impl Iterator<Item = Value> + '_ {
        self.torque_out_fl
            .iter()
            .enumerate()
            .map(|(i, v)| Value::new(i as f64 * 0.02, *v as f64))
    }

    pub fn torque_real_fr(&self) -> impl Iterator<Item = Value> + '_ {
        self.torque_out_fr
            .iter()
            .enumerate()
            .map(|(i, v)| Value::new(i as f64 * 0.02, *v as f64))
    }

    pub fn torque_real_rl(&self) -> impl Iterator<Item = Value> + '_ {
        self.torque_out_rl
            .iter()
            .enumerate()
            .map(|(i, v)| Value::new(i as f64 * 0.02, *v as f64))
    }

    pub fn torque_real_rr(&self) -> impl Iterator<Item = Value> + '_ {
        self.torque_out_rr
            .iter()
            .enumerate()
            .map(|(i, v)| Value::new(i as f64 * 0.02, *v as f64))
    }
}

impl<T: Read> ReadUtils for T {}
trait ReadUtils: Read {
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
trait SeekUtils: Seek {
    fn len(&mut self) -> io::Result<u64> {
        let pos = self.seek(SeekFrom::Current(0))?;
        let len = self.seek(SeekFrom::End(0))?;
        self.seek(SeekFrom::Start(pos))?;
        Ok(len)
    }
}
