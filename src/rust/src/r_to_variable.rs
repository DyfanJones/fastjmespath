use extendr_api::prelude::*;
use extendr_api::Result;
use jmespath::Variable;
use serde_json::Number;
use std::collections::BTreeMap;
use std::rc::Rc;

type Rcvar = Rc<Variable>;

fn null() -> Rcvar {
    Rc::new(Variable::Null)
}

fn float(x: f64) -> Rcvar {
    // NA, NaN and +/-Inf have no JSON representation
    match Number::from_f64(x) {
        Some(n) if !x.is_nan() => Rc::new(Variable::Number(n)),
        _ => null(),
    }
}

fn is_factor(obj: &Robj) -> bool {
    obj.inherits("factor")
}

/// Convert an atomic vector into one `Variable` per element (no unboxing).
fn atomic_elements(obj: &Robj) -> Result<Vec<Rcvar>> {
    if is_factor(obj) {
        let levels: Vec<String> = obj
            .get_attrib("levels")
            .and_then(|l| Strings::try_from(l).ok())
            .map(|s| s.iter().map(|x| x.as_ref().to_string()).collect())
            .unwrap_or_default();
        let codes = obj.as_integer_slice().unwrap_or(&[]);
        return Ok(codes
            .iter()
            .map(|&c| {
                if c == i32::MIN {
                    null()
                } else {
                    Rc::new(Variable::String(levels[(c - 1) as usize].clone()))
                }
            })
            .collect());
    }
    match obj.rtype() {
        Rtype::Logicals => Ok(obj
            .as_logical_slice()
            .unwrap_or(&[])
            .iter()
            .map(|b| {
                if b.is_na() {
                    null()
                } else {
                    Rc::new(Variable::Bool(b.is_true()))
                }
            })
            .collect()),
        Rtype::Integers => Ok(obj
            .as_integer_slice()
            .unwrap_or(&[])
            .iter()
            .map(|&i| {
                if i == i32::MIN {
                    null()
                } else {
                    Rc::new(Variable::Number(Number::from(i)))
                }
            })
            .collect()),
        Rtype::Doubles => Ok(obj
            .as_real_slice()
            .unwrap_or(&[])
            .iter()
            .map(|&x| float(x))
            .collect()),
        Rtype::Strings => {
            let s = Strings::try_from(obj.clone())?;
            Ok(s.iter()
                .map(|x| {
                    if x.is_na() {
                        null()
                    } else {
                        Rc::new(Variable::String(x.as_ref().to_string()))
                    }
                })
                .collect())
        }
        t => Err(Error::Other(format!("unsupported R type: {:?}", t))),
    }
}

fn data_frame(list: &List) -> Result<Rcvar> {
    let names: Vec<String> = list.names().map(|n| n.map(String::from).collect()).unwrap_or_default();
    let columns: Vec<Vec<Rcvar>> = list
        .values()
        .map(|col| column_elements(&col))
        .collect::<Result<_>>()?;
    let nrow = columns.first().map_or(0, |c| c.len());
    let rows = (0..nrow)
        .map(|i| {
            let row: BTreeMap<String, Rcvar> = names
                .iter()
                .zip(&columns)
                .map(|(n, c)| (n.clone(), c[i].clone()))
                .collect();
            Rc::new(Variable::Object(row))
        })
        .collect();
    Ok(Rc::new(Variable::Array(rows)))
}

/// One `Variable` per row of a data frame column (atomic or list column).
fn column_elements(col: &Robj) -> Result<Vec<Rcvar>> {
    match col.rtype() {
        Rtype::List => List::try_from(col.clone())?
            .values()
            .map(|v| r_to_variable(&v))
            .collect(),
        _ => atomic_elements(col),
    }
}

/// Convert an R object to a JMESPath `Variable`.
///
/// Length-1 atomic vectors unbox to scalars, longer ones become arrays.
/// Named lists become objects, unnamed lists arrays, data frames arrays of
/// row objects. `NA`, `NaN` and `Inf` become `null`.
pub fn r_to_variable(obj: &Robj) -> Result<Rcvar> {
    match obj.rtype() {
        Rtype::Null => Ok(null()),
        Rtype::List => {
            let list = List::try_from(obj.clone())?;
            if obj.inherits("data.frame") {
                return data_frame(&list);
            }
            match list.names() {
                Some(names) => {
                    let mut map = BTreeMap::new();
                    for (name, val) in names.zip(list.values()) {
                        map.insert(name.to_string(), r_to_variable(&val)?);
                    }
                    Ok(Rc::new(Variable::Object(map)))
                }
                None => {
                    let items = list
                        .values()
                        .map(|v| r_to_variable(&v))
                        .collect::<Result<Vec<_>>>()?;
                    Ok(Rc::new(Variable::Array(items)))
                }
            }
        }
        _ => {
            let mut items = atomic_elements(obj)?;
            if items.len() == 1 {
                Ok(items.pop().unwrap())
            } else {
                Ok(Rc::new(Variable::Array(items)))
            }
        }
    }
}
