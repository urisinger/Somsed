use cranelift::prelude::*;
use desmos_compiler::lang::codegen::ir::{IRScalarType, IRType};

#[derive(Debug, Clone, Copy)]
pub enum CraneliftValue {
    Scalar(CraneliftScalar),
    List(CraneliftList),
}

impl CraneliftValue {
    pub fn number(value: Value) -> Self {
        Self::Scalar(CraneliftScalar::Number([value]))
    }

    pub fn point(value: [Value; 2]) -> Self {
        Self::Scalar(CraneliftScalar::Point(value))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CraneliftScalar {
    Number([Value; 1]),
    Point([Value; 2]),
}

#[derive(Debug, Clone, Copy)]
pub enum CraneliftList {
    Number([Value; 2]),
    Point([Value; 2]),
}

impl CraneliftList {
    pub fn element_type_and_size(&self) -> (IRScalarType, usize) {
        match self {
            CraneliftList::Number(_) => (IRScalarType::Number, 8),
            CraneliftList::Point(_) => (IRScalarType::Point, 16),
        }
    }

    pub fn values(&self) -> [Value; 2] {
        match self {
            CraneliftList::Number(vals) => *vals,
            CraneliftList::Point(vals) => *vals,
        }
    }
}

impl CraneliftValue {
    /// Construct from flat Cranelift values and expected IRType
    pub fn from_values(values: &[Value], ty: IRType) -> Option<Self> {
        Some(match ty {
            IRType::Scalar(IRScalarType::Number) => {
                CraneliftValue::Scalar(CraneliftScalar::Number(*as_array(values)?))
            }
            IRType::Scalar(IRScalarType::Point) => {
                CraneliftValue::Scalar(CraneliftScalar::Point(*as_array(values)?))
            }
            IRType::List(IRScalarType::Number) => {
                CraneliftValue::List(CraneliftList::Number(*as_array(values)?))
            }
            IRType::List(IRScalarType::Point) => {
                CraneliftValue::List(CraneliftList::Point(*as_array(values)?))
            }
        })
    }

    /// Get flat Cranelift values for passing to other instructions
    pub fn as_struct(&self) -> &[Value] {
        match self {
            CraneliftValue::Scalar(CraneliftScalar::Number(v)) => v,
            CraneliftValue::Scalar(CraneliftScalar::Point(v)) => v,
            CraneliftValue::List(CraneliftList::Number(v)) => v,
            CraneliftValue::List(CraneliftList::Point(v)) => v,
        }
    }

    pub fn ty(&self) -> IRType {
        match self {
            CraneliftValue::Scalar(CraneliftScalar::Number(_)) => {
                IRType::Scalar(IRScalarType::Number)
            }
            CraneliftValue::Scalar(CraneliftScalar::Point(_)) => {
                IRType::Scalar(IRScalarType::Point)
            }
            CraneliftValue::List(CraneliftList::Number(_)) => IRType::List(IRScalarType::Number),
            CraneliftValue::List(CraneliftList::Point(_)) => IRType::List(IRScalarType::Point),
        }
    }
}

/// Number of Cranelift `Value`s required to represent an `IRType`
pub fn value_count(ty: IRType) -> usize {
    match ty {
        IRType::Scalar(IRScalarType::Number) => 1,
        IRType::Scalar(IRScalarType::Point) => 2,
        IRType::List(_) => 2,
    }
}

const fn as_array<T, const N: usize>(arr: &[T]) -> Option<&[T; N]> {
    if arr.len() == N {
        let ptr = arr.as_ptr() as *const [T; N];
        Some(unsafe { &*ptr })
    } else {
        None
    }
}
