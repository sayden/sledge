#![feature(box_syntax)]

use std::str::FromStr;

use sqlparser::ast::{BinaryOperator, Expr, Ident, SelectItem, SetExpr, Statement, Value};

fn op_equal(vs: (&serde_json::Value, serde_json::Value)) -> bool {
    *vs.0 == vs.1
}

fn op_gt(vs: (&serde_json::Value, serde_json::Value)) -> bool {
    match (vs.0, &vs.1) {
        (serde_json::Value::Number(n1), serde_json::Value::Number(n2)) => n1.as_f64().unwrap() > n2.as_f64().unwrap(),
        _ => false,
    }
}

fn values(v1: Value, v2: Value, j: &serde_json::Value) -> (&serde_json::Value, serde_json::Value) {
    (j.get(v1.to_string()).unwrap(), serde_json::Value::from(v2.to_string()))
}

fn values_i(v1: Ident, v2: Value, j: &serde_json::Value) -> (&serde_json::Value, serde_json::Value) {
    (j.get(v1).unwrap(), serde_json::Value::from(v2.to_string()))
}

fn values_i_i(v1: Ident, v2: Ident, j: &serde_json::Value) -> (&serde_json::Value, serde_json::Value) {
    (j.get(v1).unwrap(), serde_json::Value::from_str(v2.as_str()).unwrap())
}

fn a_b_f<F>(left: Box<Expr>, right: Box<Expr>, jj: &serde_json::Value, f: Box<F>) -> bool
    where F: FnOnce((&serde_json::Value, serde_json::Value)) -> bool {
    match (*left, *right) {
        (Expr::Identifier(v1), Expr::Value(v2)) => op_equal(values_i(v1, v2, &jj)),
        (Expr::Value(v1), Expr::Identifier(v2)) => op_equal(values_i(v2, v1, &jj)),
        (Expr::Identifier(v1), Expr::Identifier(v2)) => op_equal(values_i_i(v1, v2, &jj)),
        (a, b) => solve_binary(box a, jj) && solve_binary(box b, jj),
    }
}

fn solve_binary(expr: Box<Expr>, jj: &serde_json::Value) -> bool {
    match *expr {
        Expr::BinaryOp { left, op, right } => {
            match op {
                BinaryOperator::Eq => a_b_f(left, right, jj, box op_equal),
                BinaryOperator::And => a_b_f(left, right, jj, box op_gt),
                BinaryOperator::Gt => a_b_f(left, right, jj, box op_equal),
                _ => false
            }
        }
        _ => false
    }
}

fn main() {
    use sqlparser::dialect::GenericDialect;
    use sqlparser::parser::Parser;

    let sql = r#"SELECT * FROM my_table WHERE name = "mario" AND surname = "castro""#;

    let j = r#"{"name": "mario", "surname": "castro", "age": 35}"#;
    let jj: serde_json::Value = serde_json::from_str(j).unwrap();

    let dialect = GenericDialect {}; // or AnsiDialect, or your own dialect ...

    let ast = Parser::parse_sql(&dialect, sql.to_string()).unwrap();

    for statement in ast {
        match statement {
            Statement::Query(q) => {
                match q.body {
                    SetExpr::Select(c) => {
                        let res = solve_binary(box c.selection.unwrap(), &jj);
                        println!("{}", res);

                        let projection = c.projection
                            .into_iter()
                            .fold(Some(Vec::new()), |acc, v| {
                                match acc {
                                    Some(mut items) => match v {
                                        SelectItem::Wildcard => return None,
                                        SelectItem::UnnamedExpr(e) => {
                                            items.push(format!("{}", e));
                                            return Some(items);
                                        }
                                        _ => None
                                    },
                                    None => None,
                                }
                            });

                        let res = match projection {
                            None => serde_json::to_string_pretty(&jj).unwrap(),
                            Some(fields) => {
                                let value = serde_json::Value::from_str("{}").unwrap();
                                let folded = fields.into_iter()
                                    .map(|x| format!("{}", x))
                                    .fold(value, |mut acc, x| {
                                        let a = jj.get(&x).unwrap();
                                        acc[&x] = a.clone();
                                        acc
                                    });
                                serde_json::to_string_pretty(&folded).unwrap()
                            }
                        };

                        println!("{}", res);
                    }
                    SetExpr::Query(q) => {
                        println!("query: {}", q)
                    }
                    _ => println!("Some set expr")
                }
            }
            _ => println!("Other")
        }
    }
}