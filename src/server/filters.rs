use serde_json::Value;
use sqlparser::ast::{Select, SetExpr, Statement};

use crate::components::iterator::SledgeIterator;
use crate::components::sql::solve_where;
use crate::server::handlers::json_nested_value;
use crate::server::query::Query;

pub enum Filter {
    Skip(usize),
    Limit(usize),
    UntilKey(Vec<u8>),
    FieldEquals(String, String),
    Sql(Vec<Statement>),
}

pub struct Filters {
    inner: Option<Vec<Filter>>,
}

impl Filters {
    pub fn new_sql(sql: Vec<Statement>) -> Self {
        Filters { inner: Some(vec![Filter::Sql(sql)]) }
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
                let a = res.get(0).unwrap();
                let b = res.get(1).unwrap();
                itermods.push(Filter::FieldEquals(a.to_string(), b.to_string()))
            }
        }

        if itermods.is_empty() {
            return Filters { inner: None };
        }

        Filters { inner: Some(itermods) }
    }

    pub fn apply(&mut self, iter: SledgeIterator) -> SledgeIterator {
        if self.inner.as_ref().is_none() {
            return iter;
        }

        let iterators = self.inner.take().unwrap();
        iterators.into_iter().fold(iter, move |acc, m| {
            match m {
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
                Filter::UntilKey(id) =>
                    box Iterator::take_while(acc, move |x| x.id != id),
                Filter::Sql(s) => {
                    log::info!("ASDFASDFASDFASDF");
                    box Iterator::filter(acc, move |a| {
                        if let Some(statement) = s.first() {
                            if let Statement::Query(q_st) = statement {
                                if let SetExpr::Select(c) = &q_st.body {
                                    if let Ok(jj) = serde_json::from_slice::<serde_json::Value>(a.value.as_slice()) {
                                        if let Some(selection) = &c.selection {
                                            return solve_where(selection.clone(), &jj);
                                        } else {
                                            log::warn!("no selection found")
                                        }
                                    } else {
                                        log::warn!("no serde_json::Value found")
                                    }
                                } else {
                                    log::warn!("no SetExpr::Select found")
                                }
                            } else {
                                log::warn!("no Statement::Query found")
                            }
                        } else {
                            log::warn!("no statement found")
                        }
                        return false;
                    })
                }
            }
        })
    }
}
