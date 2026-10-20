# MelobudsNext (Rust)

CLI para testar o protocolo BLE do QCY Melobuds Pro, portada do projeto Python
em `MelobudsNext/`. O foco é comunicar com o fone, enviar comandos e inspecionar
o estado sem precisar de interface gráfica.

O protocolo está documentado em [`PROTOCOLO_MelobudsNext.md`](PROTOCOLO_MelobudsNext.md).

## Compilação

Requisitos: Rust 1.85 ou superior e um adaptador Bluetooth.

```sh
cargo build --release
```

No Windows, a toolchain padrão (`stable-x86_64-pc-windows-msvc`) precisa do
Visual Studio Build Tools com a opção "Desenvolvimento para desktop com C++".
Alternativa sem Visual Studio:

```powershell
rustup toolchain install stable-x86_64-pc-windows-gnu
winget install BrechtSanders.WinLibs.POSIX.UCRT
cargo +stable-x86_64-pc-windows-gnu build --release
```

## Primeira comunicação

1. Ligue o fone e desconecte-o do celular/app QCY (o fone aceita um host por vez).
2. Descubra o endereço:

   ```sh
   cargo run -- device scan
   ```

3. Salve o endereço (aceita MAC, id da plataforma ou parte do nome):

   ```sh
   cargo run -- config set-mac AA:BB:CC:DD:EE:FF
   ```

   Também é possível passar `--address` em qualquer comando `device`.

4. Teste a conexão e leia o estado completo:

   ```sh
   cargo run -- device info
   cargo run -- device info --json
   ```

`device info` lê bateria, firmware e touch diretamente e consulta ANC, game mode,
sleep, nome, balanço, volume de notificação, LDAC e detecção de uso via `0xFE`.

Se algum desses campos opcionais falhar, o comando continua e mostra o problema
como `warning:` em vez de abortar.

## Comandos do dispositivo

Cada ação abaixo sincroniza o estado, envia o comando, aplica os echos e imprime
o estado resultante. `--wait SEGUNDOS` controla a espera pelos echos (padrão 1 s)
e `--json` imprime apenas o JSON em `stdout` (mensagens vão para `stderr`).

| Comando | Efeito |
| --- | --- |
| `device info` | Conecta e sincroniza todo o estado |
| `device anc --scene indoor --level 2` | ANC indoor no nível 2 |
| `device anc --scene wind` | ANC vento |
| `device anc --intensity 4` | Transparência no nível 4 |
| `device anc --voice` | Transparência com reforço de voz |
| `device anc` | Desliga ANC/transparência |
| `device game on` | Liga o modo jogo |
| `device sleep on` | Liga o modo sono |
| `device ldac on` | Liga LDAC e reconecta (o fone reinicia) |
| `device rename "Meu Fone"` | Renomeia (até 32 bytes UTF-8) |
| `device balance 40` | Balanço 0 (esquerda) a 100 (direita) |
| `device volume medium` | Volume de notificação (`low`, `medium`, `high`, `max`) |
| `device wear on --keep-anc` | Detecção de uso mantendo ANC |
| `device query anc` | Consulta um opcode pelo nome ou número (`0x17`) |
| `device send "FF 03 09 01 01"` | Envia bytes crus |
| `device read battery` | Lê bateria, firmware ou touch (`battery`, `firmware`, `touch`) |
| `device write-touch --bind left-single=play-pause` | Grava o mapa de toque completo |
| `device inspect` | Lista serviços e características GATT |
| `device monitor` | Mostra notificações até `Ctrl+C` |

Exemplos:

```sh
cargo run -- device anc --scene daily --level 3
cargo run -- device game on --json
cargo run -- device ldac on --wait 2
cargo run -- device send "FF 05 17 03 03 01 04"
```

O mapa de toque começa zerado em `write-touch`; informe todos os pares que devem
ficar ativos, por exemplo:

```sh
cargo run -- device write-touch \
  --bind left-single=play-pause \
  --bind right-single=next \
  --bind left-double=previous
```

## Testes sem o fone

```sh
# Gera os bytes de um comando
cargo run -- build game on
cargo run -- build anc --scene indoor --level 2
cargo run -- build anc --voice
cargo run -- build query anc

# Decodifica uma notificação composta
cargo run -- decode "FF 06 09 01 01 17 03 00 00 00"
cargo run -- decode "FF 05 17 03 01 09 00" --json

# Leituras diretas
cargo run -- battery "D0 46 00"
cargo run -- firmware "02 00 06"

# Mapa de toque
cargo run -- touch encode --bind left-single=play-pause
cargo run -- touch decode "01 01 02 03 03 00 04 00 05 00 06 00 07 00 08 00 00 00 00 00"
```

## Configuração

O caminho padrão é `~/.melobudsnext/config.json` (ou `%USERPROFILE%\.melobudsnext\config.json`).
Use `--config PATH` para apontar para outro arquivo.

```json
{
  "mac_address": "AA:BB:CC:DD:EE:FF",
  "uuids": {
    "service": "0000a001-0000-1000-8000-00805f9b34fb",
    "command_write": "00001001-0000-1000-8000-00805f9b34fb",
    "notification": "00001002-0000-1000-8000-00805f9b34fb",
    "battery": "00000008-0000-1000-8000-00805f9b34fb",
    "firmware": "00000007-0000-1000-8000-00805f9b34fb",
    "touch": "0000000d-0000-1000-8000-00805f9b34fb"
  }
}
```

O formato antigo do projeto Python também é aceito na leitura:

```json
{
  "address": "AA:BB:CC:DD:EE:FF",
  "uuid_service": "0000a001-0000-1000-8000-00805f9b34fb",
  "uuid_write": "00001001-0000-1000-8000-00805f9b34fb",
  "uuid_notify": "00001002-0000-1000-8000-00805f9b34fb"
}
```

## Comportamento e diferenças em relação ao Python

- Decodificação tolerante: ANC desconhecido é preservado como `unknown(...)`,
  bateria de 2 bytes é aceita (case fica `-/-`) e o touch aceita qualquer
  quantidade par de bytes.
- `rename` vazio não apaga o nome atual.
- O `sync` não aborta por causa de uma leitura opcional; falhas viram avisos.
- As consultas `0xFE` são enviadas a cada 50 ms enquanto as notificações são
  lidas, evitando perder respostas.
- Depois de mudar LDAC, a conexão é refeita automaticamente (10 tentativas, 3 s
  entre elas), como no cliente Python.
- Saída `--json` é JSON puro em `stdout`.
- Ainda não há TUI; os comandos são de uma execução só, com `device monitor` para
  acompanhar notificações.

## Testes

```sh
cargo test
```

Os testes cobrem framing, opcodes, ANC, touch, bateria, firmware, estado e
compatibilidade do arquivo de configuração com o formato Python.
