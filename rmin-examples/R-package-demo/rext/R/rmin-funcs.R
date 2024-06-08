# nolint start

#' @docType package
#' @usage NULL
#' @useDynLib librext, .registration = TRUE
NULL

#' Return string `"Hello world!"` to R.
#' @export
fine2 <- function() .Call(.u0)

# nolint end
