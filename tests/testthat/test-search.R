people <- list(people = list(
  list(name = "a", age = 20L),
  list(name = "b", age = 30L),
  list(name = "c", age = 40L)
))

test_that("basic queries work", {
  expect_equal(jmespath_search(people, "people[0].name"), "a")
  expect_equal(jmespath_search(people, "people[*].name"), c("a", "b", "c"))
  expect_equal(jmespath_search(people, "people[?age > `25`].name"), c("b", "c"))
  expect_equal(jmespath_search(people, "length(people)"), 3L)
})

test_that("simplify controls array output", {
  expect_equal(
    jmespath_search(people, "people[*].age", simplify = FALSE),
    list(20L, 30L, 40L)
  )
})

test_that("missing keys give NULL", {
  expect_null(jmespath_search(people, "nope"))
})

test_that("R to JSON mapping", {
  expect_equal(jmespath_search(list(a = 1), "a"), 1L)
  expect_equal(jmespath_search(list(a = 1.5), "a"), 1.5)
  expect_equal(jmespath_search(list(a = 1:3), "a"), 1:3)
  expect_equal(jmespath_search(list(a = list(1)), "length(a)"), 1L)
  expect_equal(jmespath_search(list(a = factor("x")), "a"), "x")
  expect_null(jmespath_search(list(a = NA_real_), "a"))
  expect_equal(jmespath_search(list(a = c(1, NA)), "a"), c(1L, NA))
  expect_equal(jmespath_search(list(a = TRUE), "a"), TRUE)
  expect_equal(jmespath_search(list(a = c("x", NA)), "a"), c("x", NA))
})

test_that("data frames become arrays of rows", {
  df <- data.frame(x = 1:3, y = c("a", "b", "c"))
  expect_equal(jmespath_search(df, "[?x > `1`].y"), c("b", "c"))
  expect_equal(jmespath_search(df, "length(@)"), 3L)
})

test_that("objects come back as named lists", {
  res <- jmespath_search(people, "people[0]")
  expect_equal(res, list(age = 20L, name = "a"))
})

test_that("JSON strings are searched", {
  expect_equal(jmespath_search_json('{"a": {"b": 1}}', "a.b"), 1L)
  expect_equal(jmespath_search_json('{"a": [1, 2.5]}', "a"), c(1, 2.5))
  expect_equal(jmespath_search_json('{"a": [1, "x"]}', "a"), list(1L, "x"))
  expect_equal(jmespath_search_json('{"a": [1, null]}', "a"), c(1L, NA))
  expect_equal(jmespath_search_json('{"a": 5000000000}', "a"), 5e9)
  expect_error(jmespath_search_json("{bad", "a"), "Invalid JSON")
})

test_that("compiled expressions are reusable", {
  expr <- jmespath_compile("people[*].age")
  expect_equal(expr$expression(), "people[*].age")
  expect_equal(expr$search(people, TRUE), c(20L, 30L, 40L))
  expect_equal(jmespath_search(people, expr), c(20L, 30L, 40L))
})

test_that("bad expressions error", {
  expect_error(jmespath_compile("a["), "syntax")
  expect_error(jmespath_search(list(), "a["), "syntax")
})

test_that("errors have classes", {
  expect_error(jmespath_compile("a["), class = "jmespath_syntax_error")
  expect_error(jmespath_search_json("{bad", "a"), class = "jmespath_json_error")
  expect_error(jmespath_search(list(a = "x"), "abs(a)"), class = "jmespath_search_error")
  expect_error(jmespath_search(list(a = sum), "a"), class = "jmespath_error")
})
