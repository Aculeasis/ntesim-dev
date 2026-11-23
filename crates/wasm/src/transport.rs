use serde::Serialize;

use super::dispatch::*;
use super::model::*;
use super::wire;

/// Typed, ownership-safe binding. JSON remains an explicit diagnostic/export
/// format; production clients consume the binary format without JSON parsing.
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn transport(action: &str, input: &str, json: bool) -> Vec<u8> {
    match dispatch_transport(action, input.as_bytes(), json) {
        Ok(bytes) => bytes,
        Err(error) => serialize_transport_error(
            &WasmErrorResponse {
                status: "error".into(),
                action: Some(action.into()),
                error,
            },
            json,
        ),
    }
}

pub fn serialize_transport_error<T: Serialize + ?Sized>(value: &T, json: bool) -> Vec<u8> {
    serialize_transport(value, json).unwrap_or_else(|_| {
        if json {
            TRANSPORT_ERROR_JSON.to_vec()
        } else {
            TRANSPORT_ERROR_BINARY.to_vec()
        }
    })
}

// Last-resort frames cannot themselves fail serialization. Golden tests compare
// both against the real codecs and decode the binary through the JS consumer.
pub const TRANSPORT_ERROR_JSON: &[u8] =
    br#"{"status":"error","error":"internal transport error serialization failed"}"#;
pub const TRANSPORT_ERROR_BINARY: &[u8] = b"NTE\x01\x07\x04\x06\0\0\0status\x04\x05\0\0\0error\x05\x01\0\0\0\x04\x2d\0\0\0internal transport error serialization failed\x08";

pub fn serialize_transport<T: Serialize + ?Sized>(
    value: &T,
    json: bool,
) -> Result<Vec<u8>, String> {
    if json {
        serde_json::to_vec(value)
    } else {
        wire::encode(value)
    }
    .map_err(|error| format!("failed to serialize response: {error}"))
}

pub fn dispatch_transport(action: &str, input: &[u8], json: bool) -> Result<Vec<u8>, String> {
    match action {
        "version" => serialize_transport(&version_info(), json),
        "build_started_at" => serialize_transport(WASM_BUILD_STARTED_AT, json),
        "simulate_game" => with_prepared_game(input, |prepared, seed| {
            serialize_transport(&prepared.run_borrowed(seed)?, json)
        }),
        "simulate_game_batch" => serialize_transport(&dispatch_simulate_game_batch(input)?, json),
        "simulate_game_program" => {
            serialize_transport(&dispatch_simulate_game_program(input)?, json)
        }
        "simulate_game_program_batch" => {
            serialize_transport(&dispatch_simulate_game_program_batch(input)?, json)
        }
        "simulate_rotation" => serialize_transport(&dispatch_simulate_rotation(input)?, json),
        "run_batch" => serialize_transport(&dispatch_run_batch(input)?, json),
        "verify_tabular" => serialize_transport(&dispatch_verify_tabular(input)?, json),
        "enumerate_compositions" => {
            serialize_transport(&dispatch_enumerate_compositions(input)?, json)
        }
        "invoke" => serialize_transport(&dispatch_invoke(input)?, json),
        other => Err(format!("unsupported action: {other}")),
    }
}
