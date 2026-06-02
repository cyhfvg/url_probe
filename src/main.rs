use clap::Parser;
use futures::stream::{self, StreamExt};
use url_probe::cli::Args;
use url_probe::error::CliError;
use url_probe::filter::should_output;
use url_probe::input::load_targets;
use url_probe::output::{build_output_writer, finish_output, write_header, write_result};
use url_probe::probe::{build_reqwest_client, probe_once_with_retry};

/// Parse CLI options, load target URLs, probe them, and write output.
async fn run() -> Result<(), CliError> {
    let args = Args::parse();

    let targets = load_targets(&args.target)?;

    let client = build_reqwest_client(&args)?;
    if args.concurrency == 0 {
        return Err(CliError::Args("Concurrency must be at least 1".to_string()));
    }

    let mut output_writer = build_output_writer(args.output.as_deref(), args.format)?;
    write_header(&mut output_writer)?;

    // Process target URLs with bounded async concurrency and output filtering. {{{1
    let results =
        stream::iter(
            targets.into_iter().map(|target| {
                let client = client.clone();
                let method = args.method;
                let retry = args.retry;
                let request_jitter_ms = args.request_jitter_ms;

                async move {
                    probe_once_with_retry(&client, target, method, retry, request_jitter_ms).await
                }
            }),
        )
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
