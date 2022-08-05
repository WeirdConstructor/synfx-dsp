#[macro_export]
macro_rules! assert_vec_feq {
    ($vec:expr, $cmp_vec:expr) => {
        let cmp_vec = $cmp_vec;
        let res: Vec<f32> = $vec.iter().copied().collect();

        for (i, (s, scmp)) in res.iter().zip(cmp_vec.iter()).enumerate() {
            if (s - scmp).abs() > 0.0001 {
                panic!(
                    r#"
table_left: {:?}

table_right: {:?}

assertion failed: `(left[{}] == right[{}])`
      left: `{:?}`,
     right: `{:?}`"#,
                    &res[i..],
                    &(cmp_vec[i..]),
                    i,
                    i,
                    s,
                    scmp
                )
            }
        }
    };
}
