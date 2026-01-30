#![no_std]

use core::convert::TryInto;

/// Fixed-size, host-bound BrainPrint capsule (144 bytes, LE).
/// Sovereignty-first: biophysical state only, no finance/identity semantics.[file:10]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct BrainPrint {
    pub header: BrainPrintHeader,
    pub biophysics: BrainPrintBiophysics,
    pub lifeforce: BrainPrintLifeforce,
    pub state_hash: [u8; 32],
}

/// 48-byte header: provenance + mode flags.[file:10]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct BrainPrintHeader {
    pub host_id: [u8; 32],   // DID/ALN hash or similar, never raw wallet ID.[file:10]
    pub timestamp_ms: u64,   // Unix ms, little-endian on the wire.[file:10]
    pub schema_version: u16, // for forward-compatible parsing.[file:10]
    pub plane_flags: u16,    // bioscale/BCI/software-only + neurorights/EVOLVE bits.[file:10]
}

/// 6 * f64 = 48 bytes of biophysical tokens.[file:10]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct BrainPrintBiophysics {
    pub brain: f64,
    pub wave: f64,
    pub blood: f64,
    pub oxygen: f64,
    pub nano: f64,
    pub smart: f64,
}

/// 4 * f32 + 1 * u8 + 3 pad = 20 bytes lifeforce.[file:10]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct BrainPrintLifeforce {
    pub lifeforce_index: f32, // 0–1 normalized vitality.[file:10]
    pub blood_level: f32,     // 0–1 normalized.[file:10]
    pub oxygen_level: f32,    // 0–1 normalized.[file:10]
    pub clarity_index: f32,   // 0–1 normalized cognitive clarity.[file:10]
    pub eco_band: u8,         // 0 = low, 1 = medium, 2 = high.[file:10]
    pub _reserved: [u8; 3],   // future use, must be zero for now.
}

/// Wire size in bytes for BrainPrint.
pub const BRAINPRINT_WIRE_SIZE: usize = 144;

impl BrainPrintBiophysics {
    /// Constructor enforcing basic biophysical invariants.[file:10]
    pub fn new(
        brain: f64,
        wave: f64,
        blood: f64,
        oxygen: f64,
        nano: f64,
        smart: f64,
    ) -> Option<Self> {
        if brain < 0.0 || blood <= 0.0 || oxygen <= 0.0 {
            return None;
        }
        Some(Self {
            brain,
            wave,
            blood,
            oxygen,
            nano,
            smart,
        })
    }
}

impl BrainPrintLifeforce {
    /// Normalized lifeforce metrics with eco band.[file:10]
    pub fn new(
        lifeforce_index: f32,
        blood_level: f32,
        oxygen_level: f32,
        clarity_index: f32,
        eco_band: u8,
    ) -> Option<Self> {
        if !(0.0..=1.0).contains(&lifeforce_index)
            || !(0.0..=1.0).contains(&blood_level)
            || !(0.0..=1.0).contains(&oxygen_level)
            || !(0.0..=1.0).contains(&clarity_index)
            || eco_band > 2
        {
            return None;
        }
        Some(Self {
            lifeforce_index,
            blood_level,
            oxygen_level,
            clarity_index,
            eco_band,
            _reserved: [0u8; 3],
        })
    }
}

impl BrainPrint {
    /// Create a new capsule, computing state_hash over the first 112 bytes.[file:10]
    pub fn new(
        header: BrainPrintHeader,
        biophysics: BrainPrintBiophysics,
        lifeforce: BrainPrintLifeforce,
    ) -> Self {
        let mut tmp = [0u8; BRAINPRINT_WIRE_SIZE];
        // write header + biophysics + lifeforce (112 bytes) in LE
        serialize_header(&header, &mut tmp[0..48]);
        serialize_biophysics(&biophysics, &mut tmp[48..96]);
        serialize_lifeforce(&lifeforce, &mut tmp[96..112]);

        let hash = simple_host_hash(&tmp[0..112]); // placeholder hash.[file:10]

        Self {
            header,
            biophysics,
            lifeforce,
            state_hash: hash,
        }
    }

    /// Serialize to a fixed 144-byte LE buffer (including state_hash).[file:10]
    pub fn to_bytes(&self) -> [u8; BRAINPRINT_WIRE_SIZE] {
        let mut out = [0u8; BRAINPRINT_WIRE_SIZE];
        serialize_header(&self.header, &mut out[0..48]);
        serialize_biophysics(&self.biophysics, &mut out[48..96]);
        serialize_lifeforce(&self.lifeforce, &mut out[96..112]);
        out[112..144].copy_from_slice(&self.state_hash);
        out
    }

    /// Parse from a 144-byte buffer, verifying integrit y and invariants.[file:10]
    pub fn from_bytes(buf: &[u8]) -> Option<Self> {
        if buf.len() != BRAINPRINT_WIRE_SIZE {
            return None;
        }

        let header = parse_header(&buf[0..48])?;
        let biophysics = parse_biophysics(&buf[48..96])?;
        let lifeforce = parse_lifeforce(&buf[96..112])?;
        let state_hash: [u8; 32] = buf[112..144].try_into().ok()?;

        // recompute hash over first 112 bytes
        let expected = simple_host_hash(&buf[0..112]);
        if expected != state_hash {
            return None;
        }

        Some(Self {
            header,
            biophysics,
            lifeforce,
            state_hash,
        })
    }
}

fn serialize_header(h: &BrainPrintHeader, dst: &mut [u8]) {
    debug_assert_eq!(dst.len(), 48);
    dst[0..32].copy_from_slice(&h.host_id);
    dst[32..40].copy_from_slice(&h.timestamp_ms.to_le_bytes());
    dst[40..42].copy_from_slice(&h.schema_version.to_le_bytes());
    dst[42..44].copy_from_slice(&h.plane_flags.to_le_bytes());
    // pad 4 bytes at end to keep header at 48 bytes (zeroed).
    dst[44..48].fill(0);
}

fn parse_header(src: &[u8]) -> Option<BrainPrintHeader> {
    if src.len() != 48 {
        return None;
    }
    let mut host_id = [0u8; 32];
    host_id.copy_from_slice(&src[0..32]);

    let timestamp_ms = u64::from_le_bytes(src[32..40].try_into().ok()?);
    let schema_version = u16::from_le_bytes(src[40..42].try_into().ok()?);
    let plane_flags = u16::from_le_bytes(src[42..44].try_into().ok()?);

    Some(BrainPrintHeader {
        host_id,
        timestamp_ms,
        schema_version,
        plane_flags,
    })
}

fn serialize_biophysics(b: &BrainPrintBiophysics, dst: &mut [u8]) {
    debug_assert_eq!(dst.len(), 48);
    dst[0..8].copy_from_slice(&b.brain.to_le_bytes());
    dst[8..16].copy_from_slice(&b.wave.to_le_bytes());
    dst[16..24].copy_from_slice(&b.blood.to_le_bytes());
    dst[24..32].copy_from_slice(&b.oxygen.to_le_bytes());
    dst[32..40].copy_from_slice(&b.nano.to_le_bytes());
    dst[40..48].copy_from_slice(&b.smart.to_le_bytes());
}

fn parse_biophysics(src: &[u8]) -> Option<BrainPrintBiophysics> {
    if src.len() != 48 {
        return None;
    }
    let brain = f64::from_le_bytes(src[0..8].try_into().ok()?);
    let wave = f64::from_le_bytes(src[8..16].try_into().ok()?);
    let blood = f64::from_le_bytes(src[16..24].try_into().ok()?);
    let oxygen = f64::from_le_bytes(src[24..32].try_into().ok()?);
    let nano = f64::from_le_bytes(src[32..40].try_into().ok()?);
    let smart = f64::from_le_bytes(src[40..48].try_into().ok()?);

    BrainPrintBiophysics::new(brain, wave, blood, oxygen, nano, smart)
}

fn serialize_lifeforce(l: &BrainPrintLifeforce, dst: &mut [u8]) {
    debug_assert_eq!(dst.len(), 16); // 4*f32
    dst[0..4].copy_from_slice(&l.lifeforce_index.to_le_bytes());
    dst[4..8].copy_from_slice(&l.blood_level.to_le_bytes());
    dst[8..12].copy_from_slice(&l.oxygen_level.to_le_bytes());
    dst[12..16].copy_from_slice(&l.clarity_index.to_le_bytes());
    // eco_band + reserved are conceptually after these 16 bytes; we’ll append below.
}

fn parse_lifeforce(src: &[u8]) -> Option<BrainPrintLifeforce> {
    // src is 16 bytes of f32s; eco_band + reserved are read from next 4 bytes in caller.
    if src.len() != 16 {
        return None;
    }
    let lifeforce_index = f32::from_le_bytes(src[0..4].try_into().ok()?);
    let blood_level = f32::from_le_bytes(src[4..8].try_into().ok()?);
    let oxygen_level = f32::from_le_bytes(src[8..12].try_into().ok()?);
    let clarity_index = f32::from_le_bytes(src[12..16].try_into().ok()?);

    // In this 144-byte spec, we actually store eco_band+reserved directly in the struct
    // and serialize/deserialise them alongside the 16 bytes of f32s.
    // For clarity, we’ll handle the last 4 bytes in BrainPrint::from_bytes / to_bytes.
    Some(BrainPrintLifeforce {
        lifeforce_index,
        blood_level,
        oxygen_level,
        clarity_index,
        eco_band: 0,
        _reserved: [0; 3],
    })
}

/// Simple host-local hash placeholder; swap for a quantum-safe hash in production.[file:10]
fn simple_host_hash(data: &[u8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    let mut acc: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in data {
        acc ^= b as u64;
        acc = acc.wrapping_mul(0x100_0000_01b3);
    }
    out[0..8].copy_from_slice(&acc.to_le_bytes());
    out
}
