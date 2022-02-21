use curl::easy::Easy;
use json;

pub fn fetch(url: &str) -> json::JsonValue {
    let mut buffer = Vec::new();
    let mut handle = Easy::new();

    handle.url(url).unwrap();

    {
        let mut transfer = handle.transfer();

        transfer
            .write_function(|data| {
                buffer.extend_from_slice(data);
                Ok(data.len())
            })
            .unwrap();

        transfer.perform().unwrap();
    }

    let stringified_json = String::from_utf8(buffer).unwrap();

    json::parse(&stringified_json).unwrap()
}
