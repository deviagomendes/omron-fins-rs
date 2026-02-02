# omron-fins

Uma biblioteca Rust para comunicação com CLPs Omron usando o protocolo FINS.

[![Crates.io](https://img.shields.io/crates/v/omron-fins.svg)](https://crates.io/crates/omron-fins)
[![Documentation](https://docs.rs/omron-fins/badge.svg)](https://docs.rs/omron-fins)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Características

- **Biblioteca de protocolo puro** — sem lógica de negócio, polling ou schedulers
- **Execução determinística** — cada chamada produz exatamente 1 requisição e 1 resposta
- **Sem comportamento implícito** — sem retry automático, cache ou reconexão
- **API simples e previsível** — `read`, `write`, `read_bit`, `write_bit`
- **Tipos seguros** — áreas de memória como `enum`, nunca strings
- **Tratamento de erros completo** — sem `panic!` em código público

## Instalação

Adicione ao seu `Cargo.toml`:

```toml
[dependencies]
omron-fins = "0.1"
```

## Quick Start

```rust
use omron_fins::{Client, ClientConfig, MemoryArea};
use std::net::Ipv4Addr;

fn main() -> omron_fins::Result<()> {
    // Cria a configuração do cliente
    let config = ClientConfig::new(
        Ipv4Addr::new(192, 168, 1, 10),  // IP do CLP
        1,                                // Node de origem (este cliente)
        10,                               // Node de destino (o CLP)
    );

    // Conecta ao CLP
    let client = Client::new(config)?;

    // Lê 10 words a partir de DM100
    let data = client.read(MemoryArea::DM, 100, 10)?;
    println!("Dados lidos: {:?}", data);

    // Escreve valores em DM200
    client.write(MemoryArea::DM, 200, &[0x1234, 0x5678])?;

    // Lê um bit específico (CIO 0.05)
    let bit = client.read_bit(MemoryArea::CIO, 0, 5)?;
    println!("CIO 0.05 = {}", bit);

    // Escreve um bit
    client.write_bit(MemoryArea::CIO, 0, 5, true)?;

    Ok(())
}
```

## Áreas de Memória

A biblioteca suporta as seguintes áreas de memória:

| Área | Nome | Descrição | Acesso a Word | Acesso a Bit |
|------|------|-----------|:-------------:|:------------:|
| `CIO` | Core I/O | Entradas/saídas e relés internos | ✅ | ✅ |
| `WR` | Work | Bits/words de trabalho temporário | ✅ | ✅ |
| `HR` | Holding | Bits/words retentivos | ✅ | ✅ |
| `DM` | Data Memory | Armazenamento de dados numéricos | ✅ | ❌ |

```rust
use omron_fins::MemoryArea;

// Verificar se uma área suporta acesso a bit
assert!(MemoryArea::CIO.supports_bit_access());
assert!(!MemoryArea::DM.supports_bit_access());
```

## API

### Leitura de Words

```rust
// Lê 'count' words a partir de 'address'
let data: Vec<u16> = client.read(area, address, count)?;
```

**Parâmetros:**
- `area`: Área de memória (`MemoryArea::DM`, `CIO`, `WR`, `HR`)
- `address`: Endereço inicial (0-65535)
- `count`: Quantidade de words a ler (1-999)

### Escrita de Words

```rust
// Escreve uma slice de words a partir de 'address'
client.write(area, address, &[valor1, valor2, ...])?;
```

**Parâmetros:**
- `area`: Área de memória
- `address`: Endereço inicial
- `data`: Slice de words a escrever (1-999 words)

### Leitura de Bit

```rust
// Lê um bit específico
let valor: bool = client.read_bit(area, address, bit)?;
```

**Parâmetros:**
- `area`: Área de memória (apenas `CIO`, `WR`, `HR` — DM não suporta)
- `address`: Endereço do word
- `bit`: Posição do bit (0-15)

### Escrita de Bit

```rust
// Escreve um bit específico
client.write_bit(area, address, bit, value)?;
```

**Parâmetros:**
- `area`: Área de memória (apenas `CIO`, `WR`, `HR`)
- `address`: Endereço do word
- `bit`: Posição do bit (0-15)
- `value`: Valor a escrever (`true` ou `false`)

## Configuração Avançada

### Configuração Completa do Cliente

```rust
use omron_fins::ClientConfig;
use std::net::Ipv4Addr;
use std::time::Duration;

let config = ClientConfig::new(Ipv4Addr::new(192, 168, 1, 10), 1, 10)
    .with_port(9601)                        // Porta personalizada (padrão: 9600)
    .with_timeout(Duration::from_secs(5))   // Timeout personalizado (padrão: 2s)
    .with_source_network(1)                 // Rede de origem
    .with_source_unit(0)                    // Unidade de origem
    .with_dest_network(1)                   // Rede de destino
    .with_dest_unit(0);                     // Unidade de destino
```

### Endereçamento de Node

O protocolo FINS usa três componentes para endereçar um node:

| Componente | Descrição | Valor Típico |
|------------|-----------|--------------|
| Network | Número da rede | 0 (rede local) |
| Node | Número do node | 1-126 |
| Unit | Número da unidade | 0 (CPU) |

Para comunicação simples na mesma rede, apenas o número do node é necessário:

```rust
// Comunicação local simples
let config = ClientConfig::new(ip, source_node, dest_node);

// Comunicação entre redes
let config = ClientConfig::new(ip, source_node, dest_node)
    .with_source_network(1)
    .with_dest_network(2);
```

## Tratamento de Erros

Todas as operações retornam `Result<T, FinsError>`. A biblioteca nunca causa `panic!` em código público.

```rust
use omron_fins::{Client, ClientConfig, MemoryArea, FinsError};
use std::net::Ipv4Addr;

let config = ClientConfig::new(Ipv4Addr::new(192, 168, 1, 10), 1, 10);
let client = Client::new(config)?;

match client.read(MemoryArea::DM, 100, 10) {
    Ok(data) => println!("Dados: {:?}", data),
    
    Err(FinsError::Timeout) => {
        println!("Timeout de comunicação");
    }
    
    Err(FinsError::PlcError { main_code, sub_code }) => {
        println!("Erro do CLP: main=0x{:02X}, sub=0x{:02X}", main_code, sub_code);
    }
    
    Err(FinsError::InvalidAddressing { reason }) => {
        println!("Endereçamento inválido: {}", reason);
    }
    
    Err(FinsError::InvalidParameter { parameter, reason }) => {
        println!("Parâmetro inválido '{}': {}", parameter, reason);
    }
    
    Err(e) => println!("Erro: {}", e),
}
```

### Tipos de Erro

| Erro | Descrição |
|------|-----------|
| `PlcError` | Erro retornado pelo CLP (com códigos main/sub) |
| `Timeout` | Timeout de comunicação |
| `InvalidAddressing` | Endereçamento inválido (ex: bit access em DM) |
| `InvalidParameter` | Parâmetro inválido (ex: count = 0) |
| `InvalidResponse` | Resposta inválida do CLP |
| `SidMismatch` | Service ID não corresponde entre request/response |
| `Io` | Erro de I/O do sistema |

## Exemplos

### Monitoramento de I/O

```rust
use omron_fins::{Client, ClientConfig, MemoryArea};
use std::net::Ipv4Addr;

fn main() -> omron_fins::Result<()> {
    let client = Client::new(
        ClientConfig::new(Ipv4Addr::new(192, 168, 1, 10), 1, 10)
    )?;

    // Lê estado das entradas digitais (CIO 0-9)
    let inputs = client.read(MemoryArea::CIO, 0, 10)?;
    
    for (i, word) in inputs.iter().enumerate() {
        println!("CIO {:03}: 0x{:04X} ({:016b})", i, word, word);
    }

    Ok(())
}
```

### Escrita de Receita

```rust
use omron_fins::{Client, ClientConfig, MemoryArea};
use std::net::Ipv4Addr;

fn write_recipe(client: &Client, recipe_id: u16, params: &[u16]) -> omron_fins::Result<()> {
    // Escreve ID da receita em DM100
    client.write(MemoryArea::DM, 100, &[recipe_id])?;
    
    // Escreve parâmetros em DM101-DM110
    client.write(MemoryArea::DM, 101, params)?;
    
    // Seta bit de "receita pronta" em WR 0.00
    client.write_bit(MemoryArea::WR, 0, 0, true)?;
    
    Ok(())
}

fn main() -> omron_fins::Result<()> {
    let client = Client::new(
        ClientConfig::new(Ipv4Addr::new(192, 168, 1, 10), 1, 10)
    )?;

    let recipe_params = [1000, 2000, 3000, 500, 750];
    write_recipe(&client, 42, &recipe_params)?;
    
    println!("Receita enviada com sucesso!");
    Ok(())
}
```

### Leitura de Alarmes

```rust
use omron_fins::{Client, ClientConfig, MemoryArea};
use std::net::Ipv4Addr;

fn check_alarms(client: &Client) -> omron_fins::Result<Vec<usize>> {
    // Lê 10 words de alarmes (160 bits)
    let alarm_words = client.read(MemoryArea::HR, 0, 10)?;
    
    let mut active_alarms = Vec::new();
    
    for (word_idx, word) in alarm_words.iter().enumerate() {
        for bit in 0..16 {
            if (word >> bit) & 1 == 1 {
                active_alarms.push(word_idx * 16 + bit);
            }
        }
    }
    
    active_alarms
}

fn main() -> omron_fins::Result<()> {
    let client = Client::new(
        ClientConfig::new(Ipv4Addr::new(192, 168, 1, 10), 1, 10)
    )?;

    let alarms = check_alarms(&client)?;
    
    if alarms.is_empty() {
        println!("Nenhum alarme ativo");
    } else {
        println!("Alarmes ativos: {:?}", alarms);
    }
    
    Ok(())
}
```

## Constantes Úteis

```rust
use omron_fins::{DEFAULT_FINS_PORT, DEFAULT_TIMEOUT, MAX_PACKET_SIZE};

// Porta UDP padrão do FINS
assert_eq!(DEFAULT_FINS_PORT, 9600);

// Timeout padrão de comunicação
assert_eq!(DEFAULT_TIMEOUT, std::time::Duration::from_secs(2));

// Tamanho máximo do pacote FINS
assert_eq!(MAX_PACKET_SIZE, 2012);
```

## Limitações

- **Apenas UDP** — TCP não é suportado nesta versão
- **Síncrono** — operações bloqueantes (async pode ser adicionado futuramente)
- **Sem retry automático** — a aplicação deve implementar lógica de retry se necessário
- **Sem cache** — cada chamada gera uma requisição de rede
- **Sem reconexão automática** — a aplicação deve recriar o cliente se necessário

## Filosofia de Design

Esta biblioteca segue o princípio de **determinismo acima de abstração**:

1. Cada operação faz exatamente o que diz
2. Sem comportamento mágico ou implícito
3. A aplicação tem controle total sobre retry, cache e reconexão
4. Erros são sempre explícitos e descritivos

## Licença

MIT License - veja [LICENSE](LICENSE) para detalhes.

## Contribuindo

Contribuições são bem-vindas! Por favor, leia [ARCHITECTURE.md](ARCHITECTURE.md) para entender as regras de design do projeto antes de submeter PRs.
