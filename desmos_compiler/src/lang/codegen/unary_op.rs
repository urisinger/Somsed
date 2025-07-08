use crate::lang::codegen::IRSegment;
use anyhow::{bail, Result};
use parse::ast::UnaryOperator;

use super::{
    ir::{BlockID, IRScalarType, IRType, InstID, Instruction},
    IRGen,
};

impl IRGen<'_> {
    pub(crate) fn codegen_unary_op(
        segment: &mut IRSegment,
        current_block: BlockID,
        lhs: InstID,
        op: UnaryOperator,
    ) -> Result<InstID> {
        Ok(match lhs.ty() {
            IRType::Scalar(IRScalarType::Number) => {
                Self::codegen_unary_number_op(segment, current_block, lhs, op)?
            }
            IRType::Scalar(IRScalarType::Point) => match op {
                UnaryOperator::Neg => {
                    let x =
                        segment.push(current_block, Instruction::Extract(lhs, 0), IRType::NUMBER);
                    let x = segment.push(current_block, Instruction::Neg(x), IRType::NUMBER);
                    let y =
                        segment.push(current_block, Instruction::Extract(lhs, 1), IRType::NUMBER);
                    let y = segment.push(current_block, Instruction::Neg(y), IRType::NUMBER);

                    segment.push(current_block, Instruction::Point(x, y), IRType::POINT)
                }
                UnaryOperator::PointX => {
                    segment.push(current_block, Instruction::Extract(lhs, 0), IRType::NUMBER)
                }

                UnaryOperator::PointY => {
                    segment.push(current_block, Instruction::Extract(lhs, 1), IRType::NUMBER)
                }

                _ => bail!("Unary op {:?} cant be applied to point", op),
            },
            IRType::List(_) => bail!("Unary operations are not defined for lists"),
        })
    }

    pub(crate) fn codegen_unary_number_op(
        segment: &mut IRSegment,
        current_block: BlockID,
        lhs: InstID,
        op: UnaryOperator,
    ) -> Result<InstID> {
        Ok(segment.push(
            current_block,
            match op {
                UnaryOperator::Neg => Instruction::Neg,
                UnaryOperator::Sqrt => Instruction::Sqrt,
                _ => todo!(),
            }(lhs),
            IRType::NUMBER,
        ))
    }
}
