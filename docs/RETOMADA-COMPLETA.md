# Apocalipse Download Manager — Retomada Completa

Atualizado em: 2026-09-24
Branch de trabalho: `claude/stoic-ramanujan-icyww8`
Último commit nesta atualização: `f75c081` (ver Seção 0 para o histórico da reversão).

> **Aviso para quem continuar**: esta branch recebeu commits de **múltiplas sessões de IA em paralelo** e do **próprio dono do repositório (`linuxhell`) diretamente via push**. Antes de assumir que algo "não foi feito" ou "já está pronto", rode `git log --oneline -40` e leia o histórico — o estado muda rápido e pode ser revertido por decisão humana sem aviso prévio na conversa.

## 0. Reversão importante — leia isto primeiro

Uma sessão de IA anterior (a que escreveu a versão passada deste arquivo) migrou o app inteiro de `aria2` clássico para **`aria2-next`** (`AnInsomniacy/aria2-next`) e construiu uma feature nova, **Apocalipse ED2K**, no lugar do Apocalipse AI (que também foi removido). Esse trabalho está documentado em detalhe no histórico do git (commits entre `4872f64` e antes), mas **foi revertido pelo próprio dono do repositório**, em commits autorais diretos (não de uma sessão Claude):

```
de614e9  refactor(engine): restore classic aria2 and purge retired engine code
48a5a5a  fix(aria2): remove orphaned ED2K tail after classic migration
f75c081  fix(aria2): remove obsolete metadata cleanup comment
```

O commit principal (`de614e9`):
- restaura `aria2` clássico (binário `aria2c`/`aria2c.exe`, fonte `FerroDownload/aria2-static-builds`) como motor de torrent/magnet;
- **remove** `ed2k.html`, `ed2k.js`, `DownloadKind::Ed2k`, toda a lógica de servidores eD2K/server.met, o comando `open_ed2k_window` e a entrada de nav "Apocalipse ED2K";
- remove as menções ao Apocalipse AI do `README.md` (a remoção do AI em si, feita antes, **foi mantida** — só o texto do README que ainda citava o AI foi limpo agora);
- deixou um resíduo (fragmento de doc-comment + chave sobrando em `aria2.rs`) que quebrava a compilação; os dois commits seguintes (`48a5a5a`, `f75c081`), também de `linuxhell`, corrigiram isso.

**Conclusão prática**: tudo que a versão anterior deste arquivo descrevia sobre aria2-next e Apocalipse ED2K **não existe mais no código** e não deve ser re-implementado sem confirmação explícita do usuário — foi uma decisão dele, tomada fora desta conversa, diretamente no GitHub. Se o usuário pedir para retomar aria2-next/ED2K, o histórico do git antes de `de614e9` tem a implementação completa e testada para referência/cherry-pick, mas **não assuma que ele quer isso de volta só porque existiu antes**.

Estado dos testes no commit `f75c081`: `cargo test --workspace` → 117/117, `node --test tests/*.test.cjs` → 284/284.

---

## 1. Estado atual do motor de download (pós-reversão)

- Torrent/magnet: **aria2 clássico** (`aria2c`), binário baixado de `FerroDownload/aria2-static-builds`, exatamente como era antes de toda a migração para aria2-next.
- HTTP direto: motor nativo Rust (segmentação, HTTP/3 com fallback, retomada por identidade remota, ETag/Last-Modified, DNS/proxy customizados, verificação SHA-256).
- eD2K/aMule: **não existe mais no app**. Se o usuário pedir novamente, ver Seção 0 para onde recuperar a implementação anterior.
- Apocalipse AI: continua removido (essa parte da decisão anterior foi mantida pelo revert).

### O que NÃO foi revertido (permanece válido do trabalho anterior)
- Correção do tema não seguido pela janela Apocalipse Link (`link.js` `validThemes` atualizado + sincronização via `get_application_theme`/evento `theme-changed`) — isso é independente do motor de download e não foi tocado pelo revert.
- Correção do fundo da página "Sobre" (remoção da regra CSS `.about-scene::after` que recortava um canto fixo específico da arte antiga) — também independente do motor, deve continuar valendo. **Precisa reteste do usuário** (ver Seção 2).

---

## 2. Status dos bugs que o usuário reportou (build antiga `65336ef`)

Reportados com 2 ZIPs de diagnóstico + 3 screenshots: imagem nova da página "Sobre" não aparecia, botão "Analisar" travado ao capturar o link da ISO do Windows 11, download de torrent não puxava metadados.

1. **Imagem da página "Sobre"**: a correção de CSS (remover o recorte de canto fixo) segue válida após o revert, já que não mexe em motor de download. **Ainda precisa reteste em build nova** — nunca foi confirmado pelo usuário em uma build pós-correção.
2. **Botão "Analisar" travado**: causa raiz hipotetizada (não confirmada 100%) — evidência de diagnóstico mostrava duas capturas (`bridge.download`/`bridge.prompt`) chegando sem `inspect_url` de acompanhamento, coincidindo com uma análise de metadados de outro torrent travada por ~150s. Hipótese: o diálogo/botão "Analisar" é único (singleton) na UI; uma nova captura chegando enquanto ele já está ocupado fica sem feedback visual de "ocupado", parecendo travada. **Não corrigido** — precisa decisão de design (enfileirar / rejeitar com mensagem clara / diálogo separado) antes de codar. Esse problema é de UI e é **independente do motor usado** (aria2 clássico ou aria2-next), então continua em aberto do jeito que estava.
3. **Torrent não puxava metadados**: na versão aria2-next, isso tinha sido rastreado a um bug real do libtorrent (`category=libtorrent code=19: torrent already exists in session`, GID reciclado ficando com zero conexões) e corrigido com poda de transferências órfãs. **Com o revert para aria2 clássico, essa análise específica não se aplica mais** — aria2 clássico tem sua própria pilha de BitTorrent, diferente do libtorrent do aria2-next, então o mesmo bug pode ou não existir aqui. **Precisa reteste do zero em aria2 clássico** para saber se o problema de metadados trava também nesse motor, ou se era específico do aria2-next.

---

## 3. Regras permanentes (não mudam com o revert)

- **Nunca** disparar `release.yml` ou criar release pública sem pedido explícito do usuário. Builds de teste usam só `validate-portable.yml`.
- Rodar `cargo test --workspace` e `node --test tests/*.test.cjs` antes de qualquer commit.
- `validThemes` em cada arquivo JS de janela deve ser mantido manualmente sincronizado com os seletores `data-theme` de `styles.css` — não há checagem automática entre os dois.
- Múltiplas sessões (humanas e de IA) editam esta branch concorrentemente. Sempre `git fetch` + checar `git log` antes de assumir o estado atual, e esperar rebases/força de conflito ao dar push.

## 4. Próximos passos recomendados

1. Pedir ao usuário para testar uma build fresca a partir do commit atual e confirmar se a imagem da página "Sobre" aparece corretamente agora.
2. Pedir para reproduzir o teste de torrent/magnet **usando aria2 clássico** (motor atual) para saber se "metadados não carregam" ainda acontece aqui, já que a causa raiz documentada antes era específica do libtorrent do aria2-next.
3. Decidir e implementar a UX do botão "Analisar" compartilhado (item 2 da Seção 2) — segue sendo o único bug sem nenhuma correção proposta.
4. Confirmar com o usuário se ele realmente não quer mais aria2-next/eD2K antes de tocar nesse assunto de novo — a decisão de reverter foi dele, direto no GitHub, não através desta conversa.
