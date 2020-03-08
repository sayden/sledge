#![feature(box_syntax)]

use std::str::FromStr;

use sqlparser::ast::{BinaryOperator, Expr, Ident, SelectItem, SetExpr, Statement, Value};

fn op_equal(vs: (serde_json::Value, serde_json::Value)) -> bool {
    vs.0 == vs.1
}

fn op_gt(vs: (serde_json::Value, serde_json::Value)) -> bool {
    get_numbers(vs.0, vs.1, |a, b| a > b)
}

fn op_gte(vs: (serde_json::Value, serde_json::Value)) -> bool {
    get_numbers(vs.0, vs.1, |a, b| a >= b)
}

fn get_numbers<F>(a: serde_json::Value, b: serde_json::Value, f: F) -> bool
where
    F: FnOnce(f64, f64) -> bool,
{
    match (a, b) {
        (serde_json::Value::Number(n1), serde_json::Value::Number(n2)) => {
            f(n1.as_f64().unwrap(), n2.as_f64().unwrap())
        }
        _ => false,
    }
}

fn op_lt(vs: (serde_json::Value, serde_json::Value)) -> bool {
    get_numbers(vs.0, vs.1, |a, b| a < b)
}

fn op_lte(vs: (serde_json::Value, serde_json::Value)) -> bool {
    get_numbers(vs.0, vs.1, |a, b| a <= b)
}

fn values_i(
    v1: Ident,
    v2: Value,
    jj: &serde_json::Value,
) -> (serde_json::Value, serde_json::Value) {
    let a = jj.get(v1).unwrap().clone();
    let b = serde_value_from_sql_value(v2).unwrap();
    
    (a, b)
}

fn serde_value_from_sql_value(v: Value) -> Option<serde_json::Value> {
    match v {
        Value::Number(n) => serde_json::Value::from_str(n.as_str())
            .map_err(|err| println!("error trying to get value: {}", err))
            .ok(),
        Value::SingleQuotedString(s) => Some(serde_json::Value::from(s)),
        _ => None,
    }
}

fn values_i_i(
    v1: Ident,
    v2: Ident,
    jj: &serde_json::Value,
) -> (serde_json::Value, serde_json::Value) {
    let vv1 = jj.get(v1).unwrap().clone();
    let vv2 = jj.get(v2).unwrap().clone();
    (vv1, vv2)
}

fn a_b_f<F, F2>(left: Box<Expr>, right: Box<Expr>, jj: &serde_json::Value, f: F, f2: F2) -> bool
where
    F: FnOnce(bool, bool) -> bool,
    F2: FnOnce((serde_json::Value, serde_json::Value)) -> bool,
{
    match (*left, *right) {
        (Expr::Identifier(v1), Expr::Value(v2)) => f2(values_i(v1, v2, &jj)),
        (Expr::Value(v1), Expr::Identifier(v2)) => f2(values_i(v2, v1, &jj)),
        (Expr::Identifier(v1), Expr::Identifier(v2)) => f2(values_i_i(v1, v2, &jj)),
        (a, b) => f(solve_binary(a, jj), solve_binary(b, jj)),
    }
}

fn solve_binary(expr: Expr, jj: &serde_json::Value) -> bool {
    match expr {
        Expr::BinaryOp { left, op, right } => match op {
            BinaryOperator::Eq => a_b_f(left, right, jj, |a, b| a == b, op_equal),
            BinaryOperator::And => a_b_f(left, right, jj, |a, b| a && b, op_equal),
            BinaryOperator::Gt => a_b_f(left, right, jj, |a, _| a, op_gt),
            BinaryOperator::GtEq => a_b_f(left, right, jj, |a, _| a, op_gte),
            BinaryOperator::LtEq => a_b_f(left, right, jj, |_, b| b, op_lte),
            BinaryOperator::Lt => a_b_f(left, right, jj, |_, b| b, op_lt),
            BinaryOperator::Or => a_b_f(left, right, jj, |a, b| a || b, op_equal),
            _ => false,
        },
        _ => false,
    }
}

fn main() {
    use sqlparser::dialect::GenericDialect;
    use sqlparser::parser::Parser;

    let sql = r#"SELECT * FROM my_table WHERE age < 35 AND name = 'mario'"#;

    let j = r#"{"name": "mario", "surname": "castro", "age": 35, "weight": 80}"#;
    let jj: serde_json::Value = serde_json::from_str(j).unwrap();

    let dialect = GenericDialect {};

    let ast = Parser::parse_sql(&dialect, sql.to_string()).unwrap();

    for statement in ast {
        if let Statement::Query(q) = statement {
            match q.body {
                SetExpr::Select(c) => {
                    let res = solve_binary(c.selection.unwrap(), &jj);
                    println!("{}", res);

                    let projection =
                        c.projection
                            .into_iter()
                            .fold(Some(Vec::new()), |acc, v| match acc {
                                Some(mut items) => match v {
                                    SelectItem::Wildcard => None,
                                    SelectItem::UnnamedExpr(e) => {
                                        items.push(format!("{}", e));
                                        Some(items)
                                    }
                                    _ => None,
                                },
                                None => None,
                            });

                    let res = match projection {
                        None => serde_json::to_string_pretty(&jj).unwrap(),
                        Some(fields) => {
                            let value = serde_json::Value::from_str("{}").unwrap();
                            let folded = fields.into_iter().fold(value, |mut acc, x| {
                                let a = jj.get(&x).unwrap();
                                acc[&x] = a.clone();
                                acc
                            });
                            serde_json::to_string_pretty(&folded).unwrap()
                        }
                    };

                    println!("{}", res);
                }
                SetExpr::Query(q) => println!("query: {}", q),
                _ => println!("Some set expr"),
            }
        }
    }
}
