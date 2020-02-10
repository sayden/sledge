use failure::{Error, Context, Fail, Backtrace};
use std::fmt::Display;
use std::fmt;
use std::string::FromUtf8Error;

pub trait Storage {
    fn get(&self, s: &str) -> Result<Option<String>, Error>;
    fn put(&self, k: &str, v: &str) -> Result<(), Error>;
    fn range(&self, k: &str) -> Result<Box<dyn Iterator<Item=(String, String)>>, Error>;
}

//
//trait UnwrapIfOK {
//    fn unwrap_if_ok(self) -> Asdfd<Self>
//        where
//            Self: Sized;
//}
//
//#[derive(Clone)]
//struct Asdfd<I> {
//    iter: I,
//}
//
//impl Asdfd {
//    pub(super) fn new(iter: I) -> Asdfd<I> {
//        Asdfd { iter }
//    }
//}
//
//impl UnwrapIfOK for Asdfd {
//    fn unwrap_if_ok(self) -> Asdfd<Self>
//        where
//            Self: Sized,
//    {
//        Asdfd::new(self)
//    }
//}
//
//impl Iterator for Asdfd {
//    type Item = I;
//
//    fn next(&mut self) -> Option<Self::Item> {
//        let item = self.iter.next();
//        return if item.is_some() {
//            let a = item.unwrap();
//            Some(item.unwrap())
//        } else {
//            None
//        };
//    }
//}


#[cfg(test)]
mod tests {
    use std::string::FromUtf8Error;
    use failure::{Fail, ResultExt, Context};

    #[test]
    fn test_convert_to_string() {
        let result: Result<Option<Vec<Option<&str>>>, Box<dyn std::error::Error>> = Ok(Some(vec![Some("hello"), None]));

        if result.is_ok() {
            let maybe_data = result.unwrap();

            if maybe_data.is_some() {
                let v = maybe_data.unwrap();

                let final_value = v.iter().filter_map(|x| {
                    if x.is_some() {
                        Some(x.unwrap())
                    } else {
                        None
                    }
                });

                for i in final_value {
                    println!("Found some value: '{}'", i);
                }
            } else {
                println!("data not found")
            }
        } else {
            println!("error retrieving data")
        }
    }

    pub fn third(result: Result<Option<Vec<Option<&str>>>, FromUtf8Error>) -> Result<Option<String>, failure::Error> {
        let maybe_vec = result?;
        if maybe_vec.is_none() {
            bail!("not found")
        }

        let v = maybe_vec.unwrap();

        Ok(join_vec_of_maybe_str(v))
    }

    pub fn fourth(result: Result<Option<Vec<Option<&str>>>, FromUtf8Error>) -> Result<Option<String>, failure::Error> {
        let maybe_vec = result.or(Err(failure::err_msg("FromUtf8Error")))?;
        let v = maybe_vec.ok_or(failure::err_msg("values not found"))?;


        Ok(join_vec_of_maybe_str(v))
    }

    fn join_vec_of_maybe_str(v: Vec<Option<&str>>) -> Option<String> {

        let res: String = v
            .into_iter()
            .filter_map(|x| x)
            .map(|s| s.chars())
            .flatten()
            .collect();

        return if res.len() == 0 {
            None
        } else {
            Some(res)
        };
    }

    pub fn second(result: Result<Option<Vec<Option<&str>>>, failure::Error>) -> Result<Option<String>, failure::Error> {
        let maybe_vec = result?;
        if maybe_vec.is_none() {
            bail!("not found")
        }

        let v = maybe_vec.unwrap();

        Ok(join_vec_of_maybe_str(v))
    }

    #[test]
    fn test_second_approach() {
        let printer = |x: Result<Option<String>, failure::Error>| {
            println!("{}", match x {
                Ok(v) => match v {
                    Some(v) => format!("{}", v),
                    None => "value not found".to_string()
                },
                Err(e) => format!("{}", e)
            });
        };

        let result: Result<Option<Vec<Option<&str>>>, failure::Error> = Ok(Some(vec![Some("hello"), None, Some(" "), None, Some("world")]));
        let s = second(result);
        printer(s);

        let result2: Result<Option<Vec<Option<&str>>>, FromUtf8Error> = Ok(Some(vec![Some("hello"), None, Some(" "), None, Some("world")]));
        let r = third(result2);
        printer(r);

        let result3: Result<Option<Vec<Option<&str>>>, FromUtf8Error> = Ok(Some(vec![None, None]));
        let x = third(result3);
        printer(x);

        let result4: Result<Option<Vec<Option<&str>>>, FromUtf8Error> = String::from_utf8(vec![0, 159]).map(|_| None);
        let x = third(result4);
        printer(x);

//        let value = String::from_utf8(vec![0, 159]);
//        println!("{}", value.unwrap_err());

//        let a = ["1", "lol", "3", "NaN", "5"];

//        let mut iter = a.iter().filter_map(|s| {
//            let res1 = s.parse();
//            let res2 = res1.ok();
//            res2
//        });
    }
}
