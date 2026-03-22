#[derive(Debug)]
pub enum SpinTime{
    Seconds(usize),
    Default
}

#[derive(Debug)]
pub enum WakeType{
    ActiveSpin,
    SlowSpin,
    Request,
    RequestSpin(Option<SpinTime>),
}

impl Default for WakeType {
    fn default() -> Self {  
        return WakeType::Request;
    }
}
