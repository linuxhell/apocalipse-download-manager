console.log('[Apocalipse] Extensao carregada.');

const CONFIG = {
    rapidgator: {
        domains: ['rapidgator.net', 'rapidgator.com'],
        maxRetries: 1,
    },
    apocalipse: {
        url: 'http://localhost:8080',
        endpoints: {
            bridge: '/v1/bridge-download',
            status: '/v1/download-status',
        }
    }
};

chrome.downloads.onCreated.addListener(async (downloadItem) => {
    console.log('[Apocalipse] Download detectado:', downloadItem.url);
    
    const isRapidgator = CONFIG.rapidgator.domains.some(domain => 
        downloadItem.url.includes(domain) || 
        (downloadItem.referrer && downloadItem.referrer.includes(domain))
    );
    
    if (!isRapidgator) {
        console.log('[Apocalipse] Nao e Rapidgator, ignorando.');
        return;
    }
    
    console.log('[Apocalipse] Rapidgator detectado! Iniciando interceptacao...');
    await handleRapidgator(downloadItem);
});

async function handleRapidgator(downloadItem) {
    try {
        console.log('[Apocalipse] Cancelando download no Chrome...');
        await chrome.downloads.cancel(downloadItem.id);
        await chrome.downloads.erase({ id: downloadItem.id });
        console.log('[Apocalipse] Download cancelado no Chrome.');
        
        const pageUrl = downloadItem.referrer || downloadItem.url;
        const fileId = extractFileId(pageUrl);
        
        if (!fileId) {
            console.error('[Apocalipse] Nao foi possivel extrair o file ID:', pageUrl);
            await fallbackToChrome(downloadItem);
            return;
        }
        
        console.log('[Apocalipse] File ID extraido:', fileId);
        
        const correctUrl = `https://rapidgator.net/file/${fileId}`;
        console.log('[Apocalipse] URL corrigida:', correctUrl);
        
        const cookies = await getCookies('https://rapidgator.net');
        console.log('[Apocalipse] Cookies obtidos:', cookies.length);
        
        const userAgent = navigator.userAgent;
        
        console.log('[Apocalipse] Enviando para o Apocalipse...');
        const taskId = await sendToApocalipse({
            url: correctUrl,
            filename: downloadItem.filename,
            referrer: pageUrl,
            cookies: cookies,
            userAgent: userAgent,
            method: 'GET',
            rule: 'rapidgator'
        });
        
        console.log('[Apocalipse] Tarefa criada com ID:', taskId);
        await monitorTask(taskId, downloadItem);
        
    } catch (error) {
        console.error('[Apocalipse] Erro no handleRapidgator:', error);
        await fallbackToChrome(downloadItem);
    }
}

function extractFileId(url) {
    const match = url.match(/rapidgator\.net\/file\/([a-f0-9]+)/);
    return match ? match[1] : null;
}

async function getCookies(domain) {
    return new Promise((resolve) => {
        chrome.cookies.getAll({ domain: domain }, (cookies) => {
            resolve(cookies.map(cookie => ({
                name: cookie.name,
                value: cookie.value,
                domain: cookie.domain,
                path: cookie.path,
                secure: cookie.secure,
                httpOnly: cookie.httpOnly,
                sameSite: cookie.sameSite,
                expirationDate: cookie.expirationDate,
            })));
        });
    });
}

async function sendToApocalipse(data) {
    const response = await fetch(`${CONFIG.apocalipse.url}${CONFIG.apocalipse.endpoints.bridge}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(data),
    });
    
    if (!response.ok) {
        throw new Error(`Erro ao enviar: ${response.status} - ${await response.text()}`);
    }
    
    const result = await response.json();
    return result.taskId || result;
}

async function monitorTask(taskId, downloadItem) {
    console.log('[Apocalipse] Monitorando tarefa:', taskId);
    
    let attempts = 0;
    const maxAttempts = 20;
    
    return new Promise((resolve) => {
        const interval = setInterval(async () => {
            try {
                const status = await getTaskStatus(taskId);
                console.log('[Apocalipse] Status:', status);
                
                if (status.completed) {
                    console.log('[Apocalipse] Download concluido com sucesso!');
                    clearInterval(interval);
                    resolve(true);
                    return;
                }
                
                if (status.failed || status.error) {
                    console.log('[Apocalipse] Download falhou.');
                    clearInterval(interval);
                    await fallbackToChrome(downloadItem);
                    resolve(false);
                    return;
                }
                
                attempts++;
                if (attempts >= maxAttempts) {
                    console.log('[Apocalipse] Timeout do monitoramento.');
                    clearInterval(interval);
                    await fallbackToChrome(downloadItem);
                    resolve(false);
                }
            } catch (error) {
                console.error('[Apocalipse] Erro no monitoramento:', error);
                attempts++;
                if (attempts >= maxAttempts) {
                    clearInterval(interval);
                    await fallbackToChrome(downloadItem);
                    resolve(false);
                }
            }
        }, 3000);
    });
}

async function getTaskStatus(taskId) {
    const response = await fetch(`${CONFIG.apocalipse.url}${CONFIG.apocalipse.endpoints.status}/${taskId}`);
    if (!response.ok) {
        throw new Error(`Erro ao verificar status: ${response.status}`);
    }
    return await response.json();
}

async function fallbackToChrome(downloadItem) {
    console.log('[Apocalipse] Ativando fallback para o Chrome...');
    
    try {
        const key = `fallback_${downloadItem.url}`;
        const data = await chrome.storage.session.get(key);
        
        if (data[key]) {
            console.log('[Apocalipse] Fallback ja foi tentado para este download.');
            return;
        }
        
        await chrome.storage.session.set({ [key]: true });
        
        await chrome.downloads.download({
            url: downloadItem.referrer || downloadItem.url,
            filename: downloadItem.filename,
            conflictAction: 'uniquify',
        });
        
        console.log('[Apocalipse] Fallback acionado.');
    } catch (error) {
        console.error('[Apocalipse] Erro no fallback:', error);
    }
}

setInterval(async () => {
    const items = await chrome.storage.session.get(null);
    for (const [key, value] of Object.entries(items)) {
        if (key.startsWith('fallback_') && typeof value === 'boolean') {
            await chrome.storage.session.remove(key);
        }
    }
}, 3600000);

console.log('[Apocalipse] Extensao pronta para interceptar Rapidgator!');