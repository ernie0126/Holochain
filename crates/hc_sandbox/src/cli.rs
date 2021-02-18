//! Definitions of StructOpt options for use in the CLI

use crate::cmds::*;
use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Helper for generating, running, and interacting with Holochain Conductor "sandboxes".
///
/// A sandbox is a directory containing a conductor config, databases, and keystore,
/// with a single Holochain app installed in the conductor:
/// Everything you need to quickly run your app in holochain,
/// or create complex multi-conductor sandboxes for testing.
pub struct HcSandbox {
    #[structopt(subcommand)]
    command: HcSandboxSubcommand,
    /// Force the admin port that hc uses to talk to holochain to a specific value.
    /// For example `hc -f=9000,9001 run`
    /// This must be set on each run or the port will change if it's in use.
    #[structopt(short, long, value_delimiter = ",")]
    force_admin_ports: Vec<u16>,
    /// Set the path to the holochain binary.
    #[structopt(short, long, env = "HC_HOLOCHAIN_PATH", default_value = "holochain")]
    holochain_path: PathBuf,
}

/// The list of subcommands for `hc sandbox`
#[derive(Debug, StructOpt)]
#[structopt(setting = structopt::clap::AppSettings::InferSubcommands)]
pub enum HcSandboxSubcommand {
    /// Generate one or more new Holochain Conductor sandbox(es) for later use.
    ///
    /// A single app will be installed as part of this sandbox.
    /// See the help for the `<dnas>` argument below to learn how to define the app to be installed.
    Generate {
        /// Number of conductor sandboxes to create.
        #[structopt(short, long, default_value = "1")]
        num_conductors: usize,

        /// (flattened)
        #[structopt(flatten)]
        gen: Create,

        /// Automatically run the sandbox(es) that were created.
        /// This is effectively a combination of `hc generate` and `hc run`
        ///
        /// You may optionally specify app interface ports to bind when running.
        /// This allows your UI to talk to the conductor.
        ///
        /// For example, `hc generate -r=0,9000,0` will create three app interfaces.
        /// Or, use `hc generate -r` to run without attaching any app interfaces.
        ///
        /// This follows the same structure as `hc run --ports`
        #[structopt(short, long, value_delimiter = ",")]
        run: Option<Vec<u16>>,

        /// List of DNAs to use when installing the App for this sandbox.
        /// Defaults to searching the current directory for a single `*.dna` file.
        dnas: Vec<PathBuf>,
    },
    /// Run conductor(s) from existing sandbox(es).
    Run(Run),

    /// Make a call to a conductor's admin interface.
    Call(crate::calls::Call),

    /// List sandboxes found in `$(pwd)/.hc`.
    List {
        /// Show more verbose information.
        #[structopt(short, long, parse(from_occurrences))]
        verbose: usize,
    },

    /// Clean (completely remove) sandboxes that are listed in the `$(pwd)/.hc` file.
    Clean,
}

/// Options for running a sandbox
#[derive(Debug, StructOpt)]
pub struct Run {
    /// Optionally specifies app interface ports to bind when running.
    /// This allows your UI to talk to the conductor.
    /// For example, `hc -p=0,9000,0` will create three app interfaces.
    #[structopt(short, long, value_delimiter = ",")]
    ports: Vec<u16>,

    /// (flattened)
    #[structopt(flatten)]
    existing: Existing,
}

impl HcSandbox {
    /// Run this command
    pub async fn run(self) -> anyhow::Result<()> {
        match self.command {
            HcSandboxSubcommand::Generate {
                gen,
                run,
                num_conductors,
                dnas,
            } => {
                let paths = generate(&self.holochain_path, dnas, num_conductors, gen).await?;
                for (port, path) in self
                    .force_admin_ports
                    .clone()
                    .into_iter()
                    .zip(paths.clone().into_iter())
                {
                    crate::force_admin_port(path, port)?;
                }
                if let Some(ports) = run {
                    let holochain_path = self.holochain_path.clone();
                    let force_admin_ports = self.force_admin_ports.clone();
                    tokio::task::spawn(async move {
                        if let Err(e) =
                            run_n(&holochain_path, paths, ports, force_admin_ports).await
                        {
                            tracing::error!(failed_to_run = ?e);
                        }
                    });
                    tokio::signal::ctrl_c().await?;
                    crate::save::release_ports(std::env::current_dir()?).await?;
                }
            }
            HcSandboxSubcommand::Run(Run { ports, existing }) => {
                let paths = existing.load()?;
                if paths.is_empty() {
                    return Ok(());
                }
                let holochain_path = self.holochain_path.clone();
                let force_admin_ports = self.force_admin_ports.clone();
                tokio::task::spawn(async move {
                    if let Err(e) = run_n(&holochain_path, paths, ports, force_admin_ports).await {
                        tracing::error!(failed_to_run = ?e);
                    }
                });
                tokio::signal::ctrl_c().await?;
                crate::save::release_ports(std::env::current_dir()?).await?;
            }
            HcSandboxSubcommand::Call(call) => {
                crate::calls::call(&self.holochain_path, call).await?
            }
            // HcSandboxSubcommand::Task => todo!("Running custom tasks is coming soon"),
            HcSandboxSubcommand::List { verbose } => {
                crate::save::list(std::env::current_dir()?, verbose)?
            }
            HcSandboxSubcommand::Clean => crate::save::clean(std::env::current_dir()?, Vec::new())?,
        }

        Ok(())
    }
}

async fn run_n(
    holochain_path: &Path,
    paths: Vec<PathBuf>,
    app_ports: Vec<u16>,
    force_admin_ports: Vec<u16>,
) -> anyhow::Result<()> {
    let run_holochain = |holochain_path: PathBuf, path: PathBuf, ports, force_admin_port| async move {
        crate::run::run(&holochain_path, path, ports, force_admin_port).await?;
        Result::<_, anyhow::Error>::Ok(())
    };
    let mut force_admin_ports = force_admin_ports.into_iter();
    let mut app_ports = app_ports.into_iter();
    let jhs = paths
        .into_iter()
        .zip(std::iter::repeat_with(|| force_admin_ports.next()))
        .zip(std::iter::repeat_with(|| app_ports.next()))
        .map(|((path, force_admin_port), app_port)| {
            let f = run_holochain(
                holochain_path.to_path_buf(),
                path,
                app_port.map(|p| vec![p]).unwrap_or_default(),
                force_admin_port,
            );
            tokio::task::spawn(f)
        });
    futures::future::try_join_all(jhs).await?;
    Ok(())
}

async fn generate(
    holochain_path: &Path,
    dnas: Vec<PathBuf>,
    num_conductors: usize,
    create: Create,
) -> anyhow::Result<Vec<PathBuf>> {
    let dnas = crate::dna::parse_dnas(dnas)?;
    let paths = crate::sandbox::default_n(holochain_path, num_conductors, create, dnas).await?;
    crate::save::save(std::env::current_dir()?, paths.clone())?;
    Ok(paths)
}
