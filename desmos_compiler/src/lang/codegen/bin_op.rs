use anyhow::{bail, Result};

use crate::lang::codegen::ir::{IRScalarType, IRType};
use parse::ast::BinaryOperator;

use super::{
    ir::{BlockID, IRSegment, InstID, Instruction},
    IRGen,
};

impl IRGen<'_> {
    pub(crate) fn codegen_binary_op(
        segment: &mut IRSegment,
        current_block: BlockID,
        lhs: InstID,
        op: BinaryOperator,
        rhs: InstID,
    ) -> Result<InstID> {
        use IRScalarType::*;
        use IRType::*;

        match (lhs.ty(), rhs.ty()) {
            (Scalar(Number), Scalar(Number)) => {
                Self::codegen_binary_number_op(segment, current_block, lhs, op, rhs)
            }

            (Scalar(Point), Scalar(Point)) => {
                Self::codegen_binary_point_op(segment, current_block, lhs, op, rhs)
            }

            (Scalar(Number), Scalar(Point)) => {
                Self::codegen_number_point_op(segment, current_block, lhs, op, rhs)
            }

            (Scalar(Point), Scalar(Number)) => {
                Self::codegen_point_number_op(segment, current_block, lhs, op, rhs)
            }
            (List(list_t), Scalar(Number)) if op == BinaryOperator::Index => {
                Ok(segment.push(current_block, Instruction::Index(lhs, rhs), Scalar(list_t)))
            }

            // Scalar-List or List-Scalar
            (List(_), Scalar(_)) | (Scalar(_), List(_)) | (List(_), List(_)) => {
                Self::codegen_distributed_op(segment, current_block, lhs, op, rhs)
            }
        }
    }

    fn codegen_distributed_op(
        segment: &mut IRSegment,
        current_block: BlockID,
        lhs: InstID,
        op: BinaryOperator,
        rhs: InstID,
    ) -> Result<InstID> {
        use IRType::*;
        let inner_block = segment.create_block();

        Ok(match (lhs.ty(), rhs.ty()) {
            (List(lhs_ty), Scalar(rhs_ty)) => {
                // List args come first
                let lhs_inst = segment.push(
                    inner_block,
                    Instruction::BlockArg {
                        index: 0,
                        block: inner_block,
                    },
                    Scalar(lhs_ty),
                );

                let rhs_inst = segment.push(
                    inner_block,
                    Instruction::BlockArg {
                        index: 1,
                        block: inner_block,
                    },
                    Scalar(rhs_ty),
                );
                let ty =
                    Self::codegen_binary_op(segment, inner_block, lhs_inst, op, rhs_inst)?.ty();

                let ty = match ty {
                    Scalar(scalar) => List(scalar),
                    List(_) => bail!("Block incorrectly returns a list, this indicates a bug"),
                };
                //TODO: insert a with block here
                segment.push(
                    current_block,
                    Instruction::Map {
                        lists: vec![vec![lhs]],
                        block_id: inner_block,
                    },
                    ty,
                )
            }
            (Scalar(lhs_ty), List(rhs_ty)) => {
                let lhs_inst = segment.push(
                    inner_block,
                    Instruction::BlockArg {
                        index: 1,
                        block: inner_block,
                    },
                    Scalar(lhs_ty),
                );
                // List args come first
                let rhs_inst = segment.push(
                    inner_block,
                    Instruction::BlockArg {
                        index: 0,
                        block: inner_block,
                    },
                    Scalar(rhs_ty),
                );
                let ty =
                    Self::codegen_binary_op(segment, inner_block, lhs_inst, op, rhs_inst)?.ty();
                let ty = match ty {
                    Scalar(scalar) => List(scalar),
                    List(_) => bail!("Block incorrectly returns a list, this indicates a bug"),
                };

                //TODO: insert a with block here
                segment.push(
                    current_block,
                    Instruction::Map {
                        lists: vec![vec![rhs]],
                        block_id: inner_block,
                    },
                    ty,
                )
            }
            (List(lhs_ty), List(rhs_ty)) => {
                // List args come first
                let lhs_inst = segment.push(
                    inner_block,
                    Instruction::BlockArg {
                        index: 0,
                        block: inner_block,
                    },
                    Scalar(lhs_ty),
                );

                let rhs_inst = segment.push(
                    inner_block,
                    Instruction::BlockArg {
                        index: 1,
                        block: inner_block,
                    },
                    Scalar(rhs_ty),
                );
                let ty =
                    Self::codegen_binary_op(segment, inner_block, lhs_inst, op, rhs_inst)?.ty();

                let ty = match ty {
                    Scalar(scalar) => List(scalar),
                    List(_) => bail!("Block incorrectly returns a list, this indicates a bug"),
                };

                segment.push(
                    current_block,
                    Instruction::Map {
                        lists: vec![vec![lhs, rhs]],
                        block_id: inner_block,
                    },
                    ty,
                )
            }

            (Scalar(_), Scalar(_)) => bail!("cant distribute op on scalars"),
        })
    }

    fn codegen_binary_number_op(
        segment: &mut IRSegment,
        current_block: BlockID,
        lhs: InstID,
        op: BinaryOperator,
        rhs: InstID,
    ) -> Result<InstID> {
        Ok(segment.push(
            current_block,
            match op {
                BinaryOperator::Add => Instruction::Add,
                BinaryOperator::Sub => Instruction::Sub,
                BinaryOperator::Dot => Instruction::Mul,
                BinaryOperator::Mul => Instruction::Mul,
                BinaryOperator::Div => Instruction::Div,
                BinaryOperator::Pow => Instruction::Pow,
                BinaryOperator::Point => Instruction::Point,
                _ => todo!(),
            }(lhs, rhs),
            IRType::NUMBER,
        ))
    }

    fn codegen_binary_point_op(
        segment: &mut IRSegment,
        current_block: BlockID,
        lhs: InstID,
        op: BinaryOperator,
        rhs: InstID,
    ) -> Result<InstID> {
        use BinaryOperator::*;
        match op {
            Add | Sub => {
                let mut extract = |inst: InstID, index: usize| {
                    segment.push(
                        current_block,
                        Instruction::Extract(inst, index),
                        IRType::NUMBER,
                    )
                };

                let (lx, ly) = (extract(lhs, 0), extract(lhs, 1));
                let (rx, ry) = (extract(rhs, 0), extract(rhs, 1));

                let op_x = Self::codegen_binary_number_op(segment, current_block, lx, op, rx)?;
                let op_y = Self::codegen_binary_number_op(segment, current_block, ly, op, ry)?;

                Ok(segment.push(current_block, Instruction::Point(op_x, op_y), IRType::POINT))
            }
            _ => bail!("BinaryOp {:?} is not supported for Point <-> Point", op),
        }
    }

    fn codegen_number_point_op(
        segment: &mut IRSegment,
        current_block: BlockID,
        number: InstID,
        op: BinaryOperator,
        point: InstID,
    ) -> Result<InstID> {
        use BinaryOperator::*;
        match op {
            Dot | Mul => {
                let mut extract = |inst: InstID, index: usize| {
                    segment.push(
                        current_block,
                        Instruction::Extract(inst, index),
                        IRType::NUMBER,
                    )
                };

                let px = extract(point, 0);
                let py = extract(point, 1);

                let op_x = Self::codegen_binary_number_op(segment, current_block, number, op, px)?;
                let op_y = Self::codegen_binary_number_op(segment, current_block, number, op, py)?;

                Ok(segment.push(current_block, Instruction::Point(op_x, op_y), IRType::POINT))
            }
            _ => bail!("BinaryOp {:?} is not supported for Number <-> Point", op),
        }
    }

    fn codegen_point_number_op(
        segment: &mut IRSegment,
        current_block: BlockID,
        point: InstID,
        op: BinaryOperator,
        number: InstID,
    ) -> Result<InstID> {
        use BinaryOperator::*;
        match op {
            Dot | Mul | Div => {
                let mut extract = |inst: InstID, index: usize| {
                    segment.push(
                        current_block,
                        Instruction::Extract(inst, index),
                        IRType::NUMBER,
                    )
                };

                let px = extract(point, 0);
                let py = extract(point, 1);

                let op_x = Self::codegen_binary_number_op(segment, current_block, px, op, number)?;
                let op_y = Self::codegen_binary_number_op(segment, current_block, py, op, number)?;

                Ok(segment.push(current_block, Instruction::Point(op_x, op_y), IRType::POINT))
            }
            _ => bail!("BinaryOp {:?} is not supported for Point <-> Number", op),
        }
    }
}
