# Impcore-rs 
Impcore-rs is a JIT compiled Impcore runtime with some handy language extensions to make impcore easier, but not too easy.

Impcore is a very primitive language created by Norman Ramsey in his textbook "Programming Languages: Build Prove Compare".
This implementation of impcore is solely for the challenge of making something cool in impcore, to do which requires a faster implementation of impcore.

This implementation uses LLVM for an optimizing backend and benchmarks up to 14000x speedups on the original C implementation. It also guarantees no memory leaks, thanks to Rust and some simple garbage collection.

## Installation: 
  - `cargo install impcore` (TODO)

## Basic Usage:
`./impcore-rs -f myfilename.imp`

**Options**

  - `-f/--filename <filename>`, use code from file, must have extension `.imp`. 
  - `-q`, quiet mode, doesn't print top level expressions by default. 
  - `-d`, debug mode, print full ast and llvm ir to stderr. 
  - `-i`, interpreter mode uses true impcore semantics at a performance cost. 
  - `--emit-llvm`, exports an llvm IR file 

## Impcore Language Extensions:
### Arrays:
To make an array use the declaration `(val <array-name>[] <exp>)`.
This allocates and binds an array of size `<exp>` to 
`<name>`. The declaration `(val <array-name>[] <exp>)` evaluates to `<exp>`. Array values are always initialized to zero.

    
You can read and store array values by `<array-name>[<index-exp>]` or `(set <array-name>[<index-exp>] <exp>)`.
It is an undefined behaviour to access beyond the bounds of the array.

Arrays store 32 bit integer values. But you can construct other types using bit manipulations.

Functions can take and operate on arrays. For instance

```
(define array-xor-swap (A[] i j)
    (begin 
        (set A[i] (^ A[i] A[j]))
        (set A[j] (^ A[j] A[i]))
        (set A[i] (^ A[i] A[j]))
        0))
(val arr[] 10)
(set arr[5] 5)
(set arr[2] 2)
(array-xor-swap arr[] 2 5)
(check-expect arr[2] 5)
(check-expect arr[5] 2)
```
Note: Arrays can only be passed to a function via an empty indexer, `(foo <array-name>[])`, rather than `(foo <array-name>)` like in C.

### Bitwise Operators:
Although these are possible in standard impcore, they are native machine instructions in impcore-rs. Use `~`, `&`, `|`, `!`, `^`, `<<`, `>>` (signed), and `>>>` (unsigned).

### Printing:
Along with standard impcore print functions `print`,`println`,`printu`, there is also `(printc <exp>)` which will print the expression as a char, as well as `(printstr <array-name>[])` which is equivalent to the C code `printf("%s", *(char *)array_name);`. This means that there must be a null-terminator somewhere inside `<array-name>[]`.

### User Input:
If you add `#(import stdin)` somewhere at the top of your file, you will get access 
to the function `(getc)` which takes no arguments but is equivalent to the C code `fgetc(stdin);`

### Import Files:
Import files via the definition `#(import "<filename>.imp")`. Imported files must end in `.imp`, files will only be imported once on the occurent of their first definition. Files are imported recursively.

Paths are relative to the entry filename so if entry file
`imp/myfile.imp` contains a call to `#(import "stdlib/chars.imp"`, then `imp/stdlib/chars.imp` will be pasted in place of the import. If `imp/stdlib/chars.imp` contains a further call to `#(import "stdlib/word.imp")`, then `imp/stdlib/word.imp` would be imported. 

Note: if in the example above `imp/stdlib/chars.imp` called `#(import "word.imp")` instead, then `imp/word.imp` would be imported. Helpful preprocessor errors will be raised for recursive imports and failure to open files.

### Macros:
Macros in impcore must start with `'` to differentiate them from variables. It is recommended that macros are in all caps.

Define a simple replacement macro via
`#(replace 'EOF (- 0 1))`, not every instance of `'EOF` will be replaced with the expression `(- 0 1)`.

Define an inline function macro with
`#(replace ('ADD-ONE x) (+ x 1))` . This macro takes an expression as an argument and pastes it in every occurence.

Inline function macros can take any number of arguments. 

You can unbind a macro with 
`(#undef 'MACRO)`, this will remove the replacer from the macro environment. This will remove **both** inline functions and expressions with the name `'MACRO`.
 

## Warning: Unsafe differences from impcore
- **Unit testing**

  Check-error unit tests are not run, since this is resolved at compile time

- **Error handling**

  Currently impcore-rs only supports a file reading mode, so errors are 
  handled at compile time rather than at runtime  

- **Redefinition of functions**

  Redefining functions is allowed (for now) in both modes compiler execution modes, 
  but in JIT mode it will resort to javascript behaviour. In interpreter mode it will resemble standard impcore. 
  
```
(define add-one (x) (+ x 1))
(define add-two (x) (add-one (add-one x)))
(add-two 0)
(define add-one (x) (+ x 100))
(add-two 0)
(add-one 0)
```

  Will have the following differences in output
  
```
;; standard impcore 
add-one
add-two
2
add-one
200
100

;; impcore-rs (interpreter)
add-one
add-two
2
add-one
200
100

;; impcore-rs (jit)
add-one
add-two
200
add-one
200
100
```

- TODO

