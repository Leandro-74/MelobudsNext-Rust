<p align="center">
  <img src="assets/logo.svg" width="120" alt="MelobudsNext">
</p>

# MelobudsNext

**Controle seu QCY Melobuds Pro pelo computador, no Windows e no Linux.**

MelobudsNext é um aplicativo open source, escrito em Rust, que conversa com o
fone por Bluetooth Low Energy (BLE). Em vez de depender do celular e do app
oficial, você ajusta cancelamento de ruído, toque, modo jogo, LDAC e o restante
numa janela no desktop — ou, se preferir, pelo terminal.

> Projeto não oficial, sem vínculo com a QCY. Desenvolvido e testado com o QCY
> Melobuds Pro no firmware 2.0.6; outros modelos e versões não foram validados.

## O que dá para fazer

- **Cancelamento de ruído (ANC):** cenas interno, diário e ruído (cada uma com
  três níveis), vento e adaptativo.
- **Transparência:** seis níveis de intensidade, com reforço de voz opcional.
- **Modo jogo e modo sono:** ligar e desligar com um clique.
- **LDAC:** ativar ou desativar o codec de alta qualidade; o app reconecta
  sozinho depois que o fone reinicia.
- **Mapa de toque:** escolher o que cada toque faz, nos dois lados.
- **Renomear o fone** (até 32 bytes).
- **Balanço de canais** e **volume das notificações** (baixo, médio, alto,
  máximo).
- **Detecção de uso:** pausar/retomar ao tirar o fone, mantendo ou não o ANC.
- **Bateria e firmware** na tela inicial.
- **Estado ao vivo:** mudanças feitas no próprio fone aparecem na interface sem
  precisar clicar em **Atualizar**.
- **Terminal opcional:** todas as funções também estão no CLI, com saída JSON
  para scripts.

## Instalação

### Windows 10/11

1. Baixe `melobudsnext-gui.exe` na página de
   [releases](https://github.com/Leandro-74/MelobudsNext-Rust/releases).
2. Confira o SHA-256 (compare com o valor publicado na release):

   ```powershell
   Get-FileHash .\melobudsnext-gui.exe -Algorithm SHA256
   ```

3. Execute o `.exe`. Não há instalador e não é preciso instalar o Visual C++
   Redistributable.

O executável não é assinado digitalmente, então o Windows pode mostrar "O
Windows protegeu o computador". Use **Mais informações → Executar assim mesmo**
depois de conferir o checksum.

### Arch Linux

O pacote fica no próprio repositório e é instalado com `makepkg` (sem AUR). As
dependências são resolvidas pelo pacman:

```sh
git clone https://github.com/Leandro-74/MelobudsNext-Rust.git
cd MelobudsNext-Rust
./packaging/aur/build-local.sh -si
```

Depois de instalar, abra **MelobudsNext** no menu da área de trabalho ou rode
`melobudsnext-gui`.

### Outras distribuições Linux

Baixe o `.tar.gz` da [release](https://github.com/Leandro-74/MelobudsNext-Rust/releases),
confira o checksum, extraia e copie os binários:

```sh
sha256sum -c melobudsnext-<versão>-x86_64-unknown-linux-gnu.tar.gz.sha256
tar -xzf melobudsnext-<versão>-x86_64-unknown-linux-gnu.tar.gz
cd melobudsnext-<versão>-x86_64-unknown-linux-gnu
sudo install -Dm755 melobudsnext /usr/local/bin/melobudsnext
sudo install -Dm755 melobudsnext-gui /usr/local/bin/melobudsnext-gui
sudo install -Dm644 melobudsnext.desktop \
  /usr/local/share/applications/melobudsnext.desktop
sudo install -Dm644 melobudsnext.svg \
  /usr/local/share/icons/hicolor/scalable/apps/melobudsnext.svg
```

É preciso ter BlueZ e D-Bus instalados (o pacote da sua distro cuida disso).
Se o Bluetooth estiver bloqueado, libere o adaptador:

```sh
sudo systemctl enable --now bluetooth
sudo rfkill unblock bluetooth
bluetoothctl power on
```

## Primeiros passos

1. Ligue o fone e **desconecte-o do celular** (ou feche o app da QCY). O fone
   aceita um computador/celular conectado por vez.
2. Pareie o fone no sistema:
   - **Windows:** Configurações → Bluetooth e dispositivos → Adicionar
     dispositivo.
   - **Linux:** `bluetoothctl`, depois `power on`, `agent on`, `scan on`,
     `pair <endereço>`, `trust <endereço>` e `connect <endereço>`.
3. Abra o **MelobudsNext**.
4. No campo **MAC ou ID do fone**, informe o endereço (ou parte do nome) e
   clique em **Conectar**. Se o endereço já estiver salvo na configuração, a
   conexão é automática.

Para não digitar o endereço toda vez:

```sh
melobudsnext config set-mac AA:BB:CC:DD:EE:FF
```

Aceita também o ID da plataforma ou parte do nome do fone. Use um endereço
fictício nos exemplos e nunca compartilhe o endereço real do seu aparelho.

## Usando a interface

A janela tem cinco abas:

| Aba | O que faz |
| --- | --- |
| **Status** | Bateria, firmware, nome, conexão e leitura geral do estado. |
| **ANC** | Cancelamento de ruído, transparência, intensidade e reforço de voz. |
| **Áudio** | LDAC, modo jogo, modo sono e balanço de canais. |
| **Toque** | Função de cada gesto (toque simples, duplo e triplo, nos dois lados). |
| **Uso** | Detecção de uso e a opção de manter o ANC ativo. |

**Modo jogo**, **modo sono** e **LDAC** são aplicados na hora. ANC, sliders,
mapa de toque e nome usam um rascunho local: ajuste e clique em **Aplicar**.
Depois de um comando bem-sucedido o valor escolhido permanece na tela, mesmo
quando o fone não devolve a confirmação a tempo.

O cabeçalho mostra o estado da conexão e permite conectar, atualizar e
desconectar. Se o fone desligar ou sair de alcance, a interface volta para
"Desconectado" na próxima ação em vez de mostrar dados antigos.

## Usando o terminal

O CLI `melobudsnext` faz as mesmas operações, útil para scripts e para quem
prefere o terminal. Os comandos mais comuns:

| Comando | Efeito |
| --- | --- |
| `melobudsnext device scan` | Procura fones por perto. |
| `melobudsnext device info` | Conecta e mostra todo o estado. |
| `melobudsnext device anc --scene indoor --level 2` | ANC interno no nível 2. |
| `melobudsnext device anc --scene wind` | ANC para vento. |
| `melobudsnext device anc --intensity 4` | Transparência no nível 4. |
| `melobudsnext device anc --voice` | Transparência com reforço de voz. |
| `melobudsnext device anc` | Desliga ANC e transparência. |
| `melobudsnext device game on` | Liga ou desliga o modo jogo (`on`/`off`). |
| `melobudsnext device sleep on` | Liga ou desliga o modo sono (`on`/`off`). |
| `melobudsnext device ldac on` | Liga o LDAC e reconecta. |
| `melobudsnext device rename "Meu Fone"` | Renomeia o fone. |
| `melobudsnext device balance 40` | Balanço: 0 (esquerda) a 100 (direita). |
| `melobudsnext device volume medium` | Volume de notificação (`low`, `medium`, `high`, `max`). |
| `melobudsnext device wear on --keep-anc` | Detecção de uso mantendo o ANC. |
| `melobudsnext device read battery` | Lê bateria, firmware ou toque. |
| `melobudsnext device write-touch --bind left-single=play-pause` | Grava o mapa de toque. |
| `melobudsnext device monitor` | Acompanha as notificações até `Ctrl+C`. |

Todos os comandos aceitam `--json` (saída limpa para scripts), `--wait` (espera
pelos ecos, padrão 1 s) e `--address` (endereço do fone). O `--help` de cada
subcomando lista as opções.

O mapa de toque é gravado por completo: informe todos os gestos que devem
ficar ativos, por exemplo:

```sh
melobudsnext device write-touch \
  --bind left-single=play-pause \
  --bind right-single=next \
  --bind left-double=previous
```

Ferramentas para quem trabalha com o protocolo (não precisam do fone):

```sh
melobudsnext build anc --scene indoor --level 2   # bytes de um comando
melobudsnext decode "FF 06 09 01 01 17 03 00 00 00"
melobudsnext touch decode "01 01 02 03 03 00 04 00 05 00 06 00 07 00 08 00 00 00 00 00"
```

O protocolo completo está em
[`PROTOCOLO_MelobudsNext.md`](PROTOCOLO_MelobudsNext.md).

## Configuração

O arquivo padrão é `~/.melobudsnext/config.json` no Linux e
`%USERPROFILE%\.melobudsnext\config.json` no Windows. O CLI e a GUI usam o mesmo
arquivo; `--config PATH` aponta para outro.

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

Só o endereço (`mac_address`) costuma ser necessário: ele permite a conexão
automática e pode ser definido com `config set-mac` ou informado no campo da
interface. Os UUIDs são os padrões do fone. O formato antigo do cliente Python
(`address`, `uuid_*`) continua aceito na leitura. `melobudsnext config path`
mostra o caminho em uso e `melobudsnext config show` imprime o conteúdo atual.

### Opções da interface

| Flag | Efeito |
| --- | --- |
| `--offline` | Abre a interface sem Bluetooth (demonstração). |
| `--config PATH` | Usa outro arquivo de configuração. |
| `--reconnect-attempts N` | Tentativas de reconexão após o LDAC (padrão 10). |
| `--reconnect-delay SEGUNDOS` | Intervalo entre as tentativas (padrão 3). |
| `--help` / `--version` | Ajuda e versão. |

O modo offline também liga com `MELOBUDNEXT_GUI_OFFLINE=1` (aceita `true`, `yes`
e `on`; `0`, `false`, `no` e `off` desligam). Nesse modo os controles de escrita
ficam desabilitados.

## Problemas comuns

| Sintoma | O que fazer |
| --- | --- |
| O fone não aparece | Confirme que está ligado, pareado e **desconectado do celular**. Aumente a espera com `--scan-timeout 10`. |
| "Permission denied" ou "Not authorized" no Linux | Adicione seu usuário ao grupo `bluetooth` (`sudo usermod -aG bluetooth "$USER"`), saia e entre na sessão; aceite o pedido do polkit quando aparecer. |
| O Windows avisa que o app não é reconhecido | Confira o SHA-256 e use **Mais informações → Executar assim mesmo**. |
| O LDAC derrubou a conexão | É esperado: o fone reinicia ao trocar o codec e o app reconecta sozinho (1 a 2 tentativas, tipicamente). |
| O estado demorou para mudar | Alguns ecos chegam depois da janela do comando; aguarde ou clique em **Atualizar**. |
| "BLE link lost" | O fone desligou, saiu de alcance ou outro programa assumiu a conexão. Conecte de novo. |
| ANC aparece como "Desconhecido" | O firmware reportou um valor não mapeado. Escolha um modo conhecido antes de aplicar. |
| Dois programas ao mesmo tempo | Feche o app oficial da QCY (celular) e outros clientes BLE; o fone aceita uma conexão por vez. |

## Compatibilidade e limitações

- Validado com o **QCY Melobuds Pro no firmware 2.0.6**. A matriz de
  compatibilidade e o que muda em outras versões estão em
  [`PROTOCOLO_MelobudsNext.md`](PROTOCOLO_MelobudsNext.md).
- Bateria e versão do firmware são lidas ao conectar e em **Atualizar**; não há
  atualização periódica dessas duas leituras.
- O mapa de toque é relido pela interface ociosa a cada ~3 s (ele não gera
  notificação).
- Alguns ecos de estado (balanço, volume, detecção de uso e modo sono) podem
  chegar depois da janela do comando; o `device info` seguinte confirma.
- O byte de resultado do `0x28` é preservado, mas ainda não interpretado;
  `0x2F`/`0x30` não são emitidos pelo firmware 2.0.6.
- Não há suporte validado para macOS, não há pacote no AUR (instalação por
  clone + `makepkg`) e o executável de Windows não é assinado.
- O backlog completo e o que está em aberto ficam em
  [`PENDENCIAS.md`](PENDENCIAS.md).

### Status de validação

- **Linux:** Arch Linux (kernel `7.3.5-arch1-1`, Wayland, BlueZ 5.88), firmware
  2.0.6 — CLI, GUI, ANC, LDAC com reconexão e notificações.
- **Windows:** `melobudsnext-gui.exe` executado e conectado ao fone em um
  Windows real (validação do usuário, 2026-11-22); versão do Windows e firmware
  não registrados.
- **Arch:** instalação via `makepkg -si` validada em Arch Linux x86_64
  (2026-11-22), com menu e ícone.

O roteiro para repetir os testes está em
[`tests/manual/validacao-hardware.md`](tests/manual/validacao-hardware.md).

## Compilar da fonte

Requisitos: Rust 1.85 ou superior (`rustup`) e um adaptador Bluetooth acessível
pelo sistema.

```sh
git clone https://github.com/Leandro-74/MelobudsNext-Rust.git
cd MelobudsNext-Rust
cargo build --release
```

Isso gera os dois binários: `melobudsnext` (CLI) e `melobudsnext-gui`
(interface iced). Para compilar só o CLI, sem baixar o `iced`:

```sh
cargo build --release --no-default-features
```

<details>
<summary>Dependências de compilação por distro</summary>

Arch Linux:

```sh
sudo pacman -S --needed base-devel rust pkgconf dbus bluez bluez-utils \
  libxkbcommon wayland
```

Debian/Ubuntu:

```sh
sudo apt install build-essential pkg-config libdbus-1-dev \
  libxkbcommon-dev libwayland-dev bluez
```

Fedora:

```sh
sudo dnf install gcc pkgconf-pkg-config dbus-devel libxkbcommon-devel \
  wayland-devel bluez
```

`libxkbcommon` e `wayland` só são necessários para a GUI.

</details>

Testes:

```sh
cargo test --all-targets
```

O projeto documenta o protocolo, mantém testes com dublê de transporte e uma CI
que roda `fmt`, `clippy -D warnings` e a suíte em Linux e Windows. Para gerar
pacote Arch, artefatos de Linux/Windows e publicar uma release, siga
[`packaging/README.md`](packaging/README.md).

## Licença

MIT. Veja [`LICENSE`](LICENSE).
