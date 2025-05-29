Following on https://github.com/montyly/vyper_hash/tree/main (and https://blog.vyperlang.org/posts/selector-tables/)

100% of the rust code was vibe coded, so this requires further investigation :)

i
# EVM Function Dispatcher Optimization (Proof of Concept)

A proof of concept for an alternative Ethereum smart contract function dispatcher that uses mathematical operations instead of multiple comparisons. This is an experimental approach to function dispatching. :)

## How It Works

From a given set of function ids finds a simple _hashing_ function that maps each funcid into a different u8. The use the following ~600 bytes to hold little basic block that will finally jump into the final real function initial basic block. It uses 700 bytes for the dispatch code 😂. But then only 55? gas to reach the function.  

```
result_byte = ((selector * magic_number_q) >> shift_amount) & 0xFF
```

Then is uses that byte multiplied by 6 to jump to a basic block that will jump to the actuall basic block. This will fill all the blockchain.

### Bytecode Structure

1. **Dispatcher Code (78 bytes)**
   ```
   PUSH0
   CALLDATALOAD
   PUSH32 <magic_number_q>
   MUL
   PUSH32 <shift_amount>
   SHR
   PUSH1 0xFF
   AND
   PUSH1 0x06
   MUL
   JUMPDEST
   JUMP
   ```

2. **Function Dispatchers**
   - Each dispatcher is 6 bytes:
     ```
     JUMPDEST (1 byte)
     PUSH3 <function_address> (4 bytes)
     JUMP (1 byte)
     ```
   - Placed at `78 + (result_byte * 6)`

3. **Function Code Blocks**
   - Each function at its own address
   - Starts with JUMPDEST and ends with STOP

## Gas Costs

- Dispatcher Logic: 43 gas
- Function Jump: 12 gas
- **Total: 55 gas** from CALL to function execution

Note: This is a proof of concept and has not been thoroughly tested in production environments.
```
Generated function selectors and addresses:
 1: Selector: 0x6f4ff18f -> Address: 0x00f01000
 2: Selector: 0x5b4849aa -> Address: 0x00f02000
 3: Selector: 0x95ce1bb2 -> Address: 0x00f03000
 4: Selector: 0x64ae1f05 -> Address: 0x00f04000
 5: Selector: 0x05ef275b -> Address: 0x00f05000
 6: Selector: 0xbbe62f60 -> Address: 0x00f06000
 7: Selector: 0xc71b7cc8 -> Address: 0x00f07000
 8: Selector: 0xb71a3723 -> Address: 0x00f08000
 9: Selector: 0x685eddf5 -> Address: 0x00f09000
10: Selector: 0x900c4e83 -> Address: 0x00f0a000
11: Selector: 0xc6b145cc -> Address: 0x00f0b000
12: Selector: 0xa3053cf0 -> Address: 0x00f0c000
13: Selector: 0xd63782a0 -> Address: 0x00f0d000
14: Selector: 0x901401c7 -> Address: 0x00f0e000
15: Selector: 0xa28e7df6 -> Address: 0x00f0f000
16: Selector: 0x0bac62e9 -> Address: 0x00f10000
17: Selector: 0x4c566129 -> Address: 0x00f11000
18: Selector: 0x654b2207 -> Address: 0x00f12000
19: Selector: 0x4926ce1e -> Address: 0x00f13000
20: Selector: 0x3a153b0d -> Address: 0x00f14000
Found better solution after 0 attempts in 0.00 seconds (max byte: 0xf7)
Found better solution after 1 attempts in 0.00 seconds (max byte: 0xd4)
Found better solution after 151 attempts in 0.03 seconds (max byte: 0xce)
Found better solution after 222 attempts in 0.04 seconds (max byte: 0xc6)
Found better solution after 370 attempts in 0.09 seconds (max byte: 0xc5)
Found better solution after 847 attempts in 0.17 seconds (max byte: 0xba)
Found better solution after 933 attempts in 0.18 seconds (max byte: 0xb8)
Best solution found after 1024 attempts in 0.20 seconds (max byte: 0xb8)

Found magic numbers for EVM dispatch:
q (multiplier): 0xb56f598cb0ffdd45fe22433ace46d7e1a127b0d46a8695a05fe05649f7a71e19
shift: 40

Selector to Result Byte Mapping:
--------------------------------
Selector                Result Byte
--------------------------------
0x6f4ff18f              0xb8
0x5b4849aa              0x73
0x95ce1bb2              0x59
0x64ae1f05              0x3f
0x05ef275b              0x69
0xbbe62f60              0x83
0xc71b7cc8              0x44
0xb71a3723              0x4d
0x685eddf5              0x43
0x900c4e83              0x4e
0xc6b145cc              0x2e
0xa3053cf0              0x79
0xd63782a0              0x66
0x901401c7              0xb7
0xa28e7df6              0x28
0x0bac62e9              0x6c
0x4c566129              0xa7
0x654b2207              0xab
0x4926ce1e              0x0c
0x3a153b0d              0x9b
--------------------------------


EVM bytecode structure:
// Dispatcher code (78 bytes)
  0: PUSH0
  1: CALLDATALOAD
  2: PUSH32 0xb56f598cb0ffdd45fe22433ace46d7e1a127b0d46a8695a05fe05649f7a71e19 // magic number q
 35: MUL
 36: PUSH32 0x00000028 // shift amount
 69: SHR
 70: PUSH1 0xFF
 72: AND
 73: PUSH1 0x06
 75: MUL
 76: JUMPDEST
 77: JUMP

// NOP sled with function dispatchers (starts at byte 78)
// Each function entry point consists of:
// JUMPDEST (1 byte)
// PUSH3 <function_address> (4 bytes)
// JUMP (1 byte)
// Total: 6 bytes per function

// Start of NOP sled
// Gap from offset 78 to 150
150: JUMPDEST
151: PUSH3 0xf13000 // Function at 0x00f13000 (selector: 0x4926ce1e, result byte: 0x0c)
155: JUMP
// Gap from offset 156 to 318
318: JUMPDEST
319: PUSH3 0xf0f000 // Function at 0x00f0f000 (selector: 0xa28e7df6, result byte: 0x28)
323: JUMP
// Gap from offset 324 to 354
354: JUMPDEST
355: PUSH3 0xf0b000 // Function at 0x00f0b000 (selector: 0xc6b145cc, result byte: 0x2e)
359: JUMP
// Gap from offset 360 to 456
456: JUMPDEST
457: PUSH3 0xf04000 // Function at 0x00f04000 (selector: 0x64ae1f05, result byte: 0x3f)
461: JUMP
// Gap from offset 462 to 480
480: JUMPDEST
481: PUSH3 0xf09000 // Function at 0x00f09000 (selector: 0x685eddf5, result byte: 0x43)
485: JUMP
486: JUMPDEST
487: PUSH3 0xf07000 // Function at 0x00f07000 (selector: 0xc71b7cc8, result byte: 0x44)
491: JUMP
// Gap from offset 492 to 540
540: JUMPDEST
541: PUSH3 0xf08000 // Function at 0x00f08000 (selector: 0xb71a3723, result byte: 0x4d)
545: JUMP
546: JUMPDEST
547: PUSH3 0xf0a000 // Function at 0x00f0a000 (selector: 0x900c4e83, result byte: 0x4e)
551: JUMP
// Gap from offset 552 to 612
612: JUMPDEST
613: PUSH3 0xf03000 // Function at 0x00f03000 (selector: 0x95ce1bb2, result byte: 0x59)
617: JUMP
// Gap from offset 618 to 690
690: JUMPDEST
691: PUSH3 0xf0d000 // Function at 0x00f0d000 (selector: 0xd63782a0, result byte: 0x66)
695: JUMP
// Gap from offset 696 to 708
708: JUMPDEST
709: PUSH3 0xf05000 // Function at 0x00f05000 (selector: 0x05ef275b, result byte: 0x69)
713: JUMP
// Gap from offset 714 to 726
726: JUMPDEST
727: PUSH3 0xf10000 // Function at 0x00f10000 (selector: 0x0bac62e9, result byte: 0x6c)
731: JUMP
// Gap from offset 732 to 768
768: JUMPDEST
769: PUSH3 0xf02000 // Function at 0x00f02000 (selector: 0x5b4849aa, result byte: 0x73)
773: JUMP
// Gap from offset 774 to 804
804: JUMPDEST
805: PUSH3 0xf0c000 // Function at 0x00f0c000 (selector: 0xa3053cf0, result byte: 0x79)
809: JUMP
// Gap from offset 810 to 864
864: JUMPDEST
865: PUSH3 0xf06000 // Function at 0x00f06000 (selector: 0xbbe62f60, result byte: 0x83)
869: JUMP
// Gap from offset 870 to 1008
1008: JUMPDEST
1009: PUSH3 0xf14000 // Function at 0x00f14000 (selector: 0x3a153b0d, result byte: 0x9b)
1013: JUMP
// Gap from offset 1014 to 1080
1080: JUMPDEST
1081: PUSH3 0xf11000 // Function at 0x00f11000 (selector: 0x4c566129, result byte: 0xa7)
1085: JUMP
// Gap from offset 1086 to 1104
1104: JUMPDEST
1105: PUSH3 0xf12000 // Function at 0x00f12000 (selector: 0x654b2207, result byte: 0xab)
1109: JUMP
// Gap from offset 1110 to 1176
1176: JUMPDEST
1177: PUSH3 0xf0e000 // Function at 0x00f0e000 (selector: 0x901401c7, result byte: 0xb7)
1181: JUMP
1182: JUMPDEST
1183: PUSH3 0xf01000 // Function at 0x00f01000 (selector: 0x6f4ff18f, result byte: 0xb8)
1187: JUMP
// End of NOP sled

// Function code blocks

// Function at 0x00f01000
0x00f01000: JUMPDEST
// Function 1 implementation
// Selector: 0x6f4ff18f
// ... function code ...
0x00f01001: STOP

// Function at 0x00f02000
0x00f02000: JUMPDEST
// Function 2 implementation
// Selector: 0x5b4849aa
// ... function code ...
0x00f02001: STOP

// Function at 0x00f03000
0x00f03000: JUMPDEST
// Function 3 implementation
// Selector: 0x95ce1bb2
// ... function code ...
0x00f03001: STOP

// Function at 0x00f04000
0x00f04000: JUMPDEST
// Function 4 implementation
// Selector: 0x64ae1f05
// ... function code ...
0x00f04001: STOP

// Function at 0x00f05000
0x00f05000: JUMPDEST
// Function 5 implementation
// Selector: 0x05ef275b
// ... function code ...
0x00f05001: STOP

// Function at 0x00f06000
0x00f06000: JUMPDEST
// Function 6 implementation
// Selector: 0xbbe62f60
// ... function code ...
0x00f06001: STOP

// Function at 0x00f07000
0x00f07000: JUMPDEST
// Function 7 implementation
// Selector: 0xc71b7cc8
// ... function code ...
0x00f07001: STOP

// Function at 0x00f08000
0x00f08000: JUMPDEST
// Function 8 implementation
// Selector: 0xb71a3723
// ... function code ...
0x00f08001: STOP

// Function at 0x00f09000
0x00f09000: JUMPDEST
// Function 9 implementation
// Selector: 0x685eddf5
// ... function code ...
0x00f09001: STOP

// Function at 0x00f0a000
0x00f0a000: JUMPDEST
// Function 10 implementation
// Selector: 0x900c4e83
// ... function code ...
0x00f0a001: STOP

// Function at 0x00f0b000
0x00f0b000: JUMPDEST
// Function 11 implementation
// Selector: 0xc6b145cc
// ... function code ...
0x00f0b001: STOP

// Function at 0x00f0c000
0x00f0c000: JUMPDEST
// Function 12 implementation
// Selector: 0xa3053cf0
// ... function code ...
0x00f0c001: STOP

// Function at 0x00f0d000
0x00f0d000: JUMPDEST
// Function 13 implementation
// Selector: 0xd63782a0
// ... function code ...
0x00f0d001: STOP

// Function at 0x00f0e000
0x00f0e000: JUMPDEST
// Function 14 implementation
// Selector: 0x901401c7
// ... function code ...
0x00f0e001: STOP

// Function at 0x00f0f000
0x00f0f000: JUMPDEST
// Function 15 implementation
// Selector: 0xa28e7df6
// ... function code ...
0x00f0f001: STOP

// Function at 0x00f10000
0x00f10000: JUMPDEST
// Function 16 implementation
// Selector: 0x0bac62e9
// ... function code ...
0x00f10001: STOP

// Function at 0x00f11000
0x00f11000: JUMPDEST
// Function 17 implementation
// Selector: 0x4c566129
// ... function code ...
0x00f11001: STOP

// Function at 0x00f12000
0x00f12000: JUMPDEST
// Function 18 implementation
// Selector: 0x654b2207
// ... function code ...
0x00f12001: STOP

// Function at 0x00f13000
0x00f13000: JUMPDEST
// Function 19 implementation
// Selector: 0x4926ce1e
// ... function code ...
0x00f13001: STOP

// Function at 0x00f14000
0x00f14000: JUMPDEST
// Function 20 implementation
// Selector: 0x3a153b0d
// ... function code ...
0x00f14001: STOP
```