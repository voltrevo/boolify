use std::{collections::HashSet, rc::Rc};

use bristol_circuit::BristolCircuit;

use crate::{generate_bristol, BoolWire, CircuitOutput, IdGenerator, ValueWire};

pub fn boolify(arith_circuit: &BristolCircuit, bit_width: usize) -> BristolCircuit {
    if arith_circuit.io_widths.is_some() {
        panic!("Arithmetic circuit should not have io widths");
    }

    let id_gen = IdGenerator::new_rc_refcell();
    let mut wires: Vec<Option<ValueWire>> = vec![None; arith_circuit.wire_count];

    for (name, i) in &arith_circuit.info.input_name_to_wire_index {
        wires[*i] = Some(ValueWire::new_input(name, bit_width, &id_gen));
    }

    for (_, const_info) in &arith_circuit.info.constants {
        wires[const_info.wire_index] = Some(
            ValueWire::new_const(const_info.value.parse().unwrap(), &id_gen).resize(bit_width),
        );
    }

    let unary_ops = ["AUnaryAdd", "AUnarySub", "ANot", "ABitNot"]
        .iter()
        .map(|s| s.to_string())
        .collect::<HashSet<_>>();

    let binary_ops = [
        "AAdd", "ASub", "AMul", "ADiv", "AMod", "AExp", "AEq", "ANeq", "AEq", "ANeq", "ABoolAnd",
        "ABoolOr", "ALt", "ALEq", "AGt", "AGEq", "ABitAnd", "ABitOr", "AXor", "AShiftL", "AShiftR",
        "AShiftR",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect::<HashSet<_>>();

    let to_value = |b: &Rc<BoolWire>| BoolWire::as_value(b).resize(bit_width);

    for gate in &arith_circuit.gates {
        if unary_ops.contains(&gate.op) {
            assert_eq!(gate.inputs.len(), 1);
            assert_eq!(gate.outputs.len(), 1);

            let in_ = wires[gate.inputs[0]]
                .as_ref()
                .expect("Required wire not assigned");

            let out_id = gate.outputs[0];

            wires[out_id] = Some(match gate.op.as_str() {
                "AUnaryAdd" => in_.clone(),
                "AUnarySub" => ValueWire::negate(in_),
                "ANot" => to_value(&BoolWire::not(&in_.to_bool())),
                "ABitNot" => ValueWire::bit_not(in_),
                _ => unreachable!(),
            });
        } else if binary_ops.contains(&gate.op) {
            assert_eq!(gate.inputs.len(), 2);
            assert_eq!(gate.outputs.len(), 1);

            let a = wires[gate.inputs[0]]
                .as_ref()
                .expect("Required wire not assigned");

            let b = wires[gate.inputs[1]]
                .as_ref()
                .expect("Required wire not assigned");

            let out_id = gate.outputs[0];

            wires[out_id] = Some(match gate.op.as_str() {
                "AAdd" => ValueWire::add(a, b),
                "ASub" => ValueWire::sub(a, b),
                "AMul" => ValueWire::mul(a, b),
                "ADiv" => ValueWire::div(a, b),
                "AMod" => ValueWire::mod_(a, b),
                "AExp" => panic!("Not implemented: exp"), // TODO
                "AEq" => to_value(&ValueWire::equal(a, b)),
                "ANeq" => to_value(&ValueWire::not_equal(a, b)),
                "ABoolAnd" => to_value(&ValueWire::bool_and(a, b)),
                "ABoolOr" => to_value(&ValueWire::bool_or(a, b)),
                "ALt" => to_value(&ValueWire::less_than(a, b)),
                "ALEq" => to_value(&ValueWire::less_than_or_eq(a, b)),
                "AGt" => to_value(&ValueWire::greater_than(a, b)),
                "AGEq" => to_value(&ValueWire::greater_than_or_eq(a, b)),
                "ABitAnd" => ValueWire::bit_and(a, b),
                "ABitOr" => ValueWire::bit_or(a, b),
                "AXor" => ValueWire::bit_xor(a, b),
                "AShiftL" => panic!("Not implemented: shl"), // TODO
                "AShiftR" => panic!("Not implemented: shr"), // TODO
                _ => unreachable!(),
            });
        } else {
            panic!("Unrecognized op: {}", &gate.op)
        }
    }

    let mut outputs = Vec::<CircuitOutput>::new();

    for (name, i) in &arith_circuit.info.output_name_to_wire_index {
        outputs.push(CircuitOutput {
            name: name.clone(),
            value: wires[*i].clone().expect("Required wire not assigned"),
        });
    }

    let circuit = generate_bristol(&outputs);

    circuit
}
