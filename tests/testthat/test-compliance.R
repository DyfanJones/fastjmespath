# Runs the official JMESPath compliance suite (https://github.com/jmespath/jmespath.test).
# Expected values and inputs go through the same converter with `simplify = FALSE`,
# so these tests check the query semantics rather than the R type mapping.
skip_if_not_installed("jsonlite")

to_json <- function(x) {
  as.character(jsonlite::toJSON(
    x,
    auto_unbox = TRUE,
    null = "null",
    digits = NA
  ))
}
normalise <- function(json) jmespath_search_json(json, "@", simplify = FALSE)

# Known upstream bug in the jmespath crate: avg([]) yields NaN instead of null.
known_failures <- "functions.json: avg(empty_list)"

files <- list.files(
  test_path("compliance"),
  pattern = "\\.json$",
  full.names = TRUE
)

for (file in files) {
  suites <- jsonlite::read_json(file, simplifyVector = FALSE)
  for (suite in suites) {
    given <- to_json(suite$given)
    for (case in suite$cases) {
      label <- sprintf("%s: %s", basename(file), case$expression)
      if (label %in% known_failures) {
        next
      }
      if (!is.null(case$error)) {
        test_that(label, {
          expect_error(jmespath_search(
            case$given,
            case$expression,
            simplify = FALSE
          ))
          expect_error(jmespath_search_json(
            given,
            case$expression,
            simplify = FALSE
          ))
        })
      } else {
        test_that(label, {
          expect_equal(
            jmespath_search(suite$given, case$expression, simplify = FALSE),
            normalise(to_json(case$result))
          )
          expect_equal(
            jmespath_search_json(given, case$expression, simplify = FALSE),
            normalise(to_json(case$result))
          )
        })
      }
    }
  }
}
