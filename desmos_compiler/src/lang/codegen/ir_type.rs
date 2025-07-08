use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, bail, Result};

use crate::expressions::Expressions;
use parse::ast::{
    BinaryOperator, ChainedComparison, Expression, ExpressionListEntry, UnaryOperator,
};

use super::ir::{IRScalarType, IRType};

pub fn expr_ty(
    expr: &Expression,
    env: &Expressions,
    params: &HashMap<String, IRType>,
) -> anyhow::Result<IRType> {
    use IRScalarType::*;
    use IRType::*;

    match expr {
        Expression::Number(_) => Ok(Scalar(Number)),
        Expression::List(elements) => {
            // empty  →  default to Number list
            let first = elements
                .first()
                .map(|n| expr_ty(n, env, params))
                .transpose()?
                .unwrap_or(Scalar(Number));

            // assert homogeneity
            for n in elements {
                anyhow::ensure!(
                    expr_ty(n, env, params)? == first,
                    "mixed element types in list"
                );
            }
            match first {
                Scalar(st) => Ok(List(st)),
                List(_) => bail!("nested lists are not supported"),
            }
        }

        Expression::Identifier(name) => ty_ident(env, params, name),

        Expression::UnaryOperation { operation, arg } => {
            ty_unary(*operation, expr_ty(arg, env, params)?)
        }
        Expression::BinaryOperation {
            operation,
            left,
            right,
        } => {
            let lt = expr_ty(left, env, params)?;
            let rt = expr_ty(right, env, params)?;
            ty_binary(lt, *operation, rt)
        }
        Expression::CallOrMultiply { callee, args } => ty_call(env, params, callee, args),

        Expression::For { body, lists } => {
            let mut scoped_params = params.clone();

            for (name, expr) in lists {
                let ty = expr_ty(expr, env, &scoped_params)?;
                match ty {
                    IRType::List(scalar_type) => {
                        scoped_params.insert(name.clone(), IRType::Scalar(scalar_type));
                    }
                    IRType::Scalar(_) => bail!("expected List, found Scalar"),
                }
            }

            // Type of the body inside the loop (with new bindings)
            let body_ty = expr_ty(body, env, &scoped_params)?;

            match body_ty {
                Scalar(s) => Ok(List(s)),
                List(_) => bail!("Expected scalar, found list"),
            }
        }

        Expression::Call { callee, args } => todo!(),
        Expression::ChainedComparison(chained_comparison) => todo!(),
        Expression::Piecewise {
            test,
            consequent,
            alternate,
        } => todo!(),
        Expression::SumProd {
            kind,
            variable,
            lower_bound,
            upper_bound,
            body,
        } => todo!(),
        Expression::With {
            body,
            substitutions,
        } => todo!(),

        Expression::ListRange {
            before_ellipsis,
            after_ellipsis,
        } => todo!(),
    }
}

fn ty_binary(lhs: IRType, op: BinaryOperator, rhs: IRType) -> Result<IRType> {
    use IRType::*;

    match (lhs, op, rhs) {
        (Scalar(lhs), op, Scalar(rhs)) => Ok(Scalar(ty_scalar_binary(lhs, op, rhs)?)),

        (List(lhs), BinaryOperator::Index, Scalar(IRScalarType::Number)) => Ok(Scalar(lhs)),
        (List(lhs), op, Scalar(rhs))
        | (Scalar(lhs), op, List(rhs))
        | (List(lhs), op, List(rhs)) => Ok(List(ty_scalar_binary(lhs, op, rhs)?)),
    }
}

fn ty_scalar_binary(
    lhs: IRScalarType,
    op: BinaryOperator,
    rhs: IRScalarType,
) -> Result<IRScalarType> {
    use BinaryOperator::*;
    use IRScalarType::*;
    match (lhs, op, rhs) {
        /* ───────────────────  number  ⨯  number  ─────────────────── */
        (Number, Add | Sub | Dot | Mul | Div | Pow, Number) => Ok(Number),

        /* ───────────────────  point   ⨯  point   ─────────────────── */
        (IRScalarType::Point, Add | Sub, IRScalarType::Point) => Ok(IRScalarType::Point),

        /* ────────────────  number ⨯ point / point ⨯ number  ──────────────── */
        (Number, Dot | Mul, IRScalarType::Point) | (IRScalarType::Point, Dot | Mul, Number) => {
            Ok(IRScalarType::Point)
        }

        /* ─────────────────────  point  ÷  number  ───────────────────── */
        (IRScalarType::Point, Div, Number) => Ok(IRScalarType::Point),

        (Number, BinaryOperator::Point, Number) => Ok(IRScalarType::Point),

        _ => bail!("type error: {op:?} is not defined for {lhs:?} and {rhs:?}"),
    }
}

fn ty_unary(op: UnaryOperator, inner: IRType) -> Result<IRType> {
    use IRScalarType::*;
    use IRType::*;
    use UnaryOperator::*;

    match (op, inner) {
        // number → number   (all unary ops on scalars allowed)
        (_, Scalar(Number)) => Ok(Scalar(Number)),

        // point → point  (only Neg is defined in code‑gen)
        (Neg, Scalar(Point)) => Ok(Scalar(Point)),
        (PointX, Scalar(Point)) => Ok(Scalar(Number)),
        (PointY, Scalar(Point)) => Ok(Scalar(Number)),

        // everything else is invalid
        _ => bail!("unary op {op:?} is not defined for {inner:?}"),
    }
}

fn ty_ident(
    env: &Expressions,
    params: &HashMap<String, IRType>,
    name: &str,
) -> anyhow::Result<IRType> {
    if let Some(ty) = params.get(name).cloned() {
        Ok(ty)
    } else {
        match env
            .get_expr(name)
            .ok_or_else(|| anyhow!("unknown ident {name}"))?
        {
            ExpressionListEntry::Assignment { value, .. } => expr_ty(&value, env, params),
            ExpressionListEntry::FunctionDeclaration { .. } => {
                bail!("{name} is a function, not a value")
            }
            _ => bail!("{name} is not an ident, this indicates a bug"),
        }
    }
}

fn ty_call(
    env: &Expressions,
    params: &HashMap<String, IRType>,
    ident: &str,
    args: &[Expression],
) -> anyhow::Result<IRType> {
    let ExpressionListEntry::FunctionDeclaration {
        body, parameters, ..
    } = env
        .get_expr(ident)
        .ok_or_else(|| anyhow!("unknown function {ident}"))?
    else {
        bail!("{ident} is not a function")
    };

    anyhow::ensure!(
        args.len() == parameters.len(),
        "parity mismatch calling {ident}"
    );

    // compute argument types
    let arg_tys: HashMap<String, IRType> = args
        .iter()
        .zip(parameters.iter())
        .map(|(expr, key)| Ok((key.to_string(), expr_ty(expr, env, params)?)))
        .collect::<anyhow::Result<_>>()?;

    // Use the function body to get its return type,            <-- recursion!
    expr_ty(&body, env, &arg_tys)
}

pub fn used_functions(
    expr: &Expression,
    env: &Expressions,
    param_types: &HashMap<String, IRType>,
) -> Result<HashSet<(String, Vec<IRType>)>> {
    let mut fns = HashSet::new();
    collect_functions(expr, env, param_types, &mut fns)?;
    Ok(fns)
}

fn collect_functions(
    expr: &Expression,
    env: &Expressions,
    param_types: &HashMap<String, IRType>,
    fns: &mut HashSet<(String, Vec<IRType>)>,
) -> Result<()> {
    match expr {
        Expression::List(expressions) => {
            for n in expressions {
                collect_functions(n, env, param_types, fns)?;
            }
        }
        Expression::UnaryOperation { operation, arg } => {
            collect_functions(arg, env, param_types, fns)?;
        }
        Expression::BinaryOperation {
            operation,
            left,
            right,
        } => {
            collect_functions(left, env, param_types, fns)?;
            collect_functions(right, env, param_types, fns)?;
        }
        Expression::CallOrMultiply { callee, args } => {
            match env.get_expr(callee.as_str()) {
                Some(ExpressionListEntry::FunctionDeclaration { .. }) => {
                    fns.insert((
                        callee.clone(),
                        args.iter()
                            .map(|arg| expr_ty(arg, env, param_types))
                            .collect::<Result<Vec<_>, _>>()?,
                    ));
                }
                Some(ExpressionListEntry::Assignment { .. }) => {
                    if args.len() != 1 {
                        bail!(
                            "{callee} is a variable and cannot be called with {} arguments",
                            args.len()
                        );
                    }
                }
                Some(_) => {
                    bail!("{callee} is not a function or variable");
                }
                None => {
                    bail!("Unknown identifier: {callee}");
                }
            }

            for arg in args {
                collect_functions(arg, env, param_types, fns)?;
            }
        }
        Expression::ChainedComparison(ChainedComparison { operands, .. }) => {
            for expr in operands {
                collect_functions(expr, env, param_types, fns)?;
            }
        }

        Expression::For { body, lists } => {
            // Create extended param_types with loop variables bound to their scalar types
            let mut scoped_params = param_types.clone();

            for (name, expr) in lists {
                collect_functions(expr, env, param_types, fns)?;

                let ty = expr_ty(expr, env, param_types)?;
                match ty {
                    IRType::List(scalar_ty) => {
                        scoped_params.insert(name.clone(), IRType::Scalar(scalar_ty));
                    }
                    IRType::Scalar(_) => {
                        bail!("expected List in For loop, found Scalar for variable '{name}'");
                    }
                }
            }

            // Recurse into body using the updated param scope
            collect_functions(body, env, &scoped_params, fns)?;
        }

        Expression::Piecewise {
            test,
            consequent,
            alternate,
        } => todo!(),
        Expression::SumProd {
            kind,
            variable,
            lower_bound,
            upper_bound,
            body,
        } => todo!(),
        Expression::With {
            body,
            substitutions,
        } => todo!(),
        Expression::Number(_) | Expression::Identifier(_) => {}

        Expression::ListRange {
            before_ellipsis,
            after_ellipsis,
        } => todo!(),

        Expression::Call { callee, args } => todo!(),
    }

    Ok(())
}
