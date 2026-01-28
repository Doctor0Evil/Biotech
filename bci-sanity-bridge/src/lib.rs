use wasm_bindgen::prelude::*;
use crate::types::{PsychoEnvSnapshot, NanoRadsSnapshot};
use crate::mapper::map_bci_to_sanity_decay;
use crate::rads_mapper::{map_eeg_telemetry_to_rads_nano, apply_rads_nano_policy};
use biophysical_blockchain::types::{SystemAdjustment, LifeforceBand};
use bci_bioledger_bridge::{BciEvent, BciLedgerClient};

mod types;
mod mapper;
mod rads_mapper;

/// Single-shot EEG + telemetry → SANITY/DECAY/RADS/NANO adjustment.
/// Called from JS (Perplexity/Gemini/Copilot/Grok tool, etc.) but
/// only executes on host-side with valid DID/role at the ledger layer.
#[wasm_bindgen]
pub fn bridge_eeg_to_bioledger(
    bci_event_json: &str,
    psycho_snap_json: &str,
    nano_snap_json: &str,
    lifeforce_band_str: &str,
) -> Result<JsValue, JsValue> {
    let event: BciEvent = serde_json::from_str(bci_event_json)
        .map_err(|e| JsValue::from_str(&format!("bad BciEvent: {e}")))?;
    let psycho: PsychoEnvSnapshot = serde_json::from_str(psycho_snap_json)
        .map_err(|e| JsValue::from_str(&format!("bad PsychoEnvSnapshot: {e}")))?;
    let nano_snap: NanoRadsSnapshot = serde_json::from_str(nano_snap_json)
        .map_err(|e| JsValue::from_str(&format!("bad NanoRadsSnapshot: {e}")))?;

    let lifeforce_band = match lifeforce_band_str {
        "Safe" => LifeforceBand::Safe,
        "SoftWarn" => LifeforceBand::SoftWarn,
        "HardStop" => LifeforceBand::HardStop,
        _ => return Err(JsValue::from_str("invalid LifeforceBand")),
    };

    // 1. Map EEG → SANITY / DECAY.
    let (delta_sanity, delta_decay) = map_bci_to_sanity_decay(&event, &psycho);

    // 2. Base SystemAdjustment (no RADS/NANO yet).
    let mut base = SystemAdjustment::zero();
    base.reason = "EEG_SANITY_DECAY_RADS_NANO".to_string();
    base.deltabrain = 0.0;
    base.deltawave = 0.0;
    base.deltablood = 0.0;
    base.deltaoxygen = 0.0;
    base.deltanano = event.nano_load_delta; // from classifier/pipeline
    base.deltasmart = 0.0;
    base.ecocost = 0.0;

    // Encode SANITY/DECAY into inner-ledger friendly deltas (host-specific mapping).
    base = crate::mapper::fold_sanity_decay_into_adjustment(
        base,
        delta_sanity,
        delta_decay,
    );

    // 3. Map EEG + telemetry → RADS / NANO routing.
    let rads_policy = map_eeg_telemetry_to_rads_nano(
        &event,
        &nano_snap,
        lifeforce_band,
    );

    // 4. Apply NANO scaling and RADS ecocost before submit.
    let final_adj = apply_rads_nano_policy(&base, &rads_policy);

    // 5. Submit to host-local biophysical ledger (DID/role enforced down-stack).
    let client = BciLedgerClient::default();
    let result = client.submit_adjustment(final_adj)
        .map_err(|e| JsValue::from_str(&format!("ledger error: {e}")))?;

    JsValue::from_serde(&result).map_err(|e| JsValue::from_str(&format!("{e}")))
}
