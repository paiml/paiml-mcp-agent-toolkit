fn very_high_function(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            if x > 20 {
                if x > 30 {
                    if x > 40 {
                        x * 5
                    } else {
                        x * 4
                    }
                } else {
                    x * 3
                }
            } else {
                x * 2
            }
        } else if x > 5 {
            if x > 7 {
                x + 20
            } else {
                x + 10
            }
        } else {
            x + 1
        }
    } else if x < 0 {
        if x < -10 {
            if x < -20 {
                if x < -30 {
                    -x * 4
                } else {
                    -x * 3
                }
            } else {
                -x * 2
            }
        } else {
            -x
        }
    } else {
        match x {
            0 => 0,
            _ => unreachable\!(),
        }
    }
}
EOF < /dev/null