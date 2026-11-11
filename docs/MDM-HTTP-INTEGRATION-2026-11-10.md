# Integração HTTP Nibiru 0.2.8 — ADM 0.8.17

A biblioteca Rust e o backend empacotado usam a mesma revisão exata do motor,
`f2ae7df1b48707e5246d7d40e75ae617c9732bb4`,
fixada em `apps/desktop/src-tauri/Cargo.toml` e `Cargo.lock`. O CI extrai o commit
do lockfile e instala esse executável, sem selecionar um release flutuante.

O [PR do motor](https://github.com/linuxhell/nibiru-xtreme/pull/8) reúne a auditoria
do Mister Download Manager e implementação própria. Consulte também
[a arquitetura e seus limites](https://github.com/linuxhell/nibiru-xtreme/blob/f2ae7df1b48707e5246d7d40e75ae617c9732bb4/docs/HTTP-0.2.8.md).

## Comportamento

- Downloads HTTP novos aproveitam o corpo do primeiro GET, sem HEAD separado.
  A sondagem explícita e a confirmação da identidade de retomada permanecem.
- 429/503 respeitam Retry-After e backoff limitado, sem abortar conexões úteis
  nem contornar a recusa com GET integral. Repetições são de GET idempotente.
- Recusas repetidas e evidências recentes de recepção podem ensinar um teto
  conservador por origem/contexto de acesso, por dez minutos em memória.
- O slider explícito prevalece sobre o padrão por tamanho. Quantidade de blocos,
  mínimo de divisão e limites do servidor podem resultar em menos conexões;
  o máximo escolhido nunca é ultrapassado.
- Corpos parados ou interrompidos recuperam somente os blocos ainda não gravados.
  O esgotamento das tentativas preserva o parcial verificável.
- Buffers são reutilizados depois da escrita. No Windows, parciais segmentados
  podem ser preparados como esparsos antes de definir o tamanho lógico.
- TLS, validadores, proxy/headers, limite de banda e publicação atômica seguem
  obrigatórios. A interface mostra “Finalizando arquivo…” até a confirmação real
  do backend; não infere conclusão a partir apenas dos bytes recebidos.

## Extensões e escopo

Chrome/Edge/Firefox permanecem em 0.3.210. O XPI Mozilla-assinado foi preservado
byte a byte: 210874 bytes, SHA256
`5b730b9ecdd6ba2c0f80447624ef8942672ee2e436f594856a99cb352a51c202`.
A mudança no motor não exige nova extensão ou assinatura. A correção POST
anterior permanece. Não há confirmação de sucesso real em UUP Dump, Nuvid
ou de ID alto ED2K na rede do usuário.

## Verificação

Os 384 testes JavaScript locais passaram sem skips. O motor exige Clippy estrito,
todos os targets/features, regressões HTTP/RPC e interoperabilidade independente.
O CI do ADM exige testes desktop completos, transport lab e extensão em Windows,
Linux e macOS x64 antes de publicar seus artefatos validados.

O ambiente local não tem GTK/WebKit de desenvolvimento. Os testes de core/CLI
não substituem a validação desktop do CI. Ensaios sintéticos não provam
superioridade universal sobre IDM/MDM, maior capacidade de Internet ou aceleração
em torrent/ED2K.

O ensaio controlado do motor registrou 120 execuções com hashes: arquivo pequeno
de 94 ms para 51 ms; cenário limitado por conexão de 2,244 s para 1,182 s com
máximo explícito de oito. A configuração e o limite do servidor influenciam o
resultado; isso não multiplica a capacidade de um enlace já saturado.
