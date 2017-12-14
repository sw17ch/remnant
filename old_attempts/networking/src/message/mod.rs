#[derive(Debug,Serialize,Deserialize)]
pub enum Request {
    Empty,
    Ping,
}

impl Default for Request {
    fn default() -> Request { Request::Empty }
}
#[derive(Debug,Serialize,Deserialize)]
pub enum Response {
    Empty,
    Ping,
}

impl Default for Response {
    fn default() -> Response { Response::Empty }
}
