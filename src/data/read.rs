use std::io::{self, Read, Seek, SeekFrom};

use super::{Data, DataEntry, Temp, TempEntry, Version};

impl Version {
    fn data_sample_size(&self) -> usize {
        match self {
            Version::S322e => 132,
            Version::S321e => 134,
        }
    }

    fn temp_sample_size(&self) -> usize {
        match self {
            Version::S322e => 44,
            Version::S321e => 44,
        }
    }
}

impl Data {
    fn extend_capacity(&mut self, cap: usize) {
        self.entries.reserve(cap);
    }

    pub fn read_extend(
        &mut self,
        reader: &mut (impl Read + Seek),
        version: Version,
    ) -> anyhow::Result<()> {
        let len = reader.len()?;
        let samples = len as usize / version.data_sample_size();
        self.extend_capacity(samples);

        for _ in 0..samples {
            self.entries.push(DataEntry {
                ms: reader.read_f32()?,

                power: reader.read_f32()?,

                driven: reader.read_f32()?,
                energy_to_finish_factor: reader.read_f32()?,
                energy_total: reader.read_f32()?,

                gas: reader.read_f32()?,

                ams_u_min: reader.read_i16()?,
                ams_u_min_true: reader.read_i16()?,
                ams_u_avg: match version {
                    Version::S322e => 0,
                    Version::S321e => reader.read_i16()?,
                },

                l_uzk: reader.read_f32()?,
                speed_rl: reader.read_f32()?,
                torque_set_rl: reader.read_f32()?,
                speed_rr: reader.read_f32()?,
                torque_set_rr: -reader.read_f32()?,
                speed_fl: reader.read_f32()?,
                torque_set_fl: reader.read_f32()?,
                speed_fr: reader.read_f32()?,
                torque_set_fr: -reader.read_f32()?,

                accel_x: reader.read_i16()?,
                accel_y: reader.read_i16()?,
                accel_z: reader.read_i16()?,

                gyro_x: reader.read_i16()?,
                gyro_y: reader.read_i16()?,
                gyro_z: reader.read_i16()?,

                steering: reader.read_i16()?,
                break_front: reader.read_f32()?,
                break_rear: reader.read_f32()?,
                break_pedal: reader.read_f32()?,

                current: reader.read_i32()? / 1000,
                power_reduce: reader.read_f32()?,

                torque_real_rl: reader.read_f32()?,
                torque_real_rr: reader.read_f32()?,
                torque_real_fl: reader.read_f32()?,
                torque_real_fr: reader.read_f32()?,

                spring_fr: reader.read_f32()? - 1630.0 - 420.0,
                spring_fl: reader.read_f32()? - 4750.0 + 400.0,
                spring_rl: reader.read_f32()? - 3125.0 + 115.0,
                spring_rr: reader.read_f32()? - 4005.0 - 200.0,
            });
        }

        Ok(())
    }
}

impl Temp {
    fn extend_capacity(&mut self, cap: usize) {
        self.entries.reserve(cap);
    }

    pub fn read_extend(
        &mut self,
        reader: &mut (impl Read + Seek),
        version: Version,
    ) -> anyhow::Result<()> {
        let len = reader.len()?;
        let samples = len as usize / version.temp_sample_size();
        self.extend_capacity(samples);

        for _ in 0..samples {
            self.entries.push(TempEntry {
                ms: reader.read_f32()?,

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

                heatsink_temp_rl: reader.read_i16()?,
                heatsink_temp_rr: reader.read_i16()?,
                heatsink_temp_fl: reader.read_i16()?,
                heatsink_temp_fr: reader.read_i16()?,
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
