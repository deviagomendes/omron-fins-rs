//! Bindings napi-rs para expor a biblioteca omron-fins ao ecossistema Node.js / Bun.
//!
//! Este módulo cria uma camada de tradução entre os tipos Rust e os tipos JavaScript,
//! expondo todas as funcionalidades do cliente FINS como funções assíncronas (Promises).

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Arc;

use crate::client::{Client, ClientConfig};
use crate::command::{ForceSpec, ForcedBit, MultiReadSpec, PlcMode};
use crate::memory::MemoryArea;
use crate::types::{DataType, PlcValue};

// ─── Enums exportados para TypeScript ──────────────────────────────

/// Memory areas supported by Omron PLCs.
#[napi]
pub enum FinsMemoryArea {
    CIO,
    WR,
    HR,
    DM,
    AR,
}

/// Data types supported by Omron PLCs.
#[napi]
pub enum FinsDataType {
    USINT,
    UINT,
    UDINT,
    ULINT,
    SINT,
    INT,
    DINT,
    LINT,
    REAL,
    LREAL,
    WORD,
    DWORD,
    LWORD,
}

/// PLC operating modes.
#[napi]
pub enum FinsPlcMode {
    Debug,
    Monitor,
    Run,
}

/// Force operation specifications.
#[napi]
pub enum FinsForceSpec {
    ForceOn,
    ForceOff,
    Release,
}

// ─── Conversores de Enum ───────────────────────────────────────────

/// Converte FinsMemoryArea para MemoryArea interno.
fn memory_area_from_enum(area: FinsMemoryArea) -> MemoryArea {
    match area {
        FinsMemoryArea::CIO => MemoryArea::CIO,
        FinsMemoryArea::WR => MemoryArea::WR,
        FinsMemoryArea::HR => MemoryArea::HR,
        FinsMemoryArea::DM => MemoryArea::DM,
        FinsMemoryArea::AR => MemoryArea::AR,
    }
}

/// Converte string JS para MemoryArea Rust.
fn parse_memory_area(area: &str) -> Result<MemoryArea> {
    match area.to_uppercase().as_str() {
        "CIO" => Ok(MemoryArea::CIO),
        "WR" => Ok(MemoryArea::WR),
        "HR" => Ok(MemoryArea::HR),
        "DM" => Ok(MemoryArea::DM),
        "AR" => Ok(MemoryArea::AR),
        _ => Err(Error::from_reason(format!(
            "Área de memória inválida: '{}'. Valores válidos: CIO, WR, HR, DM, AR",
            area
        ))),
    }
}

/// Converte FinsMemoryArea ou string para MemoryArea interno.
fn parse_memory_area_input(area: Either<FinsMemoryArea, String>) -> Result<MemoryArea> {
    match area {
        Either::A(enum_area) => Ok(memory_area_from_enum(enum_area)),
        Either::B(string_area) => parse_memory_area(&string_area),
    }
}

/// Converte FinsPlcMode para PlcMode interno.
fn plc_mode_from_enum(mode: FinsPlcMode) -> PlcMode {
    match mode {
        FinsPlcMode::Debug => PlcMode::Debug,
        FinsPlcMode::Monitor => PlcMode::Monitor,
        FinsPlcMode::Run => PlcMode::Run,
    }
}

/// Converte string JS para PlcMode Rust.
fn parse_plc_mode(mode: &str) -> Result<PlcMode> {
    match mode.to_lowercase().as_str() {
        "debug" => Ok(PlcMode::Debug),
        "monitor" => Ok(PlcMode::Monitor),
        "run" => Ok(PlcMode::Run),
        _ => Err(Error::from_reason(format!(
            "Modo PLC inválido: '{}'. Valores válidos: debug, monitor, run",
            mode
        ))),
    }
}

/// Converte FinsPlcMode ou string para PlcMode interno.
fn parse_plc_mode_input(mode: Either<FinsPlcMode, String>) -> Result<PlcMode> {
    match mode {
        Either::A(enum_mode) => Ok(plc_mode_from_enum(enum_mode)),
        Either::B(string_mode) => parse_plc_mode(&string_mode),
    }
}

/// Converte FinsForceSpec para ForceSpec interno.
fn force_spec_from_enum(spec: FinsForceSpec) -> ForceSpec {
    match spec {
        FinsForceSpec::ForceOn => ForceSpec::ForceOn,
        FinsForceSpec::ForceOff => ForceSpec::ForceOff,
        FinsForceSpec::Release => ForceSpec::Release,
    }
}

/// Converte string JS para ForceSpec Rust.
fn parse_force_spec(spec: &str) -> Result<ForceSpec> {
    match spec.to_lowercase().as_str() {
        "force_on" | "forceon" | "on" => Ok(ForceSpec::ForceOn),
        "force_off" | "forceoff" | "off" => Ok(ForceSpec::ForceOff),
        "release" => Ok(ForceSpec::Release),
        _ => Err(Error::from_reason(format!(
            "ForceSpec inválido: '{}'. Valores válidos: force_on, force_off, release",
            spec
        ))),
    }
}

/// Converte FinsForceSpec ou string para ForceSpec interno.
fn parse_force_spec_input(spec: Either<FinsForceSpec, String>) -> Result<ForceSpec> {
    match spec {
        Either::A(enum_spec) => Ok(force_spec_from_enum(enum_spec)),
        Either::B(string_spec) => parse_force_spec(&string_spec),
    }
}

/// Converte FinsDataType para DataType interno.
fn data_type_from_enum(t: FinsDataType) -> DataType {
    match t {
        FinsDataType::USINT => DataType::USINT,
        FinsDataType::UINT => DataType::UINT,
        FinsDataType::UDINT => DataType::UDINT,
        FinsDataType::ULINT => DataType::ULINT,
        FinsDataType::SINT => DataType::SINT,
        FinsDataType::INT => DataType::INT,
        FinsDataType::DINT => DataType::DINT,
        FinsDataType::LINT => DataType::LINT,
        FinsDataType::REAL => DataType::REAL,
        FinsDataType::LREAL => DataType::LREAL,
        FinsDataType::WORD => DataType::WORD,
        FinsDataType::DWORD => DataType::DWORD,
        FinsDataType::LWORD => DataType::LWORD,
    }
}

/// Converte string JS para DataType Rust.
fn parse_data_type(t: &str) -> Result<DataType> {
    match t.to_uppercase().as_str() {
        "USINT" => Ok(DataType::USINT),
        "UINT" => Ok(DataType::UINT),
        "UDINT" => Ok(DataType::UDINT),
        "ULINT" => Ok(DataType::ULINT),
        "SINT" => Ok(DataType::SINT),
        "INT" => Ok(DataType::INT),
        "DINT" => Ok(DataType::DINT),
        "LINT" => Ok(DataType::LINT),
        "REAL" => Ok(DataType::REAL),
        "LREAL" => Ok(DataType::LREAL),
        "WORD" => Ok(DataType::WORD),
        "DWORD" => Ok(DataType::DWORD),
        "LWORD" => Ok(DataType::LWORD),
        _ => Err(Error::from_reason(format!("Tipo de dado inválido: '{}'", t))),
    }
}

/// Converte FinsDataType ou string para DataType interno.
fn parse_data_type_input(t: Either<FinsDataType, String>) -> Result<DataType> {
    match t {
        Either::A(enum_type) => Ok(data_type_from_enum(enum_type)),
        Either::B(string_type) => parse_data_type(&string_type),
    }
}

/// Converte FinsError do Rust para napi::Error do JS.
fn fins_to_js_error(e: crate::error::FinsError) -> Error {
    Error::from_reason(e.to_string())
}

// ─── Objetos JS para inputs complexos ──────────────────────────────

/// Forced bit specification (JS input).
#[napi(object)]
pub struct JsForcedBit {
    /// Memory area: FinsMemoryArea or string (e.g., "CIO", "WR", "HR", "AR")
    pub area: String,
    /// Word address
    pub address: u16,
    /// Bit position (0-15)
    pub bit: u8,
    /// Specification: FinsForceSpec or string (e.g., "force_on", "force_off", "release")
    pub spec: String,
}

/// Multi-read specification (JS input).
#[napi(object)]
pub struct JsMultiReadSpec {
    /// Memory area: FinsMemoryArea or string (e.g., "CIO", "WR", "HR", "DM", "AR")
    pub area: String,
    /// Word address
    pub address: u16,
    /// Bit position (optional, null/undefined for word read)
    pub bit: Option<u8>,
}

/// Representation of a PLC value for JavaScript.
#[napi(object)]
pub struct JsPlcValue {
    /// Data type: FinsDataType or string (e.g., "INT", "REAL", etc.)
    pub r#type: String,
    /// Value (represented as JSON string to avoid type issues)
    pub value: String,
}

/// Status and configuration of an Omron PLC (JS output).
#[napi(object)]
pub struct JsControllerStatus {
    /// Operating mode: "debug", "monitor", or "run"
    pub mode: String,
    /// Indicates if a fatal error exists
    pub fatal_error: bool,
    /// Indicates if a non-fatal error exists
    pub non_fatal_error: bool,
    /// Raw fatal error flags
    pub fatal_error_data: u16,
    /// Raw non-fatal error flags
    pub non_fatal_error_data: u16,
}

// ─── Constantes exportadas ─────────────────────────────────────────

/// Default FINS protocol UDP port.
#[napi]
pub const DEFAULT_FINS_PORT: u16 = crate::transport::DEFAULT_FINS_PORT;

/// Maximum FINS packet size.
#[napi]
pub const MAX_PACKET_SIZE: u16 = crate::transport::MAX_PACKET_SIZE as u16;

/// Maximum words per command.
#[napi]
pub const MAX_WORDS_PER_COMMAND: u16 = crate::command::MAX_WORDS_PER_COMMAND;

// ─── Cliente FINS (classe principal) ───────────────────────────────

/// FINS client for communication with Omron PLCs.
///
/// Each operation generates exactly 1 UDP request and 1 UDP response.
/// No automatic retries, caching, or reconnection.
#[napi]
pub struct FinsClient {
    inner: Arc<Client>,
}

#[napi]
impl FinsClient {
    /// Creates a new FINS client.
    ///
    /// @param host - PLC IP address (e.g., "192.168.1.250")
    /// @param sourceNode - Source node number (this client)
    /// @param destNode - Destination node number (the PLC)
    /// @param options - Advanced options (optional)
    #[napi(constructor)]
    pub fn new(
        host: String,
        source_node: u8,
        dest_node: u8,
        options: Option<JsClientOptions>,
    ) -> Result<Self> {
        let ip: std::net::Ipv4Addr = host
            .parse()
            .map_err(|e| Error::from_reason(format!("IP inválido '{}': {}", host, e)))?;

        let mut config = ClientConfig::new(ip, source_node, dest_node);

        if let Some(opts) = options {
            if let Some(port) = opts.port {
                config = config.with_port(port);
            }
            if let Some(timeout_ms) = opts.timeout_ms {
                config =
                    config.with_timeout(std::time::Duration::from_millis(timeout_ms as u64));
            }
            if let Some(src_network) = opts.source_network {
                config = config.with_source_network(src_network);
            }
            if let Some(src_unit) = opts.source_unit {
                config = config.with_source_unit(src_unit);
            }
            if let Some(dst_network) = opts.dest_network {
                config = config.with_dest_network(dst_network);
            }
            if let Some(dst_unit) = opts.dest_unit {
                config = config.with_dest_unit(dst_unit);
            }
        }

        let client = Client::new(config).map_err(fins_to_js_error)?;
        Ok(Self {
            inner: Arc::new(client),
        })
    }

    // ─── Leitura / Escrita de Words ────────────────────────────────

    /// Reads words from PLC memory (asynchronous).
    ///
    /// @param area - Memory area (FinsMemoryArea or string)
    /// @param address - Start address
    /// @param count - Number of words to read (1-999)
    /// @returns Promise<number[]> - u16 values array
    #[napi]
    pub async fn read(
        &self,
        area: Either<FinsMemoryArea, String>,
        address: u16,
        count: u16,
    ) -> Result<Vec<u32>> {
        let mem_area = parse_memory_area_input(area)?;
        let client = self.inner.clone();

        let result = tokio::task::spawn_blocking(move || client.read(mem_area, address, count))
            .await
            .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
            .map_err(fins_to_js_error)?;

        Ok(result.into_iter().map(|v| v as u32).collect())
    }

    /// Writes words to PLC memory (asynchronous).
    ///
    /// @param area - Memory area (FinsMemoryArea or string)
    /// @param address - Start address
    /// @param data - Array of u16 values to write
    #[napi]
    pub async fn write(
        &self,
        area: Either<FinsMemoryArea, String>,
        address: u16,
        data: Vec<u32>,
    ) -> Result<()> {
        let mem_area = parse_memory_area_input(area)?;
        let words: Vec<u16> = data.into_iter().map(|v| v as u16).collect();
        let client = self.inner.clone();

        tokio::task::spawn_blocking(move || client.write(mem_area, address, &words))
            .await
            .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
            .map_err(fins_to_js_error)
    }

    // ─── Leitura / Escrita de Bits ─────────────────────────────────

    /// Reads a bit from PLC memory (asynchronous).
    ///
    /// @param area - Memory area (FinsMemoryArea or string)
    /// @param address - Word address
    /// @param bit - Bit position (0-15)
    /// @returns Promise<boolean>
    #[napi]
    pub async fn read_bit(
        &self,
        area: Either<FinsMemoryArea, String>,
        address: u16,
        bit: u8,
    ) -> Result<bool> {
        let mem_area = parse_memory_area_input(area)?;
        let client = self.inner.clone();

        tokio::task::spawn_blocking(move || client.read_bit(mem_area, address, bit))
            .await
            .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
            .map_err(fins_to_js_error)
    }

    /// Writes a bit to PLC memory (asynchronous).
    ///
    /// @param area - Memory area (FinsMemoryArea or string)
    /// @param address - Word address
    /// @param bit - Bit position (0-15)
    /// @param value - Bit value (true/false)
    #[napi]
    pub async fn write_bit(
        &self,
        area: Either<FinsMemoryArea, String>,
        address: u16,
        bit: u8,
        value: bool,
    ) -> Result<()> {
        let mem_area = parse_memory_area_input(area)?;
        let client = self.inner.clone();

        tokio::task::spawn_blocking(move || client.write_bit(mem_area, address, bit, value))
            .await
            .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
            .map_err(fins_to_js_error)
    }

    // ─── Fill ──────────────────────────────────────────────────────

    /// Fills a memory region with a value (asynchronous).
    ///
    /// @param area - Memory area (FinsMemoryArea or string)
    /// @param address - Start address
    /// @param count - Number of words to fill (1-999)
    /// @param value - u16 value to fill
    #[napi]
    pub async fn fill(
        &self,
        area: Either<FinsMemoryArea, String>,
        address: u16,
        count: u16,
        value: u32,
    ) -> Result<()> {
        let mem_area = parse_memory_area_input(area)?;
        let val = value as u16;
        let client = self.inner.clone();

        tokio::task::spawn_blocking(move || client.fill(mem_area, address, count, val))
            .await
            .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
            .map_err(fins_to_js_error)
    }

    // ─── Transfer ──────────────────────────────────────────────────

    /// Transfers data from one memory area to another (asynchronous).
    ///
    /// @param srcArea - Source memory area (FinsMemoryArea or string)
    /// @param srcAddress - Source address
    /// @param dstArea - Destination memory area (FinsMemoryArea or string)
    /// @param dstAddress - Destination address
    /// @param count - Number of words to transfer
    #[napi]
    pub async fn transfer(
        &self,
        src_area: Either<FinsMemoryArea, String>,
        src_address: u16,
        dst_area: Either<FinsMemoryArea, String>,
        dst_address: u16,
        count: u16,
    ) -> Result<()> {
        let src = parse_memory_area_input(src_area)?;
        let dst = parse_memory_area_input(dst_area)?;
        let client = self.inner.clone();

        tokio::task::spawn_blocking(move || {
            client.transfer(src, src_address, dst, dst_address, count)
        })
        .await
        .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
        .map_err(fins_to_js_error)
    }

    // ─── Run / Stop ────────────────────────────────────────────────

    /// Sets the PLC to execution mode (asynchronous).
    ///
    /// @param mode - Mode (FinsPlcMode or string: "debug", "monitor", "run")
    #[napi]
    pub async fn run(&self, mode: Either<FinsPlcMode, String>) -> Result<()> {
        let plc_mode = parse_plc_mode_input(mode)?;
        let client = self.inner.clone();

        tokio::task::spawn_blocking(move || client.run(plc_mode))
            .await
            .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
            .map_err(fins_to_js_error)
    }

    /// Stops the PLC (asynchronous).
    #[napi]
    pub async fn stop(&self) -> Result<()> {
        let client = self.inner.clone();

        tokio::task::spawn_blocking(move || client.stop())
            .await
            .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
            .map_err(fins_to_js_error)
    }

    // ─── Forced Set/Reset ──────────────────────────────────────────

    /// Forces bits ON/OFF in the PLC (asynchronous).
    ///
    /// @param specs - Array of forced bit specifications
    #[napi]
    pub async fn forced_set_reset(&self, specs: Vec<JsForcedBit>) -> Result<()> {
        let forced_bits: std::result::Result<Vec<ForcedBit>, Error> = specs
            .into_iter()
            .map(|s| {
                Ok(ForcedBit {
                    area: parse_memory_area(&s.area)?,
                    address: s.address,
                    bit: s.bit,
                    spec: parse_force_spec(&s.spec)?,
                })
            })
            .collect();
        let bits = forced_bits?;
        let client = self.inner.clone();

        tokio::task::spawn_blocking(move || client.forced_set_reset(&bits))
            .await
            .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
            .map_err(fins_to_js_error)
    }

    /// Cancels all forced bits in the PLC (asynchronous).
    #[napi]
    pub async fn forced_set_reset_cancel(&self) -> Result<()> {
        let client = self.inner.clone();

        tokio::task::spawn_blocking(move || client.forced_set_reset_cancel())
            .await
            .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
            .map_err(fins_to_js_error)
    }

    // ─── Multiple Read ─────────────────────────────────────────────

    /// Reads from multiple memory areas in a single request (asynchronous).
    ///
    /// @param specs - Array of read specifications
    /// @returns Promise<number[]> - u16 values array
    #[napi]
    pub async fn read_multiple(&self, specs: Vec<JsMultiReadSpec>) -> Result<Vec<u32>> {
        let read_specs: std::result::Result<Vec<MultiReadSpec>, Error> = specs
            .into_iter()
            .map(|s| {
                Ok(MultiReadSpec {
                    area: parse_memory_area(&s.area)?,
                    address: s.address,
                    bit: s.bit,
                })
            })
            .collect();
        let rs = read_specs?;
        let client = self.inner.clone();

        let result =
            tokio::task::spawn_blocking(move || client.read_multiple(&rs))
                .await
                .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
                .map_err(fins_to_js_error)?;

        Ok(result.into_iter().map(|v| v as u32).collect())
    }

    // ─── Type Helpers ──────────────────────────────────────────────

    /// Reads an f32 (REAL) value from 2 consecutive words (asynchronous).
    ///
    /// @param area - Memory area (FinsMemoryArea or string)
    /// @param address - Word start address
    /// @returns Promise<number>
    #[napi]
    pub async fn read_f32(
        &self,
        area: Either<FinsMemoryArea, String>,
        address: u16,
    ) -> Result<f64> {
        let mem_area = parse_memory_area_input(area)?;
        let client = self.inner.clone();

        let result =
            tokio::task::spawn_blocking(move || client.read_f32(mem_area, address))
                .await
                .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
                .map_err(fins_to_js_error)?;

        Ok(result as f64)
    }

    /// Writes an f32 (REAL) value to 2 consecutive words (asynchronous).
    ///
    /// @param area - Memory area (FinsMemoryArea or string)
    /// @param address - Word start address
    /// @param value - f32 value to write
    #[napi]
    pub async fn write_f32(
        &self,
        area: Either<FinsMemoryArea, String>,
        address: u16,
        value: f64,
    ) -> Result<()> {
        let mem_area = parse_memory_area_input(area)?;
        let val = value as f32;
        let client = self.inner.clone();

        tokio::task::spawn_blocking(move || client.write_f32(mem_area, address, val))
            .await
            .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
            .map_err(fins_to_js_error)
    }

    /// Reads an f64 (LREAL) value from 4 consecutive words (asynchronous).
    ///
    /// @param area - Memory area (FinsMemoryArea or string)
    /// @param address - Word start address
    /// @returns Promise<number>
    #[napi]
    pub async fn read_f64(
        &self,
        area: Either<FinsMemoryArea, String>,
        address: u16,
    ) -> Result<f64> {
        let mem_area = parse_memory_area_input(area)?;
        let client = self.inner.clone();

        tokio::task::spawn_blocking(move || client.read_f64(mem_area, address))
            .await
            .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
            .map_err(fins_to_js_error)
    }

    /// Writes an f64 (LREAL) value to 4 consecutive words (asynchronous).
    ///
    /// @param area - Memory area (FinsMemoryArea or string)
    /// @param address - Word start address
    /// @param value - f64 value to write
    #[napi]
    pub async fn write_f64(
        &self,
        area: Either<FinsMemoryArea, String>,
        address: u16,
        value: f64,
    ) -> Result<()> {
        let mem_area = parse_memory_area_input(area)?;
        let client = self.inner.clone();

        tokio::task::spawn_blocking(move || client.write_f64(mem_area, address, value))
            .await
            .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
            .map_err(fins_to_js_error)
    }

    /// Reads an i32 (DINT) value from 2 consecutive words (asynchronous).
    ///
    /// @param area - Memory area (FinsMemoryArea or string)
    /// @param address - Word start address
    /// @returns Promise<number>
    #[napi]
    pub async fn read_i32(
        &self,
        area: Either<FinsMemoryArea, String>,
        address: u16,
    ) -> Result<i32> {
        let mem_area = parse_memory_area_input(area)?;
        let client = self.inner.clone();

        tokio::task::spawn_blocking(move || client.read_i32(mem_area, address))
            .await
            .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
            .map_err(fins_to_js_error)
    }

    /// Writes an i32 (DINT) value to 2 consecutive words (asynchronous).
    ///
    /// @param area - Memory area (FinsMemoryArea or string)
    /// @param address - Word start address
    /// @param value - i32 value to write
    #[napi]
    pub async fn write_i32(
        &self,
        area: Either<FinsMemoryArea, String>,
        address: u16,
        value: i32,
    ) -> Result<()> {
        let mem_area = parse_memory_area_input(area)?;
        let client = self.inner.clone();

        tokio::task::spawn_blocking(move || client.write_i32(mem_area, address, value))
            .await
            .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
            .map_err(fins_to_js_error)
    }

    // ─── Strings ───────────────────────────────────────────────────

    /// Writes an ASCII string to PLC memory (asynchronous).
    ///
    /// @param area - Memory area (FinsMemoryArea or string)
    /// @param address - Word start address
    /// @param value - ASCII string
    #[napi]
    pub async fn write_string(
        &self,
        area: Either<FinsMemoryArea, String>,
        address: u16,
        value: String,
    ) -> Result<()> {
        let mem_area = parse_memory_area_input(area)?;
        let client = self.inner.clone();

        tokio::task::spawn_blocking(move || client.write_string(mem_area, address, &value))
            .await
            .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
            .map_err(fins_to_js_error)
    }

    /// Reads an ASCII string from PLC memory (asynchronous).
    ///
    /// @param area - Memory area (FinsMemoryArea or string)
    /// @param address - Word start address
    /// @param wordCount - Number of words to read (1 word = 2 characters)
    /// @returns Promise<string>
    #[napi]
    pub async fn read_string(
        &self,
        area: Either<FinsMemoryArea, String>,
        address: u16,
        word_count: u16,
    ) -> Result<String> {
        let mem_area = parse_memory_area_input(area)?;
        let client = self.inner.clone();

        tokio::task::spawn_blocking(move || client.read_string(mem_area, address, word_count))
            .await
            .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
            .map_err(fins_to_js_error)
    }

    // ─── Structs ───────────────────────────────────────────────────

    /// Reads a structure from PLC memory (asynchronous).
    ///
    /// @param area - Memory area (FinsMemoryArea or string)
    /// @param address - Word start address
    /// @param types - Array of types (FinsDataType or string)
    /// @returns Promise<JsPlcValue[]>
    #[napi]
    pub async fn read_struct(
        &self,
        area: Either<FinsMemoryArea, String>,
        address: u16,
        types: Vec<Either<FinsDataType, String>>,
    ) -> Result<Vec<JsPlcValue>> {
        let mem_area = parse_memory_area_input(area)?;
        let rust_types: std::result::Result<Vec<DataType>, Error> =
            types.iter().map(|t| parse_data_type_input(t.clone())).collect();
        let ts = rust_types?;
        let client = self.inner.clone();

        let results = tokio::task::spawn_blocking(move || client.read_struct(mem_area, address, ts))
            .await
            .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
            .map_err(fins_to_js_error)?;

        Ok(results
            .into_iter()
            .map(|v| {
                let val_json = match v {
                    PlcValue::USint(x) => serde_json::to_string(&x).unwrap(),
                    PlcValue::Uint(x) => serde_json::to_string(&x).unwrap(),
                    PlcValue::Word(x) => serde_json::to_string(&x).unwrap(),
                    PlcValue::Udint(x) => serde_json::to_string(&x).unwrap(),
                    PlcValue::Dword(x) => serde_json::to_string(&x).unwrap(),
                    PlcValue::Ulint(x) => serde_json::to_string(&x.to_string()).unwrap(),
                    PlcValue::Lword(x) => serde_json::to_string(&x.to_string()).unwrap(),
                    PlcValue::Sint(x) => serde_json::to_string(&x).unwrap(),
                    PlcValue::Int(x) => serde_json::to_string(&x).unwrap(),
                    PlcValue::Dint(x) => serde_json::to_string(&x).unwrap(),
                    PlcValue::Lint(x) => serde_json::to_string(&x.to_string()).unwrap(),
                    PlcValue::Real(x) => serde_json::to_string(&x).unwrap(),
                    PlcValue::Lreal(x) => serde_json::to_string(&x).unwrap(),
                };
                JsPlcValue {
                    r#type: format!("{:?}", v.data_type()),
                    value: val_json,
                }
            })
            .collect())
    }

    /// Writes a structure to PLC memory (asynchronous).
    ///
    /// @param area - Memory area (FinsMemoryArea or string)
    /// @param address - Word start address
    /// @param values - Array of objects { type: string, value: any }
    #[napi]
    pub async fn write_struct(
        &self,
        area: Either<FinsMemoryArea, String>,
        address: u16,
        values: Vec<JsPlcValue>,
    ) -> Result<()> {
        let mem_area = parse_memory_area_input(area)?;
        let mut rust_values = Vec::with_capacity(values.len());

        for v in values {
            let data_type = parse_data_type(&v.r#type)?;
            let json_val: serde_json::Value = serde_json::from_str(&v.value)
                .map_err(|e| Error::from_reason(format!("Erro ao parsear JSON '{}': {}", v.value, e)))?;

            let val = match data_type {
                DataType::USINT => PlcValue::USint(json_val.as_u64().unwrap_or(0) as u8),
                DataType::UINT => PlcValue::Uint(json_val.as_u64().unwrap_or(0) as u16),
                DataType::UDINT => PlcValue::Udint(json_val.as_u64().unwrap_or(0) as u32),
                DataType::ULINT => PlcValue::Ulint(
                    json_val
                        .as_str()
                        .map(|s| s.parse().unwrap_or(0))
                        .unwrap_or(json_val.as_u64().unwrap_or(0)),
                ),
                DataType::SINT => PlcValue::Sint(json_val.as_i64().unwrap_or(0) as i8),
                DataType::INT => PlcValue::Int(json_val.as_i64().unwrap_or(0) as i16),
                DataType::DINT => PlcValue::Dint(json_val.as_i64().unwrap_or(0) as i32),
                DataType::LINT => PlcValue::Lint(
                    json_val
                        .as_str()
                        .map(|s| s.parse().unwrap_or(0))
                        .unwrap_or(json_val.as_i64().unwrap_or(0)),
                ),
                DataType::REAL => PlcValue::Real(json_val.as_f64().unwrap_or(0.0) as f32),
                DataType::LREAL => PlcValue::Lreal(json_val.as_f64().unwrap_or(0.0)),
                DataType::WORD => PlcValue::Word(json_val.as_u64().unwrap_or(0) as u16),
                DataType::DWORD => PlcValue::Dword(json_val.as_u64().unwrap_or(0) as u32),
                DataType::LWORD => PlcValue::Lword(
                    json_val
                        .as_str()
                        .map(|s| s.parse().unwrap_or(0))
                        .unwrap_or(json_val.as_u64().unwrap_or(0)),
                ),
            };
            rust_values.push(val);
        }

        let client = self.inner.clone();
        tokio::task::spawn_blocking(move || client.write_struct(mem_area, address, rust_values))
            .await
            .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
            .map_err(fins_to_js_error)
    }
}

// ─── Opções do cliente (objeto JS) ─────────────────────────────────

/// Advanced FINS client configuration options.
#[napi(object)]
pub struct JsClientOptions {
    /// UDP port (default: 9600)
    pub port: Option<u16>,
    /// Timeout in milliseconds (default: 2000)
    pub timeout_ms: Option<u32>,
    /// Source network number
    pub source_network: Option<u8>,
    /// Source unit number
    pub source_unit: Option<u8>,
    /// Destination network number
    pub dest_network: Option<u8>,
    /// Destination unit number
    pub dest_unit: Option<u8>,
}

// ─── Funções utilitárias ───────────────────────────────────────────

/// Gets the value of a specific bit from a u16 word.
#[napi]
pub fn get_bit(value: u32, bit: u8) -> bool {
    crate::utils::get_bit(value as u16, bit)
}

/// Sets the value of a specific bit in a u16 word.
#[napi]
pub fn set_bit(value: u32, bit: u8, on: bool) -> u32 {
    crate::utils::set_bit(value as u16, bit, on) as u32
}

/// Toggles the value of a specific bit in a u16 word.
#[napi]
pub fn toggle_bit(value: u32, bit: u8) -> u32 {
    crate::utils::toggle_bit(value as u16, bit) as u32
}

/// Converts a u16 word into an array of 16 booleans.
#[napi]
pub fn word_to_bits(value: u32) -> Vec<bool> {
    crate::utils::word_to_bits(value as u16).to_vec()
}

/// Converts an array of booleans to a u16 word.
#[napi]
pub fn bits_to_word(bits: Vec<bool>) -> u32 {
    let mut arr = [false; 16];
    for (i, &b) in bits.iter().take(16).enumerate() {
        arr[i] = b;
    }
    crate::utils::bits_to_word(&arr) as u32
}

/// Returns indices of bits that are ON in a word.
#[napi]
pub fn get_on_bits(value: u32) -> Vec<u32> {
    crate::utils::get_on_bits(value as u16)
        .into_iter()
        .map(|v| v as u32)
        .collect()
}

/// Counts how many bits are ON in a word.
#[napi]
pub fn count_on_bits(value: u32) -> u32 {
    crate::utils::count_on_bits(value as u16)
}

/// Formats a word as a binary string (e.g., "0b1010_0101_1100_0011").
#[napi]
pub fn format_binary(value: u32) -> String {
    crate::utils::format_binary(value as u16)
}

/// Formats a word as a hexadecimal string (e.g., "0xA5C3").
#[napi]
pub fn format_hex(value: u32) -> String {
    crate::utils::format_hex(value as u16)
}
