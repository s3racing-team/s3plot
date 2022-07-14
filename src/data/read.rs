use std::io::{self, Read, Seek, SeekFrom};

use super::{DataEntry, Error, TempEntry, Version};

impl Version {
    fn data_sample_size(&self) -> usize {
        match self {
            Version::S321e => 132,
            Version::S322e => 134,
        }
    }

    fn temp_sample_size(&self) -> usize {
        match self {
            Version::S321e => 44,
            Version::S322e => 44,
        }
    }
}

pub fn read_extend_data(
    reader: &mut (impl Read + Seek),
    data: &mut Vec<DataEntry>,
    version: Version,
) -> Result<(), Error> {
    let len = reader.len()?;
    let samples = len as usize / version.data_sample_size();
    data.reserve(samples);

    for _ in 0..samples {
        let entry = read_data_entry(reader, version)?;
        entry.sanity_check()?;
        data.push(entry);
    }

    Ok(())
}

fn read_data_entry(reader: &mut (impl Read + Seek), version: Version) -> Result<DataEntry, Error> {
    Ok(DataEntry {
        ms: reader.read_f32()?,

        power: reader.read_f32()?,

        driven: reader.read_f32()?,
        energy_to_finish_factor: reader.read_f32()?,
        energy_total: reader.read_f32()?,

        gas: reader.read_f32()?,

        ams_u_min: reader.read_i16()?,
        ams_u_min_true: reader.read_i16()?,
        ams_u_avg: match version {
            Version::S321e => 0,
            Version::S322e => reader.read_i16()?,
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
    })
}

impl DataEntry {
    fn sanity_check(&self) -> Result<(), Error> {
        sanity_check_f32(self.ms)?;

        sanity_check_f32(self.power)?;

        sanity_check_f32(self.driven)?;
        sanity_check_f32(self.energy_to_finish_factor)?;
        sanity_check_f32(self.energy_total)?;

        sanity_check_f32(self.gas)?;

        sanity_check_i16(self.ams_u_min)?;
        sanity_check_i16(self.ams_u_min_true)?;
        sanity_check_i16(self.ams_u_avg)?;

        sanity_check_f32(self.l_uzk)?;
        sanity_check_f32(self.speed_rl)?;
        sanity_check_f32(self.torque_set_rl)?;
        sanity_check_f32(self.speed_rr)?;
        sanity_check_f32(self.torque_set_rr)?;
        sanity_check_f32(self.speed_fl)?;
        sanity_check_f32(self.torque_set_fl)?;
        sanity_check_f32(self.speed_fr)?;
        sanity_check_f32(self.torque_set_fr)?;

        sanity_check_i16(self.accel_x)?;
        sanity_check_i16(self.accel_y)?;
        sanity_check_i16(self.accel_z)?;

        sanity_check_i16(self.gyro_x)?;
        sanity_check_i16(self.gyro_y)?;
        sanity_check_i16(self.gyro_z)?;

        sanity_check_i16(self.steering)?;
        sanity_check_f32(self.break_front)?;
        sanity_check_f32(self.break_rear)?;
        sanity_check_f32(self.break_pedal)?;

        sanity_check_i32(self.current)?;
        sanity_check_f32(self.power_reduce)?;

        sanity_check_f32(self.torque_real_rl)?;
        sanity_check_f32(self.torque_real_rr)?;
        sanity_check_f32(self.torque_real_fl)?;
        sanity_check_f32(self.torque_real_fr)?;

        sanity_check_f32(self.spring_fr)?;
        sanity_check_f32(self.spring_fl)?;
        sanity_check_f32(self.spring_rl)?;
        sanity_check_f32(self.spring_rr)?;

        Ok(())
    }
}

pub fn read_extend_temp(
    reader: &mut (impl Read + Seek),
    temp: &mut Vec<TempEntry>,
    version: Version,
) -> Result<(), Error> {
    let len = reader.len()?;
    let samples = len as usize / version.temp_sample_size();
    temp.reserve(samples);

    for _ in 0..samples {
        let entry = read_temp_entry(reader, version)?;
        entry.sanity_check()?;
        temp.push(entry);
    }

    Ok(())
}

fn read_temp_entry(reader: &mut (impl Read + Seek), _version: Version) -> Result<TempEntry, Error> {
    Ok(TempEntry {
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

impl TempEntry {
    fn sanity_check(&self) -> Result<(), Error> {
        sanity_check_f32(self.ms)?;

        sanity_check_i16(self.ams_temp_max)?;

        sanity_check_i16(self.water_temp_converter)?;
        sanity_check_i16(self.water_temp_motor)?;

        sanity_check_f32(self.temp_rl)?;
        sanity_check_f32(self.temp_rr)?;
        sanity_check_f32(self.temp_fl)?;
        sanity_check_f32(self.temp_fr)?;

        sanity_check_i16(self.room_temp_rl)?;
        sanity_check_i16(self.room_temp_rr)?;
        sanity_check_i16(self.room_temp_fl)?;
        sanity_check_i16(self.room_temp_fr)?;

        sanity_check_i16(self.heatsink_temp_rl)?;
        sanity_check_i16(self.heatsink_temp_rr)?;
        sanity_check_i16(self.heatsink_temp_fl)?;
        sanity_check_i16(self.heatsink_temp_fr)?;

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

fn sanity_check_f32(val: f32) -> Result<(), Error> {
    if val.is_nan() {
        return Err(Error::SanityCheck("Value is nan"));
    }
    if val.is_infinite() {
        return Err(Error::SanityCheck("Value is infinite"));
    }
    Ok(())
}

fn sanity_check_i16(val: i16) -> Result<(), Error> {
    if val == i16::MAX {
        return Err(Error::SanityCheck("Value is max"));
    }
    if val == i16::MIN {
        return Err(Error::SanityCheck("Value is min"));
    }
    Ok(())
}

fn sanity_check_i32(val: i32) -> Result<(), Error> {
    if val == i32::MAX {
        return Err(Error::SanityCheck("Value is max"));
    }
    if val == i32::MIN {
        return Err(Error::SanityCheck("Value is min"));
    }
    Ok(())
}
