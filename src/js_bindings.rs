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

// ─── Conversores de Enum ───────────────────────────────────────────

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

/// Converte FinsError do Rust para napi::Error do JS.
fn fins_to_js_error(e: crate::error::FinsError) -> Error {
    Error::from_reason(e.to_string())
}

// ─── Objetos JS para inputs complexos ──────────────────────────────

/// Especificação de bit forçado (input JS).
#[napi(object)]
pub struct JsForcedBit {
    /// Área de memória: "CIO", "WR", "HR", "AR"
    pub area: String,
    /// Endereço da word
    pub address: u16,
    /// Posição do bit (0-15)
    pub bit: u8,
    /// Especificação: "force_on", "force_off", "release"
    pub spec: String,
}

/// Especificação de leitura múltipla (input JS).
#[napi(object)]
pub struct JsMultiReadSpec {
    /// Área de memória: "CIO", "WR", "HR", "DM", "AR"
    pub area: String,
    /// Endereço da word
    pub address: u16,
    /// Posição do bit (opcional, null/undefined para leitura de word)
    pub bit: Option<u8>,
}

/// Representação de um valor do PLC para o JavaScript.
#[napi(object)]
pub struct JsPlcValue {
    /// Tipo do dado: "INT", "DINT", "LINT", "REAL", etc.
    pub r#type: String,
    /// Valor (representado como string JSON para evitar problemas de tipos)
    pub value: String,
}

// ─── Constantes exportadas ─────────────────────────────────────────

/// Porta UDP padrão do protocolo FINS.
#[napi]
pub const DEFAULT_FINS_PORT: u16 = crate::transport::DEFAULT_FINS_PORT;

/// Tamanho máximo do pacote FINS.
#[napi]
pub const MAX_PACKET_SIZE: u16 = crate::transport::MAX_PACKET_SIZE as u16;

/// Máximo de words por comando.
#[napi]
pub const MAX_WORDS_PER_COMMAND: u16 = crate::command::MAX_WORDS_PER_COMMAND;

// ─── Cliente FINS (classe principal) ───────────────────────────────

/// Cliente FINS para comunicação com PLCs Omron.
///
/// Cada operação gera exatamente 1 request e 1 response UDP.
/// Sem retries automáticos, caching ou reconexão.
#[napi]
pub struct FinsClient {
    inner: Arc<Client>,
}

#[napi]
impl FinsClient {
    /// Cria um novo cliente FINS.
    ///
    /// @param host - Endereço IP do PLC (ex: "192.168.1.250")
    /// @param sourceNode - Número do nó de origem (este cliente)
    /// @param destNode - Número do nó de destino (o PLC)
    /// @param options - Opções avançadas (opcional)
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

    /// Lê words da memória do PLC (assíncrono).
    ///
    /// @param area - Área de memória ("DM", "CIO", "WR", "HR", "AR")
    /// @param address - Endereço inicial
    /// @param count - Quantidade de words para ler (1-999)
    /// @returns Promise<number[]> - Array de valores u16
    #[napi]
    pub async fn read(&self, area: String, address: u16, count: u16) -> Result<Vec<u32>> {
        let mem_area = parse_memory_area(&area)?;
        let client = self.inner.clone();

        // Executa a operação blocking em uma task separada para não travar a event loop
        let result = tokio::task::spawn_blocking(move || client.read(mem_area, address, count))
            .await
            .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
            .map_err(fins_to_js_error)?;

        // Converte u16 para u32 (JS number é seguro para u16)
        Ok(result.into_iter().map(|v| v as u32).collect())
    }

    /// Escreve words na memória do PLC (assíncrono).
    ///
    /// @param area - Área de memória ("DM", "CIO", "WR", "HR", "AR")
    /// @param address - Endereço inicial
    /// @param data - Array de valores u16 para escrever
    #[napi]
    pub async fn write(&self, area: String, address: u16, data: Vec<u32>) -> Result<()> {
        let mem_area = parse_memory_area(&area)?;
        let words: Vec<u16> = data.into_iter().map(|v| v as u16).collect();
        let client = self.inner.clone();

        tokio::task::spawn_blocking(move || client.write(mem_area, address, &words))
            .await
            .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
            .map_err(fins_to_js_error)
    }

    // ─── Leitura / Escrita de Bits ─────────────────────────────────

    /// Lê um bit da memória do PLC (assíncrono).
    ///
    /// @param area - Área de memória ("CIO", "WR", "HR", "AR") — DM não suporta bits
    /// @param address - Endereço da word
    /// @param bit - Posição do bit (0-15)
    /// @returns Promise<boolean>
    #[napi]
    pub async fn read_bit(&self, area: String, address: u16, bit: u8) -> Result<bool> {
        let mem_area = parse_memory_area(&area)?;
        let client = self.inner.clone();

        tokio::task::spawn_blocking(move || client.read_bit(mem_area, address, bit))
            .await
            .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
            .map_err(fins_to_js_error)
    }

    /// Escreve um bit na memória do PLC (assíncrono).
    ///
    /// @param area - Área de memória ("CIO", "WR", "HR", "AR")
    /// @param address - Endereço da word
    /// @param bit - Posição do bit (0-15)
    /// @param value - Valor do bit (true/false)
    #[napi]
    pub async fn write_bit(
        &self,
        area: String,
        address: u16,
        bit: u8,
        value: bool,
    ) -> Result<()> {
        let mem_area = parse_memory_area(&area)?;
        let client = self.inner.clone();

        tokio::task::spawn_blocking(move || client.write_bit(mem_area, address, bit, value))
            .await
            .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
            .map_err(fins_to_js_error)
    }

    // ─── Fill ──────────────────────────────────────────────────────

    /// Preenche uma região de memória com um valor (assíncrono).
    ///
    /// @param area - Área de memória
    /// @param address - Endereço inicial
    /// @param count - Quantidade de words para preencher (1-999)
    /// @param value - Valor u16 para preencher
    #[napi]
    pub async fn fill(
        &self,
        area: String,
        address: u16,
        count: u16,
        value: u32,
    ) -> Result<()> {
        let mem_area = parse_memory_area(&area)?;
        let val = value as u16;
        let client = self.inner.clone();

        tokio::task::spawn_blocking(move || client.fill(mem_area, address, count, val))
            .await
            .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
            .map_err(fins_to_js_error)
    }

    // ─── Transfer ──────────────────────────────────────────────────

    /// Transfere dados de uma área de memória para outra (assíncrono).
    ///
    /// @param srcArea - Área de memória de origem
    /// @param srcAddress - Endereço de origem
    /// @param dstArea - Área de memória de destino
    /// @param dstAddress - Endereço de destino
    /// @param count - Quantidade de words para transferir
    #[napi]
    pub async fn transfer(
        &self,
        src_area: String,
        src_address: u16,
        dst_area: String,
        dst_address: u16,
        count: u16,
    ) -> Result<()> {
        let src = parse_memory_area(&src_area)?;
        let dst = parse_memory_area(&dst_area)?;
        let client = self.inner.clone();

        tokio::task::spawn_blocking(move || {
            client.transfer(src, src_address, dst, dst_address, count)
        })
        .await
        .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
        .map_err(fins_to_js_error)
    }

    // ─── Run / Stop ────────────────────────────────────────────────

    /// Coloca o PLC em modo de execução (assíncrono).
    ///
    /// @param mode - Modo: "debug", "monitor" ou "run"
    #[napi]
    pub async fn run(&self, mode: String) -> Result<()> {
        let plc_mode = parse_plc_mode(&mode)?;
        let client = self.inner.clone();

        tokio::task::spawn_blocking(move || client.run(plc_mode))
            .await
            .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
            .map_err(fins_to_js_error)
    }

    /// Para o PLC (assíncrono).
    #[napi]
    pub async fn stop(&self) -> Result<()> {
        let client = self.inner.clone();

        tokio::task::spawn_blocking(move || client.stop())
            .await
            .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
            .map_err(fins_to_js_error)
    }

    // ─── Forced Set/Reset ──────────────────────────────────────────

    /// Força bits ON/OFF no PLC (assíncrono).
    ///
    /// @param specs - Array de especificações de bits forçados
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

    /// Cancela todos os bits forçados no PLC (assíncrono).
    #[napi]
    pub async fn forced_set_reset_cancel(&self) -> Result<()> {
        let client = self.inner.clone();

        tokio::task::spawn_blocking(move || client.forced_set_reset_cancel())
            .await
            .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
            .map_err(fins_to_js_error)
    }

    // ─── Multiple Read ─────────────────────────────────────────────

    /// Lê de múltiplas áreas de memória em uma única requisição (assíncrono).
    ///
    /// @param specs - Array de especificações de leitura
    /// @returns Promise<number[]> - Array de valores u16
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

    /// Lê um valor f32 (REAL) de 2 words consecutivas (assíncrono).
    ///
    /// @param area - Área de memória
    /// @param address - Endereço inicial da word
    /// @returns Promise<number>
    #[napi]
    pub async fn read_f32(&self, area: String, address: u16) -> Result<f64> {
        let mem_area = parse_memory_area(&area)?;
        let client = self.inner.clone();

        let result =
            tokio::task::spawn_blocking(move || client.read_f32(mem_area, address))
                .await
                .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
                .map_err(fins_to_js_error)?;

        Ok(result as f64) // JS number é f64
    }

    /// Escreve um valor f32 (REAL) em 2 words consecutivas (assíncrono).
    ///
    /// @param area - Área de memória
    /// @param address - Endereço inicial da word
    /// @param value - Valor f32 para escrever
    #[napi]
    pub async fn write_f32(&self, area: String, address: u16, value: f64) -> Result<()> {
        let mem_area = parse_memory_area(&area)?;
        let val = value as f32;
        let client = self.inner.clone();

        tokio::task::spawn_blocking(move || client.write_f32(mem_area, address, val))
            .await
            .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
            .map_err(fins_to_js_error)
    }

    /// Lê um valor f64 (LREAL) de 4 words consecutivas (assíncrono).
    ///
    /// @param area - Área de memória
    /// @param address - Endereço inicial da word
    /// @returns Promise<number>
    #[napi]
    pub async fn read_f64(&self, area: String, address: u16) -> Result<f64> {
        let mem_area = parse_memory_area(&area)?;
        let client = self.inner.clone();

        tokio::task::spawn_blocking(move || client.read_f64(mem_area, address))
            .await
            .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
            .map_err(fins_to_js_error)
    }

    /// Escreve um valor f64 (LREAL) em 4 words consecutivas (assíncrono).
    ///
    /// @param area - Área de memória
    /// @param address - Endereço inicial da word
    /// @param value - Valor f64 para escrever
    #[napi]
    pub async fn write_f64(&self, area: String, address: u16, value: f64) -> Result<()> {
        let mem_area = parse_memory_area(&area)?;
        let client = self.inner.clone();

        tokio::task::spawn_blocking(move || client.write_f64(mem_area, address, value))
            .await
            .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
            .map_err(fins_to_js_error)
    }

    /// Lê um valor i32 (DINT) de 2 words consecutivas (assíncrono).
    ///
    /// @param area - Área de memória
    /// @param address - Endereço inicial da word
    /// @returns Promise<number>
    #[napi]
    pub async fn read_i32(&self, area: String, address: u16) -> Result<i32> {
        let mem_area = parse_memory_area(&area)?;
        let client = self.inner.clone();

        tokio::task::spawn_blocking(move || client.read_i32(mem_area, address))
            .await
            .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
            .map_err(fins_to_js_error)
    }

    /// Escreve um valor i32 (DINT) em 2 words consecutivas (assíncrono).
    ///
    /// @param area - Área de memória
    /// @param address - Endereço inicial da word
    /// @param value - Valor i32 para escrever
    #[napi]
    pub async fn write_i32(&self, area: String, address: u16, value: i32) -> Result<()> {
        let mem_area = parse_memory_area(&area)?;
        let client = self.inner.clone();

        tokio::task::spawn_blocking(move || client.write_i32(mem_area, address, value))
            .await
            .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
            .map_err(fins_to_js_error)
    }

    // ─── Strings ───────────────────────────────────────────────────

    /// Escreve uma string ASCII na memória do PLC (assíncrono).
    ///
    /// @param area - Área de memória
    /// @param address - Endereço inicial da word
    /// @param value - String ASCII para escrever (máximo 1998 caracteres)
    #[napi]
    pub async fn write_string(
        &self,
        area: String,
        address: u16,
        value: String,
    ) -> Result<()> {
        let mem_area = parse_memory_area(&area)?;
        let client = self.inner.clone();

        tokio::task::spawn_blocking(move || client.write_string(mem_area, address, &value))
            .await
            .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
            .map_err(fins_to_js_error)
    }

    /// Lê uma string ASCII da memória do PLC (assíncrono).
    ///
    /// @param area - Área de memória
    /// @param address - Endereço inicial da word
    /// @param wordCount - Quantidade de words para ler (1 word = 2 caracteres)
    /// @returns Promise<string>
    #[napi]
    pub async fn read_string(
        &self,
        area: String,
        address: u16,
        word_count: u16,
    ) -> Result<String> {
        let mem_area = parse_memory_area(&area)?;
        let client = self.inner.clone();

        tokio::task::spawn_blocking(move || client.read_string(mem_area, address, word_count))
            .await
            .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?
            .map_err(fins_to_js_error)
    }

    // ─── Structs ───────────────────────────────────────────────────

    /// Lê uma estrutura da memória do PLC (assíncrono).
    ///
    /// @param area - Área de memória
    /// @param address - Endereço inicial da word
    /// @param types - Array de tipos ("INT", "REAL", etc.)
    /// @returns Promise<JsPlcValue[]>
    #[napi]
    pub async fn read_struct(
        &self,
        area: String,
        address: u16,
        types: Vec<String>,
    ) -> Result<Vec<JsPlcValue>> {
        let mem_area = parse_memory_area(&area)?;
        let rust_types: std::result::Result<Vec<DataType>, Error> =
            types.iter().map(|t| parse_data_type(t)).collect();
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

    /// Escreve uma estrutura na memória do PLC (assíncrono).
    ///
    /// @param area - Área de memória
    /// @param address - Endereço inicial da word
    /// @param values - Array de objetos { type: string, value: any }
    #[napi]
    pub async fn write_struct(
        &self,
        area: String,
        address: u16,
        values: Vec<JsPlcValue>,
    ) -> Result<()> {
        let mem_area = parse_memory_area(&area)?;
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

/// Opções avançadas de configuração do cliente FINS.
#[napi(object)]
pub struct JsClientOptions {
    /// Porta UDP (padrão: 9600)
    pub port: Option<u16>,
    /// Timeout em milissegundos (padrão: 2000)
    pub timeout_ms: Option<u32>,
    /// Número da rede de origem
    pub source_network: Option<u8>,
    /// Número da unidade de origem
    pub source_unit: Option<u8>,
    /// Número da rede de destino
    pub dest_network: Option<u8>,
    /// Número da unidade de destino
    pub dest_unit: Option<u8>,
}

// ─── Funções utilitárias ───────────────────────────────────────────

/// Obtém o valor de um bit específico de uma word u16.
#[napi]
pub fn get_bit(value: u32, bit: u8) -> bool {
    crate::utils::get_bit(value as u16, bit)
}

/// Define o valor de um bit específico em uma word u16.
#[napi]
pub fn set_bit(value: u32, bit: u8, on: bool) -> u32 {
    crate::utils::set_bit(value as u16, bit, on) as u32
}

/// Alterna o valor de um bit específico em uma word u16.
#[napi]
pub fn toggle_bit(value: u32, bit: u8) -> u32 {
    crate::utils::toggle_bit(value as u16, bit) as u32
}

/// Converte uma word u16 em um array de 16 booleanos.
#[napi]
pub fn word_to_bits(value: u32) -> Vec<bool> {
    crate::utils::word_to_bits(value as u16).to_vec()
}

/// Converte um array de booleanos para uma word u16.
#[napi]
pub fn bits_to_word(bits: Vec<bool>) -> u32 {
    let mut arr = [false; 16];
    for (i, &b) in bits.iter().take(16).enumerate() {
        arr[i] = b;
    }
    crate::utils::bits_to_word(&arr) as u32
}

/// Retorna indices dos bits que estão ON em uma word.
#[napi]
pub fn get_on_bits(value: u32) -> Vec<u32> {
    crate::utils::get_on_bits(value as u16)
        .into_iter()
        .map(|v| v as u32)
        .collect()
}

/// Conta quantos bits estão ON em uma word.
#[napi]
pub fn count_on_bits(value: u32) -> u32 {
    crate::utils::count_on_bits(value as u16)
}

/// Formata uma word como string binária (ex: "0b1010_0101_1100_0011").
#[napi]
pub fn format_binary(value: u32) -> String {
    crate::utils::format_binary(value as u16)
}

/// Formata uma word como string hexadecimal (ex: "0xA5C3").
#[napi]
pub fn format_hex(value: u32) -> String {
    crate::utils::format_hex(value as u16)
}
