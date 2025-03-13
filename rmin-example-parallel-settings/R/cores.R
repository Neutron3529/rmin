#' @export
#' @title logical cores
#' @description logical CPU cores
#' @examples
#' mclapply(mc.cores = PS::LOGICAL_CORES, mc.preschedule = FALSE, 1:10, print)
LOGICAL_CORES = 1 # cores(T)
#' @export
#' @title physical cores
#' @description physical CPU cores
#' mclapply(mc.cores = PS::PHYSICAL_CORES, mc.preschedule = FALSE, 1:10, print)
PHYSICAL_CORES = 2 # cores(F)
#' @export
#' @title default cores
#' @description default CPU cores
CORES = PHYSICAL_CORES
#' mclapply(mc.cores = PS::CORES, mc.preschedule = FALSE, 1:10, print)

#' @export
#' @title wrapper of mclapply
#' @returns see parallel::mclapply
mclapply=function(mc.cores = PS::CORES, mc.preschedule = FALSE,...)parallel::mclapply(mc.cores = mc.cores, mc.preschedule = mc.preschedule, ...)
#' @export
#' @title wrapper of mcmapply
#' @returns see parallel::mcmapply
mcmapply=function(mc.cores = PS::CORES, mc.preschedule = FALSE,...)parallel::mcmapply(mc.cores = mc.cores, mc.preschedule = mc.preschedule, ...)


.onLoad=function(libname, pkgname) {
    LOGICAL_CORES<<-corenum(T)
    PHYSICAL_CORES<<-corenum(F)
    CORES<<-PHYSICAL_CORES
    threads = as.integer(Sys.getenv('OMP_NUM_THREADS',1))
    if(is.na(threads)) {
        threads = 1
        cat("OMP_NUM_THREADS has a wrong setting, set its default to 1.")
    }
    set_threads(threads)
}
