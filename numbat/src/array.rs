use compact_str::CompactString;
use itertools::Itertools;
use ndarray::{ArcArrayD, Array, Axis, IxDyn, Slice, concatenate};

use crate::{
    interpreter::RuntimeErrorKind, pretty_print::FormatOptions, quantity::Quantity, value::Value,
};

fn quantity_result<T>(result: crate::quantity::Result<T>) -> Result<T, Box<RuntimeErrorKind>> {
    result.map_err(|err| Box::new(RuntimeErrorKind::QuantityError(err)))
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayValue {
    elements: ArcArrayD<Value>,
}

impl ArrayValue {
    pub fn from_values(elements: Vec<Value>) -> Self {
        let len = elements.len();
        Self::from_shape_vec(vec![len], elements).expect("1-dimensional array shape must be valid")
    }

    pub fn from_rows(rows: Vec<Vec<Value>>) -> Result<Self, Box<RuntimeErrorKind>> {
        let row_count = rows.len();
        let width = rows.first().map_or(0, |row| row.len());

        if rows.iter().any(|row| row.len() != width) {
            return Err(Box::new(RuntimeErrorKind::IncompatibleArrayShape));
        }

        let mut elements = Vec::with_capacity(row_count * width);
        for row in rows {
            elements.extend(row);
        }

        Self::from_shape_vec(vec![row_count, width], elements)
    }

    pub fn from_shape_elements(
        shape: Vec<usize>,
        elements: Vec<Value>,
    ) -> Result<Self, Box<RuntimeErrorKind>> {
        Self::from_shape_vec(shape, elements)
    }

    fn from_shape_vec(
        shape: Vec<usize>,
        elements: Vec<Value>,
    ) -> Result<Self, Box<RuntimeErrorKind>> {
        if !(1..=2).contains(&shape.len()) {
            return Err(Box::new(RuntimeErrorKind::UnsupportedArrayRank));
        }

        let expected_len: usize = shape.iter().product();
        if expected_len != elements.len() {
            return Err(Box::new(RuntimeErrorKind::IncompatibleArrayShape));
        }

        let elements = Array::from_shape_vec(IxDyn(&shape), elements)
            .map_err(|_| Box::new(RuntimeErrorKind::IncompatibleArrayShape))?
            .into_shared();

        Ok(Self { elements })
    }

    fn from_array(elements: ArcArrayD<Value>) -> Result<Self, Box<RuntimeErrorKind>> {
        if !(1..=2).contains(&elements.ndim()) {
            return Err(Box::new(RuntimeErrorKind::UnsupportedArrayRank));
        }

        Ok(Self { elements })
    }

    fn reshape_array(
        elements: ArcArrayD<Value>,
        shape: &[usize],
    ) -> Result<ArcArrayD<Value>, Box<RuntimeErrorKind>> {
        elements
            .into_shape_clone(IxDyn(shape))
            .map(|array| array.into_shared())
            .map_err(|_| Box::new(RuntimeErrorKind::IncompatibleArrayShape))
    }

    fn concatenate_axis(
        axis: Axis,
        lhs: &ArcArrayD<Value>,
        rhs: &ArcArrayD<Value>,
    ) -> Result<ArcArrayD<Value>, Box<RuntimeErrorKind>> {
        concatenate(axis, &[lhs.view(), rhs.view()])
            .map(|array| array.into_shared())
            .map_err(|_| Box::new(RuntimeErrorKind::IncompatibleArrayShape))
    }

    fn shape_vec(&self) -> Vec<usize> {
        self.elements.shape().to_vec()
    }

    fn flat_values(&self) -> Vec<Value> {
        self.elements.iter().cloned().collect()
    }

    pub fn len_axis0(&self) -> usize {
        self.elements.shape().first().copied().unwrap_or(0)
    }

    pub fn into_value(self) -> Value {
        Value::Array(self)
    }

    pub fn vector_values(&self) -> Result<Vec<Value>, Box<RuntimeErrorKind>> {
        match self.elements.ndim() {
            1 => Ok(self.elements.iter().cloned().collect()),
            2 => Err(Box::new(RuntimeErrorKind::ExpectedVector)),
            _ => unreachable!("arrays are limited to rank 1 or 2"),
        }
    }

    fn quantity_vector(&self) -> Result<Vec<Quantity>, Box<RuntimeErrorKind>> {
        match self.elements.ndim() {
            1 => Ok(self
                .elements
                .iter()
                .cloned()
                .map(Value::unsafe_as_quantity)
                .collect()),
            2 => Err(Box::new(RuntimeErrorKind::ExpectedVector)),
            _ => unreachable!("arrays are limited to rank 1 or 2"),
        }
    }

    fn quantity_matrix(&self) -> Result<Vec<Vec<Quantity>>, Box<RuntimeErrorKind>> {
        match self.elements.shape() {
            [rows, cols] => Ok((0..*rows)
                .map(|row| {
                    (0..*cols)
                        .map(|col| {
                            self.elements[IxDyn(&[row, col])]
                                .clone()
                                .unsafe_as_quantity()
                        })
                        .collect::<Vec<_>>()
                })
                .collect()),
            _ => Err(Box::new(RuntimeErrorKind::ExpectedVector)),
        }
    }

    fn parse_index_values(indices: &ArrayValue) -> Result<Vec<usize>, Box<RuntimeErrorKind>> {
        if indices.elements.ndim() != 1 {
            return Err(Box::new(RuntimeErrorKind::ExpectedVector));
        }

        indices
            .elements
            .iter()
            .map(|value| {
                let quantity = value.clone().unsafe_as_quantity();
                let scalar = quantity
                    .as_scalar()
                    .map_err(|_| Box::new(RuntimeErrorKind::InvalidArrayShape))?
                    .to_f64();

                if !scalar.is_finite() || scalar < 1.0 || scalar.fract() != 0.0 {
                    return Err(Box::new(RuntimeErrorKind::InvalidArrayShape));
                }

                Ok(scalar as usize - 1)
            })
            .collect()
    }

    pub fn head(self) -> Result<Value, Box<RuntimeErrorKind>> {
        match self.elements.ndim() {
            1 => self
                .elements
                .first()
                .cloned()
                .ok_or_else(|| Box::new(RuntimeErrorKind::EmptyArray)),
            2 => Err(Box::new(RuntimeErrorKind::ExpectedVector)),
            _ => unreachable!("arrays are limited to rank 1 or 2"),
        }
    }

    pub fn index(&self, indices: ArrayValue) -> Result<Value, Box<RuntimeErrorKind>> {
        let indices = Self::parse_index_values(&indices)?;

        match (self.elements.shape(), indices.as_slice()) {
            ([_len], [idx]) => self.elements.get(IxDyn(&[*idx])).cloned().ok_or_else(|| {
                Box::new(RuntimeErrorKind::UserError(
                    "Array index out of bounds".into(),
                ))
            }),
            ([_rows, _cols], [row, col]) => self
                .elements
                .get(IxDyn(&[*row, *col]))
                .cloned()
                .ok_or_else(|| {
                    Box::new(RuntimeErrorKind::UserError(
                        "Array index out of bounds".into(),
                    ))
                }),
            _ => Err(Box::new(RuntimeErrorKind::UserError(
                "Array indexing expects one index for vectors or two indices for matrices".into(),
            ))),
        }
    }

    pub fn tail(self) -> Result<ArrayValue, Box<RuntimeErrorKind>> {
        match self.elements.ndim() {
            1 => {
                if self.elements.is_empty() {
                    return Err(Box::new(RuntimeErrorKind::EmptyArray));
                }

                ArrayValue::from_array(
                    self.elements
                        .slice_axis(Axis(0), Slice::from(1..))
                        .to_owned()
                        .into_dyn()
                        .into_shared(),
                )
            }
            2 => Err(Box::new(RuntimeErrorKind::ExpectedVector)),
            _ => unreachable!("arrays are limited to rank 1 or 2"),
        }
    }

    pub fn take(self, n: usize) -> Result<ArrayValue, Box<RuntimeErrorKind>> {
        match self.elements.ndim() {
            1 => {
                let end = n.min(self.elements.len());
                ArrayValue::from_array(
                    self.elements
                        .slice_axis(Axis(0), Slice::from(..end))
                        .to_owned()
                        .into_dyn()
                        .into_shared(),
                )
            }
            2 => Err(Box::new(RuntimeErrorKind::ExpectedVector)),
            _ => unreachable!("arrays are limited to rank 1 or 2"),
        }
    }

    pub fn drop(self, n: usize) -> Result<ArrayValue, Box<RuntimeErrorKind>> {
        match self.elements.ndim() {
            1 => {
                let start = n.min(self.elements.len());
                ArrayValue::from_array(
                    self.elements
                        .slice_axis(Axis(0), Slice::from(start..))
                        .to_owned()
                        .into_dyn()
                        .into_shared(),
                )
            }
            2 => Err(Box::new(RuntimeErrorKind::ExpectedVector)),
            _ => unreachable!("arrays are limited to rank 1 or 2"),
        }
    }

    pub fn cons(self, element: Value) -> Result<ArrayValue, Box<RuntimeErrorKind>> {
        match self.elements.ndim() {
            1 => ArrayValue::from_array(Self::concatenate_axis(
                Axis(0),
                &Array::from_vec(vec![element]).into_dyn().into_shared(),
                &self.elements,
            )?),
            2 => Err(Box::new(RuntimeErrorKind::ExpectedVector)),
            _ => unreachable!("arrays are limited to rank 1 or 2"),
        }
    }

    pub fn cons_end(self, element: Value) -> Result<ArrayValue, Box<RuntimeErrorKind>> {
        match self.elements.ndim() {
            1 => ArrayValue::from_array(Self::concatenate_axis(
                Axis(0),
                &self.elements,
                &Array::from_vec(vec![element]).into_dyn().into_shared(),
            )?),
            2 => Err(Box::new(RuntimeErrorKind::ExpectedVector)),
            _ => unreachable!("arrays are limited to rank 1 or 2"),
        }
    }

    pub fn reverse(self) -> Result<ArrayValue, Box<RuntimeErrorKind>> {
        match self.elements.ndim() {
            1 => {
                let mut elements: Vec<_> = self.elements.iter().cloned().collect();
                elements.reverse();
                Ok(ArrayValue::from_values(elements))
            }
            2 => Err(Box::new(RuntimeErrorKind::ExpectedVector)),
            _ => unreachable!("arrays are limited to rank 1 or 2"),
        }
    }

    pub fn element_at(&self, index: usize) -> Result<Value, Box<RuntimeErrorKind>> {
        match self.elements.ndim() {
            1 => self
                .elements
                .get(IxDyn(&[index]))
                .cloned()
                .ok_or_else(|| Box::new(RuntimeErrorKind::EmptyArray)),
            2 => Err(Box::new(RuntimeErrorKind::ExpectedVector)),
            _ => unreachable!("arrays are limited to rank 1 or 2"),
        }
    }

    pub fn contains(&self, needle: &Value) -> Result<bool, Box<RuntimeErrorKind>> {
        match self.elements.ndim() {
            1 => Ok(self.elements.iter().any(|value| value == needle)),
            2 => Err(Box::new(RuntimeErrorKind::ExpectedVector)),
            _ => unreachable!("arrays are limited to rank 1 or 2"),
        }
    }

    pub fn unique(self) -> Result<ArrayValue, Box<RuntimeErrorKind>> {
        match self.elements.ndim() {
            1 => {
                let mut unique = Vec::with_capacity(self.elements.len());
                for value in self.elements.iter().cloned() {
                    if !unique.iter().any(|existing| existing == &value) {
                        unique.push(value);
                    }
                }
                Ok(ArrayValue::from_values(unique))
            }
            2 => Err(Box::new(RuntimeErrorKind::ExpectedVector)),
            _ => unreachable!("arrays are limited to rank 1 or 2"),
        }
    }

    pub fn intersperse(self, separator: Value) -> Result<ArrayValue, Box<RuntimeErrorKind>> {
        match self.elements.ndim() {
            1 => {
                let mut elements = Vec::with_capacity(self.elements.len().saturating_mul(2));
                for (idx, value) in self.elements.iter().cloned().enumerate() {
                    if idx > 0 {
                        elements.push(separator.clone());
                    }
                    elements.push(value);
                }
                Ok(ArrayValue::from_values(elements))
            }
            2 => Err(Box::new(RuntimeErrorKind::ExpectedVector)),
            _ => unreachable!("arrays are limited to rank 1 or 2"),
        }
    }

    pub fn join(self, separator: CompactString) -> Result<CompactString, Box<RuntimeErrorKind>> {
        match self.elements.ndim() {
            1 => {
                let strings = self
                    .elements
                    .iter()
                    .map(|value| match value {
                        Value::String(s) => Ok(s.as_str()),
                        _ => Err(Box::new(RuntimeErrorKind::UserError(
                            "join expects an array of strings".into(),
                        ))),
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(strings.join(separator.as_str()).into())
            }
            2 => Err(Box::new(RuntimeErrorKind::ExpectedVector)),
            _ => unreachable!("arrays are limited to rank 1 or 2"),
        }
    }

    pub fn transpose(self) -> ArrayValue {
        match self.elements.ndim() {
            1 => ArrayValue::from_array(self.elements.insert_axis(Axis(1)))
                .expect("transposed vector shape must be valid"),
            2 if self.elements.shape()[1] == 1 => {
                ArrayValue::from_array(self.elements.remove_axis(Axis(1)).into_shared())
                    .expect("flattened column vector shape must be valid")
            }
            2 => ArrayValue::from_array(
                self.elements
                    .view()
                    .reversed_axes()
                    .to_owned()
                    .into_dyn()
                    .into_shared(),
            )
            .expect("transposed array rank must stay supported"),
            _ => unreachable!("arrays are limited to rank 1 or 2"),
        }
    }

    pub fn dot(self, other: ArrayValue) -> Result<Quantity, Box<RuntimeErrorKind>> {
        let lhs = self.quantity_vector()?;
        let rhs = other.quantity_vector()?;

        if lhs.len() != rhs.len() {
            return Err(Box::new(RuntimeErrorKind::IncompatibleArrayShape));
        }

        let mut acc = Quantity::from_scalar(0.0);
        for (lhs, rhs) in lhs.into_iter().zip(rhs.into_iter()) {
            let term = lhs * rhs;
            acc = quantity_result(&acc + &term)?;
        }

        Ok(acc)
    }

    pub fn cross(self, other: ArrayValue) -> Result<ArrayValue, Box<RuntimeErrorKind>> {
        let lhs = self.quantity_vector()?;
        let rhs = other.quantity_vector()?;

        if lhs.len() != 3 || rhs.len() != 3 {
            return Err(Box::new(RuntimeErrorKind::UserError(
                "cross expects two 3-element vectors".into(),
            )));
        }

        let result = vec![
            quantity_result(
                &(lhs[1].clone() * rhs[2].clone()) - &(lhs[2].clone() * rhs[1].clone()),
            )?,
            quantity_result(
                &(lhs[2].clone() * rhs[0].clone()) - &(lhs[0].clone() * rhs[2].clone()),
            )?,
            quantity_result(
                &(lhs[0].clone() * rhs[1].clone()) - &(lhs[1].clone() * rhs[0].clone()),
            )?,
        ];

        Ok(ArrayValue::from_values(
            result.into_iter().map(Value::Quantity).collect(),
        ))
    }

    pub fn shape(self) -> ArrayValue {
        let shape = self.shape_vec();
        ArrayValue::from_values(
            shape
                .into_iter()
                .map(|dim| Value::Quantity(Quantity::from_scalar(dim as f64)))
                .collect(),
        )
    }

    fn parse_shape(shape: &ArrayValue) -> Result<Vec<usize>, Box<RuntimeErrorKind>> {
        if shape.elements.ndim() != 1 {
            return Err(Box::new(RuntimeErrorKind::ExpectedVector));
        }

        let dims: Vec<_> = shape
            .elements
            .iter()
            .map(|value| {
                let quantity = value.clone().unsafe_as_quantity();
                let scalar = quantity
                    .as_scalar()
                    .map_err(|_| Box::new(RuntimeErrorKind::InvalidArrayShape))?
                    .to_f64();

                if !scalar.is_finite() || scalar < 0.0 || scalar.fract() != 0.0 {
                    return Err(Box::new(RuntimeErrorKind::InvalidArrayShape));
                }

                Ok(scalar as usize)
            })
            .collect::<Result<_, _>>()?;

        if !(1..=2).contains(&dims.len()) {
            return Err(Box::new(RuntimeErrorKind::UnsupportedArrayRank));
        }

        Ok(dims)
    }

    fn parse_constructor_shape(shape: &ArrayValue) -> Result<Vec<usize>, Box<RuntimeErrorKind>> {
        match Self::parse_shape(shape)?.as_slice() {
            [n] => Ok(vec![*n, *n]),
            [rows, cols] => Ok(vec![*rows, *cols]),
            _ => Err(Box::new(RuntimeErrorKind::UnsupportedArrayRank)),
        }
    }

    pub fn reshape(self, shape: ArrayValue) -> Result<ArrayValue, Box<RuntimeErrorKind>> {
        let dims = Self::parse_shape(&shape)?;
        ArrayValue::from_array(Self::reshape_array(self.elements, &dims)?)
    }

    #[allow(clippy::needless_range_loop)]
    pub fn matmul(self, other: ArrayValue) -> Result<Value, Box<RuntimeErrorKind>> {
        match (self.elements.shape(), other.elements.shape()) {
            ([lhs_len], [rhs_len]) => {
                let lhs = self.quantity_vector()?;
                let rhs = other.quantity_vector()?;
                if lhs_len != rhs_len {
                    return Err(Box::new(RuntimeErrorKind::IncompatibleArrayShape));
                }
                let mut acc = Quantity::from_scalar(0.0);
                for (lhs, rhs) in lhs.into_iter().zip(rhs.into_iter()) {
                    let term = lhs * rhs;
                    acc = quantity_result(&acc + &term)?;
                }
                Ok(Value::Array(ArrayValue::from_values(vec![
                    Value::Quantity(acc),
                ])))
            }
            ([lhs_rows, lhs_cols], [rhs_rows, rhs_cols]) if lhs_cols == rhs_rows => {
                let lhs = self.quantity_matrix()?;
                let rhs = other.quantity_matrix()?;
                let mut rows = Vec::with_capacity(*lhs_rows);

                for row in 0..*lhs_rows {
                    let mut out_row = Vec::with_capacity(*rhs_cols);
                    for col in 0..*rhs_cols {
                        let mut acc = Quantity::from_scalar(0.0);
                        for k in 0..*lhs_cols {
                            let term = lhs[row][k].clone() * rhs[k][col].clone();
                            acc = quantity_result(&acc + &term)?;
                        }
                        out_row.push(Value::Quantity(acc));
                    }
                    rows.push(out_row);
                }

                Ok(Value::Array(ArrayValue::from_rows(rows)?))
            }
            ([lhs_rows, lhs_cols], [rhs_len]) if lhs_cols == rhs_len => {
                let lhs = self.quantity_matrix()?;
                let rhs = other.quantity_vector()?;
                let mut out = Vec::with_capacity(*lhs_rows);

                for row in 0..*lhs_rows {
                    let mut acc = Quantity::from_scalar(0.0);
                    for k in 0..*lhs_cols {
                        let term = lhs[row][k].clone() * rhs[k].clone();
                        acc = quantity_result(&acc + &term)?;
                    }
                    out.push(Value::Quantity(acc));
                }

                Ok(Value::Array(ArrayValue::from_values(out)))
            }
            ([lhs_len], [rhs_rows, rhs_cols]) if lhs_len == rhs_rows => {
                let lhs = self.quantity_vector()?;
                let rhs = other.quantity_matrix()?;
                let mut out = Vec::with_capacity(*rhs_cols);

                for col in 0..*rhs_cols {
                    let mut acc = Quantity::from_scalar(0.0);
                    for k in 0..*rhs_rows {
                        let term = lhs[k].clone() * rhs[k][col].clone();
                        acc = quantity_result(&acc + &term)?;
                    }
                    out.push(Value::Quantity(acc));
                }

                Ok(Value::Array(ArrayValue::from_values(out)))
            }
            _ => Err(Box::new(RuntimeErrorKind::IncompatibleArrayShape)),
        }
    }

    #[allow(clippy::needless_range_loop)]
    pub fn solve(self, rhs: ArrayValue) -> Result<Value, Box<RuntimeErrorKind>> {
        let lhs_dims = self.shape_vec();
        let rhs_dims = rhs.shape_vec();

        let n = match lhs_dims.as_slice() {
            [n, n2] if n == n2 => *n,
            _ => {
                return Err(Box::new(RuntimeErrorKind::IncompatibleArrayShape));
            }
        };

        let (mut a, rhs_is_vector, mut b) = match rhs_dims.as_slice() {
            [rows] if *rows == n => {
                let matrix = rhs
                    .quantity_vector()?
                    .into_iter()
                    .map(|q| vec![q])
                    .collect();
                (self.quantity_matrix()?, true, matrix)
            }
            [rows, cols] if *rows == n => (self.quantity_matrix()?, false, rhs.quantity_matrix()?),
            _ => return Err(Box::new(RuntimeErrorKind::IncompatibleArrayShape)),
        };

        let n = a.len();
        let rhs_cols = b.first().map_or(0, |row| row.len());

        for pivot_idx in 0..n {
            let pivot_row = (pivot_idx..n)
                .find(|&row| !a[row][pivot_idx].is_zero())
                .ok_or_else(|| {
                    Box::new(RuntimeErrorKind::UserError("Matrix is singular".into()))
                })?;

            if pivot_row != pivot_idx {
                a.swap(pivot_row, pivot_idx);
                b.swap(pivot_row, pivot_idx);
            }

            let pivot = a[pivot_idx][pivot_idx].clone();
            if pivot.is_zero() {
                return Err(Box::new(RuntimeErrorKind::UserError(
                    "Matrix is singular".into(),
                )));
            }

            for col in pivot_idx..n {
                a[pivot_idx][col] = a[pivot_idx][col].clone() / pivot.clone();
            }
            for col in 0..rhs_cols {
                b[pivot_idx][col] = b[pivot_idx][col].clone() / pivot.clone();
            }

            for row in 0..n {
                if row == pivot_idx {
                    continue;
                }

                let factor = a[row][pivot_idx].clone();
                if factor.is_zero() {
                    continue;
                }

                for col in pivot_idx..n {
                    let term = a[pivot_idx][col].clone() * factor.clone();
                    a[row][col] = quantity_result(&a[row][col] - &term)?;
                }
                for col in 0..rhs_cols {
                    let term = b[pivot_idx][col].clone() * factor.clone();
                    b[row][col] = quantity_result(&b[row][col] - &term)?;
                }
            }
        }

        if rhs_is_vector {
            Ok(Value::Array(ArrayValue::from_values(
                b.into_iter()
                    .map(|row| row.into_iter().next().unwrap())
                    .map(Value::Quantity)
                    .collect(),
            )))
        } else {
            Ok(Value::Array(ArrayValue::from_rows(
                b.into_iter()
                    .map(|row| row.into_iter().map(Value::Quantity).collect())
                    .collect(),
            )?))
        }
    }

    pub fn filled(shape: ArrayValue, value: Value) -> Result<ArrayValue, Box<RuntimeErrorKind>> {
        let dims = Self::parse_constructor_shape(&shape)?;
        ArrayValue::from_array(Array::from_elem(IxDyn(&dims), value).into_shared())
    }

    pub fn zeros(shape: ArrayValue) -> Result<ArrayValue, Box<RuntimeErrorKind>> {
        Self::filled(shape, Value::Quantity(Quantity::from_scalar(0.0)))
    }

    pub fn ones(shape: ArrayValue) -> Result<ArrayValue, Box<RuntimeErrorKind>> {
        Self::filled(shape, Value::Quantity(Quantity::from_scalar(1.0)))
    }

    pub fn eye(shape: ArrayValue) -> Result<ArrayValue, Box<RuntimeErrorKind>> {
        let dims = Self::parse_constructor_shape(&shape)?;
        let rows = dims[0];
        let cols = dims[1];

        let mut array = Array::from_elem(
            IxDyn(&[rows, cols]),
            Value::Quantity(Quantity::from_scalar(0.0)),
        );
        let one = Value::Quantity(Quantity::from_scalar(1.0));
        for i in 0..rows.min(cols) {
            array[IxDyn(&[i, i])] = one.clone();
        }

        ArrayValue::from_array(array.into_shared())
    }

    pub fn vcat(self, other: ArrayValue) -> Result<ArrayValue, Box<RuntimeErrorKind>> {
        let lhs_shape = self.shape_vec();
        let rhs_shape = other.shape_vec();

        match (lhs_shape.as_slice(), rhs_shape.as_slice()) {
            ([..], [..])
                if self.elements.ndim() == rhs_shape.len() && self.elements.ndim() == 1 =>
            {
                ArrayValue::from_array(Self::concatenate_axis(
                    Axis(0),
                    &self.elements,
                    &other.elements,
                )?)
            }
            ([_, lhs_width], [rhs_width]) if lhs_width == rhs_width => {
                ArrayValue::from_array(Self::concatenate_axis(
                    Axis(0),
                    &self.elements,
                    &other.elements.insert_axis(Axis(0)),
                )?)
            }
            ([lhs_width], [_, rhs_width]) if lhs_width == rhs_width => {
                ArrayValue::from_array(Self::concatenate_axis(
                    Axis(0),
                    &self.elements.insert_axis(Axis(0)),
                    &other.elements,
                )?)
            }
            ([_, lhs_width], [_, rhs_width]) if lhs_width == rhs_width => ArrayValue::from_array(
                Self::concatenate_axis(Axis(0), &self.elements, &other.elements)?,
            ),
            _ => Err(Box::new(RuntimeErrorKind::IncompatibleArrayShape)),
        }
    }

    pub fn hcat(self, other: ArrayValue) -> Result<ArrayValue, Box<RuntimeErrorKind>> {
        let lhs_shape = self.shape_vec();
        let rhs_shape = other.shape_vec();

        match (lhs_shape.as_slice(), rhs_shape.as_slice()) {
            ([..], [..])
                if self.elements.ndim() == rhs_shape.len() && self.elements.ndim() == 1 =>
            {
                ArrayValue::from_array(Self::concatenate_axis(
                    Axis(0),
                    &self.elements,
                    &other.elements,
                )?)
            }
            ([lhs_rows, _], [rhs_rows, _]) if lhs_rows == rhs_rows => ArrayValue::from_array(
                Self::concatenate_axis(Axis(1), &self.elements, &other.elements)?,
            ),
            ([lhs_rows, _], [rhs_len]) if lhs_rows == rhs_len => {
                ArrayValue::from_array(Self::concatenate_axis(
                    Axis(1),
                    &self.elements,
                    &other.elements.insert_axis(Axis(1)),
                )?)
            }
            ([lhs_len], [rhs_rows, _]) if lhs_len == rhs_rows => {
                ArrayValue::from_array(Self::concatenate_axis(
                    Axis(1),
                    &self.elements.insert_axis(Axis(1)),
                    &other.elements,
                )?)
            }
            _ => Err(Box::new(RuntimeErrorKind::IncompatibleArrayShape)),
        }
    }

    pub fn dimensions(&self) -> &[usize] {
        self.elements.shape()
    }

    fn fmt_with(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.elements.shape() {
            [_] => write!(
                f,
                "[{}]",
                self.elements
                    .iter()
                    .map(|element| element.to_string())
                    .join(", ")
            ),
            [rows, cols] => {
                let values = self.flat_values();
                write!(f, "[")?;

                for row in 0..*rows {
                    if row > 0 {
                        write!(f, "; ")?;
                    }

                    for col in 0..*cols {
                        if col > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", values[row * cols + col])?;
                    }
                }

                write!(f, "]")
            }
            _ => unreachable!("arrays are limited to rank 1 or 2"),
        }
    }

    pub fn pretty_print_with(&self, options: &FormatOptions) -> crate::markup::Markup {
        match self.elements.shape() {
            [_] => {
                crate::markup::operator("[")
                    + itertools::Itertools::intersperse(
                        self.elements
                            .iter()
                            .map(|element| element.pretty_print_with(options)),
                        crate::markup::operator(",") + crate::markup::space(),
                    )
                    .sum()
                    + crate::markup::operator("]")
            }
            [rows, cols] => {
                let values = self.flat_values();

                crate::markup::operator("[")
                    + itertools::Itertools::intersperse(
                        (0..*rows).map(|row| {
                            itertools::Itertools::intersperse(
                                (0..*cols)
                                    .map(|col| values[row * cols + col].pretty_print_with(options)),
                                crate::markup::operator(",") + crate::markup::space(),
                            )
                            .sum::<crate::markup::Markup>()
                        }),
                        crate::markup::operator(";") + crate::markup::space(),
                    )
                    .sum()
                    + crate::markup::operator("]")
            }
            _ => unreachable!("arrays are limited to rank 1 or 2"),
        }
    }
}

impl std::fmt::Display for ArrayValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_with(f)
    }
}
