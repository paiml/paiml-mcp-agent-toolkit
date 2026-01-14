// Complex Rust fixture with nested scopes for DAP integration testing
fn outer_function(param1: i32, param2: &str) -> i32 {
    let outer_var = 100;
    let message = format!("Processing {}", param2);

    {
        let inner_var = 200;
        let combined = outer_var + inner_var + param1;

        {
            let deeply_nested = 300;  // Line 10: Test breakpoint here
            let total = combined + deeply_nested;
            println!("Total: {}", total);
        }
    }

    let result = outer_var + param1;
    result
}

fn with_shadowing() {
    let x = 10;
    println!("First x: {}", x);

    let x = "shadowed";  // Line 24: Variable shadowing test
    println!("Second x: {}", x);

    {
        let x = 3.14;
        println!("Third x: {}", x);
    }
}

fn main() {
    let main_var = 42;
    let name = "test";

    let _result = outer_function(main_var, name);
    with_shadowing();
}
