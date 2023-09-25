use nom::{
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{map_res, opt},
    sequence::tuple,
    IResult,
};

#[derive(Debug)]
pub struct ResourceInstance {
    pub object_id: u16,
    pub instance_id: u16,
    pub resource_id: u16,
}

impl ResourceInstance {
    pub fn from_str(s: &str) -> Result<Self, ()> {
        Self::parse(s)
            .map(|(_, resource_instance)| resource_instance)
            .map_err(|_| ())
    }

    fn parse(input: &str) -> IResult<&str, Self> {
        let (input, (_, object_id, _, instance_id, _, resource_id)) =
            tuple((opt(slash), digit_u16, slash, digit_u16, slash, digit_u16))(input)?;

        Ok((
            input,
            Self {
                object_id,
                instance_id,
                resource_id,
            },
        ))
    }

    pub fn to_path(&self) -> String {
        format!(
            "/{}/{}/{}",
            self.object_id, self.instance_id, self.resource_id
        )
    }

    pub fn parse_payload(&self, payload: coapium::codec::Payload) -> Payload {
        if self.resource_id == 5852 {
            let value: [u8; 4] = payload.value().try_into().unwrap();
            let value = u32::from_be_bytes(value);
            Payload::UnsignedInt(value)
        } else {
            Payload::String(String::from_utf8(payload.value().to_vec()).unwrap())
        }
    }
}

#[derive(Debug)]
pub enum Payload {
    String(String),
    UnsignedInt(u32),
}

fn slash(input: &str) -> IResult<&str, &str> {
    tag("/")(input)
}

fn digit_u16(input: &str) -> IResult<&str, u16> {
    map_res(digit1, str::parse)(input)
}
