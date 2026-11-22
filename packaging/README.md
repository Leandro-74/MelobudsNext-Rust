# Empacotamento e release

Artefatos de distribuição do MelobudsNext. O projeto ainda **não publica** uma
release; estes arquivos preparam o fluxo descrito no checklist de release em
[`../PENDENCIAS.md`](../PENDENCIAS.md).

## Conteúdo

| Caminho | Uso |
| --- | --- |
| `aur/PKGBUILD` | Pacote do Arch Linux, compilado a partir do tarball da tag. |
| `aur/.SRCINFO` | Metadados do AUR gerados por `makepkg --printsrcinfo`. |
| `melobudsnext.desktop` | Entrada de menu da GUI. |
| `linux/build-release.sh` | Gera o `.tar.gz` de Linux com os dois binários. |
| `windows/build-release.ps1` | Gera o `.zip` de Windows com os dois `.exe`. |

## Pré-requisitos

- `main` limpa e gate completo verde:
  `cargo fmt --check`,
  `cargo clippy --all-targets -- -D warnings`,
  `cargo test --all-targets` e
  `cargo build --release`.
- Versão definida em `Cargo.toml` e em `Cargo.lock` (`cargo update -p
  melobudsnext` não é necessário; `cargo check` atualiza o lock).
- `CHANGELOG.md` atualizado com as mudanças e limitações conhecidas.
- Tag anotada `vX.Y.Z` criada e enviada **somente** no passo de publicação, que
  é manual.

A versão atual do pacote é `0.1.0`. O checklist sugere `0.2.0` para a primeira
release; não altere a versão fora do lote de release.

## Artefato de Linux

Gera `dist/melobudsnext-<versão>-x86_64-unknown-linux-gnu.tar.gz` com os
binários `melobudsnext` e `melobudsnext-gui`, `README.md`, o documento do
protocolo, a licença, o `.desktop` e o ícone:

```sh
./packaging/linux/build-release.sh 0.2.0
```

O script imprime o SHA-256 e grava o arquivo `.sha256` ao lado do `.tar.gz`.

## Artefato de Windows

Em um Windows com a toolchain MSVC (ou a alternativa GNU descrita no README):

```powershell
pwsh -File packaging/windows/build-release.ps1 -Version 0.2.0
```

O script compila com `--locked`, gera
`dist/melobudsnext-<versão>-x86_64-pc-windows-msvc.zip` e grava o `.sha256`.
Para a toolchain GNU, informe `-Target x86_64-pc-windows-gnu`.

## Checksums

Reúna os artefatos em `dist/` e gere um arquivo único:

```sh
cd dist
sha256sum *.tar.gz *.zip > SHA256SUMS
```

No Windows, use `Get-FileHash` ou o `.sha256` gerado por cada script.

## Pacote do Arch Linux (AUR)

O `PKGBUILD` compila a tag `vX.Y.Z` do GitHub e instala:

- `/usr/bin/melobudsnext` e `/usr/bin/melobudsnext-gui`;
- `/usr/share/applications/melobudsnext.desktop`;
- `/usr/share/icons/hicolor/scalable/apps/melobudsnext.svg`;
- licença e documentação em `/usr/share/licenses` e `/usr/share/doc`.

Antes de publicar no AUR:

1. Atualize `pkgver` no `PKGBUILD` (e zere `pkgrel`).
2. Com a tag publicada, rode `updpkgsums` dentro de `packaging/aur` para trocar
   `sha256sums=('SKIP')` pelo checksum real do tarball.
3. Regenere os metadados:

   ```sh
   cd packaging/aur
   makepkg --printsrcinfo > .SRCINFO
   ```

4. Teste o pacote em uma máquina Arch limpa:

   ```sh
   makepkg -si
   melobudsnext --version
   melobudsnext-gui --version
   ```

   O gate extra do Lote 7 é a instalação a partir do pacote gerado e a abertura
   da GUI pelo menu ou por `melobudsnext-gui --offline`.

5. Envie `PKGBUILD` e `.SRCINFO` para o repositório AUR
   `ssh://aur@aur.archlinux.org/melobudsnext.git`.

Enquanto a tag não existir, o `sha256sums=('SKIP')` permite validar a receita,
mas **não** deve ser enviado ao AUR: o checksum real é obrigatório.

## Ordem de publicação (manual)

1. Fazer o merge do lote de release e repetir o gate na `main`.
2. Criar a tag anotada e enviar `main` + tag.
3. Gerar os artefatos e o `SHA256SUMS`.
4. Criar a release no GitHub com os binários, os checksums e as limitações
   conhecidas (`0x28`, firmware 2.0.6, COR-11/MEL-05/PRO-05 e status da
   validação em Linux/Windows).
5. Atualizar o AUR como descrito acima.

Não publique artefatos de plataformas não validadas nem faça push automático: o
push e a publicação são passos manuais do usuário.
