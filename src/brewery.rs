use std::io;

struct Brewery {
    mash_tun: MashTun,
    kettle: Kettle,
}

struct MashTun {
    liquid: bool,
}

struct Kettle {
    liquid: bool,
}

pub trait Vessel {
    fn set_target_temp(&self) -> Result<(), io::Error> {
        Ok(())
    }

    fn get_target_temp(&self) -> Result<(f32), io::Error> {
        Ok(63.0)
    }

    fn update_control_signal(&self) -> Result<(), io::Error> {
        Ok(())
    }
}
