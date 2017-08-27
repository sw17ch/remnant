#[derive(Debug,Serialize,Deserialize)]
pub enum Request {
    Empty,
    Ping,
}

#[derive(Debug,Serialize,Deserialize)]
pub enum Response {
    Empty,
    Ping,
}
