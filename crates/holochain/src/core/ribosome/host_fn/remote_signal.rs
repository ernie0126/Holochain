use crate::core::ribosome::error::RibosomeResult;
use crate::core::ribosome::CallContext;
use crate::core::ribosome::RibosomeT;
use holochain_p2p::HolochainP2pCellT;
use holochain_serialized_bytes::prelude::SerializedBytes;
use holochain_zome_types::signal::RemoteSignal;
use holochain_zome_types::zome::FunctionName;
use holochain_zome_types::zome::ZomeName;
use holochain_zome_types::RemoteSignalInput;
use holochain_zome_types::RemoteSignalOutput;
use std::convert::TryInto;
use std::sync::Arc;
use tracing::Instrument;

#[tracing::instrument(skip(_ribosome, call_context, input))]
pub fn remote_signal(
    _ribosome: Arc<impl RibosomeT>,
    call_context: Arc<CallContext>,
    input: RemoteSignalInput,
) -> RibosomeResult<RemoteSignalOutput> {
    const FN_NAME: &str = "recv_remote_signal";
    // Timeouts and errors are ignored,
    // this is a send and forget operation.
    let network = call_context.host_access().network().clone();
    let RemoteSignal { agents, signal } = input.into_inner();
    let zome_name: ZomeName = call_context.zome().into();
    let fn_name: FunctionName = FN_NAME.into();
    let request: SerializedBytes = signal.try_into()?;
    for agent in agents {
        tokio::task::spawn(
            {
                let mut network = network.clone();
                let zome_name = zome_name.clone();
                let fn_name = fn_name.clone();
                let request = request.clone();
                async move {
                    tracing::debug!("sending to {:?}", agent);
                    let result = network
                        .call_remote(agent.clone(), zome_name, fn_name, None, request)
                        .await;
                    tracing::debug!("sent to {:?}", agent);
                    if let Err(e) = result {
                        tracing::info!(
                            "Failed to send remote signal to {:?} because of {:?}",
                            agent,
                            e
                        );
                    }
                }
            }
            .in_current_span(),
        );
    }
    Ok(RemoteSignalOutput::new(()))
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;

    use super::*;
    use crate::test_utils::cool::{CoolAgents, CoolConductor};
    use crate::{conductor::p2p_store::exchange_peer_info, test_utils::cool::CoolDnaFile};
    use futures::future;
    use hdk3::prelude::*;
    use holochain_types::dna::zome::inline_zome::InlineZome;
    use holochain_zome_types::signal::AppSignal;
    use matches::assert_matches;

    #[derive(serde::Serialize, serde::Deserialize, Debug, SerializedBytes, derive_more::From)]
    #[serde(transparent)]
    #[repr(transparent)]
    struct AppString(String);

    fn zome(agents: Vec<AgentPubKey>, num_signals: Arc<AtomicUsize>) -> InlineZome {
        let entry_def = EntryDef::default_with_id("entrydef");

        InlineZome::new_unique(vec![entry_def.clone()])
            .callback("signal_others", move |api, ()| {
                let signal = AppSignal::new(AppString("Hey".to_string()).try_into().unwrap());
                let signal = RemoteSignal {
                    agents: agents.clone(),
                    signal,
                };
                tracing::debug!("sending signal to {:?}", agents);
                api.remote_signal(signal)?;
                Ok(())
            })
            .callback("recv_remote_signal", move |api, signal: AppSignal| {
                tracing::debug!("remote signal");
                num_signals.fetch_add(1, Ordering::SeqCst);
                api.emit_signal(signal).map_err(Into::into)
            })
            .callback("init", move |api, ()| {
                let mut functions: GrantedFunctions = HashSet::new();
                functions.insert((
                    api.zome_info(()).unwrap().zome_name,
                    "recv_remote_signal".into(),
                ));
                let cap_grant_entry = CapGrantEntry {
                    tag: "".into(),
                    // empty access converts to unrestricted
                    access: ().into(),
                    functions,
                };
                api.create((EntryDefId::CapGrant, Entry::CapGrant(cap_grant_entry)))
                    .unwrap();

                Ok(InitCallbackResult::Pass)
            })
    }

    // TODO [ B-03669 ]: make much less verbose
    #[tokio::test(threaded_scheduler)]
    #[cfg(feature = "test_utils")]
    async fn remote_signal_test() -> anyhow::Result<()> {
        observability::test_run().ok();
        const NUM_CONDUCTORS: usize = 5;

        let num_signals = Arc::new(AtomicUsize::new(0));

        let conductors = CoolConductor::multi_from_standard_config(NUM_CONDUCTORS).await;
        let agents =
            future::join_all(conductors.iter().map(|c| CoolAgents::one(c.keystore()))).await;

        let (dna_file, _) = CoolDnaFile::unique_from_inline_zome(
            "zome1",
            zome(agents.clone(), num_signals.clone()),
        )
        .await
        .unwrap();

        let agents_ref = &agents;

        // TODO: write helper
        let apps = future::join_all(conductors.iter().enumerate().map(|(i, conductor)| {
            let dna_file = dna_file.clone();
            async move {
                let apps = conductor
                    .setup_app_for_agents_with_no_membrane_proof(
                        "app",
                        &[agents_ref[i].clone()],
                        &[dna_file.clone().into()],
                    )
                    .await;
                apps.into_inner()
            }
        }))
        .await;

        let p2p_envs = conductors.iter().map(|c| c.envs().p2p()).collect();
        exchange_peer_info(p2p_envs);

        // TODO: write helper
        let cells: Vec<_> = apps
            .iter()
            .flat_map(|cells| cells.iter().flat_map(|app| app.cells().iter()))
            .collect();

        let mut signals = Vec::new();
        for h in conductors.iter() {
            signals.push(h.signal_broadcaster().await.subscribe())
        }
        let signals = signals.into_iter().flatten().collect::<Vec<_>>();

        let _: () = cells[0].call("zome1", "signal_others", ()).await;

        tokio::time::delay_for(std::time::Duration::from_millis(1000)).await;
        assert_eq!(num_signals.load(Ordering::SeqCst), NUM_CONDUCTORS);

        for mut signal in signals {
            let r = signal.try_recv();
            // Each handle should recv a signal
            assert_matches!(r, Ok(_))
        }

        Ok(())
    }
}
