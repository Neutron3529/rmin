#' LatexTableTemplate
#' @slot caption Default caption, contains 2 elements, caption[1] is the optional caption, and caption[2] is the optional label.
#' @slot columns contains 2 elements, controlling the table format for name columns and data columns.
#' @slot column_names Default column_names, could be empty (in this case, use data colnames instead).
#' @slot roundings control the roundings of the column, its length could be 1 (global rounding) or length(column_names) (column rounding). Data displayed in percentage format has 2 less rounding digits.
#' @slot table_rules define extra latex rules before the \\begin\{tabular\} command
#' @slot top_rules define extra latex rules after the \\begin\{tabular\} command
#' @slot bottom_rules define extra latex rules before the \\end\{tabular\} command
#' @slot footnotes define extra latex rules after the \\end\{tabular\} command
#' @slot bold_fn function control which item is wrapped with \\bold\{..\}
#' @slot italic_fn function control which item is wrapped with \\italic\{..\}
#' @slot stars_fn function calculate how much stars is used.
#' @details
#' Slot *_fn accept data as its input, the output should have length 0 (indicate this is a no-op) or length(data).
#'
#' The used column_names should have the same length that defined in columns
#' @examples
#' template <- LatexTableTemplate(caption=c("cap","lab"), columns = c("lll|","rrr%"), column_names = c("1","2","3"), roundings=4L, table_rules = character(0), top_rules="\\toprules", bottom_rules = "\\bottom_rules", footnotes = character(0), bold_fn=function(x)NULL, italic_fn=function(x)NULL, stars_fn=function(x)NULL)
#' template$data(matrix(1:6/6,2,3))
#' # That will directly print:
#' # \begin{table}
#' #     \caption{cap\label{lab}}
#' #     \begin{tabular}{lll|rrr}
#' #         \toprules
#' #          &  &  & 1 & 2 & 3\\
#' #           &&& 0.1667 & 0.5000 & 83.33%\\
#' #           &&& 0.3333 & 0.6667 & 100.00%\\
#' #         \bottom_rules
#' #     \end{tabular}
#' # \end{table}
#' @export LatexTableTemplate
#' @exportClass LatexTableTemplate
LatexTableTemplate <- setRefClass("LatexTableTemplate", fields = list(caption = "character", columns = "character", column_names = "character", roundings="integer", table_rules="character", top_rules="character", bottom_rules="character", footnotes="character", bold_fn = "function", italic_fn = "function", stars_fn = "function", use_threeparttable = "logical"), methods = list(
    data = function(data, cap = character(0), hline=0L, cline = integer(0), row_name=character(0), columns=character(0), column_names=character(0), roundings = 3L, table_rules = character(0), top_rules=character(0), bottom_rules = character(0), footnotes = character(0), use_threeparttable = FALSE){
        "display the latex format with the given data, use caption \\code{cap} and row_name \\code{row_name} if provided.\n \\code{hline} controls the location that \\\\hline command inserted in\n \\code{cline} has length 3*N, and cline[3*K+(1:3)] = c(a,b,c) where \\\\cline{b-c} will be inserted after a-th row is printed."
        if (missing(cap)) {
            cap=caption
        }
        if (missing(columns)) {
            columns = .self$columns
        }
        if (missing(column_names)) {
            column_names = .self$column_names
        }
        if (missing(roundings)) {
            roundings = .self$roundings
        }
        if (missing(table_rules)) {
            table_rules = .self$table_rules
        }
        if (missing(top_rules)) {
            top_rules = .self$top_rules
        }
        if (missing(bottom_rules)) {
            bottom_rules = .self$bottom_rules
        }
        if (missing(footnotes)) {
            footnotes = .self$footnotes
        }
        if (missing(use_threeparttable)) {
            use_threeparttable = .self$use_threeparttable
        }
        if (!is.matrix(data)) {stop("provided data should be matrix")}
        if (length(row_name) != dim(data)[1]) {
            row_name = rownames(data)
            if (length(row_name) != dim(data)[1]) {
                row_name = character(dim(data)[1])
            }
        }
        if (length(column_names) == 0) {
            column_names = colnames(data)
            if (length(column_names) == 0) {
                column_names = character(dim(data)[2])
            }
        }
        invisible(`_print`(as.double(data), as.character(row_name), as.integer(hline), as.integer(cline), as.logical(bold_fn(data)), as.logical(italic_fn(data)),as.integer(stars_fn(data)), as.character(cap), as.character(columns), as.character(column_names), as.integer(roundings), as.character(table_rules), as.character(top_rules), as.character(bottom_rules), as.character(footnotes), as.logical(use_threeparttable)))
    },
    show = function(){
        data(t(rep(0,length(column_names))),row_name = "template:")
    }
))

#' R Template
#' @description minimal R template
#' @details
#' Since it has the same name of \code{stats::rt}, this function is not exported,
#' use \code{lt:::rt} could be fine.
#' @param col_fmt control the column format of data
#' @param name_col control the format of normal columns

#' @slot caption Default caption, contains 2 elements, caption[1] is the optional caption, and caption[2] is the optional label.
#' @slot column_names Default column_names, could be empty (in this case, use data colnames instead).
#' @slot roundings control the roundings of the column, its length could be 1 (global rounding) or length(column_names) (column rounding). Data displayed in percentage format has 2 less rounding digits.
#' @slot table_rules define extra latex rules before the \\begin\{tabular\} command
#' @slot top_rules define extra latex rules after the \\begin\{tabular\} command
#' @slot bottom_rules define extra latex rules before the \\end\{tabular\} command
#' @slot footnotes define extra latex rules after the \\end\{tabular\} command
#' @slot bold_fn function control which item is wrapped with \\bold\{..\}
#' @slot italic_fn function control which item is wrapped with \\italic\{..\}
#' @slot stars_fn function calculate how much stars is used.


#' @return a \code{LatexTableTemplate} object
#' @examples
#' template <- lt:::rt("ccc%")
#' template$data(matrix(1,2,3))
rt <- function(col_fmt, name_col="l|", caption=character(0), columns = c(name_col, col_fmt), column_names = character(0), roundings=3L, table_rules = character(0), top_rules="\\toprules", bottom_rules = "\\bottom_rules", footnotes = character(0), bold_fn=function(x)NULL, italic_fn=function(x)NULL, stars_fn=function(x)NULL, use_threeparttable=F) {
    LatexTableTemplate(caption=as.character(caption), columns = as.character(columns), column_names = as.character(column_names), roundings=as.integer(roundings), table_rules = as.character(table_rules), top_rules=as.character(top_rules), bottom_rules = as.character(bottom_rules), footnotes = as.character(footnotes), bold_fn=bold_fn, italic_fn=italic_fn, stars_fn=stars_fn, use_threeparttable = use_threeparttable)
}
