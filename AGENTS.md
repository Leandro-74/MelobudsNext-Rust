# AGENTS.md — MelobudsNext (Rust)

Instruções gerais para agentes de código neste repositório. Complementam as
regras globais do ambiente; em caso de conflito, vale a regra mais restritiva.

## 1. Antes de implementar

1. Ler, nesta ordem: `AGENTS.md`, `README.md`, `PROTOCOLO_MelobudsNext.md`,
   `PENDENCIAS.md` e o código da área afetada.
2. Identificar em `PENDENCIAS.md` o lote atual, a branch, as dependências e os
   itens a implementar. O ciclo de vida detalhado de cada lote está na seção
   **"Regras do ciclo de vida de um lote"** daquele arquivo.
3. Confirmar que a `main` está limpa e com o lote anterior já integrado.
4. Se o pedido não estiver mapeado em `PENDENCIAS.md`, registrar a pendência
   antes de codar. Não fazer mudanças fora do escopo pedido.

## 2. Arquitetura e convenções

- `src/protocol/` é puro: bytes, opcodes, ANC, toque e leituras. Não importar
  BLE, UI ou I/O.
- `src/state.rs` mantém o estado em memória e aplica notificações.
- `src/ble.rs` é o transporte (btleplug); `src/main.rs` é o CLI e
  `src/bin/gui*` é a GUI iced.
- CLI e GUI devem compartilhar a mesma lógica de protocolo e estado. Evitar
  duplicar fluxos de sincronização/comando.
- Em código de biblioteca, preferir `Result` a `panic!`/`assert!` para entradas
  externas.
- Manter o estilo tolerante já adotado: valores desconhecidos do protocolo são
  preservados (`Unknown(...)`), não descartados.
- Manter compatibilidade de leitura com o formato antigo de configuração do
  projeto Python (`address`, `uuid_*`).
- Erros de biblioteca em inglês; mensagens de interface no CLI/GUI em português,
  seguindo o padrão existente.
- Não adicionar dependências sem necessidade real; se necessário, justificar no
  commit e atualizar a documentação.
- Não fazer refatorações amplas de "passagem" junto com uma correção; manter o
  diff focado.
- Mudanças de comportamento dependentes de firmware devem ser documentadas no
  protocolo, não codificadas como identidade de um fone específico.

## 3. Qualidade, testes e validação

- Toda mudança de comportamento precisa de teste automatizado.
- Toda correção de bug precisa de teste de regressão que falhe sem a correção.
- Manter os testes existentes; não enfraquecer nem remover testes para "passar".
- Antes de cada commit e antes do merge, rodar a suíte (`cargo test --all-targets`).
- No encerramento do lote, o gate completo inclui:
  `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test --all-targets` e `cargo build --release`.
- Nunca afirmar que algo foi validado em hardware sem ter executado o teste no
  dispositivo; registrar a plataforma e o firmware usados.
- Alterações de BLE devem preservar o comportamento do CLI e da GUI; validar
  ambos quando aplicável.

## 4. Documentação

- Toda alteração precisa ficar documentada na mesma entrega:
  - novo comando, flag, variável de ambiente ou comportamento → `README.md`;
  - novo opcode, formato de bytes ou particularidade de firmware →
    `PROTOCOLO_MelobudsNext.md`;
  - API pública nova ou alterada → doc comments (`///`) e, se necessário,
    exemplo de uso.
- Atualizar `PENDENCIAS.md` ao final do lote: status, commit, data e notas de
  implementação (decisões, armadilhas e dívidas criadas).
- Mudanças que divergem do cliente Python de referência devem ser explicitadas
  no README ou no documento do protocolo.
- Não deixar código morto, `TODO` solto ou comportamento não documentado.

## 5. Commits e sessão

- Commits atômicos: um item lógico por commit, no formato
  `<tipo>[escopo]: <descrição>` com tipo/escopo em inglês e descrição em
  português.
- Não mencionar ferramentas de IA nos commits.
- Rodar os testes antes de cada commit.
- **Nunca fazer push**: o usuário publica manualmente.
- Em sessões de implementação, ao encerrar:
  1. garantir que a funcionalidade do lote está completa;
  2. rodar o gate;
  3. atualizar `PENDENCIAS.md` (status + notas);
  4. fazer o merge conforme as regras do lote;
  5. garantir que a `main` está limpa (`git status` sem alterações e sem
     arquivos não rastreados) e a branch do lote removida.
- Se a sessão for interrompida, não fazer merge parcial: manter a branch,
  registrar o status como "Em andamento" e listar o que falta.
- Uma sessão implementa um lote por vez; só iniciar o próximo após o merge e a
  validação do anterior.

## 6. Dados sensíveis e dados específicos de dispositivo

Regra absoluta: **nada de dados sensíveis ou de um fone real pode ser
commitado**, em código, teste, fixture, documentação, log, commit ou issue.

Nunca versionar:

- endereço MAC, platform ID, ID do adaptador Bluetooth ou qualquer identificador
  de um dispositivo real;
- nome real do fone, do usuário ou de uma rede;
- número de série, chaves, tokens, senhas e conteúdo de `.env`;
- capturas de tráfego, logs de notificações ou dumps de firmware que contenham
  identificadores do dispositivo;
- o arquivo local `~/.melobudsnext/config.json` (ou equivalente).

Regras práticas:

- Usar sempre valores fictícios em exemplos, testes e documentação:
  `AA:BB:CC:DD:EE:FF`, `11:22:33:44:55:66`, `"Meu Fone"`, `example.com`.
- Capturas reais devem ser anonimizadas (MAC, nome, endereço, caminhos pessoais)
  antes de entrarem no repositório; preferir fixtures sintéticas.
- UUIDs de serviço/characteristic, opcodes e formatos de bytes são constantes do
  protocolo, não dados sensíveis; ainda assim, não associá-los a um aparelho
  específico.
- Não imprimir configuração real, endereço ou credenciais em mensagens de erro,
  testes ou relatórios.
- Segredos locais, se existirem, ficam apenas em `.env`; ele deve estar no
  `.gitignore` e nunca ser commitado. Manter, se necessário, um `.env.example`
  apenas com nomes de variáveis e valores fictícios.
- Se encontrar dado sensível no working tree, interromper e relatar antes de
  commitar. Não reescrever histórico sem instrução explícita.

## 7. Build e ambiente

- Toolchain mínima: Rust 1.85 (`rust-version` no `Cargo.toml`).
- O CLI é o binário padrão (`melobudsnext`); a GUI é `melobudsnext-gui` e usa
  iced com renderer tiny-skia.
- No Linux, o BLE depende de BlueZ/D-Bus; não assumir que o adaptador está
  disponível em testes automatizados.
- Não alterar versão do pacote fora do lote de release.

## 8. Referências

- `PENDENCIAS.md` — backlog, lotes, status, notas e checklist de release.
- `PROTOCOLO_MelobudsNext.md` — protocolo BLE observado no firmware.
- `README.md` — uso do CLI, da GUI, configuração e limitações.
