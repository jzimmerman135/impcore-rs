# Impcore-rs 

### Warning: Unsafe differences from impcore
- Error handling
  Currently impcore-rs only supports a file reading mode, so errors are 
  handled at compile time rather than at runtime  

- Redefinition of functions
  Redefining functions is allowed (but not recommended) the impcore code 
```
(define add-one (x) (+ x 1))
(define add-two (x) (add-one (add-one x)))
(define add-one (x) (+ x 100))
(add-two 0)
(add-one 0)
```

Will differ in output in the following way 
```
;; standard impcore 
200
100
```

```
;; impcore-rs
2
100
```

- TODO
