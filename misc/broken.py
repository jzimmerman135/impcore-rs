def add_one(x):
    return x + 1
def add_two(x):
    return add_one(add_one(x))

print(add_two(0))

def add_one(x):
    return x + 100

print(add_two(0))
print(add_one(0))
