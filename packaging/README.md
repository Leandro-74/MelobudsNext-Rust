# Empacotamento e release

Artefatos de distribuição do MelobudsNext. A release `0.2.0` está preparada; o
push da `main`/tag e a criação da release no GitHub são passos manuais. O fluxo
completo está no checklist de release em [`../PENDENCIAS.md`](../PENDENCIAS.md).

## Conteúdo

| Caminho | Uso |
| --- | --- |
| `aur/PKGBUILD` | Pacote do Arch Linux, compilado a partir do tarball da tag. |
| `aur/.SRCINFO` | Metadados do AUR gerados por `makepkg --printsrcinfo`. |
| `aur/build-local.sh` | Build/instalação local via `git archive` + `makepkg`, sem AUR. |
| `melobudsnext.desktop` | Entrada de menu da GUI. |
| `linux/build-release.sh` | Gera o `.tar.gz` de Linux com os dois binários. |
| `windows/build-release.ps1` | Gera o `.exe` único da GUI em um Windows com MSVC. |
| `windows/build-release-xwin.sh` | Gera o mesmo `.exe` no Linux com `cargo-xwin`. |

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

A versão atual do pacote é `0.2.0`; não altere a versão fora do lote de release.

## Artefato de Linux

Gera `dist/melobudsnext-<versão>-x86_64-unknown-linux-gnu.tar.gz` com os
binários `melobudsnext` e `melobudsnext-gui`, `README.md`, o documento do
protocolo, a licença, o `.desktop` e o ícone:

```sh
./packaging/linux/build-release.sh 0.2.0
```

O script imprime o SHA-256 e grava o arquivo `.sha256` ao lado do `.tar.gz`.

## Executável de Windows (GUI única, CRT estático)

O artefato de Windows é um único `melobudsnext-gui.exe`, compilado com CRT
estático (`-C target-feature=+crt-static`), então **não** é necessário instalar o
Visual C++ Redistributable. Os assets da GUI ficam embutidos no binário e o ícone
`assets/melobudsnext.ico` é gravado no executável por `build.rs`
(`winresource`), então ele aparece no Explorer, na barra de tarefas e no
Alt+Tab.

Em um Windows com a toolchain MSVC:

```powershell
pwsh -File packaging/windows/build-release.ps1
```

O script grava `dist/melobudsnext-gui.exe` e o checksum em
`dist/melobudsnext-gui.exe.sha256`. Com `-WithCli` ele também gera
`dist/melobudsnext-<versão>-x86_64-pc-windows-msvc.zip` com a GUI, o CLI, a
licença e a documentação (para quem quiser usar o terminal). Para a toolchain
GNU, informe `-Target x86_64-pc-windows-gnu`.

### Build cruzado a partir do Linux (`cargo-xwin`)

Sem uma máquina Windows, o mesmo `.exe` pode ser gerado no Linux com
[`cargo-xwin`](https://github.com/rust-cross/cargo-xwin):

```sh
sudo pacman -S --needed clang lld llvm
cargo install cargo-xwin          # ou use um binário de release
rustup target add x86_64-pc-windows-msvc
./packaging/windows/build-release-xwin.sh
```

O script compila com `--locked`, copia o executável para
`dist/melobudsnext-gui.exe` e grava `dist/melobudsnext-gui.exe.sha256`. O
primeiro build baixa o CRT/SDK da Microsoft, então precisa de rede. O linker
emite avisos `LNK4099` sobre PDBs do CRT que não acompanham o pacote do SDK; eles
não afetam o executável gerado.

Confira se o binário não depende do Redistributable:

```sh
llvm-objdump -p dist/melobudsnext-gui.exe | grep 'DLL Name'
```

A saída deve listar apenas DLLs do próprio Windows (`KERNEL32.dll`,
`USER32.dll`, `bcryptprimitives.dll` etc.), sem `VCRUNTIME140.dll` nem
`MSVCP140.dll`. O binário usa o subsistema de GUI (`Subsystem 00000002`), então
não abre uma janela de console junto com a interface.

### Aviso do SmartScreen

O `.exe` não é assinado digitalmente, então o Windows pode exibir "O Windows
protegeu o computador". Confira o SHA-256 antes de executar e use **Mais
informações → Executar assim mesmo**. Assinar o binário é uma etapa futura (não
há certificado no projeto).

## Checksums

Reúna os artefatos em `dist/` e gere um arquivo único:

```sh
cd dist
sha256sum *.tar.gz *.zip *.exe > SHA256SUMS
```

No Windows, use `Get-FileHash` ou o `.sha256` gerado por cada script.

## Arch Linux (`makepkg`, sem AUR)

O `PKGBUILD` fica em `packaging/aur` e compila a tag `vX.Y.Z` publicada no
GitHub. Enquanto a tag não existe, o build usa a fonte local (o próprio clone)
através de `git archive`:

```sh
git clone https://github.com/Leandro-74/MelobudsNext-Rust.git
cd MelobudsNext-Rust
./packaging/aur/build-local.sh -si
```

O script gera `packaging/aur/melobudsnext-<versão>.tar.gz` a partir do commit
atual, grava o SHA-256 desse tarball na linha `sha256sums` do `PKGBUILD` e roda
`makepkg`. Sem `-si` o pacote é apenas compilado; com `-si`, `makepkg` instala
(confirmando as dependências). A instalação foi validada com `makepkg -si` em um
Arch Linux x86_64 em **2026-11-22**. Depois da instalação:

```sh
melobudsnext --version
melobudsnext-gui --version
melobudsnext-gui --offline
```

A função `check()` do `PKGBUILD` roda a suíte completa
(`cargo test --release --all-targets --locked`), então a primeira compilação
demora. O pacote instala:

- `/usr/bin/melobudsnext` e `/usr/bin/melobudsnext-gui`;
- `/usr/share/applications/melobudsnext.desktop`;
- `/usr/share/icons/hicolor/scalable/apps/melobudsnext.svg`;
- licença e documentação em `/usr/share/licenses` e `/usr/share/doc`.

O checksum gravado no repositório vem de um tarball de `git archive` usado na
validação local; qualquer commit novo o desatualiza, então rode o
`build-local.sh` (ou `updpkgsums`) para o seu commit. O `makepkg` direto, sem o
script, só funciona depois de a tag existir e de o `sha256sums` ser atualizado.

### Publicar a tag e preparar o AUR (etapa posterior)

1. Atualize `pkgver` no `PKGBUILD` (e zere `pkgrel`).
2. Com a tag publicada, rode `updpkgsums` dentro de `packaging/aur` para trocar o
   checksum local pelo do tarball oficial. O tarball do GitHub extrai como
   `MelobudsNext-Rust-<versão>`; o `prepare()` do `PKGBUILD` normaliza esse nome.
3. Regenere os metadados:

   ```sh
   cd packaging/aur
   makepkg --printsrcinfo > .SRCINFO
   ```

4. Teste a instalação em uma máquina Arch limpa (chroot ou VM) com
   `makepkg -si` e confirme os dois binários.
5. Só então envie `PKGBUILD` e `.SRCINFO` para o repositório AUR
   `ssh://aur@aur.archlinux.org/melobudsnext.git`.

O AUR fica para uma etapa posterior; a distribuição atual é o clone do
repositório + `makepkg`.

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
