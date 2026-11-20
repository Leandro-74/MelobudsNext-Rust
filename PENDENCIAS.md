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
| TST-01 | Teste | P2 | Sem testes para helpers do CLI |
| TST-02 | Teste | P1 | Fluxos de sessão sem dublê de transporte |
| TST-03 | Teste | P2 | Sem testes para transições de estado da GUI |
| TST-04 | Teste | P2 | Sem testes para erros de leitura/escrita da configuração |
| TST-05 | Teste | P3 | Tolerância do framing a lixo/`body_len` não é explícita |
| TST-06 | Teste | P1 | Regressões dos defeitos COR-* ainda não cobertas |
| TST-07 | Teste | P3 | Sem checklist/script de validação com hardware |
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
| PRO-01 | Protocolo/Hardware | P3 | Opcode `0x28` (ANC Wear) não decodificado |
| PRO-02 | Protocolo/Hardware | P3 | Auto-desligar e opcodes desconhecidos não explorados |
| PRO-03 | Protocolo/Hardware | P2 | Validação em Linux ainda não registrada |
| PRO-04 | Protocolo/Hardware | P3 | Sem matriz de compatibilidade por firmware |

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

---

## Sequência sugerida

1. **P1 de correção:** COR-03, COR-04, COR-05, COR-06 + testes de regressão
   (TST-06).
2. **P1 de estrutura:** REF-01, REF-03 e TST-02 — habilitam testar o resto.
3. **P2 rápidos:** COR-01, COR-02, COR-07, COR-08, COR-09, COR-10, MEL-01,
   MEL-02, MEL-04, TST-01, TST-03, TST-04, DOC-01, DOC-02, DOC-03, DOC-05,
   INF-01, INF-02, INF-03.
4. **P2 de maior esforço:** REF-02, REF-04, REF-05, IMP-01.
5. **P3 e dependentes de hardware:** REF-06, IMP-02, IMP-03, MEL-03, TST-05,
   TST-07, DOC-04, INF-04, INF-05, PRO-01..PRO-04.
