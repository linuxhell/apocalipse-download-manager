(function (root, factory) {
  const localModel = typeof module === "object" && module.exports
    ? require("./apocalipse-ai-local-model.js")
    : root.ApocalipseAILocalModel;
  const api = factory(localModel);
  if (typeof module === "object" && module.exports) module.exports = api;
  else root.ApocalipseAI = api;
})(typeof globalThis !== "undefined" ? globalThis : this, function (localModel) {
  "use strict";

  const copy = {
    en: {
      analyzing: "I’ll analyze this problem.",
      hello: "Hello! I’m the local Apocalipse assistant. I can analyze downloads, previews, recordings, the browser extension, settings and diagnostic logs.",
      unknown: "I couldn’t identify the exact Apocalipse feature in your question. Tell me what happened, the website name and whether it involved Download, Preview or Record.",
      noEvidence: "I don’t have enough diagnostic evidence to confirm the cause. Enable advanced diagnostics, reproduce the problem and ask me to analyze it again.",
      noErrors: "I found no recent errors in the current logs.",
      errors: "I found {count} recent error(s). The latest one is: {detail}",
      installed: "This computer is running Apocalipse {app} with browser extension {extension}. The local assistant cannot confirm whether a newer public release exists without an update check.",
      tiktokGuide: "To preview a TikTok video from the extension, click “Preview”. When the share window opens, click the “Copy” button. Apocalipse will use the copied address to open the video in the player you selected.",
      rapidgator404: "I found the cause in the logs. Rapidgator rejected the temporary address with error 404. This type of address may be consumed by its first request, so retrying the same task will not work; a new address must be generated after the captcha.",
      pixeldrainRule: "Pixeldrain downloads automatically use one connection. This compatibility rule prevents failures caused by segmented requests and does not change the connection setting for other websites.",
      rateLimit: "The website temporarily limited the requests (error 429). Wait for the period indicated by the website before trying again.",
      accessDenied: "The website denied access to the file. The session may have expired or the download may require the browser’s current login.",
      incomplete: "Apocalipse found an incomplete media track and blocked it to avoid opening or downloading the wrong video.",
      disconnected: "The browser extension was disconnected from Apocalipse during this attempt. Reconnect it and repeat the action.",
      sponsored: "The video was classified as sponsored content, so it was intentionally excluded from the extension list.",
      recordOnly: "The page did not expose a complete downloadable address. Recording may remain available because it captures the media while it plays.",
      missingButton: "I found the page activity, but not enough evidence to prove why the Download button was missing. Play the video, click the area where the button should appear and ask me to analyze the new records.",
      socialMissing: "The structured social debugger found {count} visible player(s) without a live overlay on {site}. The latest confirmed reason is “{reason}” for player {player}. This conclusion comes from player-decision telemetry, not a guess.",
      socialSummary: "Social debugger for {site}: {visible} visible player(s), {eligible} eligible, {overlays} overlay(s), {missing} missing, {sponsored} sponsored, {inactive} inactive and {noAction} without a supported action.",
      genericFailure: "The latest related failure was: {detail}",
      domainEvidence: "Forensic Debugger V4 found observed evidence for {domain}. Latest relevant event: {event}. Detail: {detail}. This is recorded telemetry, not a guessed cause.",
      historyEmpty: "There are no saved corrections.",
      historyCount: "There are {count} saved correction(s). Open Correction history to view, apply or remove them.",
      knownCorrection: "I already have a locally confirmed correction for {site}: “{name}”. It worked {count} time(s), most recently with Apocalipse {version}. You can apply it again for testing if the problem returned.",
      correctionLearned: "I saved “{name}” as a new local correction candidate for {site}. It will only become confirmed knowledge after a successful manual test.",
      noPending: "There is no correction waiting for your decision.",
      correctionTesting: "“{name}” is now marked for testing. Reproduce the problem and tell me whether it worked.",
      correctionRejected: "The correction did not solve the problem. I restored its previous state and will analyze the new records for another safe possibility.",
      correctionConfirmed: "Correction “{name}” was confirmed as working.",
      correctionSaved: "Correction “{name}” was not applied and remains saved in Correction history.",
      correctionRemoved: "Correction “{name}” was removed.",
      correctionNotFound: "I couldn’t find that correction. Open Correction history and copy its exact name.",
      clarifyButton: "Are you referring to the button over the video or to the download already listed in Apocalipse?",
      clarifyAction: "Did this happen when you clicked Download, Preview or Record?",
      taskSummary: "There are {total} task(s): {active} active, {paused} paused, {failed} failed and {completed} completed.",
      downloadSlow: "A slow download may be caused by the website, the number of allowed connections, a configured speed limit, proxy/VPN routing or lack of torrent peers. Tell me the website or task so I can check its records.",
      performanceObserved: "The transfer-engine telemetry shows {speed} now, with a recorded peak of {peak}, using {connections} connection(s), {sources} verified source(s) and {protocol}. {finding}",
      performanceNeedDiagnostics: "I do not have structured transfer telemetry for this download yet. Enable Advanced diagnostics while reproducing the slow transfer so I can measure connections, mirrors, resume decisions and throughput.",
      performanceNoSamples: "I found transfer-engine decisions, but there are not enough throughput samples yet to measure the slowdown. Keep Advanced diagnostics active during the transfer and ask me again.",
      performanceSingleConnection: "Only one connection was observed. If this server limits each connection, additional safe ranges could improve throughput; the current evidence does not confirm that yet.",
      performanceManyConnections: "The engine is already using {connections} connections. Without evidence of unused bandwidth, increasing the count further is not the first change to test.",
      performanceDrop: "Observed throughput fell to about {ratio}% of the recorded peak. This confirms a slowdown occurred, but does not by itself prove server throttling.",
      performanceStable: "The latest sample is close to the recorded peak, so the current telemetry does not show a large throughput collapse.",
      performanceMirrors: "{count} segment(s) needed another verified source, so mirror fallback actively recovered failed or slower attempts.",
      performanceResumeRestarted: "A previous partial transfer was not resumed because the remote identity or byte range no longer matched the saved checkpoint. Apocalipse restarted safely instead of mixing incompatible bytes.",
      performanceResumeAccepted: "The saved checkpoint matched the remote validator and byte range, so safe resume was accepted.",
      performanceProtocolUnknown: "protocol not recorded yet",
      performanceRangeSteal: "Adaptive range stealing reassigned {count} slow-tail range(s), moving {bytes} to idle/faster workers instead of waiting for the original connection.",
      performanceRemoteChecksum: "The server advertised a SHA-256 digest and Apocalipse is using it to detect transfer corruption. Because the digest came from the same origin, it is transport-integrity evidence, not independent publisher authentication.",
      previewHelp: "Preview opens the selected media in the external player configured in Tools. It must never create a download or open the save-location window.",
      recordingHelp: "Record captures media while it plays and later exports the result. Use it when the page does not provide a complete direct download address.",
      extensionHelp: "The browser extension detects media and sends approved actions to the desktop application. Its connection status appears at the bottom of Apocalipse.",
      toolsHelp: "Tools manages FFmpeg, FFprobe, yt-dlp, aria2, N_m3u8DL-RE, QuickJS and your external media player.",
      torrentHelp: "The Torrents section manages magnet and torrent tasks, selected files, peers, progress, speed and previews.",
      ed2kHelp: "The ed2k function depends on its network engine and available servers or Kad peers. Connection and source availability determine whether a transfer starts.",
      linkHelp: "Apocalipse Link transfers files between authorized computers. Check the remote ID, password and connection status when a transfer does not start.",
      settingsHelp: "Settings contains the download folder, clipboard capture, network, proxy, DNS, credentials, associations and extension pairing.",
      filesHelp: "The destination folder can be selected for each task or set as the default in Settings. Open folder uses the completed task’s actual destination.",
      privacyHelp: "Apocalipse AI runs locally. It analyzes privacy-safe diagnostic events and does not send the conversation to an external model.",
      credentialSaved: "Credentials for {host} were saved for user {username}.",
      credentialInvalid: "I understood that you want to add site credentials, but the site, username or password is missing. Use: add a rule for site example.com username myuser password mypassword.",
      credentialFailed: "I couldn’t save the credentials. Check the site address, username and password.",
      chatCleared: "Conversation cleared. How can I help with Apocalipse?",
      acknowledged: "All right. I’m here if you want to continue.",
      thanks: "You’re welcome! I’m here if you need anything else about Apocalipse.",
      goodbye: "See you later! When you need me, I’ll be here in Apocalipse.",
      wellbeing: "I’m working normally and ready to help with Apocalipse. What would you like me to check?",
      capabilities: "I can explain Apocalipse features, check its current status and privacy-safe logs, diagnose downloads, previews, recordings and the extension, guide settings, and manage saved site credentials and correction tests.",
      helpPrompt: "Of course. Tell me what happened, the website name and whether it involved Download, Preview or Record.",
      currentTime: "The current system time is {time}.",
      updateChecking: "I’ll check the official Apocalipse release now.",
      updateAvailable: "An update is available: Apocalipse {latest}. This computer is running {current}. Open the official Releases page to download it.",
      updateReleaseLink: "Open official Releases",
      updateNavTitle: "New Apocalipse version {latest} available",
      upToDate: "Apocalipse is up to date. This computer is running {current}.",
      updateUnavailable: "I couldn’t reach the official update service right now. The installed version is {current}.",
      updateDetails: "Official notes for Apocalipse {latest}: {details}",
      updateDetailsUnavailable: "I checked Apocalipse {latest}, but the official release does not include usable release notes. Open the official Releases page for the complete publication.",
      siteLogFound: "I found {count} diagnostic event(s) for {site}. The latest record is: {detail}",
      siteLogEmpty: "I found no diagnostic records for {site} in the current log.",
      offTopic: "I’m specialized in Apocalipse Download Manager. Ask me about its downloads, sites, extension, media, settings, tools or diagnostics.",
    },
    "pt-BR": {
      analyzing: "Vou analisar esse problema.",
      hello: "Olá! Sou a assistente local do Apocalipse. Posso analisar downloads, visualizações, gravações, extensão, configurações e registros de diagnóstico.",
      unknown: "Não consegui identificar exatamente qual função do Apocalipse você mencionou. Diga o que aconteceu, o nome do site e se foi em Baixar, Visualizar ou Gravar.",
      noEvidence: "Ainda não tenho registros suficientes para confirmar a causa. Ative o diagnóstico avançado, reproduza o problema e depois me peça para analisar novamente.",
      noErrors: "Não encontrei erros recentes nos registros atuais.",
      errors: "Encontrei {count} erro(s) recente(s). O último foi: {detail}",
      installed: "Este computador está usando o Apocalipse {app} com a extensão {extension}. Sem executar a verificação de atualização, a assistente local não pode confirmar se existe uma versão pública mais recente.",
      tiktokGuide: "Para visualizar um vídeo do TikTok pela extensão, clique em “Visualizar”. Quando a janela de compartilhamento abrir, clique no botão “Copy”. O Apocalipse usará o endereço copiado para abrir o vídeo no player que você definiu.",
      rapidgator404: "Encontrei a causa nos registros. O Rapidgator recusou o endereço temporário com erro 404. Esse tipo de endereço pode ser consumido pela primeira tentativa; por isso, retomar a mesma tarefa não funcionará e será necessário gerar outro endereço após o captcha.",
      pixeldrainRule: "Os downloads do Pixeldrain usam automaticamente uma conexão. Essa regra de compatibilidade evita falhas causadas por requisições divididas e não altera a configuração dos outros sites.",
      rateLimit: "O site limitou temporariamente as tentativas, com erro 429. Aguarde o período informado pelo próprio site antes de tentar novamente.",
      accessDenied: "O site recusou o acesso ao arquivo. A sessão pode ter vencido ou o download pode depender da autenticação atual do navegador.",
      incomplete: "O Apocalipse encontrou uma faixa de mídia incompleta e a bloqueou para evitar abrir ou baixar o vídeo errado.",
      disconnected: "A extensão foi desconectada do Apocalipse durante essa tentativa. Reconecte-a e repita a ação.",
      sponsored: "O vídeo foi classificado como conteúdo patrocinado e, por isso, foi retirado intencionalmente da lista da extensão.",
      recordOnly: "A página não forneceu um endereço completo que pudesse ser baixado. A gravação pode continuar disponível porque captura a mídia enquanto ela é reproduzida.",
      missingButton: "Encontrei a atividade da página, mas ainda não há provas suficientes para confirmar por que o botão Baixar não apareceu. Reproduza o vídeo, clique na área onde o botão deveria estar e depois me peça para analisar os novos registros.",
      socialMissing: "O debugger social estruturado encontrou {count} player(s) visível(is) sem overlay ativo no {site}. O motivo confirmado mais recente é “{reason}” no player {player}. Essa conclusão vem da telemetria de decisão do player, não de uma suposição.",
      socialSummary: "Debugger social do {site}: {visible} player(s) visível(is), {eligible} elegível(is), {overlays} overlay(s), {missing} faltando, {sponsored} patrocinado(s), {inactive} inativo(s) e {noAction} sem ação suportada.",
      genericFailure: "A última falha relacionada foi: {detail}",
      domainEvidence: "O Debugger Forense V4 encontrou evidência observada para {domain}. Último evento relevante: {event}. Detalhe: {detail}. Isso vem da telemetria registrada, não de uma causa presumida.",
      historyEmpty: "Não há correções guardadas.",
      historyCount: "Existem {count} correção(ões) guardada(s). Abra o Histórico de correções para visualizar, aplicar ou apagar.",
      knownCorrection: "Já tenho uma correção confirmada localmente para {site}: “{name}”. Ela funcionou {count} vez(es), mais recentemente no Apocalipse {version}. Você pode aplicá-la novamente para teste se o problema voltou.",
      correctionLearned: "Guardei “{name}” como uma nova candidata de correção local para {site}. Ela só vira conhecimento confirmado depois de um teste manual bem-sucedido.",
      noPending: "Não existe uma correção aguardando sua decisão.",
      correctionTesting: "“{name}” foi marcada para teste. Reproduza o problema e depois informe se funcionou.",
      correctionRejected: "A correção não resolveu o problema. Restaurei o estado anterior e vou analisar os novos registros procurando outra possibilidade segura.",
      correctionConfirmed: "A correção “{name}” foi confirmada como funcional.",
      correctionSaved: "A correção “{name}” não foi aplicada e continua guardada no Histórico de correções.",
      correctionRemoved: "A correção “{name}” foi apagada.",
      correctionNotFound: "Não encontrei essa correção. Abra o Histórico de correções e copie o nome exato.",
      clarifyButton: "Você está falando do botão sobre o vídeo ou do download que já aparece na lista do Apocalipse?",
      clarifyAction: "Isso aconteceu quando você clicou em Baixar, Visualizar ou Gravar?",
      taskSummary: "Existem {total} tarefa(s): {active} ativa(s), {paused} pausada(s), {failed} com falha e {completed} concluída(s).",
      downloadSlow: "Um download lento pode ser causado pelo site, quantidade de conexões permitidas, limite de velocidade, rota de proxy/VPN ou falta de pares no torrent. Informe o site ou a tarefa para eu conferir os registros.",
      performanceObserved: "A telemetria do motor mostra {speed} agora, com pico registrado de {peak}, usando {connections} conexão(ões), {sources} fonte(s) verificada(s) e {protocol}. {finding}",
      performanceNeedDiagnostics: "Ainda não tenho telemetria estruturada do motor para esse download. Ative o Diagnóstico avançado enquanto reproduz a lentidão para eu medir conexões, mirrors, decisões de retomada e velocidade.",
      performanceNoSamples: "Encontrei decisões do motor, mas ainda não há amostras de velocidade suficientes para medir a lentidão. Mantenha o Diagnóstico avançado ativo durante a transferência e me pergunte novamente.",
      performanceSingleConnection: "Foi observada apenas uma conexão. Se o servidor limitar a velocidade por conexão, faixas adicionais seguras podem aumentar o desempenho; os registros atuais ainda não confirmam isso.",
      performanceManyConnections: "O motor já está usando {connections} conexões. Sem evidência de banda ociosa, aumentar ainda mais esse número não é a primeira mudança a testar.",
      performanceDrop: "A velocidade observada caiu para cerca de {ratio}% do pico registrado. Isso confirma que houve lentidão, mas sozinho não prova que o servidor aplicou throttling.",
      performanceStable: "A amostra mais recente está próxima do pico registrado, portanto a telemetria atual não mostra uma grande queda de velocidade.",
      performanceMirrors: "{count} segmento(s) precisaram de outra fonte verificada; o fallback de mirrors recuperou ativamente tentativas com falha ou mais lentas.",
      performanceResumeRestarted: "Uma transferência parcial anterior não foi retomada porque a identidade remota ou a faixa de bytes deixou de corresponder ao checkpoint salvo. O Apocalipse reiniciou com segurança em vez de misturar bytes incompatíveis.",
      performanceResumeAccepted: "O checkpoint salvo correspondeu ao validador remoto e à faixa de bytes, então a retomada segura foi aceita.",
      performanceProtocolUnknown: "protocolo ainda não registrado",
      performanceRangeSteal: "O range stealing adaptativo redistribuiu {count} cauda(s) lenta(s), movendo {bytes} para workers ociosos/mais rápidos em vez de esperar a conexão original.",
      performanceRemoteChecksum: "O servidor anunciou um SHA-256 e o Apocalipse está usando esse hash para detectar corrupção na transferência. Como o hash veio da própria origem, ele comprova integridade de transporte, não autenticação independente do publicador.",
      previewHelp: "Visualizar abre a mídia escolhida no player externo configurado em Ferramentas. Essa ação nunca deve criar um download nem abrir a janela de escolha do local de salvamento.",
      recordingHelp: "Gravar captura a mídia enquanto ela é reproduzida e permite exportar o resultado depois. Use quando a página não fornecer um endereço direto completo para download.",
      extensionHelp: "A extensão detecta mídias no navegador e envia as ações autorizadas ao aplicativo. O estado da conexão aparece no rodapé do Apocalipse.",
      toolsHelp: "Ferramentas administra FFmpeg, FFprobe, yt-dlp, aria2, N_m3u8DL-RE, QuickJS e o seu player externo.",
      torrentHelp: "A seção Torrents administra magnet e torrent, arquivos escolhidos, pares, progresso, velocidade e visualizações.",
      ed2kHelp: "A função ed2k depende do motor de rede e de servidores ou pares Kad disponíveis. A conexão e a existência de fontes determinam se a transferência começa.",
      linkHelp: "O Apocalipse Link transfere arquivos entre computadores autorizados. Quando não iniciar, confira o ID remoto, a senha e o estado da conexão.",
      settingsHelp: "Configurações contém pasta de download, captura da área de transferência, rede, proxy, DNS, credenciais, associações e pareamento da extensão.",
      filesHelp: "A pasta de destino pode ser escolhida em cada tarefa ou definida como padrão nas Configurações. Abrir pasta usa o destino real da tarefa concluída.",
      privacyHelp: "O Apocalipse AI funciona localmente. Ele analisa eventos de diagnóstico protegidos e não envia a conversa para um modelo externo.",
      credentialSaved: "As credenciais de {host} foram salvas para o usuário {username}.",
      credentialInvalid: "Entendi que você quer adicionar credenciais de site, mas falta o site, o usuário ou a senha. Use: adicione uma regra para o site exemplo.com nome de usuário meuusuario e senha minhasenha.",
      credentialFailed: "Não consegui salvar as credenciais. Confira o endereço do site, o usuário e a senha.",
      chatCleared: "Conversa limpa. Como posso ajudar com o Apocalipse?",
      acknowledged: "Certo. Estou aqui se quiser continuar.",
      thanks: "Por nada! Estou aqui se precisar de mais alguma coisa sobre o Apocalipse.",
      goodbye: "Até mais! Quando precisar, estarei aqui no Apocalipse.",
      wellbeing: "Estou funcionando normalmente e pronta para ajudar com o Apocalipse. O que você quer que eu confira?",
      capabilities: "Posso explicar as funções do Apocalipse, conferir o estado atual e os registros protegidos, diagnosticar downloads, visualizações, gravações e a extensão, orientar configurações e administrar credenciais de sites e testes de correções.",
      helpPrompt: "Claro. Conte o que aconteceu, o nome do site e se foi em Baixar, Visualizar ou Gravar.",
      currentTime: "Agora são {time}, conforme o relógio deste computador.",
      updateChecking: "Vou consultar agora a versão oficial do Apocalipse.",
      updateAvailable: "Há uma atualização disponível: Apocalipse {latest}. Este computador está usando a versão {current}. Abra a página oficial de Releases para baixar.",
      updateReleaseLink: "Abrir Releases oficial",
      updateNavTitle: "Nova versão do Apocalipse {latest} disponível",
      upToDate: "O Apocalipse está atualizado. Este computador está usando a versão {current}.",
      updateUnavailable: "Não consegui acessar o serviço oficial de atualização agora. A versão instalada é {current}.",
      updateDetails: "Notas oficiais do Apocalipse {latest}: {details}",
      updateDetailsUnavailable: "Conferi o Apocalipse {latest}, mas essa release não possui notas oficiais utilizáveis. Abra a página oficial de Releases para ver a publicação completa.",
      siteLogFound: "Encontrei {count} registro(s) de diagnóstico sobre {site}. O registro mais recente é: {detail}",
      siteLogEmpty: "Não encontrei registros de diagnóstico sobre {site} no log atual.",
      offTopic: "Sou especializada no Apocalipse Download Manager. Pergunte sobre downloads, sites, extensão, mídia, configurações, ferramentas ou diagnósticos.",
    },
    "zh-CN": {
      analyzing: "我会分析这个问题。",
      hello: "你好！我是 Apocalipse 本地助手。我可以分析下载、预览、录制、浏览器扩展、设置和诊断日志。",
      unknown: "我无法确定你所指的 Apocalipse 功能。请说明发生了什么、网站名称，以及问题涉及下载、预览还是录制。",
      noEvidence: "目前没有足够的诊断记录来确认原因。请开启高级诊断，重现问题，然后让我再次分析。",
      noErrors: "当前日志中没有发现近期错误。",
      errors: "我发现了 {count} 个近期错误。最后一个是：{detail}",
      installed: "此电脑正在运行 Apocalipse {app} 和浏览器扩展 {extension}。如果不执行更新检查，本地助手无法确认是否已有更新的公开版本。",
      tiktokGuide: "要通过扩展预览 TikTok 视频，请点击“预览”。分享窗口打开后，请点击“Copy”按钮。Apocalipse 将使用复制的地址，在你设置的播放器中打开视频。",
      rapidgator404: "我在日志中找到了原因。Rapidgator 以 404 错误拒绝了临时地址。此类地址可能会被第一次请求消耗，因此重试同一任务不会成功；需要在验证码后重新生成地址。",
      pixeldrainRule: "Pixeldrain 下载会自动使用一个连接。此兼容性规则可避免分段请求造成的故障，并且不会更改其他网站的连接设置。",
      rateLimit: "网站暂时限制了请求（错误 429）。请等待网站提示的时间后再试。",
      accessDenied: "网站拒绝访问该文件。登录会话可能已过期，或者下载需要浏览器当前的登录状态。",
      incomplete: "Apocalipse 发现了不完整的媒体轨道，并阻止了它，以免打开或下载错误的视频。",
      disconnected: "此次尝试期间，浏览器扩展与 Apocalipse 断开了连接。请重新连接后重试。",
      sponsored: "该视频被识别为赞助内容，因此已从扩展列表中有意排除。",
      recordOnly: "页面没有提供完整的可下载地址。录制仍可能可用，因为它会在媒体播放时进行捕获。",
      missingButton: "我找到了页面活动，但证据不足以确认下载按钮未出现的原因。请播放视频，点击按钮本应出现的区域，然后让我分析新的记录。",
      socialMissing: "结构化社交媒体调试器在 {site} 上发现 {count} 个可见播放器没有活动覆盖按钮。最近确认的原因是播放器 {player} 的“{reason}”。这个结论直接来自播放器决策遥测，而不是猜测。",
      socialSummary: "{site} 社交调试摘要：{visible} 个可见播放器，{eligible} 个可处理，{overlays} 个覆盖按钮，{missing} 个缺失，{sponsored} 个赞助内容，{inactive} 个非活动播放器，{noAction} 个没有受支持操作。",
      genericFailure: "最近一次相关故障是：{detail}",
      historyEmpty: "没有保存的修正。",
      historyCount: "已保存 {count} 个修正。打开“修正历史”可以查看、应用或删除。",
      knownCorrection: "我已经有一个针对 {site} 的本地已确认修正：“{name}”。它已成功 {count} 次，最近一次是在 Apocalipse {version}。如果问题再次出现，可以重新应用它进行测试。",
      correctionLearned: "我已将“{name}”保存为 {site} 的新本地修正候选。只有在手动测试成功后，它才会成为已确认知识。",
      noPending: "没有等待你决定的修正。",
      correctionTesting: "“{name}”已标记为测试。请重现问题，然后告诉我是否有效。",
      correctionRejected: "此修正没有解决问题。我已恢复之前的状态，并会分析新记录以寻找另一种安全方案。",
      correctionConfirmed: "修正“{name}”已确认有效。",
      correctionSaved: "修正“{name}”未应用，并保存在修正历史中。",
      correctionRemoved: "修正“{name}”已删除。",
      correctionNotFound: "找不到该修正。请打开修正历史并复制准确名称。",
      clarifyButton: "你指的是视频上方的按钮，还是已经出现在 Apocalipse 列表中的下载？",
      clarifyAction: "这是在你点击下载、预览还是录制时发生的？",
      taskSummary: "共有 {total} 个任务：{active} 个活动、{paused} 个暂停、{failed} 个失败、{completed} 个完成。",
      downloadSlow: "下载缓慢可能由网站、允许的连接数、速度限制、代理或 VPN 路由，或种子缺少节点造成。请告诉我网站或任务，以便检查日志。",
      performanceObserved: "传输引擎遥测显示当前速度为 {speed}，本次记录峰值为 {peak}，正在使用 {connections} 个连接、{sources} 个已验证来源以及 {protocol}。{finding}",
      performanceNeedDiagnostics: "目前还没有这个下载的结构化传输遥测。请在重现下载缓慢时开启“高级诊断”，这样我才能测量连接数、镜像、续传决策和吞吐量。",
      performanceNoSamples: "我找到了传输引擎的决策记录，但还没有足够的速度样本来衡量降速。请在传输期间保持“高级诊断”开启，然后再让我分析。",
      performanceSingleConnection: "目前只观察到一个连接。如果服务器按单连接限速，增加安全的分段连接可能提高速度；现有证据还不能确认这一点。",
      performanceManyConnections: "引擎已经在使用 {connections} 个连接。在没有发现带宽未被利用的证据前，不应首先继续增加连接数。",
      performanceDrop: "观察到的速度已降至本次记录峰值的大约 {ratio}%。这能确认发生了降速，但仅凭这一点不能证明服务器进行了限速。",
      performanceStable: "最新速度样本接近本次记录峰值，因此当前遥测没有显示明显的吞吐量崩塌。",
      performanceMirrors: "有 {count} 个分段需要切换到另一个已验证来源，说明镜像回退正在主动恢复失败或较慢的尝试。",
      performanceResumeRestarted: "之前的部分下载没有继续，因为远程文件身份或字节范围已不再匹配保存的检查点。Apocalipse 为避免混合不兼容的数据而进行了安全重启。",
      performanceResumeAccepted: "保存的检查点与远程验证器和字节范围一致，因此已接受安全续传。",
      performanceProtocolUnknown: "尚未记录协议",
      performanceRangeSteal: "自适应范围窃取重新分配了 {count} 个慢速尾部范围，将 {bytes} 交给空闲或更快的工作线程，而不是等待原连接完成。",
      performanceRemoteChecksum: "服务器公布了 SHA-256，Apocalipse 正用它检测传输损坏。由于该摘要来自同一来源，它只能作为传输完整性证据，不能独立证明发布者身份。",
      previewHelp: "预览会在“工具”中设置的外部播放器里打开所选媒体。它绝不能创建下载或打开保存位置窗口。",
      recordingHelp: "录制会在媒体播放时捕获内容，之后可以导出结果。当页面没有提供完整的直接下载地址时可使用此功能。",
      extensionHelp: "浏览器扩展负责检测媒体，并把获准的操作发送到桌面应用。连接状态显示在 Apocalipse 底部。",
      toolsHelp: "“工具”用于管理 FFmpeg、FFprobe、yt-dlp、aria2、N_m3u8DL-RE、QuickJS 和你的外部播放器。",
      torrentHelp: "“种子”部分管理磁力链接和种子任务、所选文件、节点、进度、速度和预览。",
      ed2kHelp: "ed2k 功能依赖网络引擎以及可用服务器或 Kad 节点。连接状态和来源数量决定传输能否开始。",
      linkHelp: "Apocalipse Link 在获授权的电脑之间传输文件。无法开始时，请检查远程 ID、密码和连接状态。",
      settingsHelp: "“设置”包含下载文件夹、剪贴板捕获、网络、代理、DNS、凭据、关联和扩展配对。",
      filesHelp: "可以为每个任务选择目标文件夹，也可以在“设置”中设为默认文件夹。“打开文件夹”使用已完成任务的实际位置。",
      privacyHelp: "Apocalipse AI 在本地运行。它分析经过隐私保护的诊断事件，不会把对话发送给外部模型。",
      credentialSaved: "已为用户 {username} 保存 {host} 的凭据。",
      credentialInvalid: "我知道你想添加网站凭据，但缺少网站、用户名或密码。请使用：为网站 example.com 添加规则，用户名 myuser，密码 mypassword。",
      credentialFailed: "无法保存凭据。请检查网站地址、用户名和密码。",
      chatCleared: "对话已清除。关于 Apocalipse，我能帮你什么？",
      acknowledged: "好的。如果你想继续，我随时在这里。",
      thanks: "不客气！如果你还需要关于 Apocalipse 的帮助，我随时在这里。",
      goodbye: "再见！需要帮助时，我会一直在 Apocalipse 里。",
      wellbeing: "我运行正常，随时可以帮助你使用 Apocalipse。你想让我检查什么？",
      capabilities: "我可以说明 Apocalipse 的功能，检查当前状态和隐私安全日志，诊断下载、预览、录制和扩展问题，指导设置，并管理网站凭据和修正测试。",
      helpPrompt: "当然可以。请告诉我发生了什么、网站名称，以及问题涉及下载、预览还是录制。",
      currentTime: "根据这台电脑的时钟，现在是 {time}。",
      updateChecking: "我现在会检查 Apocalipse 官方版本。",
      updateAvailable: "有可用更新：Apocalipse {latest}。这台电脑当前使用 {current}。请打开官方 Releases 页面下载。",
      updateReleaseLink: "打开官方 Releases",
      updateNavTitle: "Apocalipse 新版本 {latest} 可用",
      upToDate: "Apocalipse 已是最新版本。这台电脑当前使用 {current}。",
      updateUnavailable: "目前无法连接官方更新服务。已安装版本为 {current}。",
      updateDetails: "Apocalipse {latest} 的官方更新说明：{details}",
      updateDetailsUnavailable: "我已检查 Apocalipse {latest}，但该版本没有可用的官方更新说明。请打开官方 Releases 页面查看完整发布内容。",
      siteLogFound: "我找到了 {count} 条关于 {site} 的诊断记录。最新记录是：{detail}",
      siteLogEmpty: "当前日志中没有找到关于 {site} 的诊断记录。",
      offTopic: "我专用于 Apocalipse Download Manager。你可以询问下载、网站、扩展、媒体、设置、工具或诊断。",
    },
  };

  const localeOf = value => ["pt-BR", "zh-CN"].includes(value) ? value : "en";
  const say = (locale, key, values = {}) => Object.entries(values).reduce(
    (text, [name, value]) => text.replaceAll(`{${name}}`, String(value)),
    copy[localeOf(locale)][key] || copy.en[key] || key,
  );
  const fold = value => String(value || "").normalize("NFD").replace(/[\u0300-\u036f]/g, "").toLowerCase();
  const normalizeQuestion = value => fold(value)
    .replace(/\b(?:pq|pk|prq)\b/g, "por que")
    .replace(/\b(?:extencao|extensao|extençao|extenção)\b/g, "extensao")
    .replace(/\b(?:visualisar|vizualizar|vizualisar)\b/g, "visualizar")
    .replace(/\b(?:dowload|donwload|downlod)\b/g, "download")
    .replace(/\b(?:tik tok|tik-tok)\b/g, "tiktok")
    .replace(/\b(?:face book)\b/g, "facebook")
    .replace(/\b(?:obg|brigado|brigada)\b/g, "obrigado")
    .replace(/\b(?:vlw|valeu demais)\b/g, "valeu")
    .replace(/\s+/g, " ").trim();
  const safeDetail = value => detailText(value).replace(/([?&](?:token|sig|key|auth|password|cookie)=[^\s&]+)/gi, " [protected]").slice(0, 360);
  const detailText = value => typeof value === "string" ? value : value && typeof value === "object" ? JSON.stringify(value) : String(value || "");
  const eventText = event => fold(`${event?.event || ""} ${detailText(event?.detail)} ${event?.raw || ""} ${event?.source || ""}`);

  function parseCredentialCommand(input) {
    const raw = String(input || "").trim();
    const intent = /(?:credencia|nome de usu[aá]rio|usu[aá]rio|username|user name|密码|用户名|凭据)/i.test(raw)
      && /(?:site|网站|dom[ií]nio|domain)/i.test(raw);
    if (!intent) return null;
    const host = raw.match(/(?:site|网站|dom[ií]nio|domain)\s*(?:[:：=-]|do|de|for)?\s*(https?:\/\/[^\s,;]+|[a-z0-9-]+(?:\.[a-z0-9-]+)+)/i)?.[1]?.replace(/[.,;]+$/, "");
    const username = raw.match(/(?:nome\s+de\s+usu[aá]rio|usu[aá]rio|username|user\s*name|用户名)\s*(?:[:：=-]|é|is)?\s*([^\s,;，；]+)/i)?.[1];
    const password = raw.match(/(?:senha|password|密码)\s*(?:[:：=-]|é|is)?\s*(.+)$/i)?.[1]?.trim();
    const cleanPassword = password?.replace(/^["']|["']$/g, "");
    return { host: host || "", username: username || "", password: cleanPassword || "", valid: Boolean(host && username && cleanPassword) };
  }

  function redactCredentialCommand(input) {
    const parsed = parseCredentialCommand(input);
    if (!parsed?.password) return String(input || "");
    const index = String(input).lastIndexOf(parsed.password);
    return index < 0 ? String(input) : `${String(input).slice(0, index)}••••••••`;
  }

  function parseEvents(contents) {
    if (Array.isArray(contents)) return contents;
    return String(contents || "").split(/\r?\n/).filter(Boolean).map(line => {
      try { return JSON.parse(line); }
      catch { return { level: / ERROR /i.test(line) ? "ERROR" : / WARN /i.test(line) ? "WARN" : "INFO", raw: line }; }
    });
  }

  function siteFrom(question) {
    const text = fold(question);
    const known = ["rapidgator", "tiktok", "facebook", "instagram", "youtube", "pixeldrain", "uupdump", "telegram", "chatgpt"];
    const named = known.find(site => text.includes(site));
    if (named) return named;
    const domain = text.match(/\b([a-z0-9-]+(?:\.[a-z0-9-]+)+)\b/)?.[1];
    return domain || text.match(/\bsite\s+([a-z0-9_-]+)/)?.[1] || null;
  }

  function relatedEvents(events, site) {
    const all = parseEvents(events);
    if (!site) return all.slice(-500);
    return all.filter(event => eventText(event).includes(fold(site))).slice(-500);
  }

  function stateKey(task) {
    if (typeof task?.state === "string") return fold(task.state);
    return fold(Object.keys(task?.state || {})[0]);
  }

  function previousSubject(messages) {
    const previous = [...(messages || [])].reverse().find(message => message.role === "user");
    if (!previous) return "";
    const site = siteFrom(previous.text);
    const q = normalizeQuestion(previous.text);
    const area = ["download", "visualizar", "preview", "gravar", "record", "torrent", "ed2k", "extensao", "player", "botao"]
      .find(value => q.includes(value));
    return [site, area].filter(Boolean).join(" ");
  }

  function contextualQuestion(question, messages) {
    const q = normalizeQuestion(question);
    if (!/\b(ele|ela|isso|esse|essa|aquele|aquela|it|this|that|它|这个|那个)\b/.test(q)) return q;
    const subject = previousSubject((messages || []).slice(0, -1));
    return subject ? `${q} ${subject}` : q;
  }

  function formatRate(bytesPerSecond) {
    const value = Number(bytesPerSecond || 0);
    if (!Number.isFinite(value) || value <= 0) return "0 MB/s";
    const mib = value / (1024 * 1024);
    return `${mib >= 100 ? mib.toFixed(0) : mib.toFixed(1)} MB/s`;
  }

  function performanceDiagnosis(context, locale) {
    const events = parseEvents(context.engineEvents || [])
      .filter(event => /^http\.(?:engine_|performance_|segment_|resume_|transfer_|transport_|range_|remote_|integrity_)/.test(String(event?.event || "")));
    if (!events.length) return say(locale, "performanceNeedDiagnostics");

    const samples = events.filter(event => event.event === "http.performance_sample"
      && Number(event?.detail?.bytesPerSecond) >= 0);
    if (!samples.length) return say(locale, "performanceNoSamples");

    const latest = samples.at(-1);
    const current = Number(latest.detail.bytesPerSecond || 0);
    const peak = Math.max(...samples.map(event => Number(event.detail.bytesPerSecond || 0)));
    const plan = [...events].reverse().find(event => event.event === "http.engine_plan"
      || event.event === "http.transfer_started");
    const segments = events.filter(event => event.event === "http.segment_completed");
    const latestSegment = segments.at(-1);
    const connections = Number(latest?.detail?.activeConnections
      || plan?.detail?.activeConnections || 1);
    const sources = Number(plan?.detail?.sourceCount
      || latestSegment?.detail?.sourceCount || 1);
    const protocol = String(latestSegment?.detail?.transport || say(locale, "performanceProtocolUnknown"));
    const ratio = peak > 0 ? Math.round((current / peak) * 100) : 100;

    const findings = [];
    findings.push(ratio < 70
      ? say(locale, "performanceDrop", { ratio })
      : say(locale, "performanceStable"));
    if (connections <= 1) findings.push(say(locale, "performanceSingleConnection"));
    else if (connections >= 16) findings.push(say(locale, "performanceManyConnections", { connections }));

    const mirrorFallbacks = segments.filter(event => Number(event?.detail?.attempts || 1) > 1).length;
    if (mirrorFallbacks) findings.push(say(locale, "performanceMirrors", { count: mirrorFallbacks }));

    const steals = events.filter(event => event.event === "http.range_stolen");
    if (steals.length) {
      const stolenBytes = steals.reduce((sum, event) => sum + Number(event?.detail?.stolenBytes || 0), 0);
      findings.push(say(locale, "performanceRangeSteal", {
        count: steals.length,
        bytes: formatRate(stolenBytes).replace("/s", ""),
      }));
    }
    if (events.some(event => event.event === "http.remote_checksum")) {
      findings.push(say(locale, "performanceRemoteChecksum"));
    }

    const resume = [...events].reverse().find(event => event.event === "http.resume_decision");
    if (resume?.detail?.accepted === true
      && Number(resume?.detail?.resumedBytes || resume?.detail?.candidateBytes || 0) > 0) {
      findings.push(say(locale, "performanceResumeAccepted"));
    } else if (resume?.detail?.accepted === false
      && Number(resume?.detail?.candidateBytes || 0) > 0) {
      findings.push(say(locale, "performanceResumeRestarted"));
    }

    return say(locale, "performanceObserved", {
      speed: formatRate(current),
      peak: formatRate(peak),
      connections,
      sources,
      protocol,
      finding: findings.join(" "),
    });
  }

  function socialDiagnosis(events, site, locale, question = "") {
    const all = parseEvents(events).filter(event => String(event?.event || "").startsWith("social."));
    if (!all.length) return null;
    const filtered = site
      ? all.filter(event => fold(event?.detail?.platform || "").includes(fold(site)) || eventText(event).includes(fold(site)))
      : all;
    if (!filtered.length) return null;
    const missing = filtered.filter(event => event.event === "social.overlay_missing");
    const latestSummary = [...filtered].reverse().find(event => event.event === "social.scan_summary")?.detail || null;
    const q = normalizeQuestion(question);
    if (missing.length && /(botao|button|overlay|video|extensao|extension|falh|erro|problem|missing|按钮|视频|扩展)/.test(q)) {
      const latest = missing.at(-1)?.detail || {};
      return say(locale, "socialMissing", {
        site: latest.platform || site || "social",
        count: missing.length,
        reason: latest.reason || latest.decision || "unknown",
        player: latest.playerId || "unknown",
      });
    }
    if (latestSummary && /(diagnost|debug|resumo|summary|estado|status|log|registro|诊断|调试|摘要|状态)/.test(q)) {
      return say(locale, "socialSummary", {
        site: latestSummary.platform || site || "social",
        visible: latestSummary.visible ?? 0,
        eligible: latestSummary.eligible ?? 0,
        overlays: latestSummary.overlays ?? 0,
        missing: latestSummary.missing ?? 0,
        sponsored: latestSummary.sponsored ?? 0,
        inactive: latestSummary.inactive ?? 0,
        noAction: latestSummary.noAction ?? 0,
      });
    }
    return null;
  }

  function debuggerDomainDiagnosis(question, context, locale) {
    const q = normalizeQuestion(question);
    const domains = [
      { name: "aria2", test: /aria2|torrent|magnet/, event: /^(?:aria2\.|http\.engine_selected)/ },
      { name: "FFmpeg/FFprobe", test: /ffmpeg|ffprobe/, event: /ffmpeg|ffprobe|media\.preview|preview\./i },
      { name: "HLS/N_m3u8DL-RE", test: /hls|m3u8|n_m3u8dl|m3u8dl/, event: /hls|m3u8|external\./i },
      { name: "yt-dlp", test: /yt-dlp|yt_dlp/, event: /yt_dlp|external\./i },
      { name: "Apocalipse Link", test: /apocalipse link|\blink\b|computador remoto|remote computer|远程/, event: /^(?:link\.|ui\..*link)|link_transfer/i },
      { name: "thumbnail", test: /thumbnail|miniatura|缩略图/, event: /^thumbnail\./ },
      { name: "recording", test: /gravar|gravacao|recording|record|录制/, event: /recording|blob\./i },
      { name: "browser extension", test: /extensao|extension|扩展/, event: /^(?:extension\.|social\.|capture\.|overlay\.)/ },
    ];
    const domain = domains.find(item => item.test.test(q));
    if (!domain) return null;
    const events = [...parseEvents(context.events), ...parseEvents(context.engineEvents || [])]
      .filter(event => domain.event.test(String(event?.event || "")) || domain.event.test(eventText(event)));
    if (!events.length) return null;
    const latestFailure = [...events].reverse().find(event =>
      ["ERROR", "WARN"].includes(String(event?.level || "").toUpperCase())
      || /failed|error|missing|rejected|unavailable/i.test(String(event?.event || ""))
    );
    const latest = latestFailure || events.at(-1);
    return say(locale, "domainEvidence", {
      domain: domain.name,
      event: latest?.event || "unknown",
      detail: safeDetail(latest?.detail || latest?.raw || latest?.event || "unknown"),
    });
  }

  function diagnose(question, context, locale) {
    const q = contextualQuestion(question, context.messages);
    const site = siteFrom(q);
    const scoped = relatedEvents(context.events, site);
    const text = scoped.map(eventText).join("\n");
    const failures = scoped.filter(event => String(event.level || "").toUpperCase() === "ERROR" || /failed|error=/.test(eventText(event)));
    const social = socialDiagnosis([...parseEvents(context.events), ...parseEvents(context.engineEvents || [])], site, locale, q);
    if (social) return social;
    const domainEvidence = debuggerDomainDiagnosis(q, context, locale);
    if (domainEvidence) return domainEvidence;

    if (site && /(?:log|registro|diagnost|record|日志|诊断)/.test(q)) {
      const latest = scoped.at(-1);
      return latest
        ? say(locale, "siteLogFound", { site, count: scoped.length, detail: safeDetail(latest.detail || latest.raw || latest.event) })
        : say(locale, "siteLogEmpty", { site });
    }
    if (/(como (?:esta|estao).*(?:log|registro)|(?:log|registro).*(?:erro|falha|estado)|log status|errors? in (?:the )?logs?|日志.*(?:错误|状态)|(?:错误|状态).*日志)/.test(q)) {
      return failures.length
        ? say(locale, "errors", { count: failures.length, detail: safeDetail(failures.at(-1)?.detail || failures.at(-1)?.raw || failures.at(-1)?.event) })
        : say(locale, "noErrors");
    }

    if (site === "tiktok" && /(como|how|怎么|如何|visuali|preview|abr|open)/.test(q)) return say(locale, "tiktokGuide");
    if (site === "pixeldrain" && /(thread|conex|connection|falh|failed|regra|rule|线程|连接|规则)/.test(q)) return say(locale, "pixeldrainRule");
    if (/(atualiz|latest|ultima vers|最新|版本)/.test(q)) return say(locale, "installed", {
      app: context.appVersion || "—", extension: context.extensionVersion || "—",
    });
    if (/(quant|how many|状态|多少|fila|tarefas|tasks|downloads.*tem)/.test(q) && /(download|tarefa|task|fila|下载|任务)/.test(q)) {
      const tasks = context.downloads || [];
      const count = key => tasks.filter(task => stateKey(task).includes(key)).length;
      return say(locale, "taskSummary", { total: tasks.length, active: count("download"), paused: count("paus"), failed: count("fail") || count("falh"), completed: count("complete") || count("conclu") });
    }
    if (/(histor|corre|fix|修正|历史)/.test(q) && !/(nao funcion|did not work|没用|无效)/.test(q)) {
      const count = context.corrections?.length || 0;
      return say(locale, count ? "historyCount" : "historyEmpty", { count });
    }
    if (/rapidgator/.test(`${q}\n${text}`) && /(404|not found|nao iniciou|nao baix|falh|failed|不启动|失败)/.test(`${q}\n${text}`)) return say(locale, "rapidgator404");
    if (/429|too many requests|rate.?limit/.test(text)) return say(locale, "rateLimit");
    if (/403|401|forbidden|unauthorized|access denied/.test(text)) return say(locale, "accessDenied");
    if (/faixa incompleta|incomplete.*track|track.*incomplete|isolated_social_track/.test(text)) return say(locale, "incomplete");
    if (/bridge.*disconnected|extension disconnected|bridge_unavailable/.test(text)) return say(locale, "disconnected");
    if (/sponsored|patrocinado|赞助/.test(text) && /(botao|button|captur|list|按钮)/.test(q)) return say(locale, "sponsored");
    if (/canDownload=false|recording.only|visual.only|recording_only/i.test(scoped.map(event => `${event?.detail || ""} ${event?.event || ""}`).join("\n"))) return say(locale, "recordOnly");
    if (/(botao|button|按钮)/.test(q) && /(download|baix|下载)/.test(q)) return say(locale, scoped.length ? "missingButton" : "noEvidence");
    if (/(lent|devagar|slow|speed|veloc|performance|desempenho|mirror|espelho|retom|resume|慢|速度|性能|镜像|续传)/.test(q)
        && /(download|baix|transfer|motor|engine|torrent|下载|传输|引擎)/.test(q)) {
      return Array.isArray(context.engineEvents)
        ? performanceDiagnosis(context, locale)
        : say(locale, "downloadSlow");
    }
    if (/(visualizar|preview|player|播放器|预览)/.test(q)) return say(locale, "previewHelp");
    if (/(gravar|gravacao|record|capture|录制)/.test(q)) return say(locale, "recordingHelp");
    if (/(extensao|extension|扩展)/.test(q)) return say(locale, "extensionHelp");
    if (/(ffmpeg|ffprobe|yt-dlp|aria2|m3u8dl|quickjs|ferrament|tool|工具)/.test(q)) return say(locale, "toolsHelp");
    if (/(torrent|magnet|seed|peer|种子|磁力)/.test(q)) return say(locale, "torrentHelp");
    if (/(ed2k|emule|amule|kad)/.test(q)) return say(locale, "ed2kHelp");
    if (/(apocalipse link|computador remoto|remote computer|远程)/.test(q)) return say(locale, "linkHelp");
    if (/(configur|setting|proxy|vpn|dns|credencial|设置|代理)/.test(q)) return say(locale, "settingsHelp");
    if (/(pasta|diretorio|folder|directory|文件夹|目录)/.test(q)) return say(locale, "filesHelp");
    if (/(privacidade|privacy|modelo externo|external model|隐私|外部模型)/.test(q)) return say(locale, "privacyHelp");
    if (failures.length) return say(locale, "genericFailure", { detail: safeDetail(failures.at(-1)?.detail || failures.at(-1)?.raw || failures.at(-1)?.event) });
    if (/(bug|erro|error|falh|problem|问题|错误)/.test(q)) return say(locale, scoped.length ? "noErrors" : "noEvidence");
    return null;
  }

  function findCorrection(input, corrections) {
    const q = fold(input);
    return [...(corrections || [])].reverse().find(item => q.includes(fold(item.name))) || null;
  }

  function parseCorrectionTeachCommand(input) {
    const value = String(input || "").trim();
    const patterns = [
      /^(?:aprenda|aprender|guarde|guardar|salve|salvar|nova)\s+(?:esta\s+)?(?:corre[cç][aã]o|solu[cç][aã]o)(?:\s+(?:para|do|de)\s+([a-z0-9.-]+))?\s*[:\-]\s*(.{4,})$/iu,
      /^(?:learn|remember|save|new)\s+(?:this\s+)?(?:fix|correction)(?:\s+for\s+([a-z0-9.-]+))?\s*[:\-]\s*(.{4,})$/iu,
      /^(?:保存|记住|学习)(?:这个)?(?:修正|解决方案)(?:\s*([a-z0-9.-]+))?\s*[:：\-]\s*(.{2,})$/u,
    ];
    for (const pattern of patterns) {
      const match = value.match(pattern);
      if (!match) continue;
      const name = String(match[2] || "").trim().replace(/\s+/g, " ").slice(0, 160);
      if (!name) return null;
      const site = String(match[1] || siteFrom(name) || "Apocalipse").trim().toLowerCase().slice(0, 120);
      return { name, site };
    }
    return null;
  }

  function findRelevantConfirmedCorrection(input, corrections) {
    const q = normalizeQuestion(input);
    const site = siteFrom(q);
    const confirmed = [...(corrections || [])].reverse().filter(item => item?.status === "confirmed");
    if (!confirmed.length) return null;
    if (site) {
      const foldedSite = fold(site);
      const match = confirmed.find(item => {
        const savedSite = fold(item?.site || "");
        const name = fold(item?.name || "");
        return savedSite === foldedSite || savedSite.endsWith(`.${foldedSite}`) || name.includes(foldedSite);
      });
      if (match) return match;
    }
    if (!/(corre|fix|soluc|falh|erro|error|problem|修正|解决|失败|错误)/.test(q)) return null;
    return confirmed.find(item => {
      const name = fold(item?.name || "");
      return name.length >= 8 && q.includes(name);
    }) || null;
  }

  function respond(input, context = {}) {
    const locale = localeOf(context.locale);
    const q = normalizeQuestion(input);
    const corrections = context.corrections || [];
    const pending = [...corrections].reverse().find(item => ["proposed", "testing"].includes(item.status));
    if (!q) return { text: say(locale, "unknown"), intent: "unknown" };

    const taughtCorrection = parseCorrectionTeachCommand(input);
    if (taughtCorrection) {
      return {
        text: say(locale, "correctionLearned", taughtCorrection),
        intent: "correction_learned",
        learnCorrection: { ...taughtCorrection, status: "saved" },
      };
    }

    const asksUpdateDetails = /(o que mudou|o que tem de novo|novidad|corre[cç][oõ]es?.*(?:atualiz|vers)|changelog|release notes?|what changed|what(?:'s| is) new|new fixes|fixes?.*(?:update|version)|更新内容|更新了什么|新修正|修复内容)/.test(q);
    if (asksUpdateDetails) {
      const update = context.updateState || {};
      const details = String(update.notesSummary || "").trim();
      if (details) {
        return {
          text: say(locale, "updateDetails", { latest: update.latest || context.appVersion || "—", details }),
          intent: "update_details",
        };
      }
      return {
        text: say(locale, "updateChecking"),
        intent: "update_details",
        action: { type: "check_app_update", wantDetails: true },
      };
    }

    if (/(?:^|\b)(?:limpe|limpar|apague|apagar|clear|erase|delete)(?:\s+(?:essa|esta|a|the))?\s+(?:tela\s+do\s+)?(?:chat|conversa|conversation)(?:\b|$)|清除(?:聊天|对话)/.test(q)) {
      return { text: say(locale, "chatCleared"), intent: "chat_clear", action: { type: "clear_chat" } };
    }
    const conversation = localModel?.classify(q);
    if (conversation) {
      if (conversation.intent === "update_check") {
        return { text: say(locale, "updateChecking"), intent: "update_check", action: { type: "check_app_update" }, confidence: conversation.confidence };
      }
      if (conversation.intent === "current_time") {
        const now = context.now instanceof Date ? context.now : new Date();
        const formatLocale = locale === "pt-BR" ? "pt-BR" : locale === "zh-CN" ? "zh-CN" : "en-US";
        const time = new Intl.DateTimeFormat(formatLocale, { hour: "2-digit", minute: "2-digit" }).format(now);
        return { text: say(locale, "currentTime", { time }), intent: "current_time", confidence: conversation.confidence };
      }
      const responseKey = conversation.intent === "acknowledgement" ? "acknowledged"
        : conversation.intent === "help" ? "helpPrompt" : conversation.intent;
      return { text: say(locale, responseKey), intent: conversation.intent, confidence: conversation.confidence };
    }
    const credential = parseCredentialCommand(input);
    if (credential) {
      if (!credential.valid) return { text: say(locale, "credentialInvalid"), intent: "credential_invalid" };
      return {
        text: say(locale, "credentialSaved", { host: credential.host, username: credential.username }),
        intent: "credential_save",
        action: { type: "save_website_credential", ...credential },
      };
    }
    if (/^(oi|ola|bom dia|boa tarde|boa noite|hello|hi|hey|你好|早上好|下午好|晚上好)[!. ]*$/.test(q)) return { text: say(locale, "hello"), intent: "greeting" };

    if (pending && /(nao funcion|continua igual|piorou|did not work|still the same|got worse|没有用|还是一样|更糟)/.test(q)) {
      return { text: say(locale, "correctionRejected"), intent: "correction_rejected", correctionId: pending.id, status: "rejected", analyzeAgain: true };
    }
    if (/(funcionou|deu certo|worked|fixed|有效|成功|修好了)/.test(q) && pending?.status === "testing") {
      return { text: say(locale, "correctionConfirmed", { name: pending.name }), intent: "correction_confirmed", correctionId: pending.id, status: "confirmed" };
    }
    if (/^(sim|pode|aplique|yes|apply|可以|是|应用)[!. ]*$/.test(q)) {
      if (!pending) return { text: say(locale, "noPending"), intent: "correction_missing" };
      return { text: say(locale, "correctionTesting", { name: pending.name }), intent: "correction_testing", correctionId: pending.id, status: "testing" };
    }
    if (/^(aplique|aplicar|apply|应用)(?:\s|$)/.test(q)) {
      const found = findCorrection(input, corrections);
      if (!found) return { text: say(locale, "correctionNotFound"), intent: "correction_missing" };
      return { text: say(locale, "correctionTesting", { name: found.name }), intent: "correction_testing", correctionId: found.id, status: "testing" };
    }
    if (/^(nao|não|no|否|不)[!. ]*$/.test(q) && pending?.status === "proposed") {
      return { text: say(locale, "correctionSaved", { name: pending.name }), intent: "correction_saved", correctionId: pending.id, status: "saved" };
    }
    if (/^(remov|apag|delete|remove|删除)/.test(q)) {
      const found = findCorrection(input, corrections);
      if (!found) return { text: say(locale, "correctionNotFound"), intent: "correction_missing" };
      return { text: say(locale, "correctionRemoved", { name: found.name }), intent: "correction_removed", correctionId: found.id, remove: true };
    }

    const knownCorrection = findRelevantConfirmedCorrection(input, corrections);
    if (knownCorrection) {
      return {
        text: say(locale, "knownCorrection", {
          site: knownCorrection.site || siteFrom(knownCorrection.name) || "Apocalipse",
          name: knownCorrection.name,
          count: Math.max(1, Number(knownCorrection.successCount || 1)),
          version: knownCorrection.lastVerifiedVersion || context.appVersion || "—",
        }),
        intent: "known_correction",
        correctionId: knownCorrection.id,
      };
    }

    if (/(botao|button|按钮)/.test(q) && !/(video|download|baix|gravar|record|visuali|preview|视频|下载|录制|预览)/.test(q)) {
      return { text: say(locale, "clarifyButton"), intent: "clarification", confidence: 0.45 };
    }
    if (/(nao funcion|falhou|deu (?:erro|problema)|did not work|failed|problem|没有用|失败|问题)/.test(q) && !/(download|baix|visuali|preview|gravar|record|torrent|ed2k|link|下载|预览|录制)/.test(q) && !previousSubject(context.messages)) {
      return { text: say(locale, "clarifyAction"), intent: "clarification", confidence: 0.4 };
    }
    const diagnosis = diagnose(input, context, locale);
    return diagnosis
      ? { text: diagnosis, prelude: say(locale, "analyzing"), intent: "diagnosis" }
      : { text: say(locale, "offTopic"), intent: "unknown" };
  }

  return { contextualQuestion, copy, debuggerDomainDiagnosis, diagnose, findRelevantConfirmedCorrection, fold, formatRate, localeOf, normalizeQuestion, parseCorrectionTeachCommand, parseCredentialCommand, parseEvents, performanceDiagnosis, previousSubject, redactCredentialCommand, respond, say, siteFrom, socialDiagnosis };
});
