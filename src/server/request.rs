use std::collections::HashMap;

pub struct Request<'a> {
    pub method: &'a str,
    pub path: &'a str,
    pub protocol: &'a str,
    pub headers: HashMap<&'a str, &'a str>
}

impl Request<'_> {
    pub fn parse(req: &str) -> Option<Request> {
        let request_line: Vec<&str> = req.splitn(3, ' ').collect();

        if request_line.len() == 3 {
            let protocol: Vec<&str> = request_line[2].split("\r\n").collect();
            let mut headers: HashMap<&str, &str> = HashMap::new();
            for line in &protocol[1..] {
                let pair: Vec<&str> = line.splitn(2, ":").collect();
                if pair.len() == 2 {
                    headers.insert(pair[0].trim(), pair[1].trim());
                }
            }
            if protocol.len() > 1 {
                return Some(Request {
                    method: request_line[0],
                    path: request_line[1],
                    protocol: protocol[0],
                    headers
                });
            }
        }
        None
    }
}
