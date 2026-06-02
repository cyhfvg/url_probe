use crate::cli::Args;
use crate::probe::ProbeResult;

/// Return whether a probe result should be written for the current CLI options.
pub fn should_output(res: &ProbeResult, args: &Args) -> bool {
    if res.error.is_some() && !args.output_with_error {
        return false;
    }

    let mut should_output_flag = true;
    let res_http_code = res.http_code;
    let res_size_download = res.size_download;

    let filter_codes = &args.filter_http_code;
    if !filter_codes.is_empty() {
        should_output_flag =
            should_output_flag && res_http_code.is_some_and(|code| filter_codes.contains(&code));
    }

    let black_codes = &args.black_http_code;
    if !black_codes.is_empty() {
        should_output_flag =
            should_output_flag && res_http_code.is_none_or(|code| !black_codes.contains(&code));
    }

    let black_sizes = &args.black_size;
    if !black_sizes.is_empty() {
        should_output_flag =
            should_output_flag && res_size_download.is_none_or(|size| !black_sizes.contains(&size));
    }
    should_output_flag
}
