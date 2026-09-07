from pathlib import Path

# 0.3.48: ChatGPT Library only.
# Evidence from 0.3.47 diagnostics showed the same Library URL being taken over twice:
# one path from CDP/downloadWillBegin/direct interception and another from chrome.downloads.onCreated.
# Deduplicate by normalized Library URL, not by downloadId, so only one owner can open the ADM destination prompt.

p = Path('browser-extension/background.js')
s = p.read_text(encoding='utf-8')

anchor = '''const chatgptLibraryTransfers = new Set();
'''
replacement = '''const chatgptLibraryTransfers = new Set();
const chatgptLibraryUrlOwners = new Map();
const CHATGPT_LIBRARY_OWNER_TTL_MS = 15000;

const chatgptLibraryTransferKey = (value) => {
  const normalized = chatgptLibraryUrl(value || "");
  return normalized || null;
};

const claimChatgptLibraryUrl = (url, source) => {
  const key = chatgptLibraryTransferKey(url);
  if (!key) return { ok: false, key: null };
  const now = Date.now();
  const current = chatgptLibraryUrlOwners.get(key);
  if (current && now - current.startedAt < CHATGPT_LIBRARY_OWNER_TTL_MS) {
    return { ok: false, key, current };
  }
  chatgptLibraryUrlOwners.set(key, { source, startedAt: now });
  return { ok: true, key };
};

const releaseChatgptLibraryUrl = (key) => {
  if (key) chatgptLibraryUrlOwners.delete(key);
};
'''
if 'chatgptLibraryUrlOwners' not in s:
    if anchor not in s:
        raise SystemExit('chatgptLibraryTransfers anchor missing')
    s = s.replace(anchor, replacement, 1)

# streamChatGPTLibraryDownload: claim ownership by URL before opening destination prompt.
old = '''async function streamChatGPTLibraryDownload(item) {
  const url = chatgptLibraryUrl(item?.finalUrl || item?.url || "");
  if (!url) return;
  const transferKey = Number.isInteger(item?.id) ? `download:${item.id}` : `direct:${url}`;
  if (chatgptLibraryTransfers.has(transferKey)) return;
  chatgptLibraryTransfers.add(transferKey);
'''
new = '''async function streamChatGPTLibraryDownload(item, source = "unknown") {
  const url = chatgptLibraryUrl(item?.finalUrl || item?.url || "");
  if (!url) return;
  const ownership = claimChatgptLibraryUrl(url, source);
  if (!ownership.ok) {
    const duplicateState = { traceId: crypto.randomUUID(), url, pageUrl: item?.referrer || "https://chatgpt.com/", startedAt: Date.now(), bytes: 0 };
    await diagnostic("chatgpt.library.duplicate_suppressed", duplicateState, {
      detail: `source=${source} owner=${ownership.current?.source || "unknown"}`,
    });
    if (Number.isInteger(item?.id)) {
      try { chrome.downloads.cancel(item.id); } catch {}
      try { chrome.downloads.erase({ id: item.id }); } catch {}
    }
    return;
  }
  const transferKey = ownership.key;
  chatgptLibraryTransfers.add(transferKey);
'''
if old in s:
    s = s.replace(old, new, 1)
elif 'chatgpt.library.duplicate_suppressed' not in s:
    raise SystemExit('streamChatGPTLibraryDownload ownership block missing')

# Release the URL owner only when the active transfer really finishes/fails.
old = '''  } finally {
    chatgptLibraryTransfers.delete(transferKey);
  }
}
'''
new = '''  } finally {
    chatgptLibraryTransfers.delete(transferKey);
    releaseChatgptLibraryUrl(transferKey);
  }
}
'''
if old in s:
    s = s.replace(old, new, 1)
elif 'releaseChatgptLibraryUrl(transferKey)' not in s:
    raise SystemExit('ChatGPT finally block missing')

# Label each entry path so diagnostics prove which path owns/suppresses the transfer.
old = '''chrome.downloads.onCreated.addListener((item) => {
  if (!chatgptLibraryUrl(item?.finalUrl || item?.url || "")) return;
  // First instruction executed for a Library download: kill Chrome's native job.
  try { chrome.downloads.cancel(item.id); } catch {}
  void streamChatGPTLibraryDownload(item);
});
'''
new = '''chrome.downloads.onCreated.addListener((item) => {
  if (!chatgptLibraryUrl(item?.finalUrl || item?.url || "")) return;
  try { chrome.downloads.cancel(item.id); } catch {}
  try { chrome.downloads.erase({ id: item.id }); } catch {}
  void streamChatGPTLibraryDownload(item, "chrome.downloads.onCreated");
});
'''
if old in s:
    s = s.replace(old, new, 1)
elif 'chrome.downloads.onCreated\");' not in s and '"chrome.downloads.onCreated"' not in s:
    raise SystemExit('onCreated block missing')

# Direct click path.
s = s.replace(
    '''  void streamChatGPTLibraryDownload({ url, referrer: message.pageUrl || sender.tab?.url || "https://chatgpt.com/", filename: message.fileName || "" });''',
    '''  void streamChatGPTLibraryDownload({ url, referrer: message.pageUrl || sender.tab?.url || "https://chatgpt.com/", filename: message.fileName || "" }, "direct_click");''',
    1,
)

# CDP downloadWillBegin path.
s = s.replace(
    '''      void streamChatGPTLibraryDownload({
        url,
        referrer: entry.state.pageUrl || "https://chatgpt.com/",
        filename: params?.suggestedFilename || "",
      }).finally(() => void disarmChatgptLibraryDeny(source.tabId));''',
    '''      void streamChatGPTLibraryDownload({
        url,
        referrer: entry.state.pageUrl || "https://chatgpt.com/",
        filename: params?.suggestedFilename || "",
      }, "cdp.downloadWillBegin").finally(() => void disarmChatgptLibraryDeny(source.tabId));''',
    1,
)

p.write_text(s, encoding='utf-8')

# Version marker.
p = Path('browser-extension/manifest.json')
s = p.read_text(encoding='utf-8')
for old_version in ('0.3.43', '0.3.44', '0.3.45', '0.3.46', '0.3.47'):
    s = s.replace(f'"version": "{old_version}"', '"version": "0.3.48"', 1)
p.write_text(s, encoding='utf-8')
