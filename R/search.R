#' Search data with a JMESPath expression
#'
#' `jmespath_search()` evaluates a [JMESPath](https://jmespath.org) expression
#' against an R object, and `jmespath_search_json()` against a JSON string.
#' Use `jmespath_compile()` to parse an expression once and reuse it.
#'
#' @section Converting R to JSON:
#' * Length-1 atomic vectors become scalars, longer ones arrays. Wrap a
#'   single value in [list()] to force an array.
#' * Named lists become objects, unnamed lists arrays.
#' * Data frames become arrays of row objects.
#' * Factors become strings.
#' * `NULL`, `NA`, `NaN` and `Inf` become `null`.
#'
#' @section Converting results to R:
#' * Objects become named lists. Keys come back in alphabetical order.
#' * With `simplify = TRUE`, arrays of scalars of one type become atomic
#'   vectors (`null` becomes `NA`). Otherwise arrays are lists.
#' * Integers outside the 32-bit range are returned as doubles.
#'
#' @param expression A JMESPath expression string, or an object returned by
#'   `jmespath_compile()`.
#' @param data An R object (`jmespath_search()`) or a single JSON string
#'   (`jmespath_search_json()`).
#' @param simplify Whether to simplify arrays of scalars to atomic vectors.
#' @return The query result as an R object.
#' @section Errors:
#' Failures signal conditions of class `jmespath_error`, with subclasses
#' `jmespath_syntax_error` (bad expression), `jmespath_search_error`
#' (evaluation failed, e.g. a type error) and `jmespath_json_error`
#' (`data` is not valid JSON).
#' @examples
#' people <- list(people = list(
#'   list(name = "a", age = 20L),
#'   list(name = "b", age = 30L)
#' ))
#' jmespath_search(people, "people[?age > `25`].name")
#'
#' expr <- jmespath_compile("people[*].age")
#' jmespath_search(people, expr)
#' jmespath_search_json('{"a": {"b": 1}}', "a.b")
#' @export
jmespath_search <- function(data, expression, simplify = TRUE) {
  expr <- as_jmespath(expression)
  with_jmespath_errors(expr$search(data, simplify))
}

#' @rdname jmespath_search
#' @export
jmespath_search_json <- function(data, expression, simplify = TRUE) {
  if (!is.character(data) || length(data) != 1L || is.na(data)) {
    stop("`data` must be a single JSON string.", call. = FALSE)
  }
  expr <- as_jmespath(expression)
  with_jmespath_errors(expr$search_json(data, simplify))
}

#' @rdname jmespath_search
#' @export
jmespath_compile <- function(expression) {
  if (
    !is.character(expression) || length(expression) != 1L || is.na(expression)
  ) {
    stop("`expression` must be a single string.", call. = FALSE)
  }
  with_jmespath_errors(JmesExpr$new(expression))
}

as_jmespath <- function(expression) {
  if (inherits(expression, "JmesExpr")) {
    return(expression)
  }
  jmespath_compile(expression)
}

# Errors raised from Rust are plain errors; give them classes so callers can
# handle them with `tryCatch()`.
with_jmespath_errors <- function(expr) {
  tryCatch(expr, error = function(e) {
    msg <- conditionMessage(e)
    subclass <- if (startsWith(msg, "JMESPath syntax error")) {
      "jmespath_syntax_error"
    } else if (startsWith(msg, "JMESPath search error")) {
      "jmespath_search_error"
    } else if (startsWith(msg, "Invalid JSON")) {
      "jmespath_json_error"
    } else {
      NULL
    }
    stop(structure(
      class = c(subclass, "jmespath_error", "error", "condition"),
      list(message = msg, call = NULL)
    ))
  })
}
