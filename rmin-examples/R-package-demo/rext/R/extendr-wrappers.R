# nolint start

#' @docType package
#' @usage NULL
#' @useDynLib rex, .registration = TRUE
NULL

#' Return string `"Hello world!"` to R.
#' @export
Rtest <- function() .Call(test)

# nolint end
