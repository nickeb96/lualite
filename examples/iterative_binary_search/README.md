# Iterative Binary Search Example

This example creates a sorted array of integers and uses a lualite script to search for
integers in the array.  The script returns the index if the key is found, or false otherwise.

Run with:
```shell
cargo run --example iterative_binary_search
```

The output will be:

```text
array: [1, 3, 4, 6, 8, 9, 10, 11, 14, 15]

search for 0, index is false
search for 1, index is 0
search for 2, index is false
search for 3, index is 1
search for 4, index is 2
search for 5, index is false
search for 6, index is 3
search for 7, index is false
search for 8, index is 4
search for 9, index is 5
search for 10, index is 6
search for 11, index is 7
search for 12, index is false
search for 13, index is false
search for 14, index is 8
search for 15, index is 9
search for 16, index is false
```

The compiled output of `binary_search` is:

```text
registers: 9
arg count: 3
constant table:
  &0: false
function table: (empty)
bytecode:
     0  nop
     1  mov   R4 = #0
     2  sub   R5 = R2 - #1
     3  le    R7 = R4 <= R5
     4  jmp   ip 19        if !R7
     5  add   R7 = R4 + R5
     6  div   R6 = R7 / #2
     7  idx   R8 = R1[R6]
     8  lt    R7 = R3 < R8
     9  jmp   ip 11        if !R7
    10  sub   R5 = R6 - #1
    11  jmp   ip 18
    12  idx   R8 = R1[R6]
    13  gt    R7 = R3 > R8
    14  jmp   ip 16        if !R7
    15  add   R4 = R6 + #1
    16  jmp   ip 18
    17  mov   R0 = R6
    18  ret
    19  jmp   ip 2
    20  mov   R0 = &0
    21  ret
```

