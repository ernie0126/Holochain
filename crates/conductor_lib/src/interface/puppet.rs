use crate::{
    api::{self},
    interface::interface::Interface,
};
use api::ConductorApiExternal;
use async_trait::async_trait;
use futures::{channel::mpsc, stream::StreamExt};
use log::*;

/// A trivial Interface, used for proof of concept only,
/// which is driven externally by a channel in order to
/// interact with a ConductorApiExternal
pub struct PuppetInterface {
    rx: mpsc::UnboundedReceiver<bool>,
}

impl PuppetInterface {
    pub fn new(rx: mpsc::UnboundedReceiver<bool>) -> Self {
        Self { rx }
    }
}

#[async_trait]
impl Interface for PuppetInterface {
    async fn spawn(mut self, mut api: ConductorApiExternal)
    {
        dbg!("spawn start");
        while let Some(true) = self.rx.next().await {
            dbg!("x");
            if let Err(err) = api.admin(api::AdminMethod::Start("cell-handle".into())).await {
                error!("Error calling admin interface function: {}", err);
            };
        }
    }
}
