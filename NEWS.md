# fastjmespath 0.0.1

* Initial release.
* `jmespath_search()` evaluates a JMESPath expression against an R object
  (lists and data frames).
* `jmespath_search_json()` evaluates a JMESPath expression against a JSON
  string.
* `jmespath_compile()` compiles an expression once so it can be reused across
  queries.
* Built on the Rust `jmespath` crate through extendr, and tested against the
  JMESPath compliance suite.
