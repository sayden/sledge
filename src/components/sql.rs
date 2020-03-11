use std::cmp::Ordering;
use std::str::FromStr;

use futures::future::{Either, Either::Left, Either::Right};
use serde_json::Value as sValue;
use sqlparser::ast::{Expr, SelectItem};

pub mod utils {
    use sqlparser::ast::{SetExpr, Statement, TableFactor};

    pub fn get_from(ast: &[Statement]) -> Option<String> {
        let st = ast.first()?;
        if let Statement::Query(q_st) = st {
            if let SetExpr::Select(s) = &q_st.body {
                if let Some(b) = s.from.first() {
                    return match &b.relation {
                        TableFactor::Table { name, alias: _, args: _, with_hints: _ } => {
                            Some(name.0.join(""))
                        }
                        _ => None
                    };
                }
            }
        };
        None
    }
}


pub fn solve_where(expr: &Expr, jj: &sValue) -> bool {
    match expr {
        Expr::BinaryOp { left, op, right } => expr::binary_operation(left, op, right, jj),
        e => {
            println!("expression in where not recognized: {:?}", e);
            false
        },
    }
}
#[derive(Debug)]
struct SerdeValueWrapper {
    inner: sValue,
}

impl PartialOrd for SerdeValueWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.inner.as_f64().and_then(|n1| other.inner.as_f64().and_then(|n2| {
            n1.partial_cmp(&n2)
        }))
    }
}

impl PartialEq for SerdeValueWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}


impl SerdeValueWrapper {
    fn and(&self, other: &Self) -> bool {
        !self.inner.is_null() & !other.inner.is_null()
    }

    fn or(&self, other: &Self) -> bool {
        !self.inner.is_null() | !other.inner.is_null()
    }
}

pub fn check_limit_or_offset(e: &Option<Expr>)->Option<usize>{
    e.as_ref().map(|f| {
        match f {
            Expr::Value(n) => match n {
                sqlparser::ast::Value::Number(n) => n.parse::<usize>()
                .ok(),
                _ => None,
            },
            _ => None,
        }
    }).flatten()
}

trait AsValue {
    fn serde(&self, jj: &sValue) -> Either<SerdeValueWrapper, bool>;
}

impl AsValue for Expr {
    fn serde(&self, jj: &sValue) -> Either<SerdeValueWrapper, bool> {
        Left(SerdeValueWrapper {
            inner: match self {
                Expr::Identifier(i) => json_nested_value(i.as_ref(), jj).clone(),
                Expr::CompoundIdentifier(c) => json_nested_value(&c.join("."), jj).clone(),
                Expr::Value(v) => match v {
                    sqlparser::ast::Value::Number(n) => sValue::from_str(n.as_str()).ok().unwrap_or_else(|| sValue::Null),
                    sqlparser::ast::Value::SingleQuotedString(s) => serde_json::Value::from(s.as_str()),
                    _ => sValue::Null,
                },
                Expr::Nested(e) => return Right(solve_where(e, jj)),
                unknown => return Right(solve_where(unknown, jj)),
            }
        })
    }
}


mod expr {
    use futures::future::Either;
    use serde_json::Value as sValue;
    use sqlparser::ast::{BinaryOperator, Expr};

    use crate::components::sql::AsValue;

    pub fn binary_operation(left: &Expr, op: &BinaryOperator, right: &Expr, jj: &sValue) -> bool {
        let l = left.serde(jj);
        let r = right.serde(jj);

        //This duplication of code avoids boxing
        match (l, r) {
            (Either::Left(v1), Either::Left(v2)) => {
                match op {
                    BinaryOperator::Eq => v1 == v2,
                    BinaryOperator::NotEq => v1 != v2,
                    BinaryOperator::Gt => v1 > v2,
                    BinaryOperator::GtEq => v1 >= v2,
                    BinaryOperator::LtEq => v1 <= v2,
                    BinaryOperator::Lt => v1 < v2,
                    BinaryOperator::And => v1.and(&v2),
                    BinaryOperator::Or => v1.or(&v2),
                    // BinaryOperator::Divide => solve_binary(left, right, jj, | a,b | a / b, op::divide),
                    _ => false,
                }
            }
            (Either::Right(e1), Either::Right(e2)) =>
                match op {
                    BinaryOperator::Eq => e1 == e2,
                    BinaryOperator::NotEq => e1 != e2,
                    BinaryOperator::And => e1 && e2,
                    BinaryOperator::Gt => e1.gt(&e2),
                    BinaryOperator::GtEq => e1.ge(&e2),
                    BinaryOperator::LtEq => e1.le(&e2),
                    BinaryOperator::Lt => e1.lt(&e2),
                    BinaryOperator::Or => e1 || e2,
                    // BinaryOperator::Divide => solve_binary(left, right, jj, | a,b | a / b, op::divide),
                    _ => false,
                }
            _ => false,
        }
    }
}

pub fn solve_projection(projection: &[SelectItem], jj: sValue) -> Option<sValue> {
    let mut out = sValue::from_str("{}").unwrap();

    for v in projection {
        match v {
            SelectItem::Wildcard => return Some(jj),
            SelectItem::UnnamedExpr(e) => {
                match e {
                    Expr::Function(f) => println!("Function {:?}", f),
                    Expr::Identifier(i) => {
                        let a = jj.get(&i)?;
                        out[i] = a.clone();
                    }
                    Expr::Value(v) => println!("Value {:?}", v),
                    e => println!("Expression not recognized: {:?}",e),
                }
            }
            _ => (),
        }
    }

    Some(out)
}


pub fn json_nested_value<'a>(k: &str, v: &'a sValue) -> &'a sValue {
    k.replace("\"", "").split('.').fold(v, move |acc, x| &acc[x])
}