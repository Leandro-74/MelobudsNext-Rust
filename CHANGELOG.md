# Changelog

Formato baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/).
O projeto segue versionamento semântico.

## [0.2.0] — 2026-11-22

Primeira release publicável do port em Rust. Implementa o protocolo BLE do QCY
Melobuds Pro (firmware validado: 2.0.6) com um CLI e uma GUI iced.

### Adicionado

- CLI com `config`, `device` (`scan`, `info`, `query`, `send`, `read`,
  `inspect`, `monitor`, `battery`, `firmware`, `rename`, `game`, `sleep`,
  `anc`, `touch`, `balance`, `notify-volume`, `wear`, `ldac`), `build` e
  `decode`.
- GUI iced com abas de status, ANC, áudio, toque e detecção de uso, modo
  offline (`--offline`/`MELOBUDNEXT_GUI_OFFLINE`), `--config PATH` e estado em
  tempo real alimentado por uma subscrição BLE persistente.
- Camada compartilhada de ações (`actions::Controller`/`Transport`) entre CLI e
  GUI, com reconexão do LDAC e progresso de tentativas.
- Testes com dublê de transporte (conectar, sincronizar, refresh, reconectar,
  queda de link, ecos tardios) e de estado da GUI.
- Empacotamento: `PKGBUILD` para Arch Linux (instalação via `makepkg`, sem
  AUR), `.desktop`, ícone, script de release de Linux e executável único de
  Windows (`melobudsnext-gui.exe`) com CRT estático e ícone embutido.
- CI com `fmt`, `clippy -D warnings` e testes em Linux e Windows, `LICENSE`
  MIT e `.gitignore` com segredos e artefatos.
- Documentação do protocolo, das opcodes observados, da matriz de firmware, do
  roteiro de validação manual e dos pré-requisitos por plataforma.

### Corrigido

- Leitura booleana de `MELOBUDNEXT_GUI_OFFLINE` e bloqueio das ações BLE no
  modo offline.
- Detecção de link morto (a GUI volta para "Desconectado"), toggles otimistas
  que revertiam sozinhos e ANC/função de toque desconhecidos exibidos como
  valores válidos.
- `Packet::try_encode` falível em vez de pânico, escrita atômica da configuração
  e fallback de diretório `HOME` sem gravar no diretório atual.
- Reconexão do LDAC sem erro de "BLE link lost" e sem travar a GUI ociosa.
- `sync` com encerramento antecipado, stream de notificações persistente,
  tolerância a adaptador em `Unknown` e `--no-sync` no CLI.

### Limitações conhecidas

- Ecos de `0x16`, `0x1D`, `0x2C` e `0x10` podem chegar depois da janela do
  comando; o `device info` seguinte confirma o valor (MEL-05).
- O quarto byte do `0x2C` varia e segue sem significado confirmado (PRO-05).
- O byte de resultado do `0x28` é preservado, mas não interpretado; `0x2F`/`0x30`
  nunca foram emitidos pelo firmware 2.0.6.
- Bateria e versão são lidas ao conectar e em **Atualizar**; não há atualização
  periódica dessas duas characteristics.
- O pacote não está no AUR (instalação por clone + `makepkg`), o `.exe` não é
  assinado (aviso do SmartScreen) e não há validação em macOS.
- A matriz de compatibilidade cobre o firmware 2.0.6; outras versões não foram
  reproduzidas.

[0.2.0]: https://github.com/Leandro-74/MelobudsNext-Rust/releases/tag/v0.2.0
