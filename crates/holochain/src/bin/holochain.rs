use holochain_2020::conductor::{
    api::ExternalConductorApi,
    config::ConductorConfig,
    error::{ConductorError, ConductorResult},
    interface::{channel::ChannelInterface, Interface},
    interactive,
    paths::ConfigFilePath,
    Conductor,
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use structopt::StructOpt;
use sx_types::observability::{self, Output};
use tokio::sync::{mpsc, RwLock};
use tracing::*;

#[derive(Debug, StructOpt)]
#[structopt(name = "holochain", about = "The Holochain Conductor.")]
struct Opt {
    #[structopt(
        long,
        help = "Outputs structured json from logging:
    - None: No logging at all (fastest)
    - Log: Output logs to stdout with spans (human readable)
    - Compact: Same as Log but with less information
    - Json: Output logs as structured json (machine readable)
    ",
        default_value = "Log"
    )]
    structured: Output,

    #[structopt(short = "c", help = "Path to a TOML file containing conductor configuration")]
    config_path: Option<PathBuf>,

    #[structopt(short = "i", long, help = "Receive helpful prompts to create missing files and directories,
    useful when running a conductor for the first time")]
    interactive: bool,

    #[structopt(long = "example", help = "Run a very basic interface example, just to have something to do")]
    run_interface_example: bool
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();
    observability::init_fmt(opt.structured).expect("Failed to start contextual logging");
    debug!("observability initialized");

    let config_path: ConfigFilePath = opt.config_path.map(Into::into).unwrap_or_default();
    debug!("config_path: {}", config_path);

    let config: ConductorConfig = if opt.interactive {
        interactive::load_config_or_prompt_for_default(config_path)
            .expect("Could not load conductor config")
            .unwrap_or_else(|| {
                println!("Cannot continue without configuration");
                std::process::exit(1);
            })
    } else {
        ConductorConfig::load_toml(config_path).expect("Could not load conductor config")
    };

    let env_path = PathBuf::from(config.environment_path.clone());

    if opt.interactive && !env_path.is_dir() {
        interactive::prompt_for_environment_dir(&env_path).expect("Couldn't auto-create environment dir");
    }

    let conductor: Conductor = Conductor::build()
        .from_config(config)
        .await
        .expect("Could not initialize Conductor from configuration");

    let lock = Arc::new(RwLock::new(conductor));
    let api = ExternalConductorApi::new(lock);

    if opt.run_interface_example {
        interface_example(api).await;
    } else {
        println!("Conductor successfully initialized. Nothing else to do. Bye bye!");
    }
}

async fn interface_example(api: ExternalConductorApi) {
    let (mut tx_dummy, rx_dummy) = mpsc::channel(100);

    let interface_fut = ChannelInterface::new(rx_dummy).spawn(api);
    let driver_fut = async move {
        for _ in 0..50 as u32 {
            debug!("sending dummy msg");
            tx_dummy.send(true).await.unwrap();
        }
        tx_dummy.send(false).await.unwrap();
    };
    tokio::join!(interface_fut, driver_fut);
}
