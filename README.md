# MelobudsNext (Rust)

CLI para testar o protocolo BLE do QCY Melobuds Pro, portada do projeto Python
em `MelobudsNext/`. O foco é comunicar com o fone, enviar comandos e inspecionar
o estado sem precisar de interface gráfica. Também há uma GUI iced no binário
`melobudsnext-gui`, descrita mais abaixo.

O protocolo está documentado em [`PROTOCOLO_MelobudsNext.md`](PROTOCOLO_MelobudsNext.md).

## Requisitos

- Rust 1.85 ou superior (`rustup` é o caminho mais simples) e `cargo`.
- Um adaptador Bluetooth acessível pelo sistema operacional.
- Linux: BlueZ, D-Bus e as bibliotecas de desenvolvimento listadas abaixo.
- Windows: a pilha Bluetooth do Windows 10/11 e uma toolchain compatível.

### Linux (BlueZ/D-Bus)

O `btleplug` conversa com o BlueZ pelo D-Bus do sistema. Instale o daemon, as
ferramentas e as dependências de compilação, libere o adaptador e garanta que o
seu usuário possa acessá-lo.

Arch Linux:

```sh
sudo pacman -S --needed base-devel rust pkgconf dbus bluez bluez-utils \
  libxkbcommon wayland
sudo systemctl enable --now bluetooth
sudo usermod -aG bluetooth "$USER"
sudo rfkill unblock bluetooth
bluetoothctl power on
```

Debian/Ubuntu:

```sh
sudo apt update
sudo apt install build-essential pkg-config libdbus-1-dev \
  libxkbcommon-dev libwayland-dev bluez
sudo systemctl enable --now bluetooth
sudo usermod -aG bluetooth "$USER"
sudo rfkill unblock bluetooth
bluetoothctl power on
```

Fedora:

```sh
sudo dnf install gcc pkgconf-pkg-config dbus-devel libxkbcommon-devel \
  wayland-devel bluez
sudo systemctl enable --now bluetooth
sudo usermod -aG bluetooth "$USER"
sudo rfkill unblock bluetooth
bluetoothctl power on
```

Depois de `usermod`, saia da sessão e entre novamente para o grupo valer. As
operações privilegiadas do BlueZ também podem pedir autorização pelo polkit;
aceite o pedido ou configure a política para o seu usuário. `libxkbcommon` e
`wayland` são necessários para compilar a GUI; quem usa só o CLI pode compilar
com `--no-default-features` sem essas bibliotecas.

Verificação rápida:

```sh
bluetoothctl show          # o controlador deve aparecer como "Powered: yes"
rfkill list bluetooth      # não deve estar "Soft blocked: yes"
```

Se o fone estiver conectado ao celular ou ao app oficial, desconecte-o: ele
aceita um host por vez.

### Windows

A toolchain padrão (`stable-x86_64-pc-windows-msvc`) precisa do Visual Studio
Build Tools com a opção "Desenvolvimento para desktop com C++". Alternativa sem
Visual Studio:

```powershell
rustup toolchain install stable-x86_64-pc-windows-gnu
winget install BrechtSanders.WinLibs.POSIX.UCRT
cargo +stable-x86_64-pc-windows-gnu build --release
```

O Windows 10/11 já traz a pilha Bluetooth; não é necessário instalar BlueZ. O
adaptador precisa estar ligado e com o driver instalado.

## Compilação

```sh
cargo build --release
```

Por padrão são gerados os dois binários: `melobudsnext` (CLI) e
`melobudsnext-gui` (interface iced). A GUI fica atrás da feature `gui`, que vem
ligada por padrão. Para compilar apenas o CLI, sem baixar nem compilar o `iced`:

```sh
cargo build --no-default-features
cargo build --no-default-features --release
```

Nesse modo o binário `melobudsnext-gui` não é gerado e `cargo run --bin
melobudsnext-gui` falha porque o alvo exige a feature. Os testes e o clippy
podem ser executados da mesma forma com `--no-default-features`.

Para gerar um pacote instalável no Arch Linux ou os artefatos de release para
Linux e Windows, siga [`packaging/README.md`](packaging/README.md).

## Instalação

### Arch Linux (`makepkg`, sem AUR)

O `PKGBUILD` fica em `packaging/aur` e compila a tag `vX.Y.Z` publicada no
GitHub. Enquanto a tag não existe, o build usa a fonte local (o próprio clone)
através de `git archive`:

```sh
git clone https://github.com/Leandro-74/MelobudsNext-Rust.git
cd MelobudsNext-Rust
./packaging/aur/build-local.sh -si
```

O script gera um tarball do commit atual, grava o SHA-256 dele na linha
`sha256sums` do `PKGBUILD` e roda `makepkg`. Confirme a instalação:

```sh
melobudsnext --version
melobudsnext-gui --version
```

O fluxo detalhado, os arquivos instalados e os passos para publicar a tag (e,
depois, o AUR) estão em [`packaging/README.md`](packaging/README.md).

### Artefatos de release

Cada release publica um `.tar.gz` para Linux, um `melobudsnext-gui.exe` único
para Windows (CRT estático, sem instalador) e os arquivos de checksum. Confira o
checksum antes de instalar:

```sh
cd dist
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

No Windows, baixe `melobudsnext-gui.exe` e o `.sha256` correspondente, confira o
hash e execute o `.exe`:

```powershell
Get-FileHash .\melobudsnext-gui.exe -Algorithm SHA256
```

O binário não é assinado, então o SmartScreen pode pedir confirmação (**Mais
informações → Executar assim mesmo**). O CLI de Windows fica em um `.zip`
opcional ou pode ser compilado da fonte. O fluxo completo de geração dos
artefatos e da tag está em [`packaging/README.md`](packaging/README.md).

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
Quando o fone envia o resultado `0x28` depois de uma mudança de ANC, ele aparece
como `ANC wear/result` (e como `anc_wear` no JSON); esse evento não substitui o
estado de ANC.

Se algum desses campos opcionais falhar, o comando continua e mostra o problema
como `warning:` em vez de abortar.

## Comandos do dispositivo

Cada ação abaixo sincroniza o estado, envia o comando, aplica os echos e imprime
o estado resultante. `--wait SEGUNDOS` controla a espera pelos echos (padrão 1 s)
e `--json` imprime apenas o JSON em `stdout` (mensagens vão para `stderr`).
Com `--no-sync`, a conexão é aberta sem sincronizar o estado: o comando é
enviado imediatamente e a saída mostra apenas os campos devolvidos pelo eco (ou
pelo sync automático de um comando LDAC).

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
| `device ldac on` | Liga LDAC e reconecta (o fone reinicia); `--reconnect-attempts` e `--reconnect-delay` ajustam a espera |
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

## Interface gráfica

O binário `melobudsnext-gui` oferece um painel para o dia a dia sem decorar
comandos. Ele reutiliza a mesma camada BLE, o mesmo protocolo e o mesmo arquivo
de configuração do CLI:

```sh
cargo run --bin melobudsnext-gui
```

Se houver um `mac_address` na configuração, a GUI tenta conectar sozinha ao
abrir. O cabeçalho permite conectar, atualizar e desconectar; as abas cobrem
status, ANC, áudio (LDAC, modo jogo/sono, balanço), toque e detecção de uso.
LDAC e os modos jogo/sono são enviados na hora; ANC, sliders, mapa de toque e
nome usam um rascunho local e o botão **Aplicar**. Depois de um comando
bem-sucedido, o valor escolhido permanece na interface mesmo quando o firmware
não devolve o eco dentro da janela de escuta.

A GUI mantém o canal de notificações aberto em segundo plano. Mudanças feitas no
próprio fone (alternar o ANC pelo toque, mudar o modo pelo app oficial etc.)
aparecem sem clicar em **Atualizar**, e só os rascunhos dos campos alterados são
recarregados. O mapa de toque não gera notificação, então ele é relido pela GUI
ociosa a cada ~3 s.

Para abrir a interface sem acessar o Bluetooth (útil para conferir o layout ou
gravar capturas), use:

```sh
cargo run --bin melobudsnext-gui -- --offline
```

### Flags da interface gráfica

`melobudsnext-gui --help` lista as opções e `melobudsnext-gui --version` imprime
a versão do pacote. A tabela resume o comportamento:

| Flag | Efeito |
| --- | --- |
| `--offline` | Abre a interface sem acessar o Bluetooth (demonstração e capturas). |
| `--config PATH` | Usa outro arquivo de configuração; se não puder lê-lo, avisa no rodapé e segue com os padrões. |
| `--reconnect-attempts N` | Tentativas de reconexão após o reinício do LDAC (padrão 10; mínimo 1). |
| `--reconnect-delay SEGUNDOS` | Intervalo entre as tentativas (padrão 3; aceita 0 a 60). |
| `--help` | Mostra a ajuda e sai. |
| `--version` | Mostra a versão do pacote e sai. |

O modo offline também pode ser ativado por `MELOBUDNEXT_GUI_OFFLINE=1`; a flag e
a variável são combinadas com OU, ou seja, qualquer uma delas liga o modo.
Valores como `true`, `yes` e `on` também ligam; `0`, `false`, `no`, `off` e vazio
desligam. Nesse modo o botão de conexão e todos os controles que gravam no fone
ficam desabilitados; os rascunhos locais (abas, sliders e seletores) continuam
respondendo e o rodapé mostra um aviso.

A janela tem tamanho fixo de 1200x800 (layout de dashboard) e usa o renderer
**tiny-skia** (CPU). O wgpu não foi usado porque, neste ambiente, ele abria uma
janela branca; a ilustração dos fones é composta com widgets padrão do iced para
evitar deslocamentos de canvas/SVG em telas com escala de 125%.

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

## Validação em Linux

O roteiro manual está em
[`tests/manual/validacao-hardware.md`](tests/manual/validacao-hardware.md). Ele
cobre descoberta, conexão, leituras diretas, comandos, notificações, LDAC,
queda de link e a GUI, sempre sem registrar identificadores do fone.

Execução registrada em **2026-11-22**:

- **Plataforma:** Arch Linux, kernel `7.3.5-arch1-1` x86_64, sessão Wayland
  (Hyprland), BlueZ 5.88.
- **Firmware:** 2.0.6.
- **CLI:** scan, `device info`, leituras de bateria/firmware/toque, game/sleep,
  todas as cenas de ANC e transparência, balanço, volume de notificação,
  detecção de uso, renomear (com restauração do nome), `query`, `send`,
  `inspect` e `monitor` funcionaram.
- **LDAC:** `device ldac off` e `device ldac on` com
  `--reconnect-attempts 3 --reconnect-delay 1` reconectaram (1 a 2 tentativas),
  terminaram com o estado final correto e sem "BLE link lost".
- **Notificações:** `0x28` foi capturado como `FF 03 28 01 01` depois de
  mudanças de ANC; `0x2F`/`0x30` não foram emitidos e as consultas `0xFE` para
  eles não responderam; `0x14` (auto-desligar) respondeu
  `FF 06 14 04 FF FF 00 00`.
- **GUI:** `--help`, `--version` e `--offline --config <fictício>` abriram sem
  erro (a janela foi encerrada pelo timeout do teste). O toggle LDAC foi
  acionado com o fone conectado: a interface mostrou o progresso, reconectou e
  exibiu "LDAC desativado; o fone foi reconectado." sem erro.
- **Limitações encontradas:** alguns ecos de estado chegam depois da janela do
  comando (MEL-05); o `device monitor` continuou aberto quando o link foi
  derrubado por outro processo.

Para repetir a validação, siga o roteiro e registre plataforma, firmware e
resultados sem dados sensíveis.

## Configuração

O caminho padrão é `~/.melobudsnext/config.json` (ou `%USERPROFILE%\.melobudsnext\config.json`).
Use `--config PATH` no CLI ou na GUI para apontar para outro arquivo. Se
`HOME`/`USERPROFILE` não estiver definido, o CLI retorna um erro em vez de gravar
no diretório atual; nesse caso informe `--config PATH`. A GUI também aceita
`--config PATH` e avisa no rodapé quando o arquivo pedido não pode ser lido,
seguindo com os valores padrão.

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

Ao gravar (`config set-mac` ou `config init`), o CLI reescreve o arquivo no
formato novo com `mac_address`; se o mesmo arquivo for compartilhado com o TUI
Python, faça uma cópia antes, porque ele espera a chave `address`.

## Comportamento e diferenças em relação ao Python

- Decodificação tolerante: ANC desconhecido é preservado como `unknown(...)`; na
  GUI ele aparece como "Desconhecido" e o botão **Aplicar ANC** fica
  desabilitado até que um modo conhecido seja escolhido, em vez de reenviar
  "desligado". Bateria de 2 bytes é aceita e o touch aceita qualquer quantidade
  par de bytes; funções de toque desconhecidas aparecem como
  `Desconhecida (0xXX)` no seletor e são preservadas ao salvar.
- `rename` vazio não apaga o nome atual.
- O `sync` não aborta por causa de uma leitura opcional; falhas viram avisos.
- As consultas `0xFE` são enviadas a cada 50 ms enquanto as notificações são
  lidas, evitando perder respostas. O `sync` termina assim que todos os opcodes
  consultados respondem, mantendo 100 ms de tolerância para notificações
  atrasadas; se algum não responder, espera a janela completa (`--wait`).
- A subscrição de notificações é aberta uma vez por sessão e reutilizada por
  todas as ações; ela só é refeita quando a conexão é restabelecida.
- Depois de mudar LDAC, a conexão é refeita automaticamente, como no cliente
  Python: por padrão são 10 tentativas com 3 s entre elas. Durante a espera, o
  CLI imprime `Reconnect attempt N/10...` (ou em `stderr` com `--json`) e a GUI
  atualiza o cabeçalho com a tentativa atual. Use `--reconnect-attempts` e
  `--reconnect-delay` no CLI (ou na GUI) para ajustar a quantidade e o intervalo;
  `--reconnect-delay 0` remove a pausa entre as tentativas. O reinício do fone
  derruba o link antes do eco; essa queda **depois** da escrita faz parte da
  reconexão e não é reportada como "BLE link lost". Uma falha de escrita, ou uma
  queda de link em comando que não reinicia o fone, continua sendo erro.
- Na GUI, cada ação verifica o link ao terminar. Se o fone foi desligado, a
  sessão é descartada, a interface volta para "Desconectado" e o erro aparece no
  rodapé em vez de manter um estado antigo.
- Se o adaptador Bluetooth reporta estado desconhecido (`Unknown`) logo após a
  inicialização, o comando tenta novamente por até 1 s (10 tentativas de 100 ms)
  antes de falhar, em vez de abortar imediatamente.
- A GUI processa notificações em segundo plano e atualiza o estado ao vivo. Como
  a characteristic de toque não notifica, o mapa é relido periodicamente
  enquanto a interface está ociosa; os demais campos de estado chegam por
  notificação.
- Saída `--json` é JSON puro em `stdout`.
- Não há TUI; os comandos do CLI são de uma execução só, com `device monitor`
  para acompanhar notificações. A interface gráfica é um binário separado
  (`melobudsnext-gui`) e compartilha a mesma camada de BLE e protocolo.

## Limitações conhecidas

- O opcode `0x28` (ANC Wear/Result) é exibido como `ANC wear/result` e o payload
  cru é preservado. O firmware 2.0.6 envia `01` depois de mudanças de ANC, mas o
  significado desse byte não é documentado; o evento não altera o estado de ANC.
- Notificações `0x2F`/`0x30` (bateria/versão pelo canal de comando) são
  decodificadas se chegarem, mas o firmware 2.0.6 não as envia e as consultas
  `0xFE` para esses opcodes não respondem. O projeto continua usando as
  characteristics diretas para essas leituras.
- Bateria e versão do firmware são lidas ao conectar e em **Atualizar**; não há
  atualização periódica dessas duas characteristics.
- O mapa de toque tem atualização periódica só na GUI ociosa (~3 s). No CLI, use
  `device read touch` ou `device info` para reler.
- O reinício do LDAC derruba o link por alguns segundos; o comando só termina
  depois da reconexão ou do esgotamento das tentativas configuradas.
- Alguns ecos de estado (balanço, volume, detecção de uso e modo sono) podem
  chegar depois da janela do comando, deixando a saída uma ação atrasada; o
  `device info` seguinte confirma o valor (MEL-05).
- A matriz de compatibilidade cobre o firmware 2.0.6 validado; versões de
  terceiros não foram reproduzidas. Consulte
  [`PROTOCOLO_MelobudsNext.md`](PROTOCOLO_MelobudsNext.md) e o backlog em
  [`PENDENCIAS.md`](PENDENCIAS.md).
- O `melobudsnext-gui.exe` de Windows é gerado por build cruzado (`cargo-xwin`)
  com CRT estático e ícone embutido, sem depender do Redistributable. A execução
  e a conexão com o fone foram validadas manualmente pelo usuário em um Windows
  real; a instalação do pacote Arch em ambiente limpo segue pendente.

## Testes

```sh
cargo test
```

Os testes cobrem framing, opcodes, ANC, touch, bateria, firmware, estado,
compatibilidade do arquivo de configuração com o formato Python e o controlador
de ações com um dublê de transporte.
