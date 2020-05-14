use crate::core::ribosome::error::RibosomeResult;
use crate::core::ribosome::wasm_ribosome::WasmRibosome;
use crate::core::ribosome::HostContext;
use holochain_zome_types::SysTimeInput;
use holochain_zome_types::SysTimeOutput;
use std::sync::Arc;

pub async fn sys_time(
    _ribosome: Arc<WasmRibosome>,
    _host_context: Arc<HostContext>,
    _input: SysTimeInput,
) -> RibosomeResult<SysTimeOutput> {
    let start = std::time::SystemTime::now();
    let since_the_epoch = start
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards");
    Ok(SysTimeOutput::new(since_the_epoch))
}

#[cfg(test)]
pub mod wasm_test {
    use holochain_wasm_test_utils::TestWasm;
    use holochain_zome_types::{SysTimeInput, SysTimeOutput};

    #[tokio::test(threaded_scheduler)]
    #[serial_test::serial]
    async fn invoke_import_sys_time_test() {
        let _: SysTimeOutput =
            crate::call_test_ribosome!(TestWasm::Imports, "sys_time", SysTimeInput::new(()));
    }
}
