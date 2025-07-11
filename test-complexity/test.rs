fn simple() {
    println\!("simple");
}

fn complex() {
    if true {
        if false {
            if true {
                if false {
                    println\!("nested");
                }
            }
        }
    }
    
    match 5 {
        1 => println\!("1"),
        2 => println\!("2"),
        3 => println\!("3"),
        4 => println\!("4"),
        _ => println\!("other"),
    }
}

fn very_complex() {
    for i in 0..10 {
        for j in 0..10 {
            if i > 5 {
                if j < 5 {
                    match i + j {
                        10 => println\!("10"),
                        11 => println\!("11"),
                        12 => println\!("12"),
                        _ => println\!("other"),
                    }
                }
            }
        }
    }
}
EOF < /dev/null