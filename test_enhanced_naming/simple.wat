(module
  ;; Import a function from the host environment
  (import "host" "log" (func $log (param i32)))

  ;; Define memory
  (memory $memory 1)

  ;; Export memory to host
  (export "memory" (memory $memory))

  ;; Define a simple add function
  (func $add (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.add
  )

  ;; Export the add function
  (export "add" (func $add))

  ;; Define a factorial function
  (func $factorial (param $n i32) (result i32)
    (local $result i32)
    (local $counter i32)

    ;; Initialize result to 1
    i32.const 1
    local.set $result

    ;; Initialize counter to 1
    i32.const 1
    local.set $counter

    ;; Loop while counter <= n
    (loop $loop
      ;; Check if counter > n
      local.get $counter
      local.get $n
      i32.gt_s
      if
        ;; Exit loop
        br $loop
      end

      ;; result = result * counter
      local.get $result
      local.get $counter
      i32.mul
      local.set $result

      ;; counter++
      local.get $counter
      i32.const 1
      i32.add
      local.set $counter

      ;; Continue loop
      br $loop
    )

    ;; Return result
    local.get $result
  )

  ;; Export factorial function
  (export "factorial" (func $factorial))
)