use bincode::{serialize, deserialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum OpCode {
    Set,
    Get,
    Del,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Request {
    pub op: OpCode,
    pub key: String,
    pub val: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Response {
    Success,
    Return(String),
    Failed(String),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Message {
    Req(Request),
    Res(Response),
}

pub fn mangle(m: &Message) -> Vec<u8> {
    serialize(&m).unwrap()
}

pub fn demangle(buf: &Vec<u8>) -> Result<Message, ()> {
    match deserialize(&buf[..]) {
        Ok(m) => Ok(m),
        Err(_) => Err(()),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_serialize() {
        let request = Request {
            op: OpCode::Set,
            key: "SomeKey".to_string(),
            val: "Value".to_string()
        };
        let encoded: Vec<u8> = serialize(&request).unwrap();

        println!("This is some output: {:?}", encoded);

        let decoded: Request = deserialize(&encoded[..]).unwrap();

        println!("{:?}", decoded);
        assert_eq!(request, decoded);
    }
}
