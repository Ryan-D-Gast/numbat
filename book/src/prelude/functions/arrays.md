---
icon: lucide/brackets
---

# Array-related functions

Defined in: `core::arrays`

### `id`

```nbt
fn id<T>(x: T) -> T
```

### `len`
Get the length of a 1-D array.

```nbt
fn len<A>(xs: Array<A>) -> Scalar
```

!!! example "Example"
    ```nbt
    len([3, 2, 1])

        = 3
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=len%28%5B3%2C%202%2C%201%5D%29){ .md-button }

### `head`
Get the first element of a 1-D array. Yields a runtime error if the array is empty.

```nbt
fn head<A>(xs: Array<A>) -> A
```

!!! example "Example"
    ```nbt
    head([3, 2, 1])

        = 3
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=head%28%5B3%2C%202%2C%201%5D%29){ .md-button }

### `tail`
Get everything but the first element of a 1-D array. Yields a runtime error if the array is empty.

```nbt
fn tail<A>(xs: Array<A>) -> Array<A>
```

!!! example "Example"
    ```nbt
    tail([3, 2, 1])

        = [2, 1]    [Array<Scalar>]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=tail%28%5B3%2C%202%2C%201%5D%29){ .md-button }

### `cons`
Prepend an element to a 1-D array.

```nbt
fn cons<A>(x: A, xs: Array<A>) -> Array<A>
```

!!! example "Example"
    ```nbt
    cons(77, [3, 2, 1])

        = [77, 3, 2, 1]    [Array<Scalar>]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=cons%2877%2C%20%5B3%2C%202%2C%201%5D%29){ .md-button }

### `cons_end`
Append an element to the end of a 1-D array.

```nbt
fn cons_end<A>(x: A, xs: Array<A>) -> Array<A>
```

!!! example "Example"
    ```nbt
    cons_end(77, [3, 2, 1])

        = [3, 2, 1, 77]    [Array<Scalar>]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=cons%5Fend%2877%2C%20%5B3%2C%202%2C%201%5D%29){ .md-button }

### `is_empty`
Check if a 1-D array is empty.

```nbt
fn is_empty<A>(xs: Array<A>) -> Bool
```

!!! example "Example"
    ```nbt
    is_empty([3, 2, 1])

        = false    [Bool]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=is%5Fempty%28%5B3%2C%202%2C%201%5D%29){ .md-button }

!!! example "Example"
    ```nbt
    is_empty([])

        = true    [Bool]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=is%5Fempty%28%5B%5D%29){ .md-button }

### `vcat`
Concatenate two arrays along the first axis.

```nbt
fn vcat<A>(xs1: Array<A>, xs2: Array<A>) -> Array<A>
```

!!! example "Example"
    ```nbt
    vcat([3, 2, 1], [10, 11])

        = [3, 2, 1, 10, 11]    [Array<Scalar>]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=vcat%28%5B3%2C%202%2C%201%5D%2C%20%5B10%2C%2011%5D%29){ .md-button }

### `hcat`
Concatenate two arrays along the second axis.

```nbt
fn hcat<A>(xs1: Array<A>, xs2: Array<A>) -> Array<A>
```

!!! example "Example"
    ```nbt
    hcat([1, 2; 3, 4], [5; 6])

        = [1, 2, 5; 3, 4, 6]    [Array<Scalar>]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=hcat%28%5B1%2C%202%3B%203%2C%204%5D%2C%20%5B5%3B%206%5D%29){ .md-button }

### `shape`
Return the shape of an array.

```nbt
fn shape<A>(xs: Array<A>) -> Array<Scalar>
```

!!! example "Example"
    ```nbt
    shape([1, 2; 3, 4])

        = [2, 2]    [Array<Scalar>]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=shape%28%5B1%2C%202%3B%203%2C%204%5D%29){ .md-button }

### `reshape`
Reshape an array to a new 1D or 2D shape.

```nbt
fn reshape<A>(xs: Array<A>, new_shape: Array<Scalar>) -> Array<A>
```

!!! example "Example"
    ```nbt
    reshape([1, 2, 3, 4], [2, 2])

        = [1, 2; 3, 4]    [Array<Scalar>]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=reshape%28%5B1%2C%202%2C%203%2C%204%5D%2C%20%5B2%2C%202%5D%29){ .md-button }

### `take`
Get the first `n` elements of a 1-D array.

```nbt
fn take<A>(n: Scalar, xs: Array<A>) -> Array<A>
```

!!! example "Example"
    ```nbt
    take(2, [3, 2, 1, 0])

        = [3, 2]    [Array<Scalar>]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=take%282%2C%20%5B3%2C%202%2C%201%2C%200%5D%29){ .md-button }

### `drop`
Get everything but the first `n` elements of a 1-D array.

```nbt
fn drop<A>(n: Scalar, xs: Array<A>) -> Array<A>
```

!!! example "Example"
    ```nbt
    drop(2, [3, 2, 1, 0])

        = [1, 0]    [Array<Scalar>]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=drop%282%2C%20%5B3%2C%202%2C%201%2C%200%5D%29){ .md-button }

### `element_at`
Get the element at index `i` in a 1-D array.

```nbt
fn element_at<A>(i: Scalar, xs: Array<A>) -> A
```

!!! example "Example"
    ```nbt
    element_at(2, [3, 2, 1, 0])

        = 1
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=element%5Fat%282%2C%20%5B3%2C%202%2C%201%2C%200%5D%29){ .md-button }

### `index`
Index into an array using 1-based coordinates.

```nbt
fn index<A>(xs: Array<A>, indices: Array<Scalar>) -> A
```

!!! example "Example"
    ```nbt
    ([1, 2; 3, 4])[1, 2]

        = 2
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=%28%5B1%2C%202%3B%203%2C%204%5D%29%5B1%2C%202%5D){ .md-button }

### `range`
Generate a range of integer numbers from `start` to `end` (inclusive).

```nbt
fn range(start: Scalar, end: Scalar) -> Array<Scalar>
```

!!! example "Example"
    ```nbt
    range(2, 12)

        = [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]    [Array<Scalar>]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=range%282%2C%2012%29){ .md-button }

### `fill`
Create an array filled with a given value and shape.

```nbt
fn fill<A>(value: A, shape: Array<Scalar>) -> Array<A>
```

!!! example "Example"
    ```nbt
    fill(0, [2, 3])

        = [0, 0, 0; 0, 0, 0]    [forall A: Dim. Array<A>]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=fill%280%2C%20%5B2%2C%203%5D%29){ .md-button }

### `reverse`
Reverse the order of a 1-D array.

```nbt
fn reverse<A>(xs: Array<A>) -> Array<A>
```

!!! example "Example"
    ```nbt
    reverse([3, 2, 1])

        = [1, 2, 3]    [Array<Scalar>]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=reverse%28%5B3%2C%202%2C%201%5D%29){ .md-button }

### `map`
Generate a new 1-D array by applying a function to each element of the input array.

```nbt
fn map<A, B>(f: Fn[(A) -> B], xs: Array<A>) -> Array<B>
```

!!! example "Square all elements of a 1-D array."
    ```nbt
    map(sqr, [3, 2, 1])

        = [9, 4, 1]    [Array<Scalar>]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=map%28sqr%2C%20%5B3%2C%202%2C%201%5D%29){ .md-button }

### `map2`
Generate a new 1-D array by applying a function to each element of the input array. This function takes two inputs: a variable, and the element of the array.

```nbt
fn map2<A, B, C>(f: Fn[(A, B) -> C], other: A, xs: Array<B>) -> Array<C>
```

!!! example "Returns a 1-D array of bools corresponding to whether the sub-array contains a 2 or not."
    ```nbt
    map2(contains, 2, [[0], [2], [1, 2], [0, 2, 3], []])

        = [false, true, true, true, false]    [Array<Bool>]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=map2%28contains%2C%202%2C%20%5B%5B0%5D%2C%20%5B2%5D%2C%20%5B1%2C%202%5D%2C%20%5B0%2C%202%2C%203%5D%2C%20%5B%5D%5D%29){ .md-button }

### `filter`
Filter a 1-D array by a predicate.

```nbt
fn filter<A>(p: Fn[(A) -> Bool], xs: Array<A>) -> Array<A>
```

!!! example "Example"
    ```nbt
    filter(is_finite, [0, 1e10, NaN, -inf])

        = [0, 10_000_000_000]    [Array<Scalar>]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=filter%28is%5Ffinite%2C%20%5B0%2C%201e10%2C%20NaN%2C%20%2Dinf%5D%29){ .md-button }

### `foldl`
Fold a function over a 1-D array.

```nbt
fn foldl<A, B>(f: Fn[(A, B) -> A], acc: A, xs: Array<B>) -> A
```

!!! example "Join an array of strings by folding."
    ```nbt
    foldl(str_append, "", ["Num", "bat", "!"])

        = "Numbat!"    [String]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=foldl%28str%5Fappend%2C%20%22%22%2C%20%5B%22Num%22%2C%20%22bat%22%2C%20%22%21%22%5D%29){ .md-button }

### `sort_by_key`
Sort a 1-D array of elements, using the given key function that maps the element to a quantity.

```nbt
fn sort_by_key<A, D: Dim>(key: Fn[(A) -> D], xs: Array<A>) -> Array<A>
```

!!! example "Sort descending by key."
    ```nbt
    fn negate(x) = -x
    sort_by_key(negate, [701, 313, 9999, 4])

        = [9999, 701, 313, 4]    [Array<Scalar>]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=fn%20negate%28x%29%20%3D%20%2Dx%0Asort%5Fby%5Fkey%28negate%2C%20%5B701%2C%20313%2C%209999%2C%204%5D%29){ .md-button }

### `id`

```nbt
fn id<T>(x: T) -> T
```

### `sort`
Sort a 1-D array of quantities in ascending order.

```nbt
fn sort<D: Dim>(xs: Array<D>) -> Array<D>
```

!!! example "Example"
    ```nbt
    sort([3, 2, 7, 8, -4, 0, -5])

        = [-5, -4, 0, 2, 3, 7, 8]    [Array<Scalar>]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=sort%28%5B3%2C%202%2C%207%2C%208%2C%20%2D4%2C%200%2C%20%2D5%5D%29){ .md-button }

### `contains`
Returns true if the element `x` is in the 1-D array `xs`.

```nbt
fn contains<A>(x: A, xs: Array<A>) -> Bool
```

!!! example "Example"
    ```nbt
    [3, 2, 7, 8, -4, 0, -5] |> contains(0)

        = true    [Bool]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=%5B3%2C%202%2C%207%2C%208%2C%20%2D4%2C%200%2C%20%2D5%5D%20%7C%3E%20contains%280%29){ .md-button }

!!! example "Example"
    ```nbt
    [3, 2, 7, 8, -4, 0, -5] |> contains(1)

        = false    [Bool]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=%5B3%2C%202%2C%207%2C%208%2C%20%2D4%2C%200%2C%20%2D5%5D%20%7C%3E%20contains%281%29){ .md-button }

### `unique`
Remove duplicates from a given 1-D array.

```nbt
fn unique<A>(xs: Array<A>) -> Array<A>
```

!!! example "Example"
    ```nbt
    unique([1, 2, 2, 3, 3, 3])

        = [1, 2, 3]    [Array<Scalar>]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=unique%28%5B1%2C%202%2C%202%2C%203%2C%203%2C%203%5D%29){ .md-button }

### `intersperse`
Add an element between each pair of elements in a 1-D array.

```nbt
fn intersperse<A>(sep: A, xs: Array<A>) -> Array<A>
```

!!! example "Example"
    ```nbt
    intersperse(0, [1, 1, 1, 1])

        = [1, 0, 1, 0, 1, 0, 1]    [Array<Scalar>]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=intersperse%280%2C%20%5B1%2C%201%2C%201%2C%201%5D%29){ .md-button }

### `sum`
Sum all elements of a 1-D array.

```nbt
fn sum<D: Dim>(xs: Array<D>) -> D
```

!!! example "Example"
    ```nbt
    sum([3 m, 200 cm, 1000 mm])

        = 6000 mm    [Length]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=sum%28%5B3%20m%2C%20200%20cm%2C%201000%20mm%5D%29){ .md-button }

### `linspace`
Generate a 1-D array of `n_steps` evenly spaced numbers from `start` to `end` (inclusive).

```nbt
fn linspace<D: Dim>(start: D, end: D, n_steps: Scalar) -> Array<D>
```

!!! example "Example"
    ```nbt
    linspace(-5 m, 5 m, 11)

        = [-5 m, -4 m, -3 m, -2 m, -1 m, 0 m, 1 m, 2 m, 3 m, 4 m, 5 m]    [Array<Length>]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=linspace%28%2D5%20m%2C%205%20m%2C%2011%29){ .md-button }

### `join`
Convert an array of strings into a single string by concatenating them with a separator.

```nbt
fn join(xs: Array<String>, sep: String) -> String
```

!!! example "Example"
    ```nbt
    join(["snake", "case"], "_")

        = "snake_case"    [String]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=join%28%5B%22snake%22%2C%20%22case%22%5D%2C%20%22%5F%22%29){ .md-button }

