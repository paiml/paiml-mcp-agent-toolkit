(module
  ;; Import memory from host
  (import "env" "memory" (memory 1))

  ;; Simple add function
  (func $add (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.add
  )

  ;; Multiply function
  (func $multiply (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.mul
  )

  ;; Fibonacci function
  (func $fibonacci (param $n i32) (result i32)
    (local $result i32)

    local.get $n
    i32.const 1
    i32.le_s
    if
      local.get $n
      return
    end

    local.get $n
    i32.const 1
    i32.sub
    call $fibonacci

    local.get $n
    i32.const 2
    i32.sub
    call $fibonacci

    i32.add
  )

  ;; Export functions
  (export "add" (func $add))
  (export "multiply" (func $multiply))
  (export "fibonacci" (func $fibonacci))
)