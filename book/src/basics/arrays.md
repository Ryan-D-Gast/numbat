---
icon: lucide/brackets
---

# Arrays

Numbat has a built-in data type for arrays. The elements can be of any type, including other arrays.
Arrays can be created using the `[…]` syntax. For example:

```nbt
[30 cm, 110 cm, 2 m]
["a", "b", "c"]
[[1, 2], [3, 4]]
```

The type of an array is written as `Array<T>`, where `T` is the type of the elements. The types of the arrays
above are `Array<Length>`, `Array<String>`, and `Array<Array<Scalar>>`, respectively.

Arrays can be indexed using one-based coordinates:

```nbt
let xs = [10, 20, 30]
xs[2]  # 20

let m = [1, 2; 3, 4]
m[1, 2]  # 2
```

The standard library provides a [number of functions](../prelude/functions/arrays.md) to work with arrays, and
the [linear algebra functions](../prelude/functions/linalg.md) build on top of the same array type. Some useful
things to do with arrays are:
```nbt
# Get the length of an array
len([1, 2, 3])  # returns 3

# Sum all elements of an array:
sum([30 cm, 130 cm, 2 m])  # returns 360 cm

# Get the average of an array:
mean([30 cm, 130 cm, 2 m])  # returns 120 cm

# Filter an array:
filter(is_finite, [20 cm, inf, 1 m])  # returns [20 cm, 1 m]

# Map a function over an array:
map(sqr, [10 cm, 2 m])  # returns [100 cm², 4 m²]

# Generate an array of numbers:
range(1, 5)  # returns [1, 2, 3, 4, 5]

# Generate an array of evenly spaced quantities:
linspace(0 m, 1 m, 5)  # returns [0 m, 0.25 m, 0.5 m, 0.75 m, 1 m]

# Create matrices and solve systems:
zeros([2, 3])         # returns [0, 0, 0; 0, 0, 0]
eye([3, 3])           # returns [1, 0, 0; 0, 1, 0; 0, 0, 1]
[2, 1; 5, 3] \ [1; 2] # solves a linear system
```
