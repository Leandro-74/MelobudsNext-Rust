# Pendências do MelobudsNext (Rust)

Levantamento feito a partir da análise do repositório no commit `4028131`
(`fix: corrige a interface GUI em iced`).

## Baseline verificado

- `cargo test --all-targets`: **59 testes passando** (53 unitários + 6 de integração).
- `cargo clippy --all-targets`: **sem warnings**.
- `cargo fmt --check`: **limpo**.
- Worktree limpo, 7 commits, sem alterações pendentes.
- Funcionalidades do cliente Python já portadas: game mode, sleep mode, ANC
  (cenas e níveis), transparência, renomear, mapa de toque, volume de
  notificação, balanço de canais, detecção de uso, LDAC, bateria, firmware,
  scan, inspect, monitor e GUI iced.

## Legenda

| Prioridade | Significado |
| --- | --- |
| P1 | Alta: defeito visível, risco de estado incorreto ou bloqueio de uso |
| P2 | Média: melhoria importante, mas com contorno |
| P3 | Baixa: depende de hardware, decisão de produto ou acabamento |

| Categoria | Escopo |
| --- | --- |
| Correção | Defeito de comportamento ou de conteúdo já existente |
| Refatoração | Reorganização interna sem mudança de comportamento |
| Implementação | Lacuna funcional: recurso ainda não existente |
| Melhoria | Robustez, desempenho ou experiência de uso |
| Teste | Cobertura automatizada ausente |
| Documentação | Conteúdo incorreto, desatualizado ou faltante |
| Infraestrutura | Build, licença, CI, empacotamento e higiene do repositório |
| Protocolo/Hardware | Pendências que dependem de captura ou validação no fone |

## Resumo

| ID | Categoria | Prioridade | Título |
| --- | --- | --- | --- |
| COR-01 | Correção | P2 | Primeira linha do documento do protocolo corrompida |
| COR-02 | Correção | P2 | `MELOBUDNEXT_GUI_OFFLINE` trata qualquer valor como ligado |
| COR-03 | Correção | P1 | Ações da GUI continuam ativas no modo offline |
| COR-04 | Correção | P1 | GUI pode exibir "Conectado" com o link BLE morto |
| COR-05 | Correção | P1 | Toggles otimistas podem reverter sozinhos |
| COR-06 | Correção | P1 | ANC desconhecido aparece como "Desligado" na GUI |
| COR-07 | Correção | P2 | Função de toque desconhecida fica invisível no seletor |
| COR-08 | Correção | P2 | `Packet::encode` entra em pânico em vez de retornar erro |
| COR-09 | Correção | P2 | Escrita da configuração não é atômica |
| COR-10 | Correção | P2 | GUI não tem parser de argumentos (`--help`, `--version`) |
| COR-11 | Correção | P1 | LDAC retorna erro de link perdido em vez de reconectar |
| COR-12 | Correção | P1 | Escuta ao vivo pode impedir ações da GUI |
| REF-01 | Refatoração | P1 | Duplicação da orquestração entre CLI e GUI |
| REF-02 | Refatoração | P2 | `gui.rs` monolítico (2.158 linhas) |
| REF-03 | Refatoração | P1 | Falta abstração de transporte para testes |
| REF-04 | Refatoração | P2 | Re-subscribe das notificações a cada comando |
| REF-05 | Refatoração | P2 | `sync` sempre espera a janela inteira |
| REF-06 | Refatoração | P3 | Código público sem uso (`BleSession::query_all`) |
| IMP-01 | Implementação | P2 | Estado em tempo real na GUI |
| IMP-02 | Implementação | P3 | GUI não aceita `--config PATH` |
| IMP-03 | Implementação | P3 | Notificações `0x2F`/`0x30` (bateria/versão) ignoradas |
| MEL-01 | Melhoria | P2 | CLI faz sync completo antes de cada comando |
| MEL-02 | Melhoria | P2 | `ensure_powered` falha quando o estado é `Unknown` |
| MEL-03 | Melhoria | P3 | Reconexão LDAC longa e sem progresso |
| MEL-04 | Melhoria | P2 | Fallback do diretório `home` grava no diretório atual |
| MEL-05 | Melhoria | P2 | Ecos de estado podem chegar depois da janela do comando |
| TST-01 | Teste | P2 | Sem testes para helpers do CLI |
| TST-02 | Teste | P1 | Fluxos de sessão sem dublê de transporte |
| TST-03 | Teste | P2 | Sem testes para transições de estado da GUI |
| TST-04 | Teste | P2 | Sem testes para erros de leitura/escrita da configuração |
| TST-05 | Teste | P3 | Tolerância do framing a lixo/`body_len` não é explícita |
| TST-06 | Teste | P1 | Regressões dos defeitos COR-* ainda não cobertas |
| TST-07 | Teste | P3 | Sem checklist/script de validação com hardware |
| TST-08 | Teste | P3 | Helpers de diretório temporário podem colidir |
| DOC-01 | Documentação | P1 | Seção 11 do protocolo descreve pendências do projeto Python |
| DOC-02 | Documentação | P2 | Exemplo composto contradiz `body_len` |
| DOC-03 | Documentação | P2 | README não cobre pré-requisitos de BLE no Linux |
| DOC-04 | Documentação | P3 | Flags da GUI não documentadas |
| DOC-05 | Documentação | P2 | Limitações atuais (0x28, tempo real) não estão no README |
| INF-01 | Infraestrutura | P2 | Falta arquivo `LICENSE` (MIT declarado no Cargo.toml) |
| INF-02 | Infraestrutura | P2 | `.gitignore` mínimo (só `/target`) |
| INF-03 | Infraestrutura | P2 | Sem CI (fmt, clippy, testes, build) |
| INF-04 | Infraestrutura | P3 | `iced` compilado mesmo para quem só usa o CLI |
| INF-05 | Infraestrutura | P3 | Sem empacotamento (AUR, `.desktop`, release Windows) |
| INF-06 | Infraestrutura | P2 | Instalação no Arch via `makepkg` antes do AUR |
| INF-07 | Infraestrutura | P2 | Executável Windows único (GUI) com CRT estático |
| PRO-01 | Protocolo/Hardware | P3 | Opcode `0x28` (ANC Wear) não decodificado |
| PRO-02 | Protocolo/Hardware | P3 | Auto-desligar e opcodes desconhecidos não explorados |
| PRO-03 | Protocolo/Hardware | P2 | Validação em Linux ainda não registrada |
| PRO-04 | Protocolo/Hardware | P3 | Sem matriz de compatibilidade por firmware |
| PRO-05 | Protocolo/Hardware | P3 | Quarto byte do `0x2C` varia e não está confirmado |

---

## Correções

### COR-01 — Primeira linha do documento do protocolo corrompida
- **Prioridade:** P2
- **Onde:** `PROTOCOLO_MelobudsNext.md:1`
- **Problema:** o arquivo começa com `121# MelobudsNext: Documentação do Protocolo BLE`
  (número de linha colado no conteúdo). O defeito existe desde o primeiro commit.
- **Ação:** trocar por `# MelobudsNext: Documentação do Protocolo BLE`.
- **Pronto quando:** a primeira linha for um título Markdown válido.

### COR-02 — `MELOBUDNEXT_GUI_OFFLINE` trata qualquer valor como ligado
- **Prioridade:** P2
- **Onde:** `src/bin/gui.rs:254`
- **Problema:** a variável é detectada por presença (`is_some()`), então
  `MELOBUDNEXT_GUI_OFFLINE=0` ou `false` também ativam o modo offline.
- **Ação:** interpretar valores (`1`, `true`, `yes` ligam; `0`, `false`, vazio
  desligam) ou simplesmente documentar que a variável é um booleano de presença.
- **Pronto quando:** houver teste cobrindo a interpretação do valor.

### COR-03 — Ações da GUI continuam ativas no modo offline
- **Prioridade:** P1
- **Onde:** `src/bin/gui.rs:296-299` (`dispatch`), `:1217` e demais botões,
  `src/bin/gui/ble_task.rs:207`
- **Problema:** no modo offline apenas o botão **Conectar** fica desabilitado.
  Sliders, toggles e **Aplicar** disparam ações que falham com
  "fone não conectado; clique em Conectar", contrariando o propósito de
  demonstração/captura de tela.
- **Ação:** bloquear todas as ações BLE quando `offline` (ou aplicar o efeito
  apenas no estado local, se a intenção for simular).
- **Pronto quando:** nenhum controle de escrita puder gerar erro de conexão
  no modo offline.

### COR-04 — GUI pode exibir "Conectado" com o link BLE morto
- **Prioridade:** P1
- **Onde:** `src/bin/gui/ble_task.rs:96`, `src/ble.rs:371-439`
- **Problema:** `connected` é `client.is_some()`. Em `sync`, falhas de escrita
  das consultas viram apenas warning e a função retorna `Ok`; se o fone
  desligar, a GUI continua mostrando "Conectado" com estado velho.
- **Ação:** consultar `Peripheral::is_connected()` (btleplug) ao final das ações
  e descartar o cliente quando a escrita falhar por desconexão; propagar erro
  em caso de link perdido.
- **Pronto quando:** desligar o fone fizer a GUI voltar para "Desconectado"
  na próxima ação/atualização.

### COR-05 — Toggles otimistas podem reverter sozinhos
- **Prioridade:** P1
- **Onde:** `src/bin/gui.rs:458-513`, `src/bin/gui/ble_task.rs:134-171`
- **Problema:** Game/Sleep/LDAC alteram `app.state` antes de despachar e usam
  `refresh_drafts: false`. Se o eco não chegar na janela de 450 ms
  (`ble_task.rs:21`), o `ActionResult` sobrescreve o estado com a cópia antiga
  da sessão e o toggle "pula" de volta.
- **Ação:** atualizar o estado da sessão no sucesso do comando (ou sempre
  recarregar o estado ao final) para que a UI reflita o que foi enviado.
- **Pronto quando:** um comando bem-sucedido sem eco mantiver o valor escolhido.

### COR-06 — ANC desconhecido aparece como "Desligado" na GUI
- **Prioridade:** P1
- **Onde:** `src/bin/gui.rs:336-340`
- **Problema:** `AncMode::Unknown(_)` é mapeado para `AncChoice::Off` em
  `load_drafts`, mascarando o estado real reportado pelo firmware e permitindo
  que um simples **Aplicar** envie `Off`.
- **Ação:** representar o estado desconhecido na UI (rótulo próprio e nenhuma
  seleção válida) sem convertê-lo silenciosamente em desligado.
- **Pronto quando:** um ANC desconhecido não for exibido nem reenviado como off.

### COR-07 — Função de toque desconhecida fica invisível no seletor
- **Prioridade:** P2
- **Onde:** `src/bin/gui.rs:145-160`, `:1245-1266`
- **Problema:** `TouchOption::ALL` não inclui `TouchFunction::Unknown`, então um
  valor não mapeado (ex.: `0x42`) aparece em branco. O rascunho preserva o
  valor, mas a UI não informa o que está gravado.
- **Ação:** exibir uma opção "Desconhecida (0xXX)" desabilitada/somente leitura.
- **Pronto quando:** o valor desconhecido for visível e preservado ao salvar.

### COR-08 — `Packet::encode` entra em pânico em vez de retornar erro
- **Prioridade:** P2
- **Onde:** `src/protocol/framing.rs:26-30`
- **Problema:** `assert!` aborta o processo se `params.len() > 253`. Os caminhos
  atuais validam antes, mas uma biblioteca não deveria entrar em pânico.
- **Ação:** adicionar `try_encode() -> Result<...>` (ou tornar `encode`
  falível) e manter `encode` apenas como atalho documentado, se necessário.
- **Pronto quando:** entradas inválidas virarem erro tratável, com teste.

### COR-09 — Escrita da configuração não é atômica
- **Prioridade:** P2
- **Onde:** `src/config.rs:127-137`
- **Problema:** `save_to` grava direto no destino; uma interrupção no meio da
  escrita pode deixar o JSON truncado e quebrar as próximas execuções.
- **Ação:** gravar em arquivo temporário no mesmo diretório e renomear por
  cima do destino.
- **Pronto quando:** falha simulada de escrita não corromper o arquivo original.

### COR-10 — GUI não tem parser de argumentos
- **Prioridade:** P2
- **Onde:** `src/bin/gui.rs:254`, `:2149`
- **Problema:** os argumentos são lidos com `std::env::args().any(...)`; não há
  `--help`, `--version` nem validação, e qualquer argumento posicional é
  ignorado silenciosamente.
- **Ação:** migrar para `clap` (já é dependência) com `--offline` e,
  opcionalmente, `--config`.
- **Pronto quando:** `melobudsnext-gui --help` listar as flags suportadas.

### COR-11 — LDAC retorna erro de link perdido em vez de reconectar
- **Prioridade:** P1
- **Onde:** `src/actions.rs` (`Controller::command_with`), `src/ble.rs`
  (`BleSession::exchange`)
- **Problema:** ao enviar `device ldac off` ou `on` no firmware 2.0.6, o fone
  reinicia e derruba o link antes de o `exchange` terminar. O `exchange` chama
  `ensure_link` depois da escrita e devolve `BLE link lost; connect again`; o
  controlador só inicia a reconexão depois que `exchange` retorna `Ok`, então o
  comando termina em erro mesmo tendo sido aplicado. O LDAC muda de estado
  (confirmado por um `device info` seguinte), mas o CLI/GUI reportam falha e a
  GUI pode descartar a sessão.
- **Ação:** tratar a queda de link prevista nesse comando como sucesso da
  escrita e seguir para a política de reconexão; cobrir com dublê de transporte
  em que o `exchange` falha por link morto depois de registrar o pacote.
- **Pronto quando:** `device ldac on/off` concluir a reconexão e exibir o estado
  final, sem erro de link perdido.
- **Descoberto em:** validação manual do Lote 6 (Linux, firmware 2.0.6).
- **Resolvido no:** Lote 8.

### COR-12 — Escuta ao vivo pode impedir ações da GUI
- **Prioridade:** P1
- **Onde:** `src/bin/gui/app.rs` (`live_updates`), `src/bin/gui/ble_task.rs`
  (`Session::poll`/`Session::run`)
- **Problema:** quando a GUI fica ociosa, o laço de notificações reacquire o
  mutex da sessão logo depois de cada espera. Uma ação enfileirada (por exemplo,
  o toggle LDAC, que precisa reconectar) pode nunca receber o mutex: a interface
  fica presa em "Alterando LDAC e reconectando..." e o comando não é enviado.
  Descoberto na validação de hardware do Lote 8, ao testar o toggle LDAC.
- **Ação:** ceder o executor entre as esperas ociosas, para que a ação
  enfileirada assuma a sessão antes da próxima escuta.
- **Pronto quando:** uma ação disparada com a GUI ociosa sempre começa e termina,
  sem depender do intervalo das notificações.
- **Descoberto em:** validação manual do Lote 8 (Linux, firmware 2.0.6).
- **Resolvido no:** Lote 8.

---

## Refatorações

### REF-01 — Duplicação da orquestração entre CLI e GUI
- **Prioridade:** P1
- **Onde:** `src/main.rs:654-702` (`run_action`), `src/bin/gui/ble_task.rs:107-205`
- **Problema:** o fluxo "sync → enviar → aplicar ecos → reconectar → sync" está
  implementado duas vezes, com constantes e tratamentos de erro próprios.
- **Ação:** extrair uma camada de ações/controlador compartilhada na `lib`
  (ex.: `actions.rs`) consumida pelos dois binários.
- **Pronto quando:** CLI e GUI só cuidarem de entrada/saída e a orquestração
  existir em um único lugar.

### REF-02 — `gui.rs` monolítico
- **Prioridade:** P2
- **Onde:** `src/bin/gui.rs` (2.158 linhas)
- **Problema:** paleta, widgets, abas, estilos e estado no mesmo arquivo
  dificultam navegação e revisão.
- **Ação:** dividir em `gui/app.rs`, `gui/theme.rs`, `gui/widgets.rs`,
  `gui/tabs/*.rs`, seguindo o padrão já usado em `gui/ble_task.rs`.
- **Pronto quando:** nenhum arquivo da GUI passar de ~600 linhas.

### REF-03 — Falta abstração de transporte para testes
- **Prioridade:** P1
- **Onde:** `src/ble.rs`, `src/bin/gui/ble_task.rs`
- **Problema:** `BleSession` é concreto e depende de btleplug; não há como
  testar conexão, sync, exchange ou reconexão sem o fone.
- **Ação:** introduzir um trait de transporte (leitura, escrita, subscribe)
  implementado por `BleSession`, permitindo dublês nos testes.
- **Pronto quando:** for possível testar `SessionInner`/controlador sem BLE.

### REF-04 — Re-subscribe das notificações a cada comando
- **Prioridade:** P2
- **Onde:** `src/ble.rs:350-362`, `:452-456`, `:371-393`
- **Problema:** cada `exchange`/`sync` chama `subscribe_commands`, que faz
  `subscribe` e espera 150 ms. Isso adiciona latência e reexecuta uma operação
  que poderia ser mantida durante a sessão.
- **Ação:** abrir a stream uma vez por sessão e reutilizá-la; re-subscrever
  apenas após reconexão.
- **Pronto quando:** uma ação não pagar o custo de settle repetidamente.

### REF-05 — `sync` sempre espera a janela inteira
- **Prioridade:** P2
- **Onde:** `src/ble.rs:393-438`
- **Problema:** o loop roda até o deadline (1,5 s no CLI e na GUI) mesmo quando
  as 8 respostas já chegaram.
- **Ação:** encerrar antecipadamente quando todos os alvos consultados tiverem
  sido respondidos, mantendo um pequeno período de tolerância.
- **Pronto quando:** o tempo típico de sync cair sem perder respostas.

### REF-06 — Código público sem uso
- **Prioridade:** P3
- **Onde:** `src/ble.rs:442-449` (`query_all`)
- **Problema:** método público nunca chamado e sem teste, mantido apenas como
  conveniência.
- **Ação:** remover ou usar no fluxo de sincronização.
- **Pronto quando:** não houver API pública morta (ou ela estiver coberta).

---

## Implementações (lacunas funcionais)

### IMP-01 — Estado em tempo real na GUI
- **Prioridade:** P2
- **Onde:** `src/bin/gui/ble_task.rs`, `src/bin/gui.rs`
- **Problema:** a GUI só atualiza o estado quando o usuário age. Mudanças feitas
  fisicamente no fone (toque ou app oficial) não aparecem até um **Atualizar**.
  O cliente Python mantinha o estado ao vivo.
- **Ação:** manter uma subscrição persistente de notificações em background e
  enviar atualizações para a UI via `Message`.
- **Pronto quando:** alterações externas refletirem na GUI sem ação do usuário.

### IMP-02 — GUI não aceita `--config PATH`
- **Prioridade:** P3
- **Onde:** `src/bin/gui.rs:244`
- **Problema:** a GUI sempre usa `Config::load()` (caminho padrão), enquanto o
  CLI aceita `--config`; não é possível isolar perfis na GUI.
- **Ação:** adicionar a flag (junto com a migração para `clap`, COR-10) e passar
  o caminho para `Config::load_from`.
- **Pronto quando:** GUI e CLI compartilharem o mesmo arquivo por parâmetro.

### IMP-03 — Notificações `0x2F`/`0x30` (bateria/versão) ignoradas
- **Prioridade:** P3
- **Onde:** `src/state.rs:117`
- **Problema:** os opcodes estão definidos, mas notificações de bateria/versão
  via canal de comando são descartadas (o projeto usa as characteristics
  diretas). Se algum firmware passar a notificá-las, o estado não atualiza.
- **Ação:** decodificar os formatos conhecidos, se/quando observados.
- **Pronto quando:** houver captura real e teste correspondente.

---

## Melhorias

### MEL-01 — CLI faz sync completo antes de cada comando
- **Prioridade:** P2
- **Onde:** `src/main.rs:646-671`
- **Problema:** `device anc/game/sleep/...` espera 1,5 s de sync antes de enviar
  o comando, tornando cada ação lenta mesmo quando o usuário só quer aplicá-la.
- **Ação:** oferecer `--no-sync` (ou sincronizar apenas os campos necessários)
  e documentar o comportamento.
- **Pronto quando:** for possível aplicar um comando sem a espera de sync.

### MEL-02 — `ensure_powered` falha quando o estado é `Unknown`
- **Prioridade:** P2
- **Onde:** `src/ble.rs:539-551`
- **Problema:** `CentralState::Unknown` é tratado como erro definitivo. Em
  alguns backends o estado fica `Unknown` por alguns instantes após a
  inicialização do adaptador.
- **Ação:** tentar novamente por um curto período antes de abortar, com mensagem
  distinta.
- **Pronto quando:** inicialização lenta do adaptador não gerar falso negativo.

### MEL-03 — Reconexão LDAC longa e sem progresso
- **Prioridade:** P3
- **Onde:** `src/main.rs:647-648`, `src/bin/gui/ble_task.rs:23-24`
- **Problema:** até 10 tentativas com 3 s de intervalo (≈30 s) sem feedback
  intermediário; o usuário pode achar que travou.
- **Ação:** reportar tentativa atual (GUI/CLI) e permitir configurar tentativas
  e intervalo.
- **Pronto quando:** houver feedback durante a reconexão.

### MEL-04 — Fallback do diretório `home` grava no diretório atual
- **Prioridade:** P2
- **Onde:** `src/config.rs:101-106`, `:145-149`
- **Problema:** sem `HOME`/`USERPROFILE`, `Config::default_path` vira
  `./.melobudsnext/config.json`, criando arquivos inesperados no diretório de
  execução.
- **Ação:** usar o diretório de configuração da plataforma (ex.: `dirs`) ou
  retornar erro claro.
- **Pronto quando:** execução sem `HOME` não gravar no cwd silenciosamente.

### MEL-05 — Ecos de estado podem chegar depois da janela do comando
- **Prioridade:** P2
- **Onde:** `src/actions.rs` (`CommandOptions::listen`), `src/ble.rs`
  (`BleSession::exchange`)
- **Problema:** na validação do Lote 6, os ecos de `0x16` (balanço), `0x1D`
  (volume), `0x2C` (detecção de uso) e `0x10` (modo sono) chegaram depois da
  janela de espera do comando. O estado impresso ficou uma ação atrasado; o
  `device info` seguinte mostrou o valor novo. O efeito é uma saída confusa,
  não uma falha do fone.
- **Ação:** medir o atraso real desses ecos e, se necessário, aumentar a janela
  padrão, consultar o opcode depois do envio ou aplicar o valor enviado quando o
  eco não chegar (sem mascarar mudanças externas).
- **Pronto quando:** o estado exibido depois de um comando refletir o valor
  aplicado, com teste que simule o eco atrasado.
- **Descoberto em:** validação manual do Lote 6 (Linux, firmware 2.0.6).

---

## Testes

### TST-01 — Sem testes para helpers do CLI
- **Prioridade:** P2
- **Onde:** `src/main.rs` (sem `#[cfg(test)]`)
- **Problema:** `build_anc_mode`, `wait_duration` e `mapping_from_bindings` não
  têm cobertura direta; erros de validação dependem de teste manual.
- **Ação:** mover a lógica pura para a `lib` (ou testar no binário) e cobrir
  combinações inválidas (`--voice` + `--intensity`, `--level` sem `--scene`,
  limites de `--wait`, bindings malformados).
- **Pronto quando:** os caminhos de erro estiverem testados.

### TST-02 — Fluxos de sessão sem dublê de transporte
- **Prioridade:** P1
- **Onde:** `src/bin/gui/ble_task.rs`, `src/ble.rs`
- **Problema:** não há testes para conectar/desconectar, refresh, comando com
  reconexão ou sessão expirada.
- **Ação:** depende de REF-03; criar dublê e cobrir as transições de
  `SessionInner` e do futuro controlador.
- **Pronto quando:** os fluxos de estado tiverem testes sem hardware.

### TST-03 — Sem testes para transições de estado da GUI
- **Prioridade:** P2
- **Onde:** `src/bin/gui.rs`
- **Problema:** `finish`, `load_drafts`, `active_anc_mode` e o tratamento de
  offline não têm testes.
- **Ação:** extrair a lógica pura para funções testáveis e cobrir os casos de
  erro/sucesso.
- **Pronto quando:** COR-03, COR-05 e COR-06 tiverem testes de regressão.

### TST-04 — Sem testes para erros de leitura/escrita da configuração
- **Prioridade:** P2
- **Onde:** `src/config.rs:151-231`
- **Problema:** só há testes de caminho feliz (arquivo ausente, leitura,
  compatibilidade legada). Erros de permissão, JSON inválido no save e escrita
  atômica não são cobertos.
- **Ação:** adicionar testes de erro e de integridade da escrita.
- **Pronto quando:** COR-09 tiver teste de falha.

### TST-05 — Tolerância do framing a lixo/`body_len` não é explícita
- **Prioridade:** P3
- **Onde:** `src/protocol/framing.rs:83-130`
- **Problema:** o parser ignora `body_len` e descarta bloco truncado, mas só há
  um teste de truncamento simples.
- **Ação:** cobrir bytes extras após o último bloco, `body_len` inconsistente e
  notificação com múltiplos blocos válidos.
- **Pronto quando:** o comportamento documentado estiver congelado em testes.

### TST-06 — Regressões dos defeitos COR-* ainda não cobertas
- **Prioridade:** P1
- **Onde:** `tests/` e `src/**/tests`
- **Problema:** cada correção listada aqui precisa de teste que a fixe.
- **Ação:** adicionar teste de regressão junto com cada COR-* (política do
  projeto).
- **Pronto quando:** todo item COR-* tiver teste correspondente.

### TST-07 — Sem checklist/script de validação com hardware
- **Prioridade:** P3
- **Onde:** repositório (não existe `tests/manual` nem script)
- **Problema:** a validação BLE real depende de conhecimento tácito.
- **Ação:** documentar um roteiro (conectar, sync, cada comando, reconexão LDAC,
  queda de link) para execução manual em Linux/Windows.
- **Pronto quando:** o roteiro existir e citar resultados esperados.

### TST-08 — Helpers de diretório temporário podem colidir
- **Prioridade:** P3
- **Onde:** `src/config.rs` (`temporary_directory`) e o helper equivalente em
  `src/bin/gui.rs`
- **Problema:** o nome do diretório temporário combina apenas
  `SystemTime::now()` e o pid. Com testes em paralelo, duas chamadas no mesmo
  instante podem escolher o mesmo caminho. Durante o Lote 1 uma execução falhou
  em dois testes da configuração, sem reprodução em mais de 50 execuções
  seguintes.
- **Ação:** incluir um contador atômico no nome (ou usar `tempfile`), sem mudar
  a lógica testada.
- **Pronto quando:** execuções paralelas e repetidas nunca compartilharem o
  diretório de trabalho.
- **Observação:** descoberto no Lote 1; ainda sem lote atribuído.

---

## Documentação

### DOC-01 — Seção 11 do protocolo descreve pendências do projeto Python
- **Prioridade:** P1
- **Onde:** `PROTOCOLO_MelobudsNext.md:328-336`
- **Problema:** a seção "Estado atual e próximos passos" lista itens do projeto
  Python, incluindo "criar testes automatizados" (já existentes aqui) e um
  PKGBUILD em `packaging/aur` que não existe neste repositório.
- **Ação:** separar o histórico do Python do estado do port Rust, com uma seção
  própria de pendências atuais.
- **Pronto quando:** o documento não atribuir ao Rust pendências já resolvidas.

### DOC-02 — Exemplo composto contradiz `body_len`
- **Prioridade:** P2
- **Onde:** `PROTOCOLO_MelobudsNext.md:68-70`
- **Problema:** o exemplo `FF 06 09 01 01 17 03 00 00 00` tem `body_len = 06`,
  mas o corpo real tem 8 bytes. O parser ignora `body_len` de propósito, mas
  isso não está explicado.
- **Ação:** documentar que, em pacotes compostos, `body_len` não é confiável e
  os blocos devem ser lidos sequencialmente a partir do offset 2.
- **Pronto quando:** o texto não induzir a validar `body_len` como soma total.

### DOC-03 — README não cobre pré-requisitos de BLE no Linux
- **Prioridade:** P2
- **Onde:** `README.md:10-26`
- **Problema:** há instruções para Windows, mas nada sobre BlueZ, D-Bus,
  permissões (grupo `bluetooth`/polkit) ou adaptador ligado no Linux.
- **Ação:** adicionar uma seção de requisitos por plataforma.
- **Pronto quando:** um usuário Linux conseguir configurar o ambiente só pelo
  README.

### DOC-04 — Flags da GUI não documentadas
- **Prioridade:** P3
- **Onde:** `README.md:105-134`
- **Problema:** `--offline` e `MELOBUDNEXT_GUI_OFFLINE` aparecem no texto, mas
  não há tabela de flags nem menção a `--help` (inexistente hoje).
- **Ação:** atualizar após COR-10.
- **Pronto quando:** todas as flags estiverem listadas e testadas.

### DOC-05 — Limitações atuais (0x28, tempo real) não estão no README
- **Prioridade:** P2
- **Onde:** `README.md:192-205`
- **Problema:** a lista de diferenças em relação ao Python não menciona que a
  GUI não tem estado ao vivo nem que `0x28` é ignorado.
- **Ação:** incluir as limitações conhecidas e linkar este documento.
- **Pronto quando:** o README refletir o comportamento real.

---

## Infraestrutura

### INF-01 — Falta arquivo `LICENSE`
- **Prioridade:** P2
- **Onde:** raiz do repositório, `Cargo.toml:7`
- **Problema:** o pacote declara MIT, mas não há texto de licença no repositório.
- **Ação:** adicionar `LICENSE` com o texto MIT e o detentor dos direitos.
- **Pronto quando:** a licença declarada existir em arquivo.

### INF-02 — `.gitignore` mínimo
- **Prioridade:** P2
- **Onde:** `.gitignore:1`
- **Problema:** só `/target` está listado. Faltam `.env` (dados sensíveis),
  arquivos de editor/IDE e artefatos locais de empacotamento.
- **Ação:** adicionar `.env`, `.idea/`, `.vscode/`, `*.log` etc., conforme a
  convenção do projeto.
- **Pronto quando:** segredos e artefatos locais não puderem ser versionados
  por acidente.

### INF-03 — Sem CI
- **Prioridade:** P2
- **Onde:** raiz (não há `.github/workflows`)
- **Problema:** fmt, clippy `-D warnings` e testes dependem de execução manual,
  em um projeto com duas plataformas-alvo (Linux e Windows).
- **Ação:** workflow com `cargo fmt --check`, `cargo clippy --all-targets
  -- -D warnings` e `cargo test --all-targets` em Linux e Windows.
- **Pronto quando:** um PR sem testes verdes for bloqueado.

### INF-04 — `iced` compilado mesmo para quem só usa o CLI
- **Prioridade:** P3
- **Onde:** `Cargo.toml:15`, `Cargo.toml:24-26`
- **Problema:** o binário da GUI e suas dependências são compilados sempre,
  aumentando tempo de build e tamanho do workspace.
- **Ação:** feature `gui` com dependência opcional e `required-features` no
  binário.
- **Pronto quando:** `cargo build --no-default-features` gerar só o CLI.

### INF-05 — Sem empacotamento
- **Prioridade:** P3
- **Onde:** raiz (não há `packaging/`)
- **Problema:** não há PKGBUILD/AUR, `.desktop` nem processo de release para
  Windows, apesar de a seção 11 do protocolo citar esse passo.
- **Ação:** criar os artefatos e documentar o fluxo de release.
- **Pronto quando:** for possível instalar o pacote em Arch Linux e gerar o
  executável de Windows a partir do repositório.

### INF-06 — Instalação no Arch via `makepkg` antes do AUR
- **Prioridade:** P2
- **Onde:** `packaging/aur/PKGBUILD`, `packaging/README.md`, `README.md`
- **Problema:** enquanto o pacote não estiver no AUR, não há um fluxo publicado
  e testado para o usuário instalar com `makepkg`; o `PKGBUILD` ainda apontava
  para uma tag inexistente e usava `sha256sums=('SKIP')`.
- **Ação:** depois de criar a tag `vX.Y.Z`, atualizar `pkgver`, gerar o
  checksum real com `updpkgsums`, regenerar `.SRCINFO` e testar `makepkg -si`
  em uma instalação Arch limpa (chroot ou VM). Distribuir o `PKGBUILD` no
  repositório/release com instruções de clone + build; o AUR entra depois.
- **Pronto quando:** um usuário Arch instalar o pacote apenas seguindo a
  documentação, sem depender do AUR, e a instalação for validada em ambiente
  limpo.
- **Status:** Implementado no Lote 9 (build e pacote validados no Arch x86_64
  atual com fonte local; instalação em ambiente limpo ainda pendente).

### INF-07 — Executável Windows único (GUI) com CRT estático
- **Prioridade:** P2
- **Onde:** `packaging/windows/build-release.ps1`,
  `packaging/windows/build-release-xwin.sh`, `packaging/README.md`, `README.md`
- **Problema:** o script antigo gerava um `.zip` com os dois executáveis e o
  build MSVC padrão dependia do Visual C++ Redistributable. O objetivo é
  distribuir um executável simples, sem instalador.
- **Ação:** gerar `melobudsnext-gui.exe` com CRT estático
  (`RUSTFLAGS="-C target-feature=+crt-static"` no alvo
  `x86_64-pc-windows-msvc`), mantendo os assets embutidos; publicar o `.exe`
  (ou um zip de um arquivo) com checksum e orientação sobre o aviso do
  SmartScreen. O CLI pode continuar disponível via fonte ou em um zip opcional.
- **Pronto quando:** o `.exe` abrir e conectar ao fone em um Windows real sem
  instalar o Redistributable, com o fluxo documentado.
- **Status:** Implementado no Lote 9 (build cruzado com `cargo-xwin`, ícone
  embutido, imports conferidos e SHA-256 gerado; execução em Windows real
  confirmada pelo usuário).

---

## Protocolo/Hardware

### PRO-01 — Opcode `0x28` (ANC Wear) não decodificado
- **Prioridade:** P3
- **Onde:** `src/protocol/opcode.rs:15`, `src/state.rs:117`,
  `PROTOCOLO_MelobudsNext.md:303`
- **Problema:** o opcode é reconhecido e chega como notificação, mas o conteúdo
  é descartado. O significado ainda não foi mapeado.
- **Ação:** capturar notificações com o app oficial e implementar a
  decodificação com testes.
- **Pronto quando:** o estado interpretar `0x28` (ou houver evidência de que é
  irrelevante).

### PRO-02 — Auto-desligar e opcodes desconhecidos não explorados
- **Prioridade:** P3
- **Onde:** protocolo (sem opcode identificado)
- **Problema:** o auto-desligar do fone e outros opcodes observados no log do
  Python nunca foram mapeados.
- **Ação:** sessão de captura com o app oficial, documentando bytes e efeitos.
- **Pronto quando:** o comportamento estiver documentado (implementado ou não).

### PRO-03 — Validação em Linux ainda não registrada
- **Prioridade:** P2
- **Onde:** `README.md`, `PROTOCOLO_MelobudsNext.md:335`
- **Problema:** a validação original foi feita apenas no Windows; não há
  registro de teste com BlueZ (pareamento, permissões, reconexão).
- **Ação:** executar o roteiro de TST-07 em Linux e registrar resultados.
- **Pronto quando:** houver evidência de funcionamento no Linux (ou limitações
  documentadas).

### PRO-04 — Sem matriz de compatibilidade por firmware
- **Prioridade:** P3
- **Onde:** `PROTOCOLO_MelobudsNext.md`
- **Problema:** as observações são do firmware 2.0.6; não há tabela indicando o
  que muda em outras versões (ex.: toque `0x0B`, ANC `02 00 00`, wear de 3
  bytes).
- **Ação:** manter uma matriz versão × comportamento conhecido.
- **Pronto quando:** as divergências por firmware estiverem registradas.

### PRO-05 — Quarto byte do `0x2C` varia e não está confirmado
- **Prioridade:** P3
- **Onde:** `src/protocol/commands.rs` (`WearDetection::from_params`),
  `PROTOCOLO_MelobudsNext.md`
- **Problema:** o firmware 2.0.6 reporta `0x2C` com 4 bytes. O quarto byte,
  tratado como `tone`, apareceu como `00` no primeiro `device info`, mas assumiu
  valores como `0xA0` e `0x5D` depois de alternar detecção/ANC, mesmo quando o
  comando enviava `--tone 0`. O significado real (status, contador ou outro
  campo) não foi confirmado, e o parser atual o expõe como configuração.
- **Ação:** capturar novamente com o app oficial e comparar envio/report;
  enquanto isso, evitar tratar o quarto byte como valor configurável.
- **Pronto quando:** o campo estiver identificado ou marcado explicitamente como
  desconhecido e preservado sem ser reenviado.
- **Descoberto em:** validação manual do Lote 6 (Linux, firmware 2.0.6).

---

## Plano de execução em lotes

### Regras do ciclo de vida de um lote

Estas regras são específicas do plano em lotes e podem ser removidas quando o
plano for concluído.

#### Abertura da sessão

1. Ler `AGENTS.md`, `README.md`, `PROTOCOLO_MelobudsNext.md` e este arquivo.
2. Confirmar que a `main` está limpa (`git status --short` vazio) e que já
   contém o lote anterior (`git log`).
3. Confirmar na tabela "Ordem, dependências e status" a branch, as dependências
   e os itens do lote.
4. Criar a branch a partir da `main` atual (exceto o Lote 0, aplicado direto na
   `main`).
5. Registrar no início do relatório da sessão: lote, branch e escopo.

#### Durante o lote

- Implementar apenas os itens listados para o lote; não adiantar itens de lotes
  futuros.
- Commit atômico por item, no formato definido no `AGENTS.md`.
- Toda correção leva teste de regressão; toda funcionalidade leva teste novo.
- Descoberta fora do escopo: registrar um item novo neste documento (com ID e
  prioridade) em vez de implementá-lo de passagem. Se bloquear o lote, registrar
  a decisão nas notas.
- Se for necessário mudar dependência, ordem ou escopo, atualizar este documento
  no mesmo commit da mudança.
- Manter `README.md`, `PROTOCOLO_MelobudsNext.md` e doc comments atualizados
  junto com o código.

#### Encerramento obrigatório

Nenhum lote é considerado concluído antes de:

1. **Funcionalidade completa**: todos os itens entregues; o que não for
   concluído precisa ser explicitamente movido para um item novo, com motivo.
2. **Gate completo**: `cargo fmt --check`,
   `cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets` e
   `cargo build --release`.
3. **Validação manual** (quando o lote tocar BLE/GUI): executar o checklist
   aplicável e registrar plataforma e firmware.
4. **Documentação**: atualizações de README/protocolo/doc comments incluídas.
5. **Status atualizado** na tabela, com commit e data.
6. **Notas de implementação** adicionadas (template abaixo).
7. **Merge na `main`** com `git merge --no-ff <branch>`, gate repetido na
   `main` e branch apagada.
8. **`main` limpa**: `git status` sem alterações, sem arquivos não rastreados e
   sem branches de lote pendentes.

#### Atualização do status

- Ao abrir o lote: `Pendente` → `Em andamento`.
- Ao encerrar: `Implementado` + commit curto + data, por exemplo
  `Implementado (a1b2c3d, 2026-11-20)`.
- Item movido ou descartado: registrar o motivo na seção de notas do lote.

#### Notas de implementação (anedotas)

Cada lote encerrado adiciona uma subseção em **"Notas de implementação por
lote"** com:

- decisões tomadas e o porquê;
- armadilhas encontradas (firmware, btleplug, iced, plataforma);
- efeitos colaterais e dívidas criadas;
- recomendações para os próximos lotes.

Se não houver nada relevante, escrever "Sem notas".

#### Interrupção da sessão

Se a sessão terminar sem concluir o lote: manter a branch, **não** fazer merge,
mudar o status para `Em andamento` e listar no relatório o que falta e onde
parou.

#### Push e paralelismo

- Nunca fazer push; o usuário publica manualmente.
- Executar um lote por vez no mesmo diretório. Se houver sessões em paralelo,
  usar worktrees e respeitar a ordem de merge da tabela.
- Não iniciar o próximo lote antes do merge do anterior, exceto quando a tabela
  indicar independência explícita.

### Lote 0 — Correções rápidas direto na `main`

**Branch:** nenhuma (`main`)
**Dependências:** nenhuma
**Objetivo:** eliminar defeitos triviais e preparar a base antes das branches.

| Item | Categoria | Justificativa para ir direto na main |
| --- | --- | --- |
| COR-01 | Correção | Uma linha de Markdown; sem impacto em código |
| COR-02 | Correção | Alteração pontual na leitura da variável de ambiente, com teste pequeno |
| COR-08 | Correção | `try_encode` localizado no framing, com teste pequeno |
| COR-09 | Correção | Escrita atômica isolada em `config.rs`, com teste pequeno |
| REF-06 | Refatoração | Remoção de método público sem uso |
| MEL-04 | Melhoria | Fallback de diretório isolado em `config.rs` |
| TST-04 | Teste | Testes de erro da configuração |
| TST-05 | Teste | Testes de tolerância do framing |
| DOC-02 | Documentação | Ajuste textual no exemplo composto |
| INF-01 | Infraestrutura | Arquivo `LICENSE` |
| INF-02 | Infraestrutura | `.gitignore` |
| INF-03 | Infraestrutura | CI roda o gate dos lotes seguintes |

**Gate:** o gate completo (é rápido) + commit por item.
**Pronto quando:** todos os 12 itens estiverem na `main` com testes verdes.

### Lote 1 — Robustez da conexão e das ações da GUI

**Branch:** `fix/robustez-gui`
**Dependências:** Lote 0
**Arquivos principais:** `src/bin/gui.rs`, `src/bin/gui/ble_task.rs`,
`src/ble.rs`
**Objetivo:** impedir que a GUI mostre estado incorreto ou permita ações
inválidas.

| Item | Entrega |
| --- | --- |
| COR-03 | Ações BLE bloqueadas (ou simuladas) no modo offline |
| COR-04 | Link morto detectado; GUI volta para "Desconectado" |
| COR-05 | Toggles não revertem após comando bem-sucedido sem eco |
| COR-06 | ANC desconhecido não é exibido nem reenviado como "Desligado" |
| COR-07 | Função de toque desconhecida visível no seletor |
| COR-10 | `clap` na GUI (`--help`, `--version`, `--offline`) |
| IMP-02 | Flag `--config PATH` na GUI |
| TST-03 | Testes das transições de estado da GUI |
| TST-06 | Regressões de COR-03..07 e COR-10 |

**Gate extra:** teste manual no fone desligando-o durante o uso (COR-04) e
abrindo a GUI com `--offline` (COR-03).
**Pronto quando:** nenhuma ação da GUI produzir erro por estado local
inconsistente.

### Lote 2 — Fundação compartilhada e testável

**Branch:** `refactor/controlador-transporte`
**Dependências:** Lote 1
**Arquivos principais:** `src/lib.rs`, novo `src/actions.rs` (ou equivalente),
`src/main.rs`, `src/bin/gui/ble_task.rs`, `src/ble.rs`
**Objetivo:** eliminar a duplicação CLI/GUI e permitir testes sem hardware.

| Item | Entrega |
| --- | --- |
| REF-01 | Camada de ações compartilhada (`sync → enviar → aplicar → reconectar`) |
| REF-03 | Trait de transporte implementado por `BleSession` |
| TST-01 | Helpers puros do CLI movidos/testados (`build_anc_mode`, `wait_duration`, `mapping_from_bindings`) |
| TST-02 | Testes de sessão com dublê de transporte |

**Gate extra:** revisão de que CLI e GUI mantêm o mesmo comportamento
observável (mesmos comandos e mensagens).
**Pronto quando:** existir um único fluxo de orquestração e ele estiver coberto
por testes com dublê.

### Lote 3 — Sincronização e desempenho

**Branch:** `perf/sincronizacao-notificacoes`
**Dependências:** Lote 2
**Arquivos principais:** `src/ble.rs`, `src/main.rs`, controlador do Lote 2
**Objetivo:** reduzir latência e evitar re-subscrições desnecessárias.

| Item | Entrega |
| --- | --- |
| REF-04 | Stream de notificações persistente por sessão |
| REF-05 | `sync` encerra quando todas as respostas chegarem |
| MEL-01 | CLI sem sync obrigatório antes de cada comando (`--no-sync` ou equivalente) |
| MEL-02 | `ensure_powered` tolera `Unknown` transitório |

**Gate extra:** medir o tempo típico de `device info` e de uma ação antes/depois.
**Pronto quando:** não houver perda de respostas e a latência cair de forma
mensurável.

### Lote 4 — Manutenibilidade da GUI e do build

**Branch:** `refactor/gui-modulos`
**Dependências:** Lote 1
**Arquivos principais:** `src/bin/gui.rs`, novos módulos em `src/bin/gui/`,
`Cargo.toml`
**Objetivo:** reduzir o arquivo monolítico e o custo de build.

| Item | Entrega |
| --- | --- |
| REF-02 | `gui.rs` dividido em `app`, `theme`, `widgets` e `tabs` |
| INF-04 | Feature `gui` com dependência opcional do `iced` |
| MEL-03 | Feedback de progresso/configuração na reconexão LDAC |

**Gate extra:** GUI aberta em modo offline para conferir o layout após a
divisão dos módulos.
**Pronto quando:** nenhum arquivo da GUI passar de ~600 linhas e
`cargo build --no-default-features` gerar apenas o CLI.

### Lote 5 — Estado em tempo real na GUI

**Branch:** `feat/gui-tempo-real`
**Dependências:** Lotes 2, 3 e 4
**Arquivos principais:** `src/bin/gui/ble_task.rs`, módulos da GUI
**Objetivo:** refletir mudanças feitas no fone sem ação do usuário.

| Item | Entrega |
| --- | --- |
| IMP-01 | Subscrição persistente em background atualizando a UI |
| DOC-05 | README atualizado (limitações e comportamento ao vivo) |

**Gate extra:** alterar ANC/toque pelo fone e conferir a atualização na GUI sem
clicar em **Atualizar**.
**Pronto quando:** mudanças externas aparecerem na GUI em poucos segundos.

### Lote 6 — Protocolo e validação com hardware

**Branch:** `feat/protocolo-hardware`
**Dependências:** Lotes 0 e 2 (controlador) + acesso ao fone
**Arquivos principais:** `src/protocol/*`, `src/state.rs`,
`PROTOCOLO_MelobudsNext.md`
**Objetivo:** fechar as lacunas que dependem de captura real.

| Item | Entrega |
| --- | --- |
| PRO-01 | Decodificação de `0x28` (ANC Wear) com testes |
| PRO-02 | Auto-desligar e opcodes desconhecidos documentados |
| PRO-03 | Validação em Linux registrada |
| PRO-04 | Matriz de compatibilidade por firmware |
| IMP-03 | Notificações `0x2F`/`0x30` tratadas, se observadas |
| TST-07 | Roteiro/checklist de validação manual |

**Gate extra:** executar o checklist completo em Linux (e Windows, se
disponível).
**Pronto quando:** as capturas estiverem documentadas e os opcodes
implementados ou explicitamente descartados.

### Lote 7 — Documentação e entrega

**Branch:** `chore/docs-release`
**Dependências:** todos os lotes anteriores
**Arquivos principais:** `README.md`, `PROTOCOLO_MelobudsNext.md`,
`packaging/`
**Objetivo:** alinhar a documentação ao estado final e preparar distribuição.

| Item | Entrega |
| --- | --- |
| DOC-01 | Seção 11 do protocolo reescrita para o estado do Rust |
| DOC-03 | Pré-requisitos de BLE no Linux |
| DOC-04 | Flags da GUI documentadas |
| INF-05 | Empacotamento (AUR/PKGBUILD, `.desktop`, release Windows) |

**Gate extra:** instalação a partir do pacote gerado.
**Pronto quando:** um novo usuário conseguir compilar/instalar só pela
documentação.

### Lote 8 — Correção do LDAC (COR-11)

**Branch:** `fix/ldac-reconexao`
**Dependências:** Lotes 1, 2 e 6 (controlador, transporte e validação com
hardware)
**Arquivos principais:** `src/actions.rs`, `src/ble.rs`, `src/bin/gui/*`
**Objetivo:** tratar a queda de link esperada do LDAC como parte da reconexão,
sem reportar "BLE link lost" e sem mascarar quedas reais.

| Item | Entrega |
| --- | --- |
| COR-11 | LDAC reconecta e exibe o estado final após o reinício do fone |

**Critérios de implementação:**

- distinguir queda de link **antes** da escrita (erro real) de queda **depois**
  da escrita em comando que reinicia o fone (esperada);
- manter a sessão para reconectar e aplicar o estado pós-reconexão;
- não alterar o comportamento de comandos que não reconectam;
- cobrir com dublê de transporte o caso em que a escrita acontece e o link cai
  antes da coleta dos ecos.

**Gate extra:** no fone (Linux, firmware 2.0.6), executar `device ldac off` e
`device ldac on` com `--reconnect-attempts 3 --reconnect-delay 1`, confirmar a
reconexão e o estado final, e testar o toggle LDAC na GUI. Registrar plataforma
e firmware.
**Pronto quando:** `device ldac` conclui sem erro de link perdido, o estado
final corresponde ao valor aplicado e a regressão está coberta por teste.

### Lote 9 — Distribuição (Arch via makepkg e Windows standalone)

**Branch:** `chore/distribuicao`
**Dependências:** Lotes 7 e 8
**Arquivos principais:** `packaging/*`, `README.md`
**Objetivo:** disponibilizar instalação no Arch por `makepkg` (sem AUR) e um
executável Windows único.

| Item | Entrega |
| --- | --- |
| INF-06 | Fluxo `makepkg` validado em Arch limpo, com checksum real e instruções |
| INF-07 | `melobudsnext-gui.exe` único com CRT estático e checksum |

**Gate extra:** build pronto; validação Windows confirmada pelo usuário. O pacote
do Arch foi compilado, testado e inspecionado localmente; a instalação via
`makepkg -si` em um Arch limpo (chroot/VM) segue pendente.
**Pronto quando:** os dois caminhos de instalação estiverem documentados e
publicáveis no GitHub Releases; o AUR fica para uma etapa posterior.

### Ordem, dependências e status

| Lote | Branch | Depende de | Itens | Status |
| --- | --- | --- | --- | --- |
| 0 | `main` | — | 12 | Implementado (e5c4d0e, 2026-11-19) |
| 1 | `fix/robustez-gui` | 0 | 9 | Implementado (357c1f4, 2026-11-22) |
| 2 | `refactor/controlador-transporte` | 1 | 4 | Implementado (0764928, 2026-11-22) |
| 3 | `perf/sincronizacao-notificacoes` | 2 | 4 | Implementado (28963cd, 2026-11-22) |
| 4 | `refactor/gui-modulos` | 1 | 3 | Implementado (0e73c22, 2026-11-22) |
| 5 | `feat/gui-tempo-real` | 2, 3, 4 | 2 | Implementado (871a1bf, 2026-11-22) |
| 6 | `feat/protocolo-hardware` | 0, 2 | 6 | Implementado (323e95f, 2026-11-22) |
| 7 | `chore/docs-release` | todos | 4 | Implementado (d0daecb, 2026-11-22) |
| 8 | `fix/ldac-reconexao` | 1, 2, 6 | 2 | Implementado (d4d85df, 2026-11-22) |
| 9 | `chore/distribuicao` | 7, 8 | 2 | Implementado (`25348f2`, 2026-11-22; validação Windows confirmada pelo usuário) |

> Total: 48 itens nos lotes planejados (44 originais, mais COR-11 e COR-12 no
> Lote 8 e INF-06/INF-07 no Lote 9), mais TST-08, MEL-05 e PRO-05, ainda sem
> lote atribuído. O lote 4 pode ser executado em paralelo ao 2/3 por não compartilhar
> arquivos centrais, mas o merge deve seguir a ordem da tabela para manter a
> `main` sempre estável.

### Regra de promoção para a `main`

Um item do lote 0 só permanece na `main` se:

1. for resolvido em um único commit (ou dois, no máximo, código + teste);
2. não alterar API pública usada pela GUI;
3. não exigir teste manual com o fone;
4. passar no gate completo.

Caso contrário, o item é movido para o lote temático correspondente e segue o
fluxo de branch.

## Notas de implementação por lote

Cada sessão adiciona aqui a subseção do lote concluído, antes do merge.

### Modelo

```markdown
#### Lote N — <nome>
- **Status:** Implementado
- **Branch:** `<branch>` (ou `main` no Lote 0)
- **Commits:** `abc1234`, `def5678`
- **Validação:** gate completo; hardware: <plataforma/firmware ou "não aplicável">
- **Notas:**
  - decisão/armadilha/efeito colateral relevante para os próximos lotes
  - ...
```

#### Lote 0 — Correções rápidas direto na `main`

- **Status:** Implementado
- **Branch:** `main`
- **Commits:** `08d1fd7`, `2a97110`, `c7850db`, `7052885`, `086470e`,
  `9ad629b`, `6653b98`, `c7d4b39`, `d02a9c2`, `101e2d6`, `4efd8e5`, `e5c4d0e`
- **Validação:** gate completo na `main` (`cargo fmt --check`, `cargo clippy
  --all-targets -- -D warnings`, `cargo test --all-targets` com 70 testes
  unitários + 2 da GUI + 6 de integração, `cargo build --release`); hardware:
  não aplicável.
- **Notas:**
  - `MELOBUDNEXT_GUI_OFFLINE` passou a interpretar valores: `0`, `false`, `no`,
    `off` e vazio desligam; qualquer outro valor não vazio liga, preservando o
    comportamento antigo de presença para valores desconhecidos.
  - `Packet::try_encode` devolve `ProtocolError::ParamsTooLong`; `encode`
    permanece como atalho documentado que entra em pânico e
    `BleSession::write_command` usa a variante falível.
  - `Config::default_path` agora retorna `io::Result<PathBuf>` e falha com
    mensagem clara quando não há `HOME`/`USERPROFILE`, em vez de gravar no
    diretório atual. A escrita usa um `.tmp` no mesmo diretório e `rename`,
    removendo o temporário quando o rename falha.
  - A CI roda o gate em `ubuntu-latest` e `windows-latest`; no Linux instala
    `libdbus-1-dev` (btleplug/bluez), `pkg-config`, `libxkbcommon-dev` e
    `libwayland-dev`.
  - Dívidas criadas: a escrita atômica não preserva as permissões do arquivo
    original; `main.rs` ainda usa `Packet::encode` para pacotes montados
    localmente (parâmetros já validados); o teste de permissão da configuração
    não valida a negação quando a suíte roda como root.
  - Recomendação para o Lote 1: o parser de argumentos da GUI (COR-10) pode
    absorver `--offline` e a leitura de `MELOBUDNEXT_GUI_OFFLINE` em um único
    ponto.

#### Lote 1 — Robustez da conexão e das ações da GUI

- **Status:** Implementado
- **Branch:** `fix/robustez-gui`
- **Commits:** `ff79aef` (COR-10), `163bc7f` (IMP-02), `fc50f51` (COR-03),
  `364403a` (COR-04), `7e390cd` (COR-05), `402ca85` (COR-06), `7192c1e`
  (COR-07), `fda1b83` (TST-03), `357c1f4` (TST-06)
- **Validação:** gate completo na branch e na `main` (`cargo fmt --check`,
  `cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets` com
  70 testes unitários + 31 da GUI + 6 de integração, `cargo build --release`).
  Manual: GUI aberta com `--offline --config <fictício>` em Arch Linux/KDE
  Wayland; cabeçalho e pílula mostram "Offline", **Conectar** e **Aplicar**
  desabilitados e o rodapé exibe o aviso. `--help` e `--version` conferidos.
  Hardware: **não executado** — o fone pareado recusou conexão
  (`br-connection-refused`) durante a sessão, então não foi possível desligá-lo
  "durante o uso"; firmware não confirmado.
- **Notas:**
  - `Cli` (clap) centraliza `--offline`, `--config PATH`, `--help` e
    `--version`; `MELOBUDNEXT_GUI_OFFLINE` continua valendo e é combinado com a
    flag. `--config` alimenta `Config::load_from`, e um erro de leitura cai nos
    padrões com aviso no rodapé.
  - O modo offline é bloqueado em `update` (antes de qualquer mutação otimista)
    e novamente em `dispatch` como defesa. `controls_enabled()` desabilita
    botões e toggles de escrita; sliders, seletores e abas continuam locais.
    `pick_list`/`slider` do iced 0.13 não têm estado desabilitado por item, por
    isso o bloqueio efetivo fica no manipulador de mensagens.
  - `BleSession::ensure_link` roda no início e no fim de `sync` e no fim de
    `exchange`, porque a escrita pode "ter sucesso" com o link já morto. Em
    caso de erro, `drop_client_if_link_lost` descarta o cliente quando a sonda
    retorna `false` ou falha; a GUI passa a mostrar "Desconectado".
  - O estado do comando é aplicado depois do sync de reconexão: ecos
    reconhecidos continuam autoritativos; sem eco, o pacote enviado é aplicado
    ao estado da sessão para o toggle não reverter (COR-05). Isso também vale
    para o caminho LDAC, que reconecta antes de responder.
  - ANC desconhecido ganhou `AncChoice::Unknown([u8; 3])`: aparece na aba com o
    triplo em hexa, **Aplicar ANC** fica desabilitado e `active_anc_mode`
    devolve erro em vez de enviar `Off`.
  - Funções de toque desconhecidas são anexadas ao seletor como
    `Desconhecida (0xXX)` e o byte original é preservado ao gravar o mapa.
  - Armadilha: uma execução da suíte falhou em dois testes da configuração sem
    reprodução em mais de 50 execuções; a suspeita é colisão do diretório
    temporário dos testes, registrada como TST-08.
  - Dívidas: sem dublê de transporte (REF-03/Lote 2), a regressão de COR-04
    cobre a política de descarte, não o fluxo completo de sessão; a validação
    com o fone desligado em uso fica pendente de hardware disponível.
  - Recomendação para o Lote 2: mover `apply_command_result` e a política de
    link para o controlador compartilhado e cobrir queda de link com o dublê;
    no Lote 4, manter o bloqueio offline concentrado em um único ponto.

#### Lote 3 — Sincronização e desempenho

- **Status:** Implementado
- **Branch:** `perf/sincronizacao-notificacoes`
- **Commits:** `5ca768f` (REF-04), `47b293c` (REF-05), `1dc9961` (MEL-01),
  `28963cd` (MEL-02)
- **Validação:** gate completo na branch e na `main` (`cargo fmt --check`,
  `cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets` com
  114 testes da lib + 26 da GUI + 6 de integração, `cargo build --release`).
  Hardware: **não executado** — o fone configurado não respondeu (timeout de
  conexão) e o pareado recusou a conexão (`br-connection-page-timeout`) durante
  a sessão, então nenhuma medição BLE real foi feita.
- **Medições:** o trecho de espera do `sync` foi medido com uma stream sintética
  que responde aos 8 opcodes a cada 50 ms, usando a mesma lógica de encerramento
  (`QueryTracker`) da produção: **1,50 s antes → 0,51 s depois** (≈66% menos no
  trecho de espera). Projetando com as constantes do CLI (settle de 150 ms na
  primeira subscrição e `--wait 1` para o eco):
  - `device info`: ≈1,65 s → ≈0,66 s;
  - ação comum (`device game on`): ≈2,80 s → ≈1,66 s;
  - ação com `--no-sync`: ≈1,15 s (sem o sync inicial).
  Não há perda de respostas: o encerramento só ocorre depois de todos os
  opcodes consultados aparecerem, com 100 ms de tolerância; se algum não
  responder, a janela completa é mantida.
- **Notas:**
  - REF-04 moveu a subscrição para um `tokio::sync::Mutex<Option<Stream>>`
    dentro de `BleSession`. `sync` e `exchange` compartilham a mesma stream e o
    settle de 150 ms é pago uma vez por sessão. `refresh` (chamado pela
    reconexão) descarta a stream antiga, que é reaberta na próxima operação.
  - `subscribe_commands` deixou de ser público. `device send` e
    `device monitor` passaram a usar `receive()`/`next_notification()`, que
    consomem a stream da sessão; o monitor continua encerrando no `Ctrl+C`.
  - REF-05 adicionou `QueryTracker`, que remove cada opcode respondido do
    conjunto de alvos e define o fim do sync em `última resposta + 100 ms`,
    limitado à janela pedida. Respostas compostas são lidas bloco a bloco pelo
    framing normal.
  - MEL-01 expôs `Controller::connect_only` e a flag `--no-sync` em todas as
    ações do CLI. O estado é limpo ao conectar sem sync e a saída mostra apenas
    o que o comando devolver; comandos que reconectam (LDAC) ainda sincronizam
    depois da reconexão, como antes.
  - MEL-02 extraiu `poll_adapter_state`, que tenta 10 vezes com 100 ms entre
    elas quando o adaptador reporta `Unknown`. `PoweredOff` continua falhando
    na hora e o esgotamento tem mensagem própria ("remained unknown").
  - Armadilha: os testes do tracker precisam montar a notificação composta com
    um único cabeçalho `FF body_len` seguido dos blocos; repetir `FF` antes de
    cada bloco faz o parser interpretar o byte como opcode.
  - Dívidas: a validação com hardware segue pendente (PRO-03); a medição é
    sintética porque o fone não estava acessível. O Lote 4 pode começar sem
    depender deste lote, mas o Lote 5 (estado em tempo real) se beneficia da
    stream persistente e deve reutilizá-la em vez de abrir outra subscrição.

#### Lote 2 — Fundação compartilhada e testável

- **Status:** Implementado
- **Branch:** `refactor/controlador-transporte`
- **Commits:** `6505fc0` (REF-03), `0b06518` (REF-01), `8f8d168` (TST-01),
  `0764928` (TST-02)
- **Validação:** gate completo na branch e na `main` (`cargo fmt --check`,
  `cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets` com
  105 testes da lib + 26 da GUI + 6 de integração, `cargo build --release`).
  CLI conferido com `build`, `decode`, `touch encode`, `battery` e `firmware`;
  entradas inválidas de ANC continuam falhando com as mesmas mensagens. GUI:
  `--help`/`--version` e testes de estado offline. Hardware: **não executado**
  — não havia fone disponível na sessão, então nenhuma ação BLE real foi
  validada.
- **Notas:**
  - REF-03 introduziu `actions::Transport`, um trait object-safe com futuros
    encaixotados (`BoxFuture`) implementado por `BleSession`, e
    `actions::Connector` para injetar o transporte nos testes.
  - REF-01 moveu a orquestração para `actions::Controller`: conexão + sync,
    refresh, comando com reconexão e escrita de toque/renomeação. CLI e GUI
    ficaram com apenas entrada/saída; o CLI usa um `CommandObserver` para
    imprimir os ecos e a mensagem de reconexão na ordem antiga.
  - As diferenças observáveis entre CLI e GUI foram preservadas de propósito:
    a GUI usa `EchoPolicy::ApplyRequestedWhenSilent` (mantém o valor escolhido
    sem eco) e o CLI usa `EchoPolicy::EchoesOnly`. Na reconexão do CLI, os
    avisos anteriores continuam sendo descartados em favor do sync novo, como
    antes. A janela de scan do CLI continua vindo de `--scan-timeout` via
    `Controller::set_scan_timeout`.
  - TST-01 moveu `build_anc_mode`, `wait_duration` e `mapping_from_bindings`
    para `src/cli.rs`, com testes de combinações inválidas e limites.
  - TST-02 adicionou um dublê de `Transport`/`Connector` scriptável e cobre
    conectar, sincronizar, refresh, reconectar, queda de link (inclusive sonda
    inconclusiva), comandos nos dois modos de eco, rename e toque.
  - Armadilha: `CommandObserver` precisou de `Send` para o futuro do
    `Task::perform` da GUI continuar `Send`; o controlador guarda o observer
    como `&mut dyn CommandObserver` apenas durante a chamada.
  - Dívidas: `send`, `monitor`, `inspect`, `read` e `write-touch` do CLI ainda
    falam direto com `BleSession` (não são o fluxo `sync → comando`); a
    validação com hardware segue pendente. O Lote 3 pode reutilizar
    `Transport`/`Controller` para a stream persistente e o encerramento
    antecipado do sync.

#### Lote 4 — Manutenibilidade da GUI e do build

- **Status:** Implementado
- **Branch:** `refactor/gui-modulos`
- **Commits:** `7a3f022` (REF-02), `b5b882d` (MEL-03), `0e73c22` (INF-04)
- **Validação:** gate completo na branch e na `main` (`cargo fmt --check`,
  `cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets` com
  118 testes da lib + 30 da GUI + 6 de integração, `cargo build --release`).
  `cargo build --no-default-features` em um `CARGO_TARGET_DIR` limpo gerou
  apenas `melobudsnext`, e `cargo tree --no-default-features` não lista o
  `iced`. GUI aberta com `--offline --config <fictício>` em Arch Linux/KDE
  Wayland: cabeçalho, painel esquerdo, aba Status, cartões de bateria, campo de
  renomear e rodapé renderizaram no layout de 1200x800; as demais abas são
  renderizadas no teste de regressão de COR-03. Hardware: **não executado** —
  não havia fone disponível na sessão, então a reconexão LDAC e o progresso não
  foram observados no dispositivo.
- **Notas:**
  - REF-02 deixou `src/bin/gui.rs` com 48 linhas (só a entrada e os módulos) e
    distribuiu o restante em `gui/app.rs` (estado e mensagens), `cli.rs`,
    `options.rs`, `theme.rs`, `widgets.rs`, `view.rs`, `device_panel.rs` e
    `tabs/{status,anc,audio,touch,wear}.rs`. O maior arquivo é `app.rs`, com 548
    linhas. Os testes da GUI foram separados em `gui/tests/cli.rs` (flags e
    configuração) e `gui/tests/app.rs` (estado e regressões), ambos abaixo de
    600 linhas.
  - Os módulos usam `pub(crate)` e os testes acessam `crate::app`, `crate::options`
    e `crate::view`; os `include_bytes!` dos SVGs foram ajustados para
    `../../../assets/` por causa da nova profundidade.
  - INF-04 criou a feature `gui` (ligada por padrão) com `iced` opcional e
    `required-features = ["gui"]` no binário. O CLI pode ser compilado e testado
    sem o `iced`; documentado no README. Dívida: a CI continua rodando o gate
    com as features padrão, então a variante sem GUI depende de execução manual.
  - MEL-03 moveu o laço de reconexão do `BleSession` para o `Controller`, que
    agora chama `Transport::reconnect_once` por tentativa e avisa o
    `CommandObserver` antes de cada uma (`reconnect_attempt`). Assim o CLI
    imprime `Reconnect attempt N/M...` (em `stderr` no modo `--json`) e a GUI
    atualiza o rótulo de status. `ReconnectPolicy` (tentativas e intervalo) foi
    adicionada às `CommandOptions` e exposta em `device ldac` e na GUI com
    `--reconnect-attempts`/`--reconnect-delay`.
  - Armadilha: o progresso da GUI usa um canal `futures` alimentado pela própria
    `Task::stream`, sem `tokio::spawn`, porque vários testes chamam `update` fora
    de um runtime e um spawn entraria em pânico. O futuro de conclusão envia
    `Finished` pelo mesmo canal, preservando a ordem progresso → resultado.
  - Dívidas: a reconexão LDAC segue sem validação em hardware; a CI não exercita
    `--no-default-features`; a GUI não tem um teste que ligue o fluxo completo
    de progresso (a cobertura é a mensagem isolada + o observador do
    controlador).
  - Recomendação para o Lote 5: a subscrição persistente pode reaproveitar a
    `Transport`/`Controller` do Lote 3 e os módulos `app`/`tabs` criados aqui;
    ao adicionar mensagens de estado ao vivo, siga o padrão de `Task::stream`
    para não bloquear `update` nem exigir runtime nos testes.

#### Lote 5 — Estado em tempo real na GUI

- **Status:** Implementado
- **Branch:** `feat/gui-tempo-real`
- **Commits:** `9067776` (IMP-01), `871a1bf` (ajuste da leitura periódica),
  `8cea874` (organização dos testes), `2e7215d` (DOC-05)
- **Validação:** gate completo na branch e na `main` (`cargo fmt --check`,
  `cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets` com
  127 testes da lib + 36 da GUI + 6 de integração, `cargo build --release`).
  Hardware: Arch Linux/KDE Wayland, firmware 2.0.6. GUI conectada ao fone real
  via configuração existente; outro processo (CLI) alterou o ANC de
  `noise/level-3` para `indoor/level-1` e a GUI passou a exibir o novo valor sem
  clicar em **Atualizar**. O mapa de toque foi alterado externamente
  (`right-triple` de `next` para `previous`) e a GUI o releu pelo ciclo ocioso
  sem ação do usuário; ANC e toque foram restaurados ao final. Não foi possível
  executar o gesto físico nem usar o app oficial nesta sessão; a mudança externa
  foi feita por outro cliente no mesmo adaptador.
- **Notas:**
  - IMP-01 adicionou `Transport::wait_notification`/`read_touch` e, no
    controlador, `poll_notifications`/`refresh_touch`. A GUI consome a stream
    persistente do Lote 3 em um laço `Task::stream` com espera limitada
    (`LIVE_WAIT` de 250 ms), no mesmo padrão do progresso do Lote 4: não há
    `tokio::spawn`, então os testes continuam rodando fora de um runtime.
  - A notificação é aplicada ao estado da sessão e enviada à UI como
    `Message::Live`; o `update` é ignorado enquanto uma ação está em andamento
    para não competir com o resultado autoritativo do controlador.
  - Só os rascunhos dos campos que mudaram são recarregados (ANC, toque, balanço,
    volume, uso e nome). Assim uma mudança externa de ANC não descarta um ajuste
    de balanço ou de toque ainda não aplicado.
  - O mapa de toque não gera notificação na characteristic de comando. Por isso
    ele é relido a cada 12 ciclos ociosos (≈3 s); os demais campos chegam por
    notificação.
  - Armadilha: durante a validação, um segundo cliente no mesmo adaptador
    derrubou o link da GUI ao terminar. A leitura periódica de toque falhou com
    `Not connected` e a GUI voltou para "Desconectado", como esperado; um ajuste
    posterior passou a ocultar falhas transitórias dessa leitura quando a sonda
    confirma que o link continua vivo.
  - Dívidas: a releitura periódica de toque adiciona tráfego BLE a cada ~3 s na
    GUI ociosa; bateria e firmware continuam atualizando só ao conectar ou em
    **Atualizar**; `0x28` e `0x2F`/`0x30` seguem sem decodificação (Lote 6).
  - Recomendação para o Lote 6: usar a escuta ao vivo para capturar `0x28` e os
    opcodes desconhecidos com o app oficial, registrar firmware e plataforma, e
    evitar manter dois clientes conectados ao mesmo fone durante a captura.

#### Lote 6 — Protocolo e validação com hardware

- **Status:** Implementado
- **Branch:** `feat/protocolo-hardware`
- **Commits:** `e013dee` (PRO-01), `08dbe6d` (IMP-03), `87b67ba` (PRO-02),
  `da07842` (PRO-04), `4964386` (TST-07), `323e95f` (PRO-03)
- **Validação:** gate completo na branch e na `main` (`cargo fmt --check`,
  `cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets` com
  130 testes da lib + 36 da GUI + 8 de integração, `cargo build --release`).
  Manual: Arch Linux, kernel `7.3.5-arch1-1` x86_64, Wayland/Hyprland, BlueZ
  5.88, firmware 2.0.6. CLI: scan, `device info`, leituras diretas, game/sleep,
  todas as cenas de ANC e transparência, balanço, volume, detecção de uso,
  rename (com restauração), `query`, `send`, `inspect` e `monitor`. GUI:
  `--help`, `--version` e `--offline --config <fictício>` abriram sem erro. Não
  foi possível executar o gesto físico de toque nem o app oficial; a captura foi
  feita por um cliente BLE próprio na mesma máquina, sem outro cliente
  conectado.
- **Capturas:** `FF 03 28 01 01` (após mudanças de ANC), `FF 06 14 04 FF FF 00
  00` (resposta de auto-desligar), `FF 03 FE 01 2F` e `FF 03 FE 01 30` sem
  resposta, `FF 03 FE 01 06` e `FF 03 FE 01 29` sem resposta.
- **Notas:**
  - PRO-01 adicionou `AncWearResult`, que preserva o payload cru e expõe o
    primeiro byte. A notificação `0x28` não substitui `DeviceState::anc`; o
    `0xFE 0x28` não responde. O evento aparece como `ANC wear/result` no texto e
    `anc_wear` no JSON. O significado do byte `01` não foi confirmado.
  - IMP-03 implementou os decodificadores de `0x2F` (bateria L/R/caixa) e
    `0x30` (versão de 3 ou 6 bytes; os três primeiros bytes alimentam o estado e
    a versão direita é ignorada). No firmware 2.0.6 nenhum dos dois é notificado
    e o `0xFE` não responde; as characteristics diretas continuam sendo a fonte
    normal. Os testes usam payloads sintéticos, não uma captura real.
  - PRO-02 documentou o `0x14` (auto-desligar) com a captura acima e a ausência
    de resposta de `0x06`/`0x29`. Nenhum comando de auto-desligar foi
    implementado; o `0x14` só é reconhecido na documentação.
  - PRO-04 criou a matriz de compatibilidade: 2.0.6 validado; 2.0.4 listado como
    relato de terceiros não reproduzido; demais versões desconhecidas. A matriz
    também registra as regras para adicionar novos firmwares.
  - TST-07 criou `tests/manual/validacao-hardware.md` com passos, resultados
    esperados, teste de queda de link, GUI e regras para não versionar dados do
    fone.
  - PRO-03 registrou a execução em Linux no README e na seção 10.1 do protocolo.
    Descobertas fora do escopo foram registradas em vez de corrigidas:
    **COR-11** (LDAC aplica o valor, mas termina com "BLE link lost" antes de
    reconectar), **MEL-05** (ecos de `0x10`, `0x16`, `0x1D` e `0x2C` podem chegar
    depois da janela do comando) e **PRO-05** (o quarto byte de `0x2C` variou
    entre `00`, `A0` e `5D`, mesmo com `--tone 0`).
  - Armadilha: em uma das conexões o `0xFE 0x18` devolveu 32 bytes zero e o nome
    ficou `null` no estado; um `device rename` para um valor fictício e a
    restauração pelo nome do adaptador resolveram. O comportamento não foi
    reproduzido de forma controlada e não foi transformado em item.
  - O `device monitor` não encerrou sozinho quando outro processo derrubou o
    link; ficou aguardando até o timeout do teste. A GUI já cobre a queda de link
    pelo caminho próprio (Lote 1).
  - Dívidas: COR-11, MEL-05 e PRO-05 seguem abertos; a validação de Windows não
    foi executada; a captura de `0x2F`/`0x30` continua sem evidência real.
  - Recomendação para o Lote 7: o Lote 7 (documentação/entrega) pode começar,
    mas o release deve decidir o que fazer com o COR-11 antes de publicar, já
    que ele afeta uma ação documentada. Atualizar a seção 11 do protocolo
    (DOC-01), os pré-requisitos de BLE no Linux (DOC-03), as flags da GUI
    (DOC-04) e o empacotamento (INF-05).

#### Lote 7 — Documentação e entrega

- **Status:** Implementado
- **Branch:** `chore/docs-release`
- **Commits:** `a8f93eb` (DOC-01), `d6d008f` (DOC-03), `0442157` (DOC-04),
  `d0daecb` (INF-05)
- **Validação:** gate completo na branch e na `main` (`cargo fmt --check`,
  `cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets` com
  130 testes da lib + 36 da GUI + 8 de integração, `cargo build --release`).
  Documentação: comandos de compilação e instalação seguidos a partir do README;
  `desktop-file-validate` sem erros; PKGBUILD validado com
  `makepkg --printsrcinfo`, pacote local gerado e extraído em um diretório
  temporário com `melobudsnext --version` e `melobudsnext-gui --version` OK; o
  `.tar.gz` de Linux foi gerado pelo script, o conteúdo conferido e o checksum
  validado com `sha256sum -c`. Hardware: não aplicável. Windows: script não
  executado (sem sessão Windows/pwsh).
- **Notas:**
  - DOC-01 separou o histórico do cliente Python do estado do port Rust na seção
    11, com funcionalidades, validação, pendências atuais (COR-11, MEL-05,
    PRO-05, captura de `0x2F`/`0x30` e Windows) e empacotamento. As seções
    anteriores continuam descrevendo os bytes observados e a referência Python.
  - DOC-03 adicionou requisitos por plataforma: BlueZ/D-Bus, `bluetoothctl`,
    `rfkill`, grupo `bluetooth`/polkit e as dependências de build por distro
    (Arch, Debian/Ubuntu e Fedora), incluindo a separação entre CLI e GUI.
  - DOC-04 documentou `--offline`, `--config PATH`, `--reconnect-attempts`,
    `--reconnect-delay`, `--help` e `--version`, além da combinação por OU entre
    a flag `--offline` e `MELOBUDNEXT_GUI_OFFLINE`.
  - INF-05 criou `packaging/aur/PKGBUILD` e `.SRCINFO`, o `.desktop`, o ícone, o
    script de release de Linux, o script de release de Windows e o roteiro em
    `packaging/README.md`. O pacote instala os dois binários, `.desktop`, ícone,
    licença e documentação; `dist/` passou a ser ignorado.
  - Armadilha: o `.sha256` gerado pelo script guarda apenas o nome do arquivo,
    então `sha256sum -c` precisa rodar dentro de `dist/`; o comando do README foi
    ajustado para isso.
  - O `PKGBUILD` mantém `sha256sums=('SKIP')` enquanto a tag `vX.Y.Z` não
    existe. Antes de enviar ao AUR, rode `updpkgsums` e regenere o `.SRCINFO`;
    `SKIP` serve só para validar a receita localmente.
  - Dívidas: a release não foi publicada (versão continua `0.1.0`), o AUR não foi
    submetido, o artefato de Windows não foi gerado e o script PowerShell não foi
    executado nesta sessão. A validação do pacote usou o `target/` local via
    symlink, não uma compilação limpa em chroot.
  - Recomendação: o checklist de release pode ser iniciado, mas decida o COR-11
    antes de publicar, já que ele afeta uma ação documentada; valide o pacote em
    uma máquina Arch limpa e teste o `.zip` de Windows em uma sessão real.

#### Lote 8 — Correção do LDAC (COR-11)

- **Status:** Implementado
- **Branch:** `fix/ldac-reconexao`
- **Commits:** `d4d85df` (COR-11), `df7e3e6` (COR-12), `9349a0a` (docs/validação)
- **Validação:** gate completo na branch (`cargo fmt --check`,
  `cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets` com
  135 testes da lib + 36 da GUI + 8 de integração, `cargo build --release`).
  Hardware: Arch Linux, kernel `7.3.5-arch1-1` x86_64, Wayland/Hyprland, BlueZ
  5.88, firmware 2.0.6.
  - `device ldac off --reconnect-attempts 3 --reconnect-delay 1`: saída 0,
    tentativas 1/3 e 2/3, estado final `LDAC: off`, sem "BLE link lost".
  - `device ldac on --reconnect-attempts 3 --reconnect-delay 1`: saída 0,
    tentativa 1/3, estado final `LDAC: on`, sem "BLE link lost".
  - GUI conectada ao fone: o toggle LDAC foi desligado e religado; a interface
    mostrou "Reconectando ao fone… tentativa 1 de 3", voltou para "Conectado" e
    exibiu "LDAC desativado; o fone foi reconectado." e depois "LDAC ativado; o
    fone foi reconectado.", sem erro. `device info` final confirmou LDAC ligado.
- **Notas:**
  - `BleSession::exchange` passou a devolver `ExchangeReport` com
    `notifications` e `link_lost`. Uma falha de escrita continua sendo `Err`;
    a queda **depois** da escrita é sinalizada em `link_lost`. O controlador só
    segue para a reconexão quando `CommandOptions::reconnect` está ligado; nos
    demais comandos a queda vira o mesmo erro "BLE link lost; connect again" de
    antes. `rename` usa a mesma regra, porque não reinicia o fone.
  - O CLI mantém a política antiga de descartar os avisos anteriores à
    reconexão em favor do sync novo. A GUI aplica o valor pedido depois da
    reconexão quando o sync não traz o campo, então o toggle não volta atrás.
  - O teste de regressão do dublê cobre escrita seguida de link perdido
    (`push_exchange_with_link_lost`), além de falha de escrita antes do envio,
    queda em comando sem reconexão e queda no `rename`.
  - Armadilha encontrada no hardware: com a GUI ociosa, o laço de notificações
    reacqueria o mutex da sessão logo após cada espera e a ação enfileirada
    nunca começava; a interface ficava presa em "Alterando LDAC e
    reconectando...". Um `tokio::task::yield_now()` entre as esperas ociosas
    resolveu (COR-12). A regressão é de temporização e foi validada no fone;
    não há teste automatizado determinístico para essa corrida.
  - A reconexão do LDAC leva de uma a duas tentativas no firmware 2.0.6; o
    tempo varia com o reinício do fone. O CLI e a GUI usam os mesmos limites
    (`--reconnect-attempts`/`--reconnect-delay`).
  - Nenhuma dependência nova. Dívidas: a corrida do COR-12 não tem teste
    automatizado; MEL-05 e PRO-05 seguem abertos.
  - Recomendação para o Lote 9: a distribuição pode começar; o COR-11 não
    bloqueia mais a release.

#### Lote 9 — Distribuição (Arch via makepkg e Windows standalone)

- **Status:** Implementado (build pronto; validação Windows confirmada pelo
  usuário)
- **Branch:** `chore/distribuicao`, `fix/icone-windows`
- **Commits:** `cccc2fe` (Arch), `25348f2` (Windows), `232a739` (subsistema de
  GUI), `93e0485` (docs), `22404d4` (ícone)
- **Validação:** gate completo na branch e na `main` (`cargo fmt --check`,
  `cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets` com
  135 testes da lib + 36 da GUI + 8 de integração, `cargo build --release`).
  Arch: `./packaging/aur/build-local.sh` (fonte local via `git archive`) gerou
  `melobudsnext-0.1.0-1-x86_64.pkg.tar.zst` e o pacote de debug, com `check()`
  rodando a suíte completa; a lista de arquivos do pacote foi conferida e, após
  extrair o `.pkg.tar.zst` em diretório temporário, `melobudsnext --version` e
  `melobudsnext-gui --version` responderam `0.1.0`; o pacote também foi
  instalado com `pacman -U` em um root isolado sob `fakeroot`
  (`--root /tmp/.../archroot`), com a lista de arquivos conferida e os dois
  binários executando. A instalação de verdade foi executada pelo usuário com
  `./packaging/aur/build-local.sh -si` em um Arch Linux x86_64 (2026-11-22), e o
  pacote, o menu e o ícone funcionaram. O `prepare()` foi testado com um tarball
  com prefixo
  `MelobudsNext-Rust-0.1.0/` (formato do GitHub) e normalizou o diretório
  corretamente. Windows: `cargo-xwin` 0.24.1 gerou `dist/melobudsnext-gui.exe`
  (7,3 MiB, SHA-256
  `3ebfbcfdac049d608e1d81c87e83b5aa7d8c7c4be9a836297de4d91ba7e5295b`); os
  imports não incluem `VCRUNTIME140.dll`/`MSVCP140.dll`, o subsistema é
  `00000002 (Windows GUI)` e os recursos contêm os 7 tamanhos de ícone. A
  execução, a interface e a conexão com o fone foram validadas manualmente pelo
  usuário em um Windows real (versão do Windows e firmware não registrados).
- **Notas:**
  - O `PKGBUILD` apontava para uma tag inexistente e o tarball do GitHub extrai
    como `MelobudsNext-Rust-<versão>` (o GitHub remove o `v` do diretório), mas
    `build()`/`package()` faziam `cd "$pkgname-$pkgver"`. Um `prepare()` agora
    move o diretório para `melobudsnext-<versão>` quando necessário, aceitando
    tanto o tarball do GitHub quanto o gerado por `git archive`.
  - `sha256sums` deixou de ser `SKIP`: o valor gravado é o do último snapshot
    validado por `build-local.sh`. Como o hash não pode fazer parte do próprio
    tarball, o script sempre regenera o tarball com `git archive`, atualiza a
    linha com `sed` e só então chama `makepkg`; o `PKGBUILD` fica modificado no
    working tree de propósito (revisar antes de commitar). Depois da tag, rode
    `updpkgsums` e regenere o `.SRCINFO`.
  - `packaging/aur/build-local.sh` valida o caminho sem AUR: `git archive` do
    commit atual em `packaging/aur/melobudsnext-<versão>.tar.gz` (ignorado pelo
    git), checksum, `makepkg "$@"`; `-si` instala. O `.gitignore` passou a cobrir
    `src/`, `pkg/`, `*.pkg.tar.*` e o tarball local de `packaging/aur`.
  - Windows: o script PowerShell agora gera só `melobudsnext-gui.exe` com
    `-C target-feature=+crt-static` (mais o `.sha256`); `-WithCli` opcional cria
    o `.zip` com os dois binários. `build-release-xwin.sh` faz o mesmo build
    cruzado no Linux e é o caminho usado nesta validação.
  - A primeira build de Windows saiu sem ícone. `assets/melobudsnext.ico`
    (16–256 px) foi gerado a partir de `assets/logo.svg` com ImageMagick e é
    embutido por `build.rs` via `winresource` — build-dependency nova,
    justificada por localizar o `rc.exe` do MSVC no Windows e usar `llvm-rc` no
    build cruzado, sem depender de caminho fixo. `llvm-readobj
    --coff-resources` confirma os 7 tamanhos no `.exe`; o ícone também entra no
    CLI de Windows.
  - Descoberta no build cruzado: o binário da GUI saía com subsistema de console
    (`00000003`) porque faltava `#![windows_subsystem = "windows"]`; o atributo
    foi adicionado em `src/bin/gui.rs` com `cfg_attr(windows, ...)`, sem efeito
    nas outras plataformas. Sem ele, um duplo clique abriria uma janela de
    console junto da interface.
  - Armadilha: o linker do `cargo-xwin` emite `LNK4099` porque os PDBs do CRT não
    acompanham o SDK baixado. É ruído esperado e documentado em
    `packaging/README.md`; não altera o executável.
  - O artefato de Windows fica em `dist/` (ignorado pelo git); o checksum é
    publicado junto do `.exe` no GitHub Releases. O `.exe` não é assinado, então
    o SmartScreen pedirá confirmação.
  - Dívidas: a instalação em um Arch limpo (chroot/VM) não foi feita, mas o
    `makepkg -si` foi validado no Arch do próprio usuário. A versão `0.1.0` foi
    promovida para `0.2.0` no lote de release; o AUR não foi submetido e o
    `.SRCINFO` precisa ser regerado depois do `updpkgsums` pós-tag. O ícone foi
    adicionado depois da validação inicial do Windows e conferido depois.
  - Recomendação para a release: criar a tag, rodar `updpkgsums`, regerar o
    `.SRCINFO`, instalar via `makepkg -si` em um Arch limpo e reabrir o `.exe`
    (conferindo o ícone) em Windows; só depois publicar os artefatos e o
    `SHA256SUMS`.

### Release 0.2.0 (publicada)

- **Versão:** `0.2.0` em `Cargo.toml`, `Cargo.lock` e `packaging/aur/PKGBUILD`;
  `CHANGELOG.md` criado.
- **Tag:** `v0.2.0` publicada em `bf38ee2` (merge da release).
- **GitHub Release:** publicada em 2026-11-22 com os artefatos abaixo.
- **Artefatos publicados:**
  - `melobudsnext-0.2.0-x86_64-unknown-linux-gnu.tar.gz` (5,7 MiB) — SHA-256
    `e67cca9ca79d8a756c8698bfd4907710a415b7facab79c03d03e2a25da37b41b`;
  - `melobudsnext-gui.exe` (7.635.968 bytes; subsistema GUI, CRT estático,
    ícone embutido) — SHA-256
    `ef15aaa1f89ce5ca88437a1373bf7bfdc765738f083b6a2d9a50862e9d4de328`;
  - `SHA256SUMS` e os `.sha256` individuais.
- **Validação:** `makepkg` gerou `melobudsnext 0.2.0-1` com `check()` (135 + 36
  + 8 testes) e os binários respondem `0.2.0`; a instalação via `makepkg -si` foi
  confirmada pelo usuário no Arch; o `.exe` foi validado no Windows pelo usuário
  (interface, conexão e ícone). Gate completo verde na `main`.
- **Checksum do `PKGBUILD`:** `e950444f9ffdab3d94350aa20108484ca6a7b352e04177b30071e4d54887e791`,
  do tarball oficial da tag (gerado com `updpkgsums`). O caminho público
  (`makepkg` baixando a tag) foi validado depois: download, checksum,
  `prepare()` e `check()` com a suíte completa.
- **README:** reescrito para o público final em 2026-11-22 (`5315886`) e já está
  na `main`. Os artefatos da release 0.2.0 preservam o README antigo; o novo
  texto entra no `.tar.gz` e no pacote a partir da próxima release (0.2.1).
- **Pendente:** submeter o `PKGBUILD`/`.SRCINFO` (checksum oficial já fixado) ao
  AUR e decidir a próxima versão para levar o README novo aos artefatos.

## Checklist de release (após o Lote 9)

Pré-requisitos: Lotes 0–9 concluídos, `main` limpa, gate verde e limitações
conhecidas documentadas.

1. Definir a versão (`0.2.0` por padrão; `1.0.0` apenas com PRO-03 validada em
   hardware).
2. Atualizar `Cargo.toml` + `Cargo.lock` e criar/atualizar `CHANGELOG.md`.
3. Rodar o gate completo e gerar os binários com `cargo build --release`.
4. Criar a tag anotada `vX.Y.Z` — o `PKGBUILD` e o script de Windows dependem
   dela.
5. **Arch (makepkg, sem AUR):** em `packaging/aur`, atualizar `pkgver`, rodar
   `updpkgsums`, regerar o `.SRCINFO` e validar com
   `./packaging/aur/build-local.sh -si` (ou `makepkg -si` com a tag publicada)
   em um Arch limpo. Publicar o `PKGBUILD`/`.SRCINFO` no repositório e anexar as
   instruções à release; o AUR entra depois.
6. **Windows (executável único):** rodar
   `packaging/windows/build-release.ps1` (ou `build-release-xwin.sh` no Linux)
   para gerar `melobudsnext-gui.exe` com CRT estático, ícone e o SHA-256; abrir o
   `.exe` em um Windows real, conferir o ícone e conectar ao fone. Opcional:
   `-WithCli` gera um `.zip` com o CLI para quem quiser usar o terminal.
7. Publicar a release no GitHub com o `.exe`, checksums, `PKGBUILD`, notas e
   limitações conhecidas (MEL-05, PRO-05, firmware 2.0.6 e status da validação
   Linux/Windows).
8. Push manual da `main` e da tag; depois `gh release create`.
9. AUR como etapa separada, quando houver disponibilidade.

Não assinar código (não há certificado) e não publicar artefatos de plataformas
não validadas (macOS).
