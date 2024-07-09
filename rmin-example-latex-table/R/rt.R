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
#' @export
LatexTableTemplate <- setRefClass("LatexTableTemplate", fields = list(caption = "character", columns = "character", column_names = "character", roundings="integer", table_rules="character", top_rules="character", bottom_rules="character", footnotes="character", bold_fn = "function", italic_fn = "function", stars_fn = "function"), methods = list(
    data = function(data, cap = character(0), row_name=character(0)){
        "display the latex format with the given data, use caption \\code{cap} and row_name \\code{row_name} if provided."
        if (!is.matrix(data)) {stop("provided data should be matrix")}
        if (length(row_name) != dim(data)[1]) {
            row_name = rownames(data)
            if (length(row_name) != dim(data)[1]) {
                row_name = character(dim(data)[1])
            }
        }
        if (length(column_names) == 0) {
            real_column_names = colnames(data)
            if (length(real_column_names) == 0) {
                real_column_names = character(dim(data)[2])
            }
        } else {
            real_column_names = column_names
        }
        if (length(real_column_names) != dim(data)[2]) {
            stop("`dim(data)` does not met `length(column_names)`")
        }
        if (length(cap) == 0) {
            cap=caption
        }
        invisible(`_print`(as.double(data), as.character(row_name), as.logical(bold_fn(data)), as.logical(italic_fn(data)),as.integer(stars_fn(data)), cap, columns, real_column_names, roundings, table_rules, top_rules, bottom_rules, footnotes))
    },
    show = function(){
        data(t(rep(0,length(column_names))),row_name = "template:")
    }
))

