// 0.3.130: second-generation exact social resolver used by the popup.
// It keeps the 0.3.129 resolver installed for compatibility, but uses a new
// message type so failures can be traced stage-by-stage with explicit reasons.
(() => {
  if (globalThis.ADM_SOCIAL_PLAYER_RESOLVER_V2) return;
  globalThis.ADM_DIAG?.register?.('social-player-resolution-v2.js');

  const UUID_RE = /^[a-f0-9]{8}(?:-[a-f0-9]{4}){3}-[a-f0-9]{12}$/i;
  const wait = ms => new Promise(resolve => setTimeout(resolve, ms));
  const emit = (event, detail, traceId, level = 'INFO') => {
    void globalThis.ADM_DIAG?.emit?.(event, detail, traceId, level);
  };
  const platform = () => /(^|\.)facebook\.com$/i.test(location.hostname) ? 'facebook'
    : /(^|\.)tiktok\.com$/i.test(location.hostname) ? 'tiktok' : 'other';
  const rectFor = element => { try { return element?.getBoundingClientRect?.() || null; } catch { return null; } };
  const visible = element => {
    if (!element || element.isConnected === false || element.hidden || element.getAttribute?.('aria-hidden') === 'true') return false;
    const rect = rectFor(element);
    if (!rect || rect.width < 40 || rect.height < 20) return false;
    if (rect.bottom <= 0 || rect.right <= 0 || rect.top >= innerHeight || rect.left >= innerWidth) return false;
    let style = null; try { style = getComputedStyle(element); } catch {}
    return style?.display !== 'none' && style?.visibility !== 'hidden' && Number.parseFloat(style?.opacity ?? '1') > 0.01;
  };
  const distance = (a, b) => !a || !b ? Infinity
    : Math.abs(a.left - b.left) + Math.abs(a.top - b.top) + Math.abs(a.width - b.width) + Math.abs(a.height - b.height);

  const exactPlayer = request => {
    if (!request?.rect) return { video: null, reason: 'request_rect_missing', bindingSource: 'none', visibleCount: 0 };
    const players = [...document.querySelectorAll('video')].filter(visible);
    const scored = players.map(video => ({ video, score: distance(rectFor(video), request.rect) })).sort((a, b) => a.score - b.score);
    const candidates = scored.filter(item => item.score <= 12);
    const bindingSource = request.playerBindingValidated ? 'player_id_scan' : request.playerId ? 'geometry_fallback' : 'geometry';
    if (!candidates[0]) return { video: null, reason: request.playerBindingValidated ? 'bound_player_geometry_missing' : 'no_geometry_match',
      bindingSource, visibleCount: players.length, candidateCount: 0, bestScore: scored[0]?.score ?? null };
    if (candidates[1] && candidates[1].score - candidates[0].score < 3) return { video: null,
      reason: request.playerBindingValidated ? 'bound_player_geometry_ambiguous' : 'ambiguous_geometry', bindingSource,
      visibleCount: players.length, candidateCount: candidates.length, bestScore: candidates[0].score, secondScore: candidates[1].score };
    const video = candidates[0].video;
    const durationDelta = Number.isFinite(request.duration) && request.duration > 0 && Number.isFinite(video.duration) && video.duration > 0
      ? Math.abs(video.duration - request.duration) : null;
    if (durationDelta != null && durationDelta > 0.75) return { video: null,
      reason: request.playerBindingValidated ? 'bound_player_duration_mismatch' : 'duration_mismatch', bindingSource,
      visibleCount: players.length, candidateCount: candidates.length, bestScore: candidates[0].score, durationDelta };
    return { video, reason: request.playerBindingValidated ? 'exact_player_id_scan_match' : 'exact_geometry_match', bindingSource,
      visibleCount: players.length, candidateCount: candidates.length, bestScore: candidates[0].score, secondScore: candidates[1]?.score ?? null,
      durationDelta };
  };

  const scopeFor = video => {
    const nodes = []; let hiddenPreloads = 0, visibleCompetitors = 0;
    for (let node = video, depth = 0; node && depth < 20; node = node.parentElement, depth += 1) {
      if (node === document.body || node === document.documentElement || /^(BODY|HTML)$/.test(node.tagName || '')) break;
      const videos = [...(node.querySelectorAll?.('video') || [])];
      hiddenPreloads = Math.max(hiddenPreloads, videos.filter(other => other !== video && !visible(other)).length);
      const competing = videos.filter(other => other !== video && visible(other));
      if (competing.length) { visibleCompetitors = Math.max(visibleCompetitors, competing.length); break; }
      nodes.push(node);
      if (node !== video && node.matches?.('article,[role="article"],[data-e2e="recommend-list-item-container"],[data-e2e="feed-video"],[data-e2e="browse-video"]')) break;
    }
    return { nodes, hiddenPreloads, visibleCompetitors };
  };

  const facebookInfo = value => {
    try {
      const url = new URL(value, location.href);
      if (!/(^|\.)facebook\.com$/i.test(url.hostname)) return { url: null, reason: 'non_facebook_host', canonicalClass: 'rejected' };
      if (/\/(?:photo|photos)(?:\.php|\/|$)/i.test(url.pathname)) return { url: null, reason: 'facebook_photo_route', canonicalClass: 'rejected' };
      if (url.searchParams.get('v') || url.searchParams.get('story_fbid') || url.searchParams.get('fbid'))
        return { url: url.href, reason: 'specific_query_id', canonicalClass: 'query_id' };
      const path = url.pathname.replace(/\/{2,}/g, '/');
      const media = path.match(/\/(reel|reels|videos|posts)\/([^/?#]+)/i);
      if (media?.[2] && !/^(?:hashtag|live|discover|topics?)$/i.test(media[2]))
        return { url: url.href, reason: `specific_${media[1].toLowerCase()}`, canonicalClass: media[1].toLowerCase() };
      const shared = path.match(/\/share\/([rv])\/([^/?#]+)/i);
      if (shared?.[2]) return { url: url.href, reason: 'specific_share', canonicalClass: `share_${shared[1]}` };
      if (/\/watch\/\d{5,}(?:\/|$)/i.test(path)) return { url: url.href, reason: 'specific_watch_id', canonicalClass: 'watch_id' };
      if (/\/watch(?:\/|$)/i.test(path)) return { url: null, reason: 'generic_facebook_watch_url', canonicalClass: 'rejected' };
      return { url: null, reason: 'facebook_non_specific_url', canonicalClass: 'rejected' };
    } catch { return { url: null, reason: 'invalid_facebook_url', canonicalClass: 'rejected' }; }
  };

  const facebookDom = video => {
    const scope = scopeFor(video), selector = [
      'a[href*="/reel/"]','a[href*="/reels/"]','a[href*="/videos/"]','a[href*="/posts/"]','a[href*="/watch/"]',
      'a[href*="/watch?"]','a[href*="/permalink.php"]','a[href*="/story.php"]','a[href*="/share/r/"]','a[href*="/share/v/"]',
    ].join(',');
    const page = facebookInfo(location.href);
    if (page.url && [...document.querySelectorAll('video')].filter(visible).length === 1)
      return { ...page, source: 'address_bar', scope, anchorCandidates: 0, canonicalCandidates: 1, rejectedGeneric: 0 };
    let anchorCandidates = 0, rejectedGeneric = 0;
    for (const node of scope.nodes) {
      const infos = [...(node.querySelectorAll?.(selector) || [])].map(anchor => facebookInfo(anchor.href));
      anchorCandidates += infos.length;
      rejectedGeneric += infos.filter(info => info.reason === 'generic_facebook_watch_url').length;
      const urls = [...new Set(infos.map(info => info.url).filter(Boolean))];
      if (urls.length === 1) return { ...facebookInfo(urls[0]), source: 'scoped_anchor', scope, anchorCandidates, canonicalCandidates: 1, rejectedGeneric };
      if (urls.length > 1) return { url: null, reason: 'ambiguous_facebook_anchors', canonicalClass: 'rejected', source: 'scoped_anchor', scope,
        anchorCandidates, canonicalCandidates: urls.length, rejectedGeneric };
      const path = String(node.innerHTML || '').replaceAll('\\/', '/').match(/\/(?:reel|reels|videos|posts|share\/[rv])\/[A-Za-z0-9._-]+/i)?.[0];
      const markup = path ? facebookInfo(path) : null;
      if (markup?.url) return { ...markup, source: 'scoped_markup', scope, anchorCandidates, canonicalCandidates: 1, rejectedGeneric };
    }
    return { url: null, reason: rejectedGeneric ? 'only_generic_facebook_watch_candidates' : 'facebook_specific_permalink_not_found',
      canonicalClass: 'rejected', source: 'dom_probe', scope, anchorCandidates, canonicalCandidates: 0, rejectedGeneric };
  };

  const copyLabel = node => /(?:copiar link|copy link|复制链接|複製連結|링크 복사|リンクをコピー)/i.test(node?.textContent || '');
  const facebookMenu = async video => {
    const videoRect = rectFor(video), post = video.closest?.('[role="article"],article') || video.parentElement;
    if (!videoRect || !post) return { url: null, reason: 'facebook_post_container_missing', source: 'menu_copy_link' };
    let container = post;
    for (let depth = 0; container?.parentElement && depth < 6; depth += 1) {
      const rect = rectFor(container); if (rect && rect.top <= videoRect.top - 20 && rect.right >= videoRect.right - 20) break;
      container = container.parentElement;
    }
    const buttons = [...(container?.querySelectorAll?.('button,[role="button"]') || [])];
    const labeled = buttons.filter(button => /(?:ações|acoes|opções|opcoes|actions|options|more|menu|更多|更多选项|\.\.\.|…|⋯)/i
      .test(`${button.getAttribute?.('aria-label') || ''} ${button.title || ''} ${button.textContent || ''}`));
    const menuButton = (labeled.length ? labeled : buttons).filter(button => { const r = rectFor(button); return r && r.width > 0 && r.height > 0 && r.top < videoRect.top + 100; })
      .sort((a,b) => { const ar=rectFor(a), br=rectFor(b); return Math.abs(ar.right-videoRect.right)+Math.abs(ar.bottom-videoRect.top)-Math.abs(br.right-videoRect.right)-Math.abs(br.bottom-videoRect.top); })[0];
    if (!menuButton) return { url: null, reason: 'facebook_menu_button_not_found', source: 'menu_copy_link', buttonCandidates: buttons.length, labeledButtons: labeled.length };
    const before = new Set([...document.querySelectorAll('[role="menuitem"],[role="menuitemradio"]')].filter(visible));
    menuButton.click(); let copy = null, menuItemsSeen = 0;
    for (let i=0;i<20 && !copy;i+=1) { await wait(100); const items=[...document.querySelectorAll('[role="menuitem"],[role="menuitemradio"]')].filter(visible);
      menuItemsSeen=Math.max(menuItemsSeen,items.length); copy=items.find(item=>!before.has(item)&&copyLabel(item))||items.find(copyLabel); }
    if (!copy) return { url:null, reason:'facebook_copy_link_item_not_found', source:'menu_copy_link', buttonCandidates:buttons.length,labeledButtons:labeled.length,menuItemsSeen };
    let previous=''; try { previous=await navigator.clipboard.readText(); } catch {}
    copy.click(); let lastReason='facebook_clipboard_unchanged';
    for (let i=0;i<20;i+=1) { await wait(100); try { const copied=await navigator.clipboard.readText(); if(!copied||copied===previous) continue;
      const info=facebookInfo(copied); if(info.url) return {...info,source:'menu_copy_link',buttonCandidates:buttons.length,labeledButtons:labeled.length,menuItemsSeen,clipboardChanged:true};
      lastReason=info.reason; } catch { lastReason='facebook_clipboard_read_failed'; } }
    return { url:null,reason:lastReason,source:'menu_copy_link',buttonCandidates:buttons.length,labeledButtons:labeled.length,menuItemsSeen,clipboardChanged:false };
  };

  const canonicalTikTok = value => { try { const url=new URL(String(value||'').replaceAll('\\/','/'),location.href);
    const match=url.pathname.match(/^\/@([A-Za-z0-9._-]+)\/video\/(\d+)\/?$/); return /(^|\.)tiktok\.com$/i.test(url.hostname)&&match ? `https://www.tiktok.com/@${match[1]}/video/${match[2]}`:null; } catch { return null; } };
  const tiktokDom = video => {
    let resolverReason='resolver_unavailable'; try { const direct=globalThis.ApocalipseTikTokIdentity?.resolve?.(video); resolverReason=globalThis.ApocalipseTikTokIdentity?.diagnosticState?.(video)?.reason||'resolver_unresolved';
      const url=canonicalTikTok(direct); if(url) return {url,reason:'tiktok_identity_resolver',source:'identity_resolver',resolverReason,scope:scopeFor(video),anchorCandidates:0,explicitIds:0}; } catch { resolverReason='resolver_exception'; }
    const scope=scopeFor(video), anchors=[], ids=new Set();
    for(const node of scope.nodes){ for(const name of ['data-video-id','data-item-id','data-aweme-id']){const id=node.getAttribute?.(name);if(id&&/^\d+$/.test(id))ids.add(id);}
      for(const anchor of node.querySelectorAll?.('a[href*="/video/"]')||[]) anchors.push(anchor.href); }
    const urls=[...new Set(anchors.map(canonicalTikTok).filter(Boolean))];
    if(ids.size===1){const id=[...ids][0], match=urls.find(url=>url.endsWith(`/video/${id}`)); if(match)return{url:match,reason:'tiktok_explicit_id_match',source:'scoped_anchor',resolverReason,scope,anchorCandidates:anchors.length,explicitIds:1};}
    if(ids.size>1)return{url:null,reason:'tiktok_multiple_explicit_ids',source:'dom_probe',resolverReason,scope,anchorCandidates:anchors.length,explicitIds:ids.size};
    if(urls.length===1)return{url:urls[0],reason:'tiktok_scoped_anchor',source:'scoped_anchor',resolverReason,scope,anchorCandidates:anchors.length,explicitIds:ids.size};
    return{url:null,reason:urls.length>1?'tiktok_ambiguous_scoped_permalinks':'tiktok_no_permalink_in_card',source:'dom_probe',resolverReason,scope,anchorCandidates:anchors.length,explicitIds:ids.size};
  };

  const tiktokMenu = async video => {
    const scope=scopeFor(video), buttons=[];
    for(const node of scope.nodes) for(const button of node.querySelectorAll?.('button,[role="button"],[data-e2e*="share"]')||[]) if(!buttons.includes(button)&&visible(button))buttons.push(button);
    const labeled=buttons.filter(button=>/(?:share|compartilhar|compartilhe|分享|共享|공유|シェア)/i.test(`${button.getAttribute?.('data-e2e')||''} ${button.getAttribute?.('aria-label')||''} ${button.title||''} ${button.textContent||''}`));
    const share=labeled[0]; if(!share)return{url:null,reason:'tiktok_share_button_not_found',source:'menu_copy_link',scope,buttonCandidates:buttons.length,labeledButtons:labeled.length};
    const selector='[data-e2e*="copy-link"],[data-e2e*="copylink"],[data-e2e*="copy"],button,[role="button"],[role="menuitem"]';
    const before=new Set([...document.querySelectorAll(selector)].filter(visible)); share.click(); let copy=null,copyCandidatesSeen=0;
    for(let i=0;i<20&&!copy;i+=1){await wait(100);const fresh=[...document.querySelectorAll(selector)].filter(item=>visible(item)&&!before.has(item));copyCandidatesSeen=Math.max(copyCandidatesSeen,fresh.length);
      copy=fresh.find(item=>/(?:copy.?link|copiar link|复制链接|複製連結|링크 복사|リンクをコピー)/i.test(`${item.getAttribute?.('data-e2e')||''} ${item.getAttribute?.('aria-label')||''} ${item.title||''} ${item.textContent||''}`));}
    if(!copy)return{url:null,reason:'tiktok_copy_link_item_not_found',source:'menu_copy_link',scope,buttonCandidates:buttons.length,labeledButtons:labeled.length,copyCandidatesSeen};
    for(const name of ['data-clipboard-text','data-url','href']){const direct=canonicalTikTok(copy.getAttribute?.(name)||copy[name]);if(direct)return{url:direct,reason:'tiktok_copy_link_attribute',source:'menu_copy_link',scope,buttonCandidates:buttons.length,labeledButtons:labeled.length,copyCandidatesSeen};}
    let previous='';try{previous=await navigator.clipboard.readText();}catch{} copy.click(); let lastReason='tiktok_clipboard_unchanged';
    for(let i=0;i<20;i+=1){await wait(100);try{const copied=await navigator.clipboard.readText();if(!copied||copied===previous)continue;const url=canonicalTikTok(copied);if(url)return{url,reason:'tiktok_copy_link_clipboard',source:'menu_copy_link',scope,buttonCandidates:buttons.length,labeledButtons:labeled.length,copyCandidatesSeen,clipboardChanged:true};lastReason='tiktok_copied_url_not_canonical';}catch{lastReason='tiktok_clipboard_read_failed';}}
    return{url:null,reason:lastReason,source:'menu_copy_link',scope,buttonCandidates:buttons.length,labeledButtons:labeled.length,copyCandidatesSeen,clipboardChanged:false};
  };

  const stableDuring = async (video, operation) => {
    const page=location.href, source=String(video.currentSrc||video.src||''), rect=rectFor(video); let changed=false; const mark=()=>{changed=true;};
    for(const event of ['emptied','loadstart','loadedmetadata'])video.addEventListener?.(event,mark,true);
    try{const value=await operation(), after=rectFor(video), rectDelta=distance(rect,after);const reason=changed?'player_lifecycle_changed':!video.isConnected?'player_disconnected':location.href!==page?'page_changed_during_lookup':String(video.currentSrc||video.src||'')!==source?'player_source_changed':rectDelta>12?'player_geometry_changed':!visible(video)?'player_became_invisible':'player_stable';
      return{value,stable:reason==='player_stable',reason,rectDelta};}finally{for(const event of ['emptied','loadstart','loadedmetadata'])video.removeEventListener?.(event,mark,true);}
  };

  const resolve = async (request, traceId) => {
    const p=platform(), match=exactPlayer(request); emit('player.resolve_stage',{platform:p,stage:'player_match',result:match.video?'matched':'rejected',reason:match.reason,bindingSource:match.bindingSource,visibleCount:match.visibleCount,candidateCount:match.candidateCount||0,bestScore:match.bestScore??null,secondScore:match.secondScore??null,durationDelta:match.durationDelta??null},traceId,match.video?'INFO':'WARN');
    if(!match.video)return{item:null,failureStage:'player_match',reason:match.reason,platform:p}; const video=match.video;
    const stable=await stableDuring(video,async()=>{
      let dom=p==='facebook'?facebookDom(video):tiktokDom(video); const scope=dom.scope||{};
      emit('player.resolve_stage',{platform:p,stage:'dom_permalink',result:dom.url?'resolved':'unresolved',reason:dom.reason,permalinkSource:dom.source,resolverReason:dom.resolverReason||'none',scopeNodes:scope.nodes?.length||0,hiddenPreloads:scope.hiddenPreloads||0,visibleCompetitors:scope.visibleCompetitors||0,anchorCandidates:dom.anchorCandidates||0,canonicalCandidates:dom.canonicalCandidates||0,rejectedGeneric:dom.rejectedGeneric||0,explicitIds:dom.explicitIds||0,url:dom.url||''},traceId,dom.url?'INFO':'WARN');
      if(dom.url)return dom; const menu=p==='facebook'?await facebookMenu(video):await tiktokMenu(video); const ms=menu.scope||{};
      emit('player.resolve_stage',{platform:p,stage:'menu_copy_link',result:menu.url?'resolved':'unresolved',reason:menu.reason,permalinkSource:menu.source,scopeNodes:ms.nodes?.length||0,buttonCandidates:menu.buttonCandidates||0,labeledButtons:menu.labeledButtons||0,menuItemsSeen:menu.menuItemsSeen||0,copyCandidatesSeen:menu.copyCandidatesSeen||0,clipboardChanged:Boolean(menu.clipboardChanged),url:menu.url||''},traceId,menu.url?'INFO':'WARN'); return menu;});
    emit('player.resolve_stage',{platform:p,stage:'stability_check',result:stable.stable?'stable':'rejected',reason:stable.reason,rectDelta:Number.isFinite(stable.rectDelta)?stable.rectDelta:null},traceId,stable.stable?'INFO':'WARN');
    const value=stable.value; if(!value?.url)return{item:null,failureStage:value?.source==='menu_copy_link'?'menu_copy_link':'dom_permalink',reason:value?.reason||'media_identity_unresolved',platform:p};
    if(!stable.stable)return{item:null,failureStage:'stability_check',reason:stable.reason,platform:p};
    return{item:{url:value.url,extractorUrl:value.url,kind:'video',pageExtractor:true,visualOnly:false,recommended:true,ambiguousSocialTrack:false,title:document.title||'Video',thumbnail:request.thumbnail||'',duration:Number.isFinite(video.duration)&&video.duration>0?video.duration:request.duration||null,size:null,ext:'mp4'},failureStage:'none',reason:value.reason,platform:p};
  };

  chrome.runtime.onMessage.addListener((message,_sender,reply)=>{
    if(message?.type!=='APOCALIPSE_RESOLVE_VISIBLE_SOCIAL_MEDIA_V2')return;
    const traceId=UUID_RE.test(message.traceId||'')?message.traceId:globalThis.ADM_DIAG?.begin?.('player.resolve_requested',{platform:platform(),stage:'request_received'})||null;
    if(UUID_RE.test(message.traceId||''))emit('player.resolve_requested',{platform:platform(),stage:'request_received',hasPlayerId:Boolean(message.request?.playerId),bindingSource:message.request?.playerBindingValidated?'player_id_scan':'none'},traceId);
    resolve(message.request||{},traceId).then(outcome=>{emit('player.resolve_summary',{platform:outcome.platform,result:outcome.item?'resolved':'unresolved',failureStage:outcome.failureStage,reason:outcome.reason,url:outcome.item?.url||''},traceId,outcome.item?'INFO':'WARN');
      reply(outcome.item?{ok:true,item:outcome.item,platform:outcome.platform,traceId}:{ok:false,error:'media_identity_unresolved',failureStage:outcome.failureStage,reason:outcome.reason,platform:outcome.platform,traceId});
    }).catch(error=>{emit('player.resolve_summary',{platform:platform(),result:'unresolved',failureStage:'resolver_exception',reason:'resolver_exception',errorRef:String(error)},traceId,'ERROR');reply({ok:false,error:'media_identity_unresolved',failureStage:'resolver_exception',reason:'resolver_exception',platform:platform(),traceId});}); return true;
  });

  globalThis.ADM_SOCIAL_PLAYER_RESOLVER_V2={
    facebookInfo, facebookCanonicalInfo: facebookInfo,
    tiktokDom, tiktokFromVisibleCardDetailed: tiktokDom,
    tiktokMenu, tiktokFromShareMenuDetailed: tiktokMenu,
    exactPlayer, findExactVisibleVideoDetailed: exactPlayer, resolve,
  };
})();
