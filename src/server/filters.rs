use serde_json::Value;
use sqlparser::ast::{SetExpr, Statement};

use crate::components::iterator::BoxedSledgeIter;
use crate::components::simple_pair::SimplePair;
use crate::components::sql::check_limit_or_offset;
use crate::components::sql::{solve_projection, solve_where};
use crate::server::handlers::json_nested_value;
use crate::server::query::Query;

pub enum Filter {
    Skip(usize),
    Limit(usize),
    UntilKey(Vec<u8>),
    FieldEquals(String, String),
    Sql(Box<sqlparser::ast::Query>),
}

pub struct Filters {
    inner: Option<Vec<Filter>>,
}

impl Filters {
    pub fn new_sql(sql: Vec<Statement>) -> Self {
        if sql.first().is_none() {
            log::error!("no statements found on sql");
            return Filters { inner: None };
        }

        let query = if let Statement::Query(temp) = sql.first().unwrap() {
            temp
        } else {
            log::warn!("no 'query' found on sql");
            return Filters { inner: None };
        };

        let mut itermods: Vec<Filter> = Vec::new();

        itermods.push(Filter::Sql(query.clone()));

        if let Some(offset) = check_limit_or_offset(&query.offset) {
            itermods.push(Filter::Skip(offset));
        }

        if let Some(limit) = check_limit_or_offset(&query.limit) {
            itermods.push(Filter::Limit(limit));
        }

        Filters { inner: Some(itermods) }
    }

    pub fn new(query: &Query) -> Self {
        let mut itermods = Vec::new();
        if let Some(skip) = query.skip {
            itermods.push(Filter::Skip(skip))
        }

        if let Some(limit) = query.limit {
            itermods.push(Filter::Limit(limit))
        }

        if let Some(until_key) = query.until_id.as_ref().cloned() {
            itermods.push(Filter::UntilKey(Vec::from(until_key)))
        }

        if let Some(field_equals) = query.field_equals.as_ref().cloned() {
            let res: Vec<&str> = field_equals.split(':').collect();
            if res.len() != 2 {
                log::warn!("no 'field equals' iterator could be created")
            } else {
                let a = res.get(0);
                let b = res.get(1);
                if let (Some(a), Some(b)) = (a, b) {
                    itermods.push(Filter::FieldEquals((*a).to_string(), (*b).to_string()))
                }
            }
        }

        if itermods.is_empty() {
            return Filters { inner: None };
        }

        Filters { inner: Some(itermods) }
    }

    pub fn apply(&mut self, iter: BoxedSledgeIter) -> BoxedSledgeIter {
        if self.inner.as_ref().is_none() {
            return iter;
        }

        let iterators = self.inner.take().unwrap();
        iterators.into_iter().fold(iter, move |acc, m| match m {
            Filter::Limit(n) => box Iterator::take(acc, n),
            Filter::Skip(n) => box Iterator::skip(acc, n),
            Filter::FieldEquals(k, val) => box Iterator::filter(acc, move |x| {
                let body: Value = match serde_json::from_slice(x.value.as_slice()) {
                    Ok(v) => v,
                    Err(err) => {
                        log::warn!("error getting value record in 'field_equals': {}", err);
                        return false;
                    }
                };
                let left = json_nested_value(k.as_str(), &body);
                let right: Value = Value::String(val.clone());
                *left == right
            }),
            Filter::UntilKey(id) => box Iterator::take_while(acc, move |x| x.id != id),
            Filter::Sql(query) => box Iterator::filter_map(acc, move |a| {
                // println!("WHAT? {:?}\n", q_st.body);

                let c = if let SetExpr::Select(temp) = &query.body {
                    temp
                } else {
                    log::warn!("no 'Select' found on sql");
                    return None;
                };

                let jj = serde_json::from_slice::<serde_json::Value>(a.value.as_slice())
                    .map_err(|err| {
                        log::warn!(
                            "[sql] error trying to get json from db result value: {}",
                            err.to_string()
                        )
                    })
                    .ok()?;

                // If no selection is found, this is probably a "SELECT * FROM [table]" query.
                if let Some(selection) = &c.selection {
                    if !solve_where(selection, &jj) {
                        return None;
                    }
                }

                let p = if let Some(temp) = solve_projection(&c.projection, jj) {
                    temp
                } else {
                    log::warn!("error trying to solve sql projection");
                    return None;
                };

                let res = serde_json::to_vec(&p)
                    .map_err(|err| {
                        log::warn!("error trying to get projection: {}", err.to_string())
                    })
                    .ok()?;

                Some(SimplePair {
                    id: a.id,
                    value: res,
                })
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use sqlparser::dialect::GenericDialect;
    use sqlparser::parser::Parser;

    use crate::components::simple_pair::SimplePair;
    use crate::server::filters::*;
    extern crate chrono;

    #[test]
    fn test_sql() {
        let sql_query = std::fs::read_to_string("src/server/tests.sql").expect("Something went \
                                                                                wrong reading \
                                                                                the file");

        let data = vec![r#"{"name":"mario","age": 35}"#,
                        r#"{"name":"ula","age": 31}"#,];

        let dialect = GenericDialect {};
        let ast = Parser::parse_sql(&dialect, sql_query).unwrap();

        let mut f = Filters::new_sql(ast);

        let vs = data.into_iter().map(|x| {
                                     SimplePair { id: Vec::from("asdas"),
                                                  value: Vec::from(x) }
                                 });

        let res = f.apply(box vs).collect::<Vec<SimplePair>>();

        assert!(!res.is_empty());

        for i in res {
            println!("SimplePair: {}",
                     std::str::from_utf8(i.value.as_slice()).unwrap());
        }
    }
}
