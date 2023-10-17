#[derive(Clone)]
pub struct Port(u16);

impl From<u16> for Port {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

#[derive(Clone)]
pub struct Target(String);

impl From<String> for Target {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<Target> for String {
    fn from(value: Target) -> Self {
        value.0
    }
}

#[derive(Clone)]
pub struct ServiceRecord {
    pub target: Target,
    pub port: Port,
}
