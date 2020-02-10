use std::string::FromUtf8Error;

pub fn convert_vec_pairs(x: Vec<u8>, y: Vec<u8>) -> Result<(String, String), failure::Error> {
    let x1= String::from_utf8(x)?;
    let y1 = String::from_utf8(y)?;

    Ok((x1, y1))
}

pub fn convert_vec_pairs_u8(x: &[u8], y: &[u8]) -> Result<(String, String), failure::Error> {
    let x1= std::str::from_utf8(x)?;
    let y1 = std::str::from_utf8(y)?;

    Ok((x1.to_string(), y1.to_string()))
}
