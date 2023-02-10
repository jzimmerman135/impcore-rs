import sys
sys.setrecursionlimit(100000)

def has_divisor(n, d):
    if d > n / 2:
        return 0
    else:
        if 0 == n % d:
            return 1
        else:
            return has_divisor(n, d + 1)

def prime(n):
    if n < 2:
        return 0
    else:
        return not has_divisor(n, 2)

def next_prime(p, n):
    if prime(p):
        if n == 1:
            return p
        else:
            return next_prime(p + 1, n - 1)
    else:
        return next_prime(p + 1, n)

def nthprime(n):
    return next_prime(2, n)

print(nthprime(1000))
print(nthprime(1001))
print(nthprime(1002))
print(nthprime(1003))
print(nthprime(1004))
## print(nthprime(1005))
## print(nthprime(1006))
## print(nthprime(1007))
## print(nthprime(1008))
## print(nthprime(1009))
## print(nthprime(1010))
## print(nthprime(1011))
## print(nthprime(1012))
## print(nthprime(1013))
## print(nthprime(1014))
## print(nthprime(1015))
## print(nthprime(1016))
## print(nthprime(1017))
## print(nthprime(1018))
## print(nthprime(1019))
## print(nthprime(1020))
## print(nthprime(1021))
## print(nthprime(1022))
## print(nthprime(1023))
## print(nthprime(1024))
## print(nthprime(1025))
## print(nthprime(1026))
## print(nthprime(1027))
## print(nthprime(1028))
## print(nthprime(1029))