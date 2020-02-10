use std::string::FromUtf8Error;

pub fn convert_vec_pairs(x: Vec<u8>, y: Vec<u8>) -> Result<(String, String), failure::Error> {
    let x1= String::from_utf8(x)?;
    let y1 = String::from_utf8(y)?;

    Ok((x1, y1))
}
