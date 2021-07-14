use super::*;

impl<E: KitsuneP2pEventSender> ShardedGossip<E> {
    /// Incoming accept gossip round message.
    /// - Send back the agent bloom and ops bloom gossip messages.
    /// - Only send the agent bloom if this is a recent gossip type.
    pub(super) async fn incoming_accept(
        &self,
        con: Tx2ConHnd<wire::Wire>,
        remote_arc_set: Vec<ArcInterval>,
    ) -> KitsuneResult<()> {
        let local_agents = self.inner.share_mut(|i, _| Ok(i.local_agents.clone()))?;

        // Choose any local agent so we can send requests to the store.
        let agent = local_agents.iter().cloned().next();

        // If we don't have a local agent then there's nothing to do.
        let agent = match agent {
            Some(agent) => agent,
            // No local agents so there's no one to initiate gossip from.
            None => return Ok(()),
        };

        // Get the local intervals.
        let local_intervals =
            store::local_agent_arcs(&self.evt_sender, &self.space, &local_agents, &agent).await?;

        let peer_cert = con.peer_cert();

        let mut gossip = Vec::with_capacity(2);

        // Generate the bloom filters and new state.
        let state = self
            .generate_blooms(
                &agent,
                &local_agents,
                local_intervals,
                remote_arc_set,
                &mut gossip,
            )
            .await?;

        self.inner.share_mut(|inner, _| {
            // TODO: What happen if we are in the middle of a new outgoing and
            // a stale accept comes in for the same peer cert?
            inner.state_map.insert(peer_cert.clone(), state);
            for g in gossip {
                inner.outgoing.push_back((
                    GossipTgt::new(Vec::with_capacity(0), peer_cert.clone()),
                    HowToConnect::Con(con.clone()),
                    g,
                ));
            }
            Ok(())
        })?;
        Ok(())
    }
}
