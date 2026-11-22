# MelobudsNext: Documentação do Protocolo BLE

Documentação do protocolo de comunicação entre o computador e o fone **QCY Melobuds Pro**, extraída da análise do código-fonte do projeto MelobudsNext (versão 1.1.0). Os valores foram validados por engenharia reversa, com o app oficial usado como oráculo. Quando a documentação de terceiros divergia dos bytes observados, o projeto adotou os bytes.

> Projeto não oficial, sem vínculo com a QCY. Testado no firmware 2.0.6.

---

## 1. Visão geral do projeto

O MelobudsNext é uma ferramenta em Python que controla o fone QCY Melobuds Pro via Bluetooth Low Energy (BLE), sem depender do aplicativo oficial. A interface é uma TUI construída com Textual, e a comunicação BLE é feita com a biblioteca `bleak`.

O projeto é organizado em camadas:

| Módulo | Responsabilidade |
| --- | --- |
| `commands.py` | Framing dos pacotes, definição dos opcodes e fábricas de comandos |
| `keys.py` | Protocolo de mapeamento de toque (characteristic `0000000D`) |
| `state.py` | Estado ao vivo do fone, mantido somente em memória |
| `device.py` | Conexão BLE, sincronização inicial, envio, recebimento e leituras diretas |
| `tui_app.py` | Fluxo da interface e ações do usuário |
| `config.py` | Persistência apenas do endereço MAC e dos UUIDs, em `~/.melobudsnext/config.json` |

Funcionalidades cobertas: Game Mode, ANC (com cenas e níveis), Modo Transparência, leitura de bateria por lado, versão do firmware, renomear o fone, personalização de toque, volume de notificação, equilíbrio de canal, detecção de uso e LDAC. O estado é atualizado em tempo real, então mudanças feitas pelo toque no fone ou pelo app oficial aparecem na ferramenta.

---

## 2. Camada BLE: serviço e characteristics

### 2.1 Serviço e characteristics de comando

| Função | UUID | Modo |
| --- | --- | --- |
| Serviço principal | `0000a001-0000-1000-8000-00805f9b34fb` | — |
| Escrita de comandos | `00001001-0000-1000-8000-00805f9b34fb` | Write sem resposta |
| Notificações (eco e estado) | `00001002-0000-1000-8000-00805f9b34fb` | Notify |

### 2.2 Characteristics de leitura direta

| Dado | UUID | Modo | Formato |
| --- | --- | --- | --- |
| Bateria | `00000008-0000-1000-8000-00805f9b34fb` | Read | `[L, R, reservado]` |
| Versão do firmware | `00000007-0000-1000-8000-00805f9b34fb` | Read | `[major, minor, patch]` |
| Mapeamento de toque | `0000000d-0000-1000-8000-00805f9b34fb` | Read / Write | 20 bytes (ver seção 5) |

Os dados de bateria e versão não passam pelo canal de comandos. São lidos diretamente das characteristics acima.

---

## 3. Framing dos pacotes (canal de comandos)

Todo comando enviado e toda notificação recebida no canal `00001001` / `00001002` seguem o mesmo formato:

```
[0xFF] [body_len] [opcode] [param_len] [params...]
```

| Campo | Tamanho | Descrição |
| --- | --- | --- |
| `0xFF` | 1 byte | Cabeçalho fixo |
| `body_len` | 1 byte | `2 + param_len` (conta opcode, param_len e os parâmetros) |
| `opcode` | 1 byte | Identificador do comando |
| `param_len` | 1 byte | Quantidade de bytes de parâmetros |
| `params` | `param_len` bytes | Dados do comando |

Exemplo de montagem: o comando Game Mode ligado tem `param_len = 1` e `body_len = 3`, portanto o pacote completo é `FF 03 09 01 01`.

### 3.1 Pacotes compostos

Uma notificação pode trazer mais de um comando concatenado. O parser lê sequencialmente cada bloco `[opcode, param_len, params]` a partir do offset 2. Se um bloco estiver truncado, ele é descartado.

O `body_len` do cabeçalho **não é confiável em pacotes compostos**: ele pode refletir apenas o primeiro bloco (ou um valor fixo do firmware) em vez da soma do corpo inteiro. Exemplo: `FF 06 09 01 01 17 03 00 00 00` traz `body_len = 06`, mas o corpo tem 8 bytes e contém dois blocos (`0x09` com 1 parâmetro e `0x17` com 3). Por isso o parser ignora `body_len` e lê os blocos até o fim dos bytes recebidos, descartando somente o bloco final incompleto. Não valide `body_len` como soma total do corpo.

### 3.2 Eco de comandos

O fone responde a comandos com uma notificação de eco contendo o mesmo opcode, com o estado resultante. Por exemplo, após enviar Game Mode, o fone notifica `0x09` com o novo valor. É dessa forma que o estado ao vivo é mantido.

---

## 4. Tabela de opcodes

### 4.1 Opcodes de comando

| Opcode | Nome | Parâmetros | Observação |
| --- | --- | --- | --- |
| `0x09` | Game Mode | `[01]` ligar, `[02]` desligar | Validado |
| `0x10` | Sleep Mode | `[01]` ligar, `[02]` desligar | Implementado no código; não listado na validação do README |
| `0x16` | Equilíbrio do canal | `[valor]`, `0–100`, centro `50` | Implementado no código |
| `0x17` | ANC / Transparência | `[mode, sub, noise]` (3 bytes) | Validado (ver seção 6) |
| `0x18` | Renomear | Nome em UTF-8 | Validado |
| `0x1D` | Volume de notificação | `[byte, byte]` (duplicado) | Implementado no código (ver seção 7.4) |
| `0x23` | LDAC | `[01]` ligar, `[02]` desligar | Implementado no código |
| `0x2C` | Detecção de uso | `[enable, music_index, anc_enable, tone]` | Implementado; formato "espelho" do report |
| `0xFE` | Consulta de estado | `[opcode alvo]` | Validado |

O `0xFE` também foi usado para consultar opcodes ainda não implementados. O
firmware 2.0.6 respondeu a `0x14`, mas não a `0x06`, `0x28`, `0x29`, `0x2F` e
`0x30`. Esses casos estão registrados na seção 4.3.

### 4.2 Opcodes de estado (notificações)

| Opcode | Nome | Uso |
| --- | --- | --- |
| `0x09`, `0x10`, `0x16`, `0x17`, `0x18`, `0x1D`, `0x23`, `0x2C` | Eco / resposta | Mesmos formatos dos comandos acima |
| `0x28` | ANC Wear / Result | Resultado de uma alteração de ANC; payload preservado (seção 7.11) |
| `0x2F` | Bateria | `[esquerda, direita, caixa]` (seção 7.12); o firmware 2.0.6 não notifica nem responde ao `0xFE` |
| `0x30` | Versão | 3 ou 6 bytes (seção 7.13); o firmware 2.0.6 não notifica nem responde ao `0xFE` |

### 4.3 Opcodes observados fora do escopo

Estes opcodes apareceram em capturas ou em consultas, mas não são usados pelo
cliente. A tabela registra apenas o que foi observado no firmware 2.0.6; a
semântica não confirmada fica explicitamente marcada.

| Opcode | Consulta enviada | Resposta em 2.0.6 | Situação |
| --- | --- | --- | --- |
| `0x14` | `FF 03 FE 01 14` | `FF 06 14 04 FF FF 00 00` | Auto-desligar; formato em 7.10, não implementado |
| `0x06` | `FF 03 FE 01 06` | nenhuma em 350 ms | Detecção in-ear; não explorado |
| `0x29` | `FF 03 FE 01 29` | nenhuma em 350 ms | "ANC Wear" em documentação de terceiros; não explorado |
| `0x28` | `FF 03 FE 01 28` | nenhuma em 350 ms | Só foi visto como notificação espontânea (7.11) |
| `0x2F` | `FF 03 FE 01 2F` | nenhuma em 350 ms | Bateria via comando; usar a characteristic `00000008` |
| `0x30` | `FF 03 FE 01 30` | nenhuma em 350 ms | Versão via comando; usar a characteristic `00000007` |

### 4.4 Resumo de exemplos de pacotes completos

| Ação | Bytes enviados |
| --- | --- |
| Game Mode ligado | `FF 03 09 01 01` |
| Game Mode desligado | `FF 03 09 01 02` |
| Consulta do ANC | `FF 03 FE 01 17` |
| ANC desligado | `FF 05 17 03 00 00 00` |
| ANC Interior, nível 2 | `FF 05 17 03 01 01 01` |
| Transparência, intensidade 4 | `FF 05 17 03 03 01 04` |
| Transparência, aprimoramento vocal | `FF 05 17 03 03 01 00` |
| Renomear para "Fone" | `FF 06 18 04 46 6F 6E 65` |
| Equilíbrio em 60 | `FF 03 16 01 3C` |
| Volume de notificação, categoria 2 | `FF 04 1D 02 06 06` |
| Detecção de uso ligada, ANC ao remover ligado | `FF 06 2C 04 01 01 01 00` |

---

## 5. Protocolo de toque (characteristic `0000000D`)

Diferente do canal de comandos, esta characteristic **não usa o framing `0xFF`**. É um bloco fixo de 20 bytes, lido e gravado de uma vez.

### 5.1 Layout

O bloco contém 8 pares `[tecla, função]`, seguidos de 4 bytes de padding `00`.

```
[k1 f1] [k2 f2] [k3 f3] [k4 f4] [k5 f5] [k6 f6] [k7 f7] [k8 f8] [00 00 00 00]
```

A ordem de armazenamento é fixa: `Esq.1, Dir.1, Esq.2, Dir.2, Esq.3, Dir.3, Esq.4, Dir.4`.

### 5.2 Teclas

| Byte | Tecla |
| --- | --- |
| `0x01` | Esquerda, 1 toque |
| `0x02` | Direita, 1 toque |
| `0x03` | Esquerda, 2 toques |
| `0x04` | Direita, 2 toques |
| `0x05` | Esquerda, 3 toques |
| `0x06` | Direita, 3 toques |
| `0x07` | Esquerda, 4 toques |
| `0x08` | Direita, 4 toques |

### 5.3 Funções atribuíveis

| Byte | Função |
| --- | --- |
| `0x00` | Nenhuma (desativado) |
| `0x01` | Play/Pause |
| `0x02` | Faixa anterior |
| `0x03` | Próxima faixa |
| `0x04` | Assistente de voz |
| `0x05` | Volume + |
| `0x06` | Volume − |
| `0x07` | Alternar Game Mode |
| `0x08` | Atender chamada |
| `0x09` | Recusar chamada |
| `0x0A` | Chamada em espera |
| `0x0B` | Modo ANC **(neste firmware)** |

A documentação padrão da QCY indica que `0x0B` é "rediscar", mas no firmware 2.0.6 ele corresponde ao Modo ANC.

Para desativar o toque, todas as funções devem ser `0x00`.

### 5.4 Exemplo

Esquerda 1 toque com Play/Pause e direita 1 toque com Próxima faixa, demais teclas sem função:

```
01 01  02 03  03 00  04 00  05 00  06 00  07 00  08 00  00 00 00 00
```

### 5.5 Leitura e gravação

Leitura: `read_gatt_char(0000000D)` e interpretação dos pares, ignorando pares com tecla `0x00`.

Gravação: `write_gatt_char(0000000D, 20 bytes, response=False)`. O mapeamento sempre é gravado por inteiro, com as teclas não informadas recebendo `0x00`.

---

## 6. Controle de ANC e Transparência (opcode `0x17`)

### 6.1 Formato

Os três parâmetros são `[mode, sub, noise]`.

| Mode | Significado |
| --- | --- |
| `0x00` | Desligado (comando) |
| `0x01` | ANC |
| `0x03` | Transparência |

### 6.2 Cenas de ANC (`mode = 0x01`)

| Cena | `sub` | `noise` | Níveis |
| --- | --- | --- | --- |
| Interior | `0x01` | `00`, `01`, `02` | Nível 1 a 3 |
| Viagens Diárias | `0x02` | `00`, `01`, `02` | Nível 1 a 3 |
| Barulho | `0x03` | `00`, `01`, `02` | Nível 1 a 3 |
| Contra o vento | `0x04` | `00` | Sem níveis |
| Adaptativo | `0x05` | `00` | Sem níveis |

Fórmula para cenas com nível: `noise = nivel − 1`.

### 6.3 Transparência (`mode = 0x03`)

| Função | `sub` | `noise` |
| --- | --- | --- |
| Aprimoramento vocal | `0x01` | `0x00` |
| Intensidade 1 a 6 | `0x01` | `0x01` a `0x06` |

### 6.4 Desligar ANC

Comando: `00 00 00`. Observação: o fone **reporta** o estado desligado como `02 00 00`, então o parser aceita os dois padrões como "Desligado".

### 6.5 Ponto de atenção

O opcode `0x0C` (ANC simples, descrito em alguma documentação de terceiros) **não existe neste firmware**. Todo controle de ANC passa pelo `0x17`.

---

## 7. Detalhes de cada função

### 7.1 Game Mode (`0x09`)

- Envio: `[01]` ligar, `[02]` desligar.
- Estado: notificação com `[01]` indica ligado; qualquer outro valor é lido como desligado.

### 7.2 Renomear (`0x18`)

- Envio: nome em UTF-8 como parâmetros. O tamanho máximo do campo no fone é de 32 bytes.
- Leitura: enviar `0xFE` com parâmetro `0x18`. A resposta traz um campo de 32 bytes com padding `NUL`, que é removido antes de decodificar.

### 7.3 Equilíbrio do canal (`0x16`)

- Parâmetro: valor de `0` a `100`, com `50` como centro. Valores fora desse intervalo são limitados pelo código.
- Valores menores que 50 inclinam para a esquerda, e maiores que 50 para a direita.

### 7.4 Volume de notificação (`0x1D`)

O fone usa um byte real para cada categoria, e o parâmetro é enviado duplicado:

| Categoria | Byte | Envio completo |
| --- | --- | --- |
| 1. Volume mais baixo | `0x04` | `FF 04 1D 02 04 04` |
| 2. Volume médio | `0x06` | `FF 04 1D 02 06 06` |
| 3. Volume mais alto | `0x08` | `FF 04 1D 02 08 08` |
| 4. Volume máximo | `0x0A` | `FF 04 1D 02 0A 0A` |

Os valores foram mapeados com o app oficial como referência. Não são sequenciais, então não devem ser interpretados como uma escala linear.

### 7.5 Detecção de uso (`0x2C`)

Parâmetros `[enable, music_index, anc_enable, tone]`:

| Byte | Significado |
| --- | --- |
| `enable` | `01` ligada, `00` desligada |
| `music_index` | Índice da música de aviso (padrão `01`) |
| `anc_enable` | `01` mantém ANC ao remover, `00` não |
| `tone` | Tom (padrão `00`) |

O fone pode enviar 3 ou 4 bytes de parâmetro. Quando vier com 3, o campo `tone` é tratado como `00`.

### 7.6 LDAC (`0x23`)

- Envio: `[01]` ligar, `[02]` desligar.
- Estado: `[01]` indica ligado.

### 7.7 Sleep Mode (`0x10`)

- Envio: `[01]` ligar, `[02]` desligar.
- Estado: `[01]` indica ligado.

### 7.8 Consulta de estado (`0xFE`)

Envia `FE 01 <opcode alvo>` para pedir o estado atual de um opcode. A resposta chega como notificação com o formato normal do opcode consultado. No início da conexão, o projeto consulta na sequência: `0x17`, `0x09`, `0x10`, `0x18`, `0x16`, `0x1D`, `0x23` e `0x2C`, com 50 ms entre as consultas.

### 7.9 Bateria e versão (leitura direta)

**Bateria** (`00000008`): três bytes `[L, R, reservado]`.
- Bit 7: carregando (`1`) ou não (`0`).
- Bits 0 a 6: nível de 0 a 127%, na prática de 0 a 100.
- Nível `0` é exibido como `-/-` (sem dados).
- O terceiro byte é ignorado.

**Versão** (`00000007`): três bytes, formatados como `major.minor.patch`. Exemplo: `02 00 06` vira `2.0.6`.

### 7.10 Auto-desligar (`0x14`)

A consulta `FF 03 FE 01 14` foi a única dos opcodes explorados que respondeu no
firmware 2.0.6:

```
FF 06 14 04 FF FF 00 00
```

O bloco tem quatro parâmetros:

| Byte | Campo | Formato |
| --- | --- | --- |
| 0–1 | tempo configurado para desligar | `u16` little-endian, em minutos |
| 2–3 | tempo decorrido | `u16` little-endian, em minutos |

No ensaio, o tempo configurado veio como `FF FF` (65535). A documentação de
terceiros usa o mesmo campo para o timer de auto-desligar, mas o significado de
`0xFFFF` (desligado, "sem timer" ou valor máximo) não foi confirmado. O cliente
não envia `0x14` e não altera essa configuração; o opcode está apenas
documentado para não ser tratado como desconhecido em capturas futuras.

### 7.11 Resultado de ANC Wear (`0x28`)

O firmware 2.0.6 envia uma notificação com um único parâmetro depois de uma
mudança de ANC:

```
FF 03 28 01 01
```

O byte `01` foi o único observado, inclusive ao alternar entre uma cena de ANC e
"desligado". O nome do opcode no log do cliente Python é `ANC Wear/Result`, e a
documentação de terceiros o chama de "ANC Result"; o significado do byte não é
documentado pelo fabricante.

Decisões de implementação:

- o payload cru é preservado (`AncWearResult`), em vez de ser convertido em um
  booleano adivinhado;
- a notificação **não** substitui o estado de ANC (`0x17`), que continua sendo a
  fonte autoritativa da cena e do nível;
- payloads com mais de um byte são aceitos e mostrados com o tamanho, para que
  uma captura futura possa ser interpretada sem perda;
- o cliente responde a `0xFE 0x28` com silêncio, como registrado na seção 4.3.

### 7.12 Bateria via comando (`0x2F`)

Formato documentado por terceiros e aceito pelo parser:

```
[esquerda, direita, caixa]
```

Cada byte usa o mesmo formato da characteristic `00000008`: bit 7 indica
carregamento e os bits 0–6 o nível. O firmware 2.0.6 não enviou essa notificação
durante a captura e não respondeu à consulta `FF 03 FE 01 2F`; por isso a
leitura direta continua sendo o caminho normal. O decodificador aceita 2 ou 3
bytes; o terceiro byte (caixa) não é armazenado no estado atual.

### 7.13 Versão via comando (`0x30`)

Formato documentado por terceiros:

- 3 bytes: `[major, minor, patch]`;
- 6 bytes: `[Lmajor, Lminor, Lpatch, Rmajor, Rminor, Rpatch]`.

O firmware 2.0.6 não enviou essa notificação e não respondeu à consulta
`FF 03 FE 01 30`. O cliente decodifica os dois formatos, mas mantém apenas os
três primeiros bytes em `DeviceState::firmware`; a versão do lado direito não é
exposta separadamente. A leitura direta da characteristic `00000007` continua
sendo a fonte normal.

---

## 8. Particularidades deste firmware

Estas são divergências em relação ao protocolo padrão da QCY, observadas no firmware 2.0.6:

1. `0x0C` (ANC simples) não existe. Use apenas `0x17`.
2. `0xFE` não responde para bateria nem versão. Essas informações vêm das leituras diretas.
3. O fone fica silencioso em repouso e só envia notificações quando o estado muda.
4. O byte `0x0B` do toque corresponde ao Modo ANC, e não a "rediscar".
5. O opcode `0x28` (ANC Wear/Result) é recebido depois de mudanças de ANC; o payload é preservado (seção 7.11).
6. Bateria e versão via comando (`0x2F`/`0x30`) não foram observadas; as consultas `0xFE` para esses opcodes ficam sem resposta.

### 8.1 Matriz de compatibilidade por firmware

A matriz registra o que foi efetivamente observado. Uma linha marcada como
"não reproduzido" vem de relato de terceiros e não deve ser tratada como
validação deste projeto.

| Versão | Origem | `0x17` (ANC) | `0x28` | `0x2F`/`0x30` | Toque `0x0B` | Detecção de uso | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 2.0.6 | captura local (Linux, 2026-11-22) | envia `00 00 00`; reporta `02 00 00` como desligado | notifica `FF 03 28 01 01` após mudança | não notifica; `0xFE` sem resposta | Modo ANC | reporta 4 bytes (`FF 06 2C 04 ...`) | Validado |
| 2.0.4 | relato de terceiros (`qcy-control-macos`, não reproduzido) | exemplos usam `02 00 00` para desligado | não informado | não informado | não informado | não informado | Não validado |
| outras | — | — | — | — | — | — | Desconhecido |

Regras para atualizar a matriz:

- uma linha nova só entra como "Validado" com captura ou teste registrado na
  mesma entrega;
- divergências de firmware devem ser descritas no comportamento do opcode, não
  codificadas como identidade de um aparelho;
- se um firmware novo mudar um formato, o parser deve continuar tolerante e
  preservar o valor desconhecido.

---

## 9. Fluxo típico de uma sessão

1. Carregar `~/.melobudsnext/config.json` (MAC e UUIDs) ou solicitar o MAC na primeira execução.
2. Conectar via BLE e resolver as characteristics do serviço `0000a001`.
3. Ativar notificações em `00001002`.
4. Ler bateria (`00000008`), versão (`00000007`) e toque (`0000000D`).
5. Enviar as consultas `0xFE` para sincronizar o estado restante.
6. Durante a sessão, cada notificação é decodificada por `parse_packet` e aplicada ao `DeviceState`.
7. Em caso de queda, a ferramenta tenta reconectar até 10 vezes, com intervalo de 3 segundos.

---

## 10. Método de validação (resumo)

- Scripts de laboratório enviam e leem pacotes crus.
- O app oficial é usado como referência para confirmar o significado de cada byte.
- Só depois de validado, o mecanismo entra na ferramenta.
- Divergências entre documentação de terceiros e os bytes observados são resolvidas a favor dos bytes.

---

## 11. Estado atual e próximos passos

Pendências declaradas no README:

- Explorar o auto-desligar e os opcodes desconhecidos.
- Decodificar o opcode `0x28`.
- Criar testes automatizados de parse e montagem de pacotes.
- Testar no Linux, já que a validação foi feita somente no Windows.
- Empacotar como `.exe` e para Arch Linux (já existe um PKGBUILD em `packaging/aur`).
