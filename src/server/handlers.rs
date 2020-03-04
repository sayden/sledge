use std::convert::Infallible;

use bytes::Bytes;
use http::Response;
use hyper::Body;

use crate::channels::parser::{Channel, parse_and_modify_u8};
use crate::components::rocks;
use crate::components::storage::Error;
use crate::server::query::Query;
use crate::server::responses::{new_read_error, new_read_ok, new_write_error, new_write_ok};

pub async fn put(cf: &str, maybe_query: Option<Query>, maybe_path_id: Option<&str>, req: Body, maybe_channel: Option<Channel>)
                 -> Result<Response<Body>, Infallible>
{
    let whole_body = match hyper::body::to_bytes(req).await {
        Err(err) => return Ok(new_write_error(err, None, cf)),
        Ok(body) => body,
    };

    let id = match rocks::get_id(&maybe_query, maybe_path_id, Some(&whole_body)) {
        Some(id) => id,
        None => return Ok(new_write_error("no id found", None, cf))
    };

    let maybe_data = match pass_through_channel(maybe_query, whole_body.as_ref(), maybe_channel) {
        Ok(d) => d,
        Err(response) => return response,
    };

    let data = maybe_data.unwrap_or(whole_body);

    match rocks::put(cf, &id, data) {
        Ok(()) => Ok(new_write_ok(id, cf)),
        Err(err) => Ok(new_write_error(err, Some(id), cf)),
    }
}

pub fn get(maybe_query: Option<Query>, cf: String, id: String, maybe_channel: Option<Channel>) -> Result<Response<Body>, Infallible> {
    let value = match rocks::get(&cf, &id) {
        Ok(res) => res,
        Err(err) => return Ok(new_read_error(err, id.into(), cf.into()))
    };

    let data = match pass_through_channel(maybe_query, value.as_ref(), maybe_channel) {
        Ok(d) => d,
        Err(response) => return response,
    };

    Ok(new_read_ok(data.unwrap_or(Bytes::from(value)).as_ref(), id, cf))
}

fn pass_through_channel(maybe_query: Option<Query>, whole_body: &[u8], maybe_channel: Option<Channel>) -> Result<Option<Bytes>, Result<Response<Body>, Infallible>> {
    let maybe_channel = match get_channel(&maybe_query) {
        Ok(res) => res.or(maybe_channel),
        Err(err) => return Err(Ok(new_write_error(err, None, "_channel"))),
    };

    match maybe_channel {
        Some(c) => match parse_and_modify_u8(whole_body, &c) {
            Ok(v) => Ok(Some(Bytes::from(v))),
            Err(err) => return Err(Ok(new_write_error(err, None, "_channel"))),
        },
        None => Ok(None),
    }
}

pub fn get_channel(maybe_query: &Option<Query>) -> Result<Option<Channel>, Error>
{
    match maybe_query {
        None => Ok(None),
        Some(query) => match &query.channel {
            Some(channel_id) => {
                let res = rocks::get(&"_channel".to_string(), &channel_id.clone())?;
                let c = Channel::new_vec(res)?;
                return Ok(Some(c));
            }
            None => Ok(None),
        }
    }
}