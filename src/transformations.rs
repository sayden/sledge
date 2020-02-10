use std::string::FromUtf8Error;

pub fn convert_vec_pairs(x: Vec<u8>, y: Vec<u8>) -> Result<(String, String), failure::Error> {
    let x1: Result<String, FromUtf8Error> = String::from_utf8(x.to_vec());
    let y1: Result<String, FromUtf8Error> = String::from_utf8(y.to_vec());

    let (x2, y2) = match (x1, y1) {
        (Ok(x3), Ok(y3)) => (x3, y3),
        (Err(e1), Err(e2)) => bail!(format!("{}, {}", e1, e2)),
        (_, Err(e2)) => bail!(e2),
        (Err(e1), _) => bail!(e1),
    };

    Ok((x2, y2))
}
