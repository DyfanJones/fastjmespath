use extendr_api::prelude::*;
use extendr_api::Result;
use jmespath::{Expression, Variable};

mod r_to_variable;
mod variable_to_r;

use r_to_variable::r_to_variable;
use variable_to_r::variable_to_r;

/// A compiled JMESPath expression.
///
/// Compile once with `jmespath_compile()` and reuse for many searches.
/// @noRd
#[derive(Debug, Clone)]
#[extendr]
pub struct JmesExpr {
    expr: Expression<'static>,
}

fn compile(expr: &str) -> Result<Expression<'static>> {
    jmespath::compile(expr).map_err(|e| Error::Other(format!("JMESPath syntax error: {}", e)))
}

fn run(expr: &Expression<'static>, data: &Variable, simplify: bool) -> Result<Robj> {
    let result = expr
        .search(data)
        .map_err(|e| Error::Other(format!("JMESPath search error: {}", e)))?;
    variable_to_r(&result, simplify)
}

#[extendr]
impl JmesExpr {
    fn new(expr: &str) -> Self {
        match compile(expr) {
            Ok(expr) => Self { expr },
            Err(e) => throw_r_error(e.to_string()),
        }
    }

    fn expression(&self) -> String {
        self.expr.as_str().to_string()
    }

    fn search(&self, data: Robj, simplify: bool) -> Result<Robj> {
        let data = r_to_variable(&data)?;
        run(&self.expr, &data, simplify)
    }

    fn search_json(&self, json: &str, simplify: bool) -> Result<Robj> {
        let data = Variable::from_json(json).map_err(|e| Error::Other(format!("Invalid JSON: {}", e)))?;
        run(&self.expr, &data, simplify)
    }
}

extendr_module! {
    mod fastjmespath;
    impl JmesExpr;
}
