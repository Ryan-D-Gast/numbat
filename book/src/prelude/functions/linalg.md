---
icon: lucide/square-function
---

# Linear algebra functions

Defined in: `core::linalg`

### `transpose`
Transpose a matrix.

```nbt
fn transpose<D>(xs: Array<D>) -> Array<D>
```

!!! example "Example"
    ```nbt
    transpose([1, 2; 3, 4])

        = [1, 3; 2, 4]    [Array<Scalar>]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=transpose%28%5B1%2C%202%3B%203%2C%204%5D%29){ .md-button }

### `matmul`
Multiply arrays and matrices.

```nbt
fn matmul<D: Dim, E: Dim>(lhs: Array<D>, rhs: Array<E>) -> Array<D × E>
```

!!! example "Example"
    ```nbt
    matmul([1, 2; 3, 4], [5; 6])

        = [17; 39]    [Array<Scalar>]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=matmul%28%5B1%2C%202%3B%203%2C%204%5D%2C%20%5B5%3B%206%5D%29){ .md-button }

### `mat_dot`
Compute the dot product of two vectors.

```nbt
fn mat_dot<D: Dim, E: Dim>(lhs: Array<D>, rhs: Array<E>) -> D × E
```

!!! example "Example"
    ```nbt
    mat_dot([1, 2, 3], [4, 5, 6])

        = 32
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=mat%5Fdot%28%5B1%2C%202%2C%203%5D%2C%20%5B4%2C%205%2C%206%5D%29){ .md-button }

### `mat_cross`
Compute the cross product of two 3-vectors.

```nbt
fn mat_cross<D: Dim, E: Dim>(lhs: Array<D>, rhs: Array<E>) -> Array<D × E>
```

!!! example "Example"
    ```nbt
    mat_cross([1, 0, 0], [0, 1, 0])

        = [0, 0, 1]    [Array<Scalar>]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=mat%5Fcross%28%5B1%2C%200%2C%200%5D%2C%20%5B0%2C%201%2C%200%5D%29){ .md-button }

### `linear_solve`
Solve a linear system using backslash notation.

```nbt
fn linear_solve<D: Dim, E: Dim>(lhs: Array<D>, rhs: Array<E>) -> Array<E / D>
```

!!! example "Example"
    ```nbt
    [2, 1; 5, 3] \ [1; 2]

        = [1; -1]    [Array<Scalar>]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=%5B2%2C%201%3B%205%2C%203%5D%20%5C%20%5B1%3B%202%5D){ .md-button }

### `zeros`
Create an array of zeros with the given shape.

```nbt
fn zeros(shape: Array<Scalar>) -> Array<Scalar>
```

!!! example "Example"
    ```nbt
    zeros([2, 3])

        = [0, 0, 0; 0, 0, 0]    [Array<Scalar>]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=zeros%28%5B2%2C%203%5D%29){ .md-button }

### `ones`
Create an array of ones with the given shape.

```nbt
fn ones(shape: Array<Scalar>) -> Array<Scalar>
```

!!! example "Example"
    ```nbt
    ones([2, 3])

        = [1, 1, 1; 1, 1, 1]    [Array<Scalar>]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=ones%28%5B2%2C%203%5D%29){ .md-button }

### `eye`
Create an identity matrix.

```nbt
fn eye(shape: Array<Scalar>) -> Array<Scalar>
```

!!! example "Example"
    ```nbt
    eye([3, 3])

        = [1, 0, 0; 0, 1, 0; 0, 0, 1]    [Array<Scalar>]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=eye%28%5B3%2C%203%5D%29){ .md-button }

### `identity`
Create an identity matrix.

```nbt
fn identity(shape: Array<Scalar>) -> Array<Scalar>
```

!!! example "Example"
    ```nbt
    identity([3, 3])

        = [1, 0, 0; 0, 1, 0; 0, 0, 1]    [Array<Scalar>]
    ```
    [:material-play-circle: Run this example](https://numbat.dev/?q=identity%28%5B3%2C%203%5D%29){ .md-button }

