(module
  (import "ethereum" "getCodeSize" (func $getCodeSize (result i32)))
  (import "ethereum" "codeCopy" (func $codeCopy (param i32 i32 i32)))
  (import "ethereum" "useGas" (func $useGas (param i64)))
  (import "ethereum" "finish" (func $finish (param i32 i32)))
  (memory 1)
  (export "memory" (memory 0))
  (export "main" (func $main))
  (func $main
    ;; load total code size
    (local $size i32)
    (local $payload_offset i32)
    (local $payload_size i32)
    (set_local $size (call $getCodeSize))

    ;; copy entire thing into memory at offset 0
    (call $codeCopy (i32.const 0) (i32.const 0) (get_local $size))

    ;; retrieve payload size (the last 4 bytes treated as a little endian 32 bit number)
    (set_local $payload_size (i32.load (i32.sub (get_local $size) (i32.const 4))))

    ;; start offset is calculated as $size - 4 - $payload_size
    (set_local $payload_offset (i32.sub (i32.sub (get_local $size) (i32.const 4)) (get_local $payload_size)))

    ;; return the payload
    (call $finish (get_local $payload_offset) (get_local $payload_size))
  )
)
