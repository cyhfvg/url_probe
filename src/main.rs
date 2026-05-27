mod cli;
mod error;
mod filter;
mod input;
mod output;
mod probe;

use crate::cli::Args;
use crate::error::CliError;
use crate::filter::should_output;
use crate::input::load_targets;
use crate::output::{build_output_writer, finish_output, write_header, write_result};
use crate::probe::{build_reqwest_client, probe_once_with_retry};
use clap::Parser;
use futures::stream::{self, StreamExt};

/// 主运行函数，处理命令行参数，加载目标URL，执行探测，并输出结果
async fn run() -> Result<(), CliError> {
    let args = Args::parse();

    let targets = load_targets(&args.target)?;

    let client = build_reqwest_client(&args)?;
    if args.concurrency == 0 {
        return Err(CliError::Args("Concurrency must be at least 1".to_string()));
    }

    let mut output_writer = build_output_writer(args.output.as_deref(), args.format)?;
    write_header(&mut output_writer)?;

    // 使用异步流处理目标URL，限制并发数量，并根据过滤条件输出结果 {{{1
    let results = stream::iter(targets.into_iter().map(|target| {
        let client = client.clone();
        let method = args.method;
        let retry = args.retry;

        async move { probe_once_with_retry(&client, target, method, retry).await }
    }))
    .buffer_unordered(args.concurrency);

    tokio::pin!(results);

    while let Some(probe_result) = results.next().await {
        if should_output(&probe_result, &args) {
            write_result(&mut output_writer, &probe_result)?;
        }
    }
    // }}}

    finish_output(&mut output_writer)?;
    Ok(())
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    if let Err(why) = run().await {
        eprintln!("Error: {}", why);
        std::process::exit(1);
    }
}
