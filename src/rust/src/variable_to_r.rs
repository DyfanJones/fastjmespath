use extendr_api::prelude::*;
use extendr_api::Result;
use jmespath::Variable;
use serde_json::Number;

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Bool,
    Int,
    Dbl,
    Str,
}

fn as_i32(n: &Number) -> Option<i32> {
    n.as_i64().and_then(|i| i32::try_from(i).ok())
}

fn scalar_kind(v: &Variable) -> Option<Kind> {
    match v {
        Variable::Bool(_) => Some(Kind::Bool),
        Variable::Number(n) => Some(if as_i32(n).is_some() { Kind::Int } else { Kind::Dbl }),
        Variable::String(_) => Some(Kind::Str),
        _ => None,
    }
}

/// Work out the vector type an array can be simplified to. `null` elements
/// are allowed (they become `NA`) as long as at least one value is present.
fn common_kind(items: &[std::rc::Rc<Variable>]) -> Option<Kind> {
    let mut kind: Option<Kind> = None;
    for item in items {
        if matches!(**item, Variable::Null) {
            continue;
        }
        let k = scalar_kind(item)?;
        kind = match (kind, k) {
            (None, k) => Some(k),
            (Some(a), b) if a == b => Some(a),
            (Some(Kind::Int), Kind::Dbl) | (Some(Kind::Dbl), Kind::Int) => Some(Kind::Dbl),
            _ => return None,
        };
    }
    kind
}

fn simplify_array(items: &[std::rc::Rc<Variable>], kind: Kind) -> Robj {
    match kind {
        Kind::Bool => Logicals::from_values(items.iter().map(|v| match **v {
            Variable::Bool(b) => Rbool::from(b),
            _ => Rbool::na(),
        }))
        .into(),
        Kind::Int => Integers::from_values(items.iter().map(|v| match &**v {
            Variable::Number(n) => Rint::from(as_i32(n).unwrap_or(i32::MIN)),
            _ => Rint::na(),
        }))
        .into(),
        Kind::Dbl => Doubles::from_values(items.iter().map(|v| match &**v {
            Variable::Number(n) => Rfloat::from(n.as_f64().unwrap_or(f64::NAN)),
            _ => Rfloat::na(),
        }))
        .into(),
        Kind::Str => Strings::from_values(items.iter().map(|v| match &**v {
            Variable::String(s) => Rstr::from(s.as_str()),
            _ => Rstr::na(),
        }))
        .into(),
    }
}

/// Convert a JMESPath result back to R.
///
/// Objects become named lists. With `simplify`, arrays of scalars of one
/// type become atomic vectors; otherwise arrays are always lists. Integers
/// outside the `i32` range are returned as doubles.
pub fn variable_to_r(v: &Variable, simplify: bool) -> Result<Robj> {
    Ok(match v {
        Variable::Null => ().into(),
        Variable::Bool(b) => (*b).into(),
        Variable::String(s) => s.as_str().into(),
        Variable::Number(n) => match as_i32(n) {
            Some(i) => i.into(),
            None => n.as_f64().unwrap_or(f64::NAN).into(),
        },
        Variable::Array(items) => {
            if simplify {
                if let Some(kind) = common_kind(items) {
                    return Ok(simplify_array(items, kind));
                }
            }
            let values = items
                .iter()
                .map(|i| variable_to_r(i, simplify))
                .collect::<Result<Vec<_>>>()?;
            List::from_values(values).into()
        }
        Variable::Object(map) => {
            let values = map
                .values()
                .map(|i| variable_to_r(i, simplify))
                .collect::<Result<Vec<_>>>()?;
            List::from_names_and_values(map.keys(), values)?.into()
        }
        Variable::Expref(_) => return Err(Error::Other("expression references cannot be returned".into())),
    })
}
