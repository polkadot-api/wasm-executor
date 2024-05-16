use wasm_bindgen::prelude::*;

use parity_scale_codec::{Decode, Encode};
use smoldot::{
    executor::{
        host::{Config, HeapPages, HostVmPrototype},
        runtime_call::{self, RuntimeCall},
        storage_diff::TrieDiff,
    },
    json_rpc::methods::HexString,
};

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
type HexString = `0x${string}`
/**
 * Get metadata from WASM runtime.
 *
 * @param {HexString} buf
 * @returns {HexString}
 */
export function getMetadataFromRuntime(buf: HexString): HexString"#;

/// Get metadata from WASM runtime.
///
/// @param {HexString} buf
/// @returns {HexString}
#[wasm_bindgen(js_name = getMetadataFromRuntime, skip_typescript, skip_jsdoc)]
pub fn get_metadata(buf: JsValue) -> Result<JsValue, JsValue> {
    let wasm = serde_wasm_bindgen::from_value::<HexString>(buf)?;
    let vm_proto = HostVmPrototype::new(Config {
        module: wasm,
        heap_pages: HeapPages::from(2048),
        exec_hint: smoldot::executor::vm::ExecHint::ValidateAndExecuteOnce,
        allow_unresolved_imports: true,
    })
    .unwrap();

    let params = match run_call(vm_proto.clone(), "Metadata_metadata_versions", Vec::new()) {
        Ok(res) => {
            let avail_versions = Vec::<u32>::decode(&mut &res.0[..]).unwrap();
            if avail_versions.contains(&15) {
                vec![HexString(<u32>::encode(&15))]
            } else {
                vec![HexString(<u32>::encode(&14))]
            }
        }
        Err(_) => {
            // let's try with Metadata_metadata, since not all chains have
            // Metadata_metadata_versions
            let metadata_v14 = run_call(vm_proto.clone(), "Metadata_metadata", Vec::new())?;
            return Ok(serde_wasm_bindgen::to_value(&metadata_v14)?);
        }
    };
    let val = run_call(vm_proto.clone(), "Metadata_metadata_at_version", params)?;

    // @polkadot-api/wasm-executor
    // metadata_at_version returns an optional
    // since we verified that it exists, we remove the first byte
    Ok(serde_wasm_bindgen::to_value(&HexString(
        val.0[1..].to_vec(),
    ))?)
}

fn run_call(
    vm_proto: HostVmPrototype,
    func: &str,
    parameter: Vec<HexString>,
) -> Result<HexString, JsValue> {
    let vm = runtime_call::run(runtime_call::Config {
        virtual_machine: vm_proto,
        function_to_call: func,
        parameter: parameter.into_iter().map(|x| x.0),
        storage_main_trie_changes: TrieDiff::default(),
        max_log_level: 0,
        calculate_trie_changes: false,
    });

    match vm {
        Ok(vm) => match vm {
            RuntimeCall::Finished(res) => match res {
                Ok(res) => Ok(HexString(res.virtual_machine.value().as_ref().to_vec())),
                _ => Err(serde_wasm_bindgen::to_value("Error retrieving data")?),
            },
            _ => Err(serde_wasm_bindgen::to_value("VM execution not expected")?),
        },
        Err((_start_err, _host_vm_proto)) => Err(serde_wasm_bindgen::to_value(
            &format!("Method {func} err").as_str(),
        )?),
    }
}
