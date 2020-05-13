use holochain_types::dna::wasm::DnaWasm;
pub extern crate strum;
#[macro_use]
extern crate strum_macros;
use holochain_zome_types::zome::ZomeName;

#[derive(EnumIter, Clone, Copy)]
pub enum TestWasm {
    Debug,
    Foo,
    Imports,
    InitPass,
    InitFail,
    MigrateAgentPass,
    MigrateAgentFail,
    PostCommitSuccess,
    PostCommitFail,
    Validate,
    ValidateInvalid,
    ValidateValid,
}

impl From<TestWasm> for ZomeName {
    fn from(test_wasm: TestWasm) -> ZomeName {
        ZomeName::from(match test_wasm {
            TestWasm::Debug => "debug",
            TestWasm::Foo => "foo",
            TestWasm::Imports => "imports",
            TestWasm::InitPass => "init_pass",
            TestWasm::InitFail => "init_fail",
            TestWasm::MigrateAgentPass => "migrate_agent_pass",
            TestWasm::MigrateAgentFail => "migrate_agent_fail",
            TestWasm::PostCommitSuccess => "post_commit_success",
            TestWasm::PostCommitFail => "post_commit_fail",
            TestWasm::Validate => "validate",
            TestWasm::ValidateInvalid => "validate_invalid",
            TestWasm::ValidateValid => "validate_valid",
        })
    }
}

impl From<TestWasm> for DnaWasm {
    fn from(test_wasm: TestWasm) -> DnaWasm {
        DnaWasm::from(match test_wasm {
            TestWasm::Debug => include_bytes!(concat!(
                env!("OUT_DIR"),
                "/wasm32-unknown-unknown/release/test_wasm_debug.wasm"
            ))
            .to_vec(),
            TestWasm::Foo => include_bytes!(concat!(
                env!("OUT_DIR"),
                "/wasm32-unknown-unknown/release/test_wasm_foo.wasm"
            ))
            .to_vec(),
            TestWasm::Imports => include_bytes!(concat!(
                env!("OUT_DIR"),
                "/wasm32-unknown-unknown/release/test_wasm_imports.wasm"
            ))
            .to_vec(),
            TestWasm::InitPass => include_bytes!(concat!(
                env!("OUT_DIR"),
                "/wasm32-unknown-unknown/release/test_wasm_init_pass.wasm"
            ))
            .to_vec(),
            TestWasm::InitFail => include_bytes!(concat!(
                env!("OUT_DIR"),
                "/wasm32-unknown-unknown/release/test_wasm_init_fail.wasm"
            ))
            .to_vec(),
            TestWasm::MigrateAgentPass => include_bytes!(concat!(
                env!("OUT_DIR"),
                "/wasm32-unknown-unknown/release/test_wasm_migrate_agent_pass.wasm"
            ))
            .to_vec(),
            TestWasm::MigrateAgentFail => include_bytes!(concat!(
                env!("OUT_DIR"),
                "/wasm32-unknown-unknown/release/test_wasm_migrate_agent_fail.wasm"
            ))
            .to_vec(),
            TestWasm::PostCommitSuccess => include_bytes!(concat!(
                env!("OUT_DIR"),
                "/wasm32-unknown-unknown/release/test_wasm_post_commit_success.wasm"
            ))
            .to_vec(),
            TestWasm::PostCommitFail => include_bytes!(concat!(
                env!("OUT_DIR"),
                "/wasm32-unknown-unknown/release/test_wasm_post_commit_fail.wasm"
            ))
            .to_vec(),
            TestWasm::Validate => include_bytes!(concat!(
                env!("OUT_DIR"),
                "/wasm32-unknown-unknown/release/test_wasm_validate.wasm"
            ))
            .to_vec(),
            TestWasm::ValidateInvalid => include_bytes!(concat!(
                env!("OUT_DIR"),
                "/wasm32-unknown-unknown/release/test_wasm_validate_invalid.wasm"
            ))
            .to_vec(),
            TestWasm::ValidateValid => include_bytes!(concat!(
                env!("OUT_DIR"),
                "/wasm32-unknown-unknown/release/test_wasm_validate_valid.wasm"
            ))
            .to_vec(),
        })
    }
}
