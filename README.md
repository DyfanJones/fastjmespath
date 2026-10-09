<!-- badges: start -->
[![R-CMD-check](https://github.com/DyfanJones/fastjmespath/actions/workflows/R-CMD-check.yaml/badge.svg)](https://github.com/DyfanJones/fastjmespath/actions/workflows/R-CMD-check.yaml)
[![Codecov test coverage](https://codecov.io/gh/DyfanJones/fastjmespath/graph/badge.svg)](https://app.codecov.io/gh/DyfanJones/fastjmespath)
<!-- badges: end -->

# fastjmespath

Fast [JMESPath](https://jmespath.org) queries for R objects and JSON strings.

fastjmespath evaluates JMESPath expressions against R lists, data frames and
JSON. It is built on the Rust [`jmespath`](https://crates.io/crates/jmespath)
crate through [extendr](https://extendr.github.io/), and expressions can be
compiled once and reused across queries.

## Installation

Building from source needs a Rust toolchain (Cargo and `rustc >= 1.65.0`) and
`xz`. See <https://www.rust-lang.org/tools/install> to install Rust.

```r
# install.packages("pak")
pak::pak("<user>/fastjmespath")
```

## Usage

```r
library(fastjmespath)

people <- list(people = list(
  list(name = "a", age = 20L),
  list(name = "b", age = 30L)
))

# Search an R object
jmespath_search(people, "people[?age > `25`].name")
#> [1] "b"

# Search a JSON string
jmespath_search_json('{"a": {"b": 1}}', "a.b")
#> [1] 1

# Compile once, reuse many times
expr <- jmespath_compile("people[*].age")
jmespath_search(people, expr)
#> [1] 20 30
```

### Data frames

A data frame is treated as an array of row objects, so the expression applies
to the rows directly.

```r
df <- data.frame(
  name = c("a", "b", "c"),
  age = c(20L, 30L, 40L)
)

# Project a column
jmespath_search(df, "[*].age")
#> [1] 20 30 40

# Filter rows, then project
jmespath_search(df, "[?age > `25`].name")
#> [1] "b" "c"

# Filtered rows come back as a list of named lists (keys in alphabetical order)
str(jmespath_search(df, "[?age > `25`]"))
#> List of 2
#>  $ :List of 2
#>   ..$ age : int 30
#>   ..$ name: chr "b"
#>  $ :List of 2
#>   ..$ age : int 40
#>   ..$ name: chr "c"
```

### Functions

| Function                | Purpose                                              |
|-------------------------|------------------------------------------------------|
| `jmespath_search()`     | Evaluate an expression against an R object          |
| `jmespath_search_json()`| Evaluate an expression against a single JSON string |
| `jmespath_compile()`    | Parse an expression once for reuse                   |

Both search functions take `simplify = TRUE` by default, which turns arrays of
same-typed scalars into atomic vectors. Use `simplify = FALSE` to always get
lists.

### Converting R to JSON

* Length-1 atomic vectors become scalars, longer ones arrays. Wrap a single
  value in `list()` to force an array.
* Named lists become objects, unnamed lists arrays.
* Data frames become arrays of row objects.
* Factors become strings.
* `NULL`, `NA`, `NaN` and `Inf` become `null`.

### Converting results to R

* Objects become named lists. Keys come back in alphabetical order.
* With `simplify = TRUE`, arrays of scalars of one type become atomic vectors
  (`null` becomes `NA`). Otherwise arrays are lists.
* Integers outside the 32-bit range are returned as doubles.

### Error handling

Failures signal conditions of class `jmespath_error`, with subclasses:

* `jmespath_syntax_error`: the expression is invalid
* `jmespath_search_error`: evaluation failed, e.g. a type error
* `jmespath_json_error`: the data is not valid JSON

```r
tryCatch(
  jmespath_search(list(a = 1), "a["),
  jmespath_syntax_error = function(e) conditionMessage(e)
)
```

## Development

```r
devtools::document()  # regenerate docs
devtools::test()      # run the testthat suite, including JMESPath compliance tests
```

After changing Rust code, regenerate the extendr wrappers with
`rextendr::document()`.

## License

MIT. See [LICENSE.md](LICENSE.md).
