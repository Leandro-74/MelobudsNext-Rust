# Validação manual com hardware

Roteiro para validar o CLI e a GUI em um fone real. Ele complementa os testes
automatizados (`cargo test --all-targets`), que não dependem de Bluetooth.

> **Dados sensíveis:** não registre endereço MAC, nome real do fone, nome da
> rede, número de série, caminho pessoal nem capturas com identificadores. Use
> `AA:BB:CC:DD:EE:FF`, `"Meu Fone"` e `example.com` nos relatórios.

## 1. Preparação

1. Ligue o fone, retire os dois lados da caixa e desconecte-o do celular/app
   oficial. O fone aceita um host por vez.
2. No Linux, confirme o adaptador e o BlueZ:

   ```sh
   bluetoothctl show
   bluetoothctl info AA:BB:CC:DD:EE:FF
   ```

   O adaptador deve estar `Powered: yes` e o fone `Paired: yes` e
   `Trusted: yes`.
3. Compile e confirme que o binário abre:

   ```sh
   cargo build --release
   ./target/release/melobudsnext --version
   ./target/release/melobudsnext-gui --version
   ```
4. Registre a plataforma e o firmware no relatório (seção 8) **antes** de
   começar.

## 2. Descoberta e conexão

| Passo | Comando | Resultado esperado |
| --- | --- | --- |
| 2.1 | `melobudsnext device scan --scan-timeout 8` | O fone aparece com endereço e nome; anote apenas o nome fictício no relatório |
| 2.2 | `melobudsnext config set-mac AA:BB:CC:DD:EE:FF` | Configuração gravada em `~/.melobudsnext/config.json` |
| 2.3 | `melobudsnext device info` | Bateria L/R, firmware, ANC, game/sleep, balanço, volume, LDAC, detecção de uso e toque são exibidos |
| 2.4 | `melobudsnext device info --json` | `stdout` contém somente JSON válido; avisos vão para `stderr` |
| 2.5 | `melobudsnext device read battery` | Bateria compatível com a seção 2.3 |
| 2.6 | `melobudsnext device read firmware` | Versão compatível com a seção 2.3 |
| 2.7 | `melobudsnext device read touch` | Mapa de toque com 8 teclas |
| 2.8 | `melobudsnext device inspect` | Serviço `0000a001` e as characteristics `00001001`, `00001002`, `00000008`, `00000007` e `0000000d` listados |

## 3. Comandos de estado

Execute cada item e confirme o eco/estado depois do comando. Restaure o valor
anterior ao final.

| Passo | Comando | Resultado esperado |
| --- | --- | --- |
| 3.1 | `melobudsnext device game on` / `off` | Eco `0x09` e estado alterado |
| 3.2 | `melobudsnext device sleep on` / `off` | Eco `0x10` (pode chegar atrasado) e estado alterado no `device info` seguinte |
| 3.3 | `melobudsnext device anc --scene indoor --level 2` | Eco `0x17` com `01 01 01` |
| 3.4 | `melobudsnext device anc --scene daily --level 3` | Eco `0x17` com `01 02 02` |
| 3.5 | `melobudsnext device anc --scene wind` | Eco `0x17` com `01 04 00` |
| 3.6 | `melobudsnext device anc --scene adaptive` | Eco `0x17` com `01 05 00` |
| 3.7 | `melobudsnext device anc --intensity 4` | Eco `0x17` com `03 01 04` |
| 3.8 | `melobudsnext device anc --voice` | Eco `0x17` com `03 01 00` |
| 3.9 | `melobudsnext device anc` | Eco `0x17` com `00 00 00` (o fone pode reportar `02 00 00` como desligado) |
| 3.10 | `melobudsnext device balance 40` e `60` | Eco `0x16`; o estado pode aparecer no `device info` seguinte (MEL-05) |
| 3.11 | `melobudsnext device volume medium` | Eco `0x1D` com `06 06`; idem MEL-05 |
| 3.12 | `melobudsnext device wear on --keep-anc` / `off` | Eco `0x2C` com 4 bytes; ANC mantido ou não ao remover; idem MEL-05 |
| 3.13 | `melobudsnext device rename "Meu Fone"` | Nome alterado; restaure o nome original ao final |
| 3.14 | `melobudsnext device query anc` | Resposta `0x17` com o estado atual |
| 3.15 | `melobudsnext device send "FF 03 09 01 01"` | Bytes enviados; eco quando o firmware responder |

## 4. Notificações e captura de protocolo

Mantenha `device monitor` aberto em um terminal e execute as ações em outro
quando o firmware permitir mais de um cliente. Se o link cair ao abrir o segundo
cliente, use `device info`/`device monitor` em sequência e registre a limitação.

| Passo | Ação | Resultado esperado |
| --- | --- | --- |
| 4.1 | `melobudsnext device monitor` | Notificações aparecem em hexadecimal com o opcode identificado |
| 4.2 | Alterar ANC pelo comando `device anc ...` ou pelo toque | Notificação `0x17` e, no firmware 2.0.6, `FF 03 28 01 01` (`0x28`) |
| 4.3 | Consultar `0x2F`/`0x30` via `device query` ou `device send "FF 03 FE 01 2F"` | No firmware 2.0.6, sem resposta; a leitura direta continua funcionando |
| 4.4 | Consultar `0x14` com `device send "FF 03 FE 01 14"` | Resposta `FF 06 14 04 ...` (auto-desligar); registrar os bytes |
| 4.5 | Alterar ANC e observar a GUI | A GUI atualiza sem clicar em **Atualizar** (IMP-01) |

Anote os bytes observados **sem** identificadores do fone e compare com
[`PROTOCOLO_MelobudsNext.md`](../../PROTOCOLO_MelobudsNext.md), seções 4.3, 7.10
e 7.11.

## 5. LDAC e reconexão

| Passo | Comando | Resultado esperado |
| --- | --- | --- |
| 5.1 | `melobudsnext device ldac off` | O fone reinicia, a ferramenta reconecta e o estado final mostra LDAC desligado, sem "BLE link lost" |
| 5.2 | `melobudsnext device ldac on --wait 2` | Idem 5.1, com LDAC ligado no estado final |
| 5.3 | Repetir 5.1/5.2 com `--reconnect-attempts 3 --reconnect-delay 1` | Mensagens `Reconnect attempt N/3...` aparecem, o intervalo é respeitado e o comando termina com o estado final |
| 5.4 | Repetir com o fone fora de alcance | O comando falha depois das tentativas, sem travar e sem mascarar a queda |

## 6. Queda de link

1. Conecte com `melobudsnext device info` ou abra a GUI.
2. Desligue o fone ou desligue o Bluetooth do computador.
3. Execute outra ação no CLI ou clique em um controle da GUI.
4. Resultado esperado: erro claro (CLI) ou retorno para **Desconectado** (GUI),
   sem manter um estado "Conectado" obsoleto.
5. Ligue o fone e reconecte. O estado deve ser sincronizado novamente.

Observação: em `device monitor`, a queda de link feita por outro processo não
encerrou o comando (ele ficou aguardando até o timeout do teste). Registre se o
monitor sai sozinho ou não.

## 7. GUI

| Passo | Comando/ação | Resultado esperado |
| --- | --- | --- |
| 7.1 | `melobudsnext-gui --offline` | Cabeçalho "Offline"; **Conectar**, **Aplicar** e controles de escrita desabilitados |
| 7.2 | `melobudsnext-gui --help` | Lista `--offline`, `--config`, `--reconnect-attempts`, `--reconnect-delay` |
| 7.3 | `melobudsnext-gui --config /tmp/perfil.json` | Usa o arquivo indicado e avisa no rodapé se ele não puder ser lido |
| 7.4 | Conectar e trocar ANC/toque pelo fone | A GUI reflete a mudança sem **Atualizar** |
| 7.5 | Alterar o mapa de toque pelo app oficial | A GUI relê o mapa em até ~3 s enquanto ociosa |

## 8. Relatório da execução

Preencha uma cópia deste bloco em `PENDENCIAS.md` (notas do lote) ou em um
arquivo local não versionado. Não inclua identificadores reais.

```markdown
- Data:
- Plataforma: <distro, versão do kernel, sessão Wayland/X11>
- BlueZ:
- Adaptador: <modelo ou "adaptador interno">
- Firmware: <major.minor.patch>
- CLI: <passou/falhou> — observações
- GUI: <passou/falhou> — observações
- LDAC/reconexão: <passou/falhou> — observações
- Queda de link: <passou/falhou> — observações
- Capturas: <opcodes e bytes, sem identificadores>
- Limitações encontradas:
```

## 9. Critério de conclusão

O roteiro é considerado concluído quando:

- todos os passos de 2 a 7 tiverem resultado registrado (passou, falhou ou
  bloqueado por hardware);
- o firmware e a plataforma estiverem identificados;
- as capturas novas estiverem refletidas no documento do protocolo ou em
  `PENDENCIAS.md`;
- nenhum dado sensível tiver sido commitado.
