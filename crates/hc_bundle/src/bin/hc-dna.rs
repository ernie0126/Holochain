use holochain_cli_bundle::HcDnaBundle;
use structopt::StructOpt;

/// Main `hc-dna` executable entrypoint.
#[tokio::main]
pub async fn main() {
    let opt = HcDnaBundle::from_args();
    if let Err(err) = opt.run().await {
        eprintln!("hc-dna: {}", err);
        std::process::exit(1);
    }
}
