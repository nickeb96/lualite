# GCD Example

This example uses a lualite script to find the greatest common denominator of 250 and 135.
It also prints out the source code of the function, then the bytecode and
metadata once it's compiled.

Run with:
```shell
cargo run --example greatest_common_denominator
```

The output will be:

```text
function gcd(a, b)
  while a != b do
    if a > b then
      a = a - b
    else
      b = b - a
    end
  end
  return a
end

"gcd":
registers: 4
arg count: 2
constant table: (empty)
function table: (empty)
bytecode:
     0  nop
     1  ne    R3 = R1 != R2
     2  jmp   ip 8         if !R3
     3  gt    R3 = R1 > R2
     4  jmp   ip 6         if !R3
     5  sub   R1 = R1 - R2
     6  jmp   ip 7       
     7  sub   R2 = R2 - R1
     8  jmp   ip 0       
     9  mov   R0 = R1
    10  ret

running gcd with 250 and 135
result: 5
```

