#[cfg(test)]
mod ag_tests {
    use kenga::autograd::*;
    use kenga::bytecode::Value;
    use kenga::tensor::tensor_from;

    #[test]
    fn matmul_mse_learns() {
        ag_clear();
        let w = tensor_from(
            Value::List(vec![Value::I64(1), Value::I64(1)]),
            Value::List(vec![Value::F64(0.0)]),
        )
        .unwrap();
        let x = tensor_from(
            Value::List(vec![Value::I64(1), Value::I64(1)]),
            Value::List(vec![Value::F64(2.0)]),
        )
        .unwrap();
        let y = tensor_from(
            Value::List(vec![Value::I64(1), Value::I64(1)]),
            Value::List(vec![Value::F64(6.0)]),
        )
        .unwrap();
        let mut wv = w;
        for _ in 0..40 {
            ag_clear();
            let wid = ag_param(wv.clone()).unwrap();
            let xid = ag_const(x.clone()).unwrap();
            let yid = ag_const(y.clone()).unwrap();
            let pred = ag_matmul(wid, xid).unwrap();
            let loss = ag_mse(pred, yid).unwrap();
            ag_backward(loss).unwrap();
            wv = ag_step(wid, 0.1).unwrap();
        }
        match wv {
            Value::Tensor { data, .. } => {
                assert!((data[0] - 3.0).abs() < 0.2, "got {}", data[0]);
            }
            _ => panic!("expected tensor"),
        }
    }

    #[test]
    fn transpose_neg_roundtrip_grad() {
        ag_clear();
        let a = tensor_from(
            Value::List(vec![Value::I64(2), Value::I64(1)]),
            Value::List(vec![Value::F64(1.0), Value::F64(2.0)]),
        )
        .unwrap();
        let id = ag_param(a).unwrap();
        let t = ag_transpose(id).unwrap();
        let n = ag_neg(t).unwrap();
        let loss = ag_sum(n).unwrap();
        ag_backward(loss).unwrap();
        let g = ag_grad(id).unwrap();
        match g {
            Value::Tensor { data, .. } => {
                assert!((data[0] + 1.0).abs() < 1e-9);
                assert!((data[1] + 1.0).abs() < 1e-9);
            }
            _ => panic!("expected tensor grad"),
        }
    }
}
