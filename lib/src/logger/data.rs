pub struct DataLog {}

impl DataLog {
    pub fn write(data: &DataEntry) {
        println!("{:?}", data);
    }
}

#[derive(Debug)]
pub struct DataEntry {}
