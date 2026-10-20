//! Bounded Kad2 routing and direct-source lookup, independently implemented from the wire format.
//! Uses plain Kad2 UDP with v5 HELLO and legacy endpoint challenges; no obfuscation or firewall callbacks.
use nibiru_xtreme::ed2k::{metadata::Node, FileSource};
use serde::Serialize;
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    io,
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
use tokio::{
    net::UdpSocket,
    sync::{mpsc, oneshot, watch},
};

const MAX_CONTACTS: usize = 512;
const MAX_PENDING: usize = 32;
const QUERY_TIMEOUT: Duration = Duration::from_secs(3);
const VERIFIED_TTL: Duration = Duration::from_secs(20 * 60);
const MAX_HELLO_BYTES: usize = 512;
const MAX_HELLO_TAGS: u8 = 16;
const MAX_PAYLOAD_BYTES: usize = 32768;
const MAX_SOURCE_PACKETS: usize = 6;
const MAX_SOURCE_ROWS: usize = 300;
const MAX_SOURCE_BYTES: usize = MAX_PAYLOAD_BYTES * MAX_SOURCE_PACKETS;
// Advertising v6+ makes modern peers send obfuscated HELLO replies; v8 also adds keyed ACKs.
const KAD_VERSION: u8 = 5;
type Id = [u8; 16];

struct Contact {
    node: Node,
    verified: Option<Instant>,
    failed: u8,
    attempted: Option<Instant>,
}
impl Contact {
    fn is_verified(&self) -> bool {
        self.failed < 3 && self.verified.is_some_and(|t| t.elapsed() < VERIFIED_TTL)
    }
}
struct Hello {
    node: Node,
    udp_firewalled: bool,
    requests_ack: bool,
}
struct Pending {
    serial: u64,
    peer_id: Id,
    opcode: u8,
    target: Id,
    sender: ReplySender,
}
enum ReplySender {
    Single(oneshot::Sender<Vec<u8>>),
    Sources {
        sender: mpsc::Sender<Vec<u8>>,
        packets: usize,
        rows: usize,
        bytes: usize,
    },
}
struct PendingGuard {
    inner: Arc<Inner>,
    peer: SocketAddr,
    serial: u64,
}
impl Drop for PendingGuard {
    fn drop(&mut self) {
        if let Ok(mut pending) = self.inner.pending.lock() {
            if pending
                .get(&self.peer)
                .is_some_and(|p| p.serial == self.serial)
            {
                pending.remove(&self.peer);
            }
        }
    }
}
struct Inner {
    identity: Id,
    tcp_port: u16,
    socket: Arc<UdpSocket>,
    contacts: Mutex<HashMap<Id, Contact>>,
    pending: Mutex<HashMap<SocketAddr, Pending>>,
    path: PathBuf,
    allow_local: bool,
    running: AtomicBool,
    serial: AtomicU64,
    lookup_lock: tokio::sync::Mutex<()>,
}
pub struct KadSession {
    inner: Arc<Inner>,
    shutdown: watch::Sender<bool>,
}

#[derive(Serialize)]
pub struct Status {
    pub running: bool,
    pub tcp_port: u16,
    pub udp_port: u16,
    pub kad_version: u8,
    pub hello: bool,
    pub contacts: usize,
    pub verified_contacts: usize,
    pub routing: bool,
    pub direct_source_lookup: bool,
    pub keyword_search: bool,
    pub publishing: bool,
    pub obfuscation: bool,
    pub buddy_callbacks: bool,
    pub public_reachability_verified: bool,
}

fn address(node: &Node) -> SocketAddr {
    (node.host, node.udp_port).into()
}
fn retryable_udp_error(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::ConnectionReset
            | io::ErrorKind::ConnectionRefused
            | io::ErrorKind::TimedOut
            | io::ErrorKind::Interrupted
            | io::ErrorKind::WouldBlock
    ) || (cfg!(windows) && error.raw_os_error() == Some(10040)) // WSAEMSGSIZE, defensively.
}
async fn udp_retry_delay(shutdown: &mut watch::Receiver<bool>) -> bool {
    tokio::select! {
        _ = shutdown.changed() => false,
        _ = tokio::time::sleep(Duration::from_millis(100)) => true,
    }
}
fn valid_ip(ip: Ipv4Addr, local: bool) -> bool {
    !ip.is_unspecified()
        && !ip.is_multicast()
        && !ip.is_broadcast()
        && (local
            || (!ip.is_private()
                && !ip.is_loopback()
                && !ip.is_link_local()
                && ip.octets()[0] != 0
                && ip.octets()[0] < 224))
}
fn distance(a: Id, b: Id) -> Id {
    let mut out = [0; 16];
    // Kad CUInt128 uses four little-endian u32 words on the wire, most significant word first.
    for i in 0..4 {
        let start = i * 4;
        let x = u32::from_le_bytes(a[start..start + 4].try_into().unwrap())
            ^ u32::from_le_bytes(b[start..start + 4].try_into().unwrap());
        out[start..start + 4].copy_from_slice(&x.to_be_bytes());
    }
    out
}
fn write_node(out: &mut Vec<u8>, node: &Node) {
    out.extend(node.id);
    out.extend(u32::from_be_bytes(node.host.octets()).to_le_bytes());
    out.extend(node.udp_port.to_le_bytes());
    out.extend(node.tcp_port.to_le_bytes());
    out.push(node.version);
}
fn kad_payload(protocol: u8, bytes: &[u8]) -> Result<Cow<'_, [u8]>, String> {
    if bytes.len() > MAX_PAYLOAD_BYTES {
        return Err("kad_payload_limit".into());
    }
    match protocol {
        0xe4 => Ok(Cow::Borrowed(bytes)),
        0xe5 => {
            // Kad compression changes only the protocol byte: opcode remains outside zlib.
            // A fixed output capacity and one inflate call bound expansion and CPU work.
            let mut output = Vec::with_capacity(MAX_PAYLOAD_BYTES);
            let mut decoder = flate2::Decompress::new(true);
            let status = decoder
                .decompress_vec(bytes, &mut output, flate2::FlushDecompress::Finish)
                .map_err(|_| "kad_invalid_zlib")?;
            if status != flate2::Status::StreamEnd
                || decoder.total_in() != bytes.len() as u64
                || output.len() > MAX_PAYLOAD_BYTES
            {
                return Err("kad_zlib_truncated_trailing_or_oversized".into());
            }
            Ok(Cow::Owned(output))
        }
        _ => Err("kad_unsupported_protocol".into()),
    }
}
fn hello_packet(opcode: u8, identity: Id, tcp_port: u16) -> Vec<u8> {
    let mut packet = vec![0xe4, opcode];
    packet.extend(identity);
    packet.extend(tcp_port.to_le_bytes());
    packet.extend([KAD_VERSION, 0]); // No v8 ACK/firewall claims; the observed UDP port is authoritative.
    packet
}
fn hello_response(bytes: &[u8], peer: SocketAddr, local: bool) -> Result<Hello, String> {
    let SocketAddr::V4(peer) = peer else {
        return Err("kad_invalid_hello_endpoint".into());
    };
    if bytes.len() > MAX_HELLO_BYTES || !valid_ip(*peer.ip(), local) || peer.port() == 0 {
        return Err("kad_invalid_hello_endpoint_or_size".into());
    }
    let mut reader = Reader(bytes);
    let id: Id = reader.take(16)?.try_into().unwrap();
    let tcp_port = reader.u16()?;
    let version = reader.byte()?;
    let tags = reader.byte()?;
    if id == [0; 16] || tcp_port == 0 || version < 2 {
        return Err("kad_invalid_hello_identity".into());
    }
    if tags > MAX_HELLO_TAGS {
        return Err("kad_hello_tag_limit".into());
    }
    let (mut source_port, mut options) = (None, None);
    for _ in 0..tags {
        let (name, value) = reader.tag()?;
        match name {
            0xfc => {
                let port = value
                    .and_then(|v| u16::try_from(v).ok())
                    .filter(|p| *p != 0)
                    .ok_or("kad_invalid_hello_udp_port")?;
                if source_port.replace(port).is_some() {
                    return Err("kad_duplicate_hello_tag".into());
                }
            }
            0xf2 => {
                let value = value
                    .and_then(|v| u8::try_from(v).ok())
                    .ok_or("kad_invalid_hello_options")?;
                if options.replace(value).is_some() {
                    return Err("kad_duplicate_hello_tag".into());
                }
            }
            _ => {}
        }
    }
    // FC is a host-order integer UDP port. An unproven HELLO must not redirect our replies
    // or change the endpoint to which an outstanding query/challenge is bound.
    if source_port.is_some_and(|port| port != peer.port()) || !reader.0.is_empty() {
        return Err("kad_hello_endpoint_or_length_mismatch".into());
    }
    Ok(Hello {
        node: Node {
            id,
            host: *peer.ip(),
            udp_port: peer.port(),
            tcp_port,
            version,
        },
        udp_firewalled: options.unwrap_or(0) & 1 != 0,
        requests_ack: version >= 8 && options.unwrap_or(0) & 4 != 0,
    })
}
fn routing_response(bytes: &[u8], target: Id, local: bool) -> Result<Vec<Node>, String> {
    if bytes.len() < 17
        || bytes[..16] != target
        || bytes[16] > 32
        || bytes.len() != 17 + usize::from(bytes[16]) * 25
    {
        return Err("kad_invalid_routing_response".into());
    }
    let mut nodes = Vec::new();
    for row in bytes[17..].chunks_exact(25) {
        let node = Node {
            id: row[..16].try_into().unwrap(),
            host: Ipv4Addr::from(u32::from_le_bytes(row[16..20].try_into().unwrap()).to_be_bytes()),
            udp_port: u16::from_le_bytes(row[20..22].try_into().unwrap()),
            tcp_port: u16::from_le_bytes(row[22..24].try_into().unwrap()),
            version: row[24],
        };
        if node.id != [0; 16]
            && node.udp_port > 0
            && node.version >= 2
            && valid_ip(node.host, local)
        {
            nodes.push(node);
        }
    }
    Ok(nodes)
}

impl Inner {
    fn add(&self, node: Node) {
        if node.id == [0; 16]
            || node.id == self.identity
            || node.udp_port == 0
            || node.version < 2
            || !valid_ip(node.host, self.allow_local)
        {
            return;
        }
        let Ok(mut table) = self.contacts.lock() else {
            return;
        };
        if table.contains_key(&node.id) {
            return;
        }
        if table.values().any(|c| address(&c.node) == address(&node)) {
            return;
        }
        if !self.allow_local
            && table
                .values()
                .filter(|c| c.node.host.octets()[..3] == node.host.octets()[..3])
                .count()
                >= 3
        {
            return;
        }
        if table.len() >= MAX_CONTACTS {
            let discard = table
                .iter()
                .filter(|(_, c)| !c.is_verified())
                .max_by_key(|(_, c)| distance(self.identity, c.node.id))
                .map(|(id, _)| *id);
            if let Some(id) = discard {
                table.remove(&id);
            } else {
                return;
            }
        }
        table.insert(
            node.id,
            Contact {
                node,
                verified: None,
                failed: 0,
                attempted: None,
            },
        );
    }
    fn nearest(&self, target: Id, verified: bool, limit: usize) -> Vec<Node> {
        let Ok(table) = self.contacts.lock() else {
            return Vec::new();
        };
        let mut nodes = table
            .values()
            .filter(|c| c.failed < 3 && (!verified || c.is_verified()))
            .map(|c| c.node.clone())
            .collect::<Vec<_>>();
        nodes.sort_by_key(|n| distance(n.id, target));
        nodes.truncate(limit);
        nodes
    }
    fn save(&self) -> Result<(), String> {
        use std::io::Write;
        let nodes = self.nearest(self.identity, true, MAX_CONTACTS);
        let parent = self.path.parent().ok_or("kad_invalid_store_path")?;
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        let mut file = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
        serde_json::to_writer(&mut file, &nodes).map_err(|e| e.to_string())?;
        file.flush()
            .and_then(|_| file.as_file().sync_all())
            .map_err(|e| e.to_string())?;
        file.persist(&self.path).map_err(|e| e.to_string())?;
        Ok(())
    }
    fn failed(&self, node: &Node) {
        if let Ok(mut table) = self.contacts.lock() {
            if let Some(c) = table
                .get_mut(&node.id)
                .filter(|c| address(&c.node) == address(node))
            {
                c.failed = c.failed.saturating_add(1);
            }
        }
    }
    fn register(
        self: &Arc<Self>,
        node: &Node,
        target: Id,
        opcode: u8,
        sender: ReplySender,
    ) -> Result<PendingGuard, String> {
        if !self.running.load(Ordering::Acquire) {
            return Err("kad_stopped".into());
        }
        let peer = address(node);
        let serial = self.serial.fetch_add(1, Ordering::Relaxed);
        {
            let mut pending = self.pending.lock().map_err(|e| e.to_string())?;
            if pending.len() >= MAX_PENDING || pending.contains_key(&peer) {
                return Err("kad_query_busy".into());
            }
            pending.insert(
                peer,
                Pending {
                    serial,
                    peer_id: node.id,
                    opcode,
                    target,
                    sender,
                },
            );
        }
        if let Ok(mut table) = self.contacts.lock() {
            if let Some(c) = table.get_mut(&node.id).filter(|c| address(&c.node) == peer) {
                c.attempted = Some(Instant::now());
            }
        }
        Ok(PendingGuard {
            inner: self.clone(),
            peer,
            serial,
        })
    }
    async fn query(
        self: &Arc<Self>,
        node: &Node,
        target: Id,
        opcode: u8,
        payload: Vec<u8>,
    ) -> Result<Vec<u8>, String> {
        let (tx, rx) = oneshot::channel();
        let _cleanup = self.register(node, target, opcode, ReplySender::Single(tx))?;
        self.socket
            .send_to(&payload, address(node))
            .await
            .map_err(|e| e.to_string())?;
        match tokio::time::timeout(QUERY_TIMEOUT, rx).await {
            // Ordinary lookups and HELLO replies have predictable targets. Only the separate
            // random challenge below can establish or renew endpoint verification.
            Ok(Ok(bytes)) => Ok(bytes),
            _ => {
                self.failed(node);
                Err("kad_query_timeout".into())
            }
        }
    }
    async fn query_sources(
        self: &Arc<Self>,
        node: &Node,
        target: Id,
        payload: Vec<u8>,
    ) -> Result<Vec<FileSource>, String> {
        let (tx, mut rx) = mpsc::channel(MAX_SOURCE_PACKETS);
        let _cleanup = self.register(
            node,
            target,
            0x3b,
            ReplySender::Sources {
                sender: tx,
                packets: 0,
                rows: 0,
                bytes: 0,
            },
        )?;
        self.socket
            .send_to(&payload, address(node))
            .await
            .map_err(|e| e.to_string())?;
        let deadline = tokio::time::Instant::now() + QUERY_TIMEOUT;
        let mut received = false;
        let mut sources = Vec::new();
        while let Ok(Some(packet)) = tokio::time::timeout_at(deadline, rx.recv()).await {
            received = true;
            sources.extend(source_response(&packet, node.id, target, self.allow_local)?);
        }
        if !received {
            self.failed(node);
            return Err("kad_query_timeout".into());
        }
        Ok(sources)
    }
    async fn ensure_verified(self: &Arc<Self>, node: &Node) -> Result<Node, String> {
        {
            let table = self.contacts.lock().map_err(|e| e.to_string())?;
            if let Some(c) = table
                .get(&node.id)
                .filter(|c| address(&c.node) == address(node))
            {
                if c.is_verified() {
                    return Ok(c.node.clone());
                }
            }
        }
        let reply = self
            .query(
                node,
                node.id,
                0x19,
                hello_packet(0x11, self.identity, self.tcp_port),
            )
            .await?;
        let hello = hello_response(&reply, address(node), self.allow_local)?;
        if hello.udp_firewalled || hello.requests_ack {
            self.failed(node);
            return Err("kad_hello_firewalled_or_keyed_ack_unsupported".into());
        }
        // Kad's pre-obfuscation challenge: the peer must receive an unpredictable target
        // at this exact endpoint and accept its own ID in KADEMLIA2_REQ before echoing it.
        let nonce = *uuid::Uuid::new_v4().as_bytes();
        let mut request = vec![0xe4, 0x21, 2];
        request.extend(nonce);
        request.extend(node.id);
        self.query(node, nonce, 0x29, request).await?;
        if !self.running.load(Ordering::Acquire) {
            return Err("kad_stopped".into());
        }
        self.add(hello.node.clone());
        let mut table = self.contacts.lock().map_err(|e| e.to_string())?;
        let contact = table
            .get_mut(&node.id)
            .filter(|c| address(&c.node) == address(node))
            .ok_or("kad_contact_not_admitted")?;
        contact.node = hello.node.clone();
        contact.verified = Some(Instant::now());
        contact.failed = 0;
        contact.attempted = Some(Instant::now());
        Ok(hello.node)
    }
    async fn find(self: &Arc<Self>, node: &Node, target: Id) -> Result<Vec<Node>, String> {
        let node = self.ensure_verified(node).await?;
        let mut request = vec![0xe4, 0x21, 10];
        request.extend(target);
        request.extend(node.id);
        let data = self.query(&node, target, 0x29, request).await?;
        let nodes = routing_response(&data, target, self.allow_local)?;
        for contact in &nodes {
            self.add(contact.clone());
        }
        Ok(nodes)
    }
    async fn refresh_one(self: &Arc<Self>, node: &Node) {
        // Background refresh must not occupy a peer between a foreground lookup and its
        // source query. Yield to foreground work; hold this guard for only one contact.
        let Ok(_serial) = self.lookup_lock.try_lock() else {
            return;
        };
        let _ = self.find(node, self.identity).await;
    }
}

impl KadSession {
    pub async fn start(
        identity: Id,
        tcp_port: u16,
        path: PathBuf,
        seeds: Vec<Node>,
        allow_local: bool,
    ) -> Result<Arc<Self>, String> {
        let socket = match UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 4672)).await {
            Ok(socket) => socket,
            Err(_) => UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))
                .await
                .map_err(|e| e.to_string())?,
        };
        let socket = Arc::new(socket);
        Self::start_on_socket(identity, tcp_port, path, seeds, allow_local, socket).await
    }
    async fn start_on_socket(
        identity: Id,
        tcp_port: u16,
        path: PathBuf,
        seeds: Vec<Node>,
        allow_local: bool,
        socket: Arc<UdpSocket>,
    ) -> Result<Arc<Self>, String> {
        if identity == [0; 16] || tcp_port == 0 {
            return Err("kad_invalid_local_identity_or_tcp_port".into());
        }
        let inner = Arc::new(Inner {
            identity,
            tcp_port,
            socket,
            contacts: Mutex::new(HashMap::new()),
            pending: Mutex::new(HashMap::new()),
            path,
            allow_local,
            running: AtomicBool::new(true),
            serial: AtomicU64::new(1),
            lookup_lock: tokio::sync::Mutex::new(()),
        });
        // Stored contacts are candidates until a fresh HELLO and random challenge succeed.
        if let Ok(meta) = std::fs::metadata(&inner.path) {
            if meta.len() <= 1024 * 1024 {
                if let Ok(bytes) = std::fs::read(&inner.path) {
                    if let Ok(nodes) = serde_json::from_slice::<Vec<Node>>(&bytes) {
                        for node in nodes.into_iter().take(MAX_CONTACTS) {
                            inner.add(node);
                        }
                    }
                }
            }
        }
        for node in seeds.into_iter().take(MAX_CONTACTS) {
            inner.add(node);
        }
        let (shutdown, mut rx) = watch::channel(false);
        let session = Arc::new(Self {
            inner: inner.clone(),
            shutdown,
        });
        let receive = inner.clone();
        let mut receiver = rx.clone();
        tokio::spawn(async move {
            // Receive any complete IPv4 UDP datagram, then enforce the smaller payload cap.
            // A too-small recv_from buffer returns WSAEMSGSIZE on Windows and would stop Kad.
            let mut buffer = [0; 65536];
            let mut budget = 20u8;
            let mut reset = Instant::now();
            loop {
                let result = tokio::select! {_=receiver.changed()=>break,result=receive.socket.recv_from(&mut buffer)=>result};
                let (length, peer) = match result {
                    Ok(packet) => packet,
                    // In particular, Windows may report an offline UDP peer's ICMP error
                    // on this shared socket. That is not a failure of the Kad session.
                    Err(error) if retryable_udp_error(&error) => {
                        if !udp_retry_delay(&mut receiver).await {
                            break;
                        }
                        continue;
                    }
                    Err(_) => break,
                };
                if length < 2
                    || length > MAX_PAYLOAD_BYTES + 2
                    || !matches!(buffer[0], 0xe4 | 0xe5)
                    || !peer.is_ipv4()
                {
                    continue;
                }
                let opcode = buffer[1];
                if !matches!(opcode, 0x11 | 0x19 | 0x21 | 0x29 | 0x3b | 0x60) {
                    continue;
                }
                if matches!(opcode, 0x19 | 0x29 | 0x3b) {
                    // Avoid even inflating an unsolicited response. Full target/ID checks
                    // follow decompression, still within the same pending registration.
                    if !receive
                        .pending
                        .lock()
                        .unwrap()
                        .get(&peer)
                        .is_some_and(|p| p.opcode == opcode)
                    {
                        continue;
                    }
                } else {
                    if reset.elapsed() >= Duration::from_secs(1) {
                        budget = 20;
                        reset = Instant::now();
                    }
                    if budget == 0 {
                        continue;
                    }
                    budget -= 1;
                }
                let Ok(payload) = kad_payload(buffer[0], &buffer[2..length]) else {
                    continue;
                };
                let data = payload.as_ref();
                let target = match opcode {
                    0x19 | 0x29 => data.get(..16),
                    0x3b => data.get(16..32),
                    _ => None,
                };
                if let Some(target) = target {
                    let mut pending = receive.pending.lock().unwrap();
                    if pending
                        .get(&peer)
                        .is_some_and(|p| p.opcode == opcode && p.target.as_slice() == target)
                    {
                        if opcode == 0x19
                            && pending.get(&peer).is_some_and(|p| {
                                !hello_response(data, peer, receive.allow_local)
                                    .is_ok_and(|hello| hello.node.id == p.peer_id)
                            })
                        {
                            continue;
                        }
                        if opcode == 0x3b
                            && pending.get(&peer).is_some_and(|p| {
                                source_response(data, p.peer_id, p.target, receive.allow_local)
                                    .is_err()
                            })
                        {
                            continue;
                        }
                        if opcode == 0x29
                            && routing_response(
                                data,
                                target.try_into().unwrap(),
                                receive.allow_local,
                            )
                            .is_err()
                        {
                            continue;
                        }
                        if let Some(mut p) = pending.remove(&peer) {
                            match p.sender {
                                ReplySender::Single(sender) => {
                                    let _ = sender.send(data.to_vec());
                                }
                                ReplySender::Sources {
                                    sender,
                                    mut packets,
                                    mut rows,
                                    mut bytes,
                                } => {
                                    let count = usize::from(u16::from_le_bytes(
                                        data[32..34].try_into().unwrap(),
                                    ));
                                    if packets < MAX_SOURCE_PACKETS
                                        && rows + count <= MAX_SOURCE_ROWS
                                        && bytes + data.len() <= MAX_SOURCE_BYTES
                                        && sender.try_send(data.to_vec()).is_ok()
                                    {
                                        packets += 1;
                                        rows += count;
                                        bytes += data.len();
                                        if packets < MAX_SOURCE_PACKETS
                                            && rows < MAX_SOURCE_ROWS
                                            && bytes < MAX_SOURCE_BYTES
                                        {
                                            p.sender = ReplySender::Sources {
                                                sender,
                                                packets,
                                                rows,
                                                bytes,
                                            };
                                            pending.insert(peer, p);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    continue;
                }
                let mut reply = vec![0xe4];
                if opcode == 0x11 {
                    let Ok(hello) = hello_response(data, peer, receive.allow_local) else {
                        continue;
                    };
                    if hello.node.id == receive.identity {
                        continue;
                    }
                    if !hello.udp_firewalled {
                        // A HELLO can only introduce a candidate. The bounded refresh worker
                        // performs the challenge; receiving a packet never spawns another task.
                        receive.add(hello.node);
                    }
                    reply = hello_packet(0x19, receive.identity, receive.tcp_port);
                } else if opcode == 0x60 && data.is_empty() {
                    reply.push(0x61);
                    reply.extend(peer.port().to_le_bytes());
                } else if opcode == 0x21
                    && data.len() == 33
                    && data[17..33] == receive.identity
                    && data[0] & 31 > 0
                {
                    let known = receive
                        .contacts
                        .lock()
                        .unwrap()
                        .values()
                        .any(|c| address(&c.node) == peer && c.is_verified());
                    let target: Id = data[1..17].try_into().unwrap();
                    // A peer verifying our legacy HELLO must be able to obtain its nonce
                    // even before we have verified it. Do not reveal the table to that peer.
                    let nodes = if known {
                        receive.nearest(target, true, usize::from(data[0] & 31).min(10))
                    } else {
                        Vec::new()
                    };
                    reply.push(0x29);
                    reply.extend(target);
                    reply.push(nodes.len() as u8);
                    for node in nodes {
                        write_node(&mut reply, &node);
                    }
                } else {
                    continue;
                }
                let _ = receive.socket.send_to(&reply, peer).await;
            }
            receive.running.store(false, Ordering::Release);
            receive.pending.lock().unwrap().clear();
        });
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(Duration::from_secs(30));
            loop {
                tokio::select! {_=rx.changed()=>break,_=tick.tick()=>{}}
                // Refresh old contacts rather than only the closest nodes on every pass.
                let nodes = {
                    let table = inner.contacts.lock().unwrap();
                    let mut rows = table
                        .values()
                        .filter(|c| {
                            c.failed < 3
                                || c.attempted
                                    .is_none_or(|t| t.elapsed() >= Duration::from_secs(300))
                        })
                        .collect::<Vec<_>>();
                    rows.sort_by_key(|c| c.attempted);
                    rows.iter()
                        .take(4)
                        .map(|c| c.node.clone())
                        .collect::<Vec<_>>()
                };
                for node in nodes {
                    tokio::select! {_=rx.changed()=>return,_=inner.refresh_one(&node)=>{}}
                }
                let _ = inner.save();
            }
        });
        Ok(session)
    }
    pub fn status(&self) -> Status {
        let contacts = self.inner.contacts.lock().unwrap();
        Status {
            running: self.inner.running.load(Ordering::Acquire),
            tcp_port: self.inner.tcp_port,
            udp_port: self
                .inner
                .socket
                .local_addr()
                .map(|a| a.port())
                .unwrap_or(0),
            kad_version: KAD_VERSION,
            hello: true,
            contacts: contacts.len(),
            verified_contacts: contacts.values().filter(|c| c.is_verified()).count(),
            routing: true,
            direct_source_lookup: true,
            keyword_search: false,
            publishing: false,
            obfuscation: false,
            buddy_callbacks: false,
            public_reachability_verified: false,
        }
    }
    pub fn stop(&self) {
        let _ = self.shutdown.send(true);
        self.inner.running.store(false, Ordering::Release);
    }
    pub async fn lookup(&self, target: Id) -> Result<Vec<Node>, String> {
        let _serial = self.inner.lookup_lock.lock().await;
        self.lookup_locked(target).await
    }
    async fn lookup_locked(&self, target: Id) -> Result<Vec<Node>, String> {
        let mut visited = HashSet::new();
        for _ in 0..8 {
            let peers = self
                .inner
                .nearest(target, false, MAX_CONTACTS)
                .into_iter()
                .filter(|n| !visited.contains(&n.id))
                .take(3)
                .collect::<Vec<_>>();
            if peers.is_empty() {
                break;
            }
            let mut queries = tokio::task::JoinSet::new();
            for peer in peers {
                visited.insert(peer.id);
                let inner = self.inner.clone();
                queries.spawn(async move {
                    let _ = inner.find(&peer, target).await;
                });
            }
            while queries.join_next().await.is_some() {}
        }
        Ok(self.inner.nearest(target, true, 10))
    }
    pub async fn sources(&self, hash: Id, size: u64) -> Result<Vec<FileSource>, String> {
        let _serial = self.inner.lookup_lock.lock().await;
        let mut target = hash;
        for chunk in target.chunks_exact_mut(4) {
            chunk.reverse();
        }
        let peers = self.lookup_locked(target).await?;
        let mut sources = Vec::new();
        let mut seen = HashSet::new();
        for peer in peers.into_iter().take(3) {
            let mut request = vec![0xe4, 0x34];
            request.extend(target);
            request.extend(0u16.to_le_bytes());
            request.extend(size.to_le_bytes());
            if let Ok(found) = self.inner.query_sources(&peer, target, request).await {
                for source in found {
                    if seen.insert((source.host, source.port)) {
                        sources.push(source);
                    }
                }
            }
        }
        Ok(sources)
    }
}
impl Drop for KadSession {
    fn drop(&mut self) {
        self.stop();
    }
}

struct Reader<'a>(&'a [u8]);
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        if self.0.len() < n {
            return Err("kad_truncated_packet".into());
        }
        let (a, b) = self.0.split_at(n);
        self.0 = b;
        Ok(a)
    }
    fn byte(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, String> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn tag(&mut self) -> Result<(u8, Option<u64>), String> {
        let raw = self.byte()?;
        let kind = raw & 0x7f;
        let name = if raw & 0x80 != 0 {
            self.byte()?
        } else {
            let len = usize::from(self.u16()?);
            let name = self.take(len)?;
            if len == 1 {
                name[0]
            } else {
                0
            }
        };
        let value = match kind {
            3 => Some(u64::from(self.u32()?)),
            8 => Some(u64::from(self.u16()?)),
            9 => Some(u64::from(self.byte()?)),
            11 => Some(u64::from_le_bytes(self.take(8)?.try_into().unwrap())),
            1 => {
                self.take(16)?;
                None
            }
            2 => {
                let n = usize::from(self.u16()?);
                self.take(n)?;
                None
            }
            4 => {
                self.take(4)?;
                None
            }
            5 => {
                self.take(1)?;
                None
            }
            7 => {
                let n = self.u32()? as usize;
                self.take(n)?;
                None
            }
            0x11..=0x26 => {
                self.take(usize::from(kind - 0x10))?;
                None
            }
            _ => return Err("kad_unsupported_tag".into()),
        };
        Ok((name, value))
    }
}
fn source_response(
    bytes: &[u8],
    peer: Id,
    target: Id,
    local: bool,
) -> Result<Vec<FileSource>, String> {
    let mut reader = Reader(bytes);
    if reader.take(16)? != peer || reader.take(16)? != target {
        return Err("kad_response_identity_mismatch".into());
    }
    let count = reader.u16()?;
    if count > 128 {
        return Err("kad_source_limit".into());
    }
    let mut result = Vec::new();
    for _ in 0..count {
        reader.take(16)?;
        let tags = reader.byte()?;
        if tags > 64 {
            return Err("kad_tag_limit".into());
        }
        let (mut host, mut port, mut kind, mut crypto) = (0u64, 0u64, 0u64, 0u64);
        for _ in 0..tags {
            let (name, value) = reader.tag()?;
            if let Some(value) = value {
                match name {
                    0xfe => host = value,
                    0xfd => port = value,
                    0xff => kind = value,
                    0xf3 => crypto = value,
                    _ => {}
                }
            }
        }
        // Only high-ID direct sources are usable here; buddy/UDP callbacks and required crypt are not.
        let ip = Ipv4Addr::from(u32::try_from(host).unwrap_or(0).to_be_bytes());
        if matches!(kind, 1 | 4)
            && port > 0
            && port <= u64::from(u16::MAX)
            && valid_ip(ip, local)
            && crypto & 4 == 0
        {
            result.push(FileSource {
                client_id: u32::from_le_bytes(ip.octets()),
                host: Some(ip),
                port: port as u16,
                requires_callback: false,
            });
        }
    }
    if !reader.0.is_empty() {
        return Err("kad_trailing_source_data".into());
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn node(port: u16) -> Node {
        Node {
            id: [2; 16],
            host: Ipv4Addr::LOCALHOST,
            udp_port: port,
            tcp_port: 4662,
            version: 5,
        }
    }
    fn sources_packet(peer: Id, target: Id, kind: u8) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend(peer);
        data.extend(target);
        data.extend(1u16.to_le_bytes());
        data.extend([3; 16]);
        data.push(3);
        data.extend([0x83, 0xfe]);
        data.extend(u32::from_be_bytes([127, 0, 0, 1]).to_le_bytes());
        data.extend([0x88, 0xfd]);
        data.extend(4662u16.to_le_bytes());
        data.extend([0x89, 0xff, kind]);
        data
    }
    // Fixtures construct the wire independently of the production packet writers.
    fn fixture_hello(opcode: u8, id: Id, version: u8) -> Vec<u8> {
        let mut packet = vec![0xe4, opcode];
        packet.extend_from_slice(&id);
        packet.extend_from_slice(&[0x36, 0x12, version, 0]); // TCP 4662, zero tags.
        packet
    }
    fn fixture_challenge(target: Id, identity: Id) -> Vec<u8> {
        let mut packet = vec![0xe4, 0x21, 2];
        packet.extend_from_slice(&target);
        packet.extend_from_slice(&identity);
        packet
    }
    fn fixture_echo(target: &[u8]) -> Vec<u8> {
        let mut packet = vec![0xe4, 0x29];
        packet.extend_from_slice(target);
        packet.push(0);
        packet
    }
    fn fixture_packed(packet: &[u8]) -> Vec<u8> {
        // Independent zlib fixture: one uncompressed DEFLATE block and its Adler-32.
        // No production packet writer or flate2 compressor is used to produce the wire.
        let data = &packet[2..];
        let len = u16::try_from(data.len()).unwrap();
        let mut packed = vec![0xe5, packet[1], 0x78, 0x01, 0x01];
        packed.extend_from_slice(&len.to_le_bytes());
        packed.extend_from_slice(&(!len).to_le_bytes());
        packed.extend_from_slice(data);
        let (a, b) = data.iter().fold((1u32, 0u32), |(a, b), byte| {
            let a = (a + u32::from(*byte)) % 65521;
            (a, (b + a) % 65521)
        });
        packed.extend_from_slice(&((b << 16) | a).to_be_bytes());
        packed
    }
    fn fixture_source_batch(peer: Id, target: Id, count: u16, kind: u8) -> Vec<u8> {
        let one = sources_packet(peer, target, kind);
        let mut packet = vec![0xe4, 0x3b];
        packet.extend_from_slice(&one[..32]);
        packet.extend_from_slice(&count.to_le_bytes());
        for _ in 0..count {
            packet.extend_from_slice(&one[34..]);
        }
        packet
    }
    fn fixture_source_request(target: Id) -> Vec<u8> {
        let mut packet = vec![0xe4, 0x34];
        packet.extend_from_slice(&target);
        packet.extend_from_slice(&[0, 0, 100, 0, 0, 0, 0, 0, 0, 0]);
        packet
    }
    // Generated independently with Python zlib.compress(bytes(n), 9).
    const ZLIB_AT_LIMIT: &[u8] = &[
        120, 218, 237, 193, 1, 1, 0, 0, 0, 128, 144, 254, 175, 238, 8, 10, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 24, 128, 0, 0, 1,
    ];
    const ZLIB_OVER_LIMIT: &[u8] = &[
        120, 218, 237, 193, 1, 1, 0, 0, 0, 128, 144, 254, 175, 238, 8, 10, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 104, 128, 1, 0, 1,
    ];
    // Python zlib.compress(..., 9): eight direct source rows, 274 bytes before compression.
    // Peer ID 02*16; target is MD4 bytes 00..0f converted to four little-endian Kad words.
    const PACKED_SOURCES: &[u8] = &[
        229, 59, 120, 218, 93, 201, 185, 17, 192, 32, 12, 4, 64, 73, 124, 2, 63, 204, 184, 24, 183,
        71, 64, 98, 23, 237, 39, 190, 219, 116, 205, 80, 48, 149, 146, 83, 92, 90, 245, 190, 111,
        171, 75, 96, 243, 85, 145, 113, 61, 231, 113, 127, 26, 9, 110, 34, 184, 153, 224, 22, 130,
        235, 4, 183, 18, 220, 70, 112, 127, 168, 222, 46, 177,
    ];
    #[test]
    fn zlib_payload_requires_complete_stream_no_trailing_data_and_bounded_expansion() {
        let plain = fixture_echo(&[9; 16]);
        let packed = fixture_packed(&plain);
        assert_eq!(
            kad_payload(0xe5, &packed[2..]).unwrap().as_ref(),
            &plain[2..]
        );
        assert!(matches!(
            kad_payload(0xe4, &plain[2..]).unwrap(),
            Cow::Borrowed(_)
        ));
        assert_eq!(
            kad_payload(0xe5, ZLIB_AT_LIMIT).unwrap().len(),
            MAX_PAYLOAD_BYTES
        );
        assert!(kad_payload(0xe5, ZLIB_OVER_LIMIT).is_err());
        assert!(kad_payload(0xe4, &vec![0; MAX_PAYLOAD_BYTES + 1]).is_err());
        assert!(kad_payload(0xe3, &plain[2..]).is_err());
        for end in 0..ZLIB_AT_LIMIT.len() {
            assert!(kad_payload(0xe5, &ZLIB_AT_LIMIT[..end]).is_err());
        }
        let mut extra = ZLIB_AT_LIMIT.to_vec();
        extra.push(0);
        assert!(kad_payload(0xe5, &extra).is_err());
        extra.extend_from_slice(ZLIB_AT_LIMIT);
        assert!(kad_payload(0xe5, &extra).is_err());
        let mut corrupt = ZLIB_AT_LIMIT.to_vec();
        *corrupt.last_mut().unwrap() ^= 1;
        assert!(kad_payload(0xe5, &corrupt).is_err());
        let sources = kad_payload(PACKED_SOURCES[0], &PACKED_SOURCES[2..]).unwrap();
        assert_eq!(sources.len(), 274);
        assert_eq!(
            source_response(
                &sources,
                [2; 16],
                [3, 2, 1, 0, 7, 6, 5, 4, 11, 10, 9, 8, 15, 14, 13, 12],
                true
            )
            .unwrap()
            .len(),
            8
        );
    }
    async fn recv_packet(socket: &UdpSocket) -> (Vec<u8>, SocketAddr) {
        let mut packet = [0; 2048];
        let (len, from) =
            tokio::time::timeout(Duration::from_secs(5), socket.recv_from(&mut packet))
                .await
                .expect("fixture response timed out")
                .unwrap();
        (packet[..len].to_vec(), from)
    }
    async fn packet_barrier(socket: &UdpSocket, to: SocketAddr) {
        // The receiver handles packets in order. A PONG proves the preceding malformed or
        // uncorrelated packet from this socket was processed, without an arbitrary sleep.
        socket.send_to(&[0xe4, 0x60], to).await.unwrap();
        let (packet, from) = recv_packet(socket).await;
        assert_eq!(from, to);
        assert_eq!(&packet[..2], &[0xe4, 0x61]);
        assert_eq!(
            &packet[2..],
            &socket.local_addr().unwrap().port().to_le_bytes()
        );
    }
    async fn unseeded_session(path: PathBuf) -> Arc<KadSession> {
        KadSession::start_on_socket(
            [1; 16],
            31337,
            path,
            Vec::new(),
            true,
            Arc::new(UdpSocket::bind("127.0.0.1:0").await.unwrap()),
        )
        .await
        .unwrap()
    }
    #[test]
    fn hello_parser_bounds_tags_and_rejects_endpoint_redirection() {
        let from: SocketAddr = "127.0.0.1:4663".parse().unwrap();
        let basic = fixture_hello(0x19, [2; 16], 8);
        let hello = hello_response(&basic[2..], from, true).unwrap();
        assert_eq!(hello.node.id, [2; 16]);
        assert_eq!(hello.node.tcp_port, 4662);
        assert_eq!(hello.node.udp_port, 4663);
        assert_eq!(hello.node.version, 8);
        assert!(!hello.udp_firewalled);

        let mut tagged = basic.clone();
        tagged[21] = 2;
        // aMule's named uint16 FC and uint8 F2 tags, little-endian values.
        tagged.extend_from_slice(&[8, 1, 0, 0xfc, 0x37, 0x12, 9, 1, 0, 0xf2, 1]);
        assert!(
            hello_response(&tagged[2..], from, true)
                .unwrap()
                .udp_firewalled
        );
        let mut compact = basic.clone();
        compact[21] = 2;
        compact.extend_from_slice(&[0x88, 0xfc, 0x37, 0x12, 0x89, 0xf2, 2]);
        assert!(
            !hello_response(&compact[2..], from, true)
                .unwrap()
                .udp_firewalled
        );
        assert!(hello_response(&tagged[2..], from, false).is_err());

        let mut invalid = Vec::new();
        let mut changed = basic.clone();
        changed[2..18].fill(0);
        invalid.push(changed);
        let mut changed = basic.clone();
        changed[18..20].fill(0);
        invalid.push(changed);
        let mut changed = basic.clone();
        changed[20] = 1;
        invalid.push(changed);
        let mut changed = basic.clone();
        changed[21] = 17;
        invalid.push(changed);
        let mut changed = tagged.clone();
        changed[26] = 0x38; // A different advertised UDP port cannot redirect this HELLO.
        invalid.push(changed);
        let mut changed = tagged.clone();
        changed[21] = 3;
        changed.extend_from_slice(&[0x89, 0xf2, 0]); // Conflicting duplicate options.
        invalid.push(changed);
        let mut changed = basic.clone();
        changed[21] = 1;
        changed.extend_from_slice(&[2, 1, 0, 0xf2, 1, 0, b'1']); // Options must be integer.
        invalid.push(changed);
        let mut changed = basic.clone();
        changed[21] = 1;
        changed.extend_from_slice(&[0x83, 0xfc, 0, 0, 1, 0]); // UDP value overflows u16.
        invalid.push(changed);
        let mut changed = basic.clone();
        changed.resize(MAX_HELLO_BYTES + 3, 0);
        invalid.push(changed);
        let mut changed = basic.clone();
        changed.push(0);
        invalid.push(changed);
        for packet in invalid {
            assert!(
                hello_response(&packet[2..], from, true).is_err(),
                "{packet:?}"
            );
        }
        for end in 2..basic.len() {
            assert!(hello_response(&basic[2..end], from, true).is_err());
        }
        for end in basic.len()..tagged.len() {
            assert!(hello_response(&tagged[2..end], from, true).is_err());
        }
    }
    #[tokio::test]
    async fn actual_udp_hello_requires_id_endpoint_and_unpredictable_challenge() {
        let root = tempfile::tempdir().unwrap();
        let fixture = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let impostor = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let peer = node(fixture.local_addr().unwrap().port());
        let tcp = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let tcp_port = tcp.local_addr().unwrap().port();
        let session = KadSession::start_on_socket(
            [1; 16],
            tcp_port,
            root.path().join("routing.json"),
            Vec::new(),
            true,
            Arc::new(UdpSocket::bind("127.0.0.1:0").await.unwrap()),
        )
        .await
        .unwrap();
        let inner = session.inner.clone();
        let work = tokio::spawn(async move { inner.ensure_verified(&peer).await });

        let (request, from) = recv_packet(&fixture).await;
        assert_eq!(&request[..2], &[0xe4, 0x11]);
        assert_eq!(&request[2..18], &[1; 16]);
        assert_eq!(&request[18..20], &tcp_port.to_le_bytes());
        assert_eq!(&request[20..], &[5, 0]); // The actual bound TCP port and plain Kad v5.
        fixture
            .send_to(&fixture_hello(0x19, [3; 16], 8), from)
            .await
            .unwrap();
        let mut malformed = fixture_hello(0x19, [2; 16], 8);
        malformed[21] = 1; // Truncated tag list must not complete HELLO.
        fixture.send_to(&malformed, from).await.unwrap();
        packet_barrier(&fixture, from).await;
        impostor
            .send_to(&fixture_hello(0x19, [2; 16], 8), from)
            .await
            .unwrap();
        packet_barrier(&impostor, from).await;
        assert_eq!(session.status().verified_contacts, 0);
        assert_eq!(session.inner.pending.lock().unwrap().len(), 1);

        fixture
            .send_to(&fixture_hello(0x19, [2; 16], 8), from)
            .await
            .unwrap();
        let (challenge, challenge_from) = recv_packet(&fixture).await;
        assert_eq!(challenge_from, from);
        assert_eq!(challenge.len(), 35);
        assert_eq!(&challenge[..3], &[0xe4, 0x21, 2]);
        assert_eq!(&challenge[19..], &[2; 16]);
        let nonce = &challenge[3..19];
        assert_ne!(nonce, &[0; 16]);
        assert_ne!(nonce, &[1; 16]);
        assert_ne!(nonce, &[2; 16]);
        assert_eq!(nonce[6] & 0xf0, 0x40); // Random UUID v4, not a stable lookup target.
        assert_eq!(nonce[8] & 0xc0, 0x80);
        assert_eq!(session.status().verified_contacts, 0);

        fixture
            .send_to(&fixture_echo(&[9; 16]), from)
            .await
            .unwrap();
        let mut malformed = fixture_echo(nonce);
        malformed[18] = 1; // A count without its contact bytes is invalid even for a challenge.
        fixture.send_to(&malformed, from).await.unwrap();
        packet_barrier(&fixture, from).await;
        impostor.send_to(&fixture_echo(nonce), from).await.unwrap();
        packet_barrier(&impostor, from).await;
        assert_eq!(session.status().verified_contacts, 0);
        assert_eq!(session.inner.pending.lock().unwrap().len(), 1);
        let packed = fixture_packed(&fixture_echo(nonce));
        fixture
            .send_to(&packed[..packed.len() - 1], from)
            .await
            .unwrap();
        let mut trailing = packed.clone();
        trailing.push(0);
        fixture.send_to(&trailing, from).await.unwrap();
        let mut bomb = vec![0xe5, 0x29];
        bomb.extend_from_slice(ZLIB_OVER_LIMIT);
        fixture.send_to(&bomb, from).await.unwrap();
        packet_barrier(&fixture, from).await;
        assert_eq!(session.status().verified_contacts, 0);
        fixture.send_to(&packed, from).await.unwrap();

        let verified = work.await.unwrap().unwrap();
        assert_eq!(verified.id, [2; 16]);
        assert_eq!(verified.version, 8);
        assert_eq!(session.status().verified_contacts, 1);
        assert_eq!(session.status().tcp_port, tcp_port);
        assert!(!session.status().public_reachability_verified);
        assert!(!session.status().obfuscation);
        assert!(session.inner.pending.lock().unwrap().is_empty());

        let proved_at = session.inner.contacts.lock().unwrap()[&verified.id].verified;
        let inner = session.inner.clone();
        let work = tokio::spawn(async move { inner.find(&verified, [7; 16]).await });
        let (request, from) = recv_packet(&fixture).await;
        assert_eq!(&request[..3], &[0xe4, 0x21, 10]);
        fixture
            .send_to(&fixture_echo(&request[3..19]), from)
            .await
            .unwrap();
        work.await.unwrap().unwrap();
        assert_eq!(
            session.inner.contacts.lock().unwrap()[&[2; 16]].verified,
            proved_at
        );
        session.stop();
    }
    #[tokio::test]
    async fn stable_routing_reply_never_establishes_contact_verification() {
        let root = tempfile::tempdir().unwrap();
        let fixture = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let peer = node(fixture.local_addr().unwrap().port());
        let session = unseeded_session(root.path().join("routing.json")).await;
        let inner = session.inner.clone();
        let work = tokio::spawn(async move {
            inner
                .query(&peer, [1; 16], 0x29, fixture_challenge([1; 16], peer.id))
                .await
        });
        let (_, from) = recv_packet(&fixture).await;
        fixture
            .send_to(&fixture_echo(&[1; 16]), from)
            .await
            .unwrap();
        work.await.unwrap().unwrap();
        assert_eq!(session.status().verified_contacts, 0);
        session.stop();
    }
    #[tokio::test]
    async fn udp_firewalled_or_keyed_ack_hello_cannot_be_verified() {
        for options in [1, 4] {
            let root = tempfile::tempdir().unwrap();
            let fixture = UdpSocket::bind("127.0.0.1:0").await.unwrap();
            let peer = node(fixture.local_addr().unwrap().port());
            let session = unseeded_session(root.path().join("routing.json")).await;
            let inner = session.inner.clone();
            let work = tokio::spawn(async move { inner.ensure_verified(&peer).await });
            let (_, from) = recv_packet(&fixture).await;
            let mut hello = fixture_hello(0x19, [2; 16], 8);
            hello[21] = 1;
            hello.extend_from_slice(&[9, 1, 0, 0xf2, options]);
            fixture.send_to(&hello, from).await.unwrap();
            assert!(work.await.unwrap().is_err());
            packet_barrier(&fixture, from).await; // No challenge or unsupported keyed ACK was sent.
            assert_eq!(session.status().verified_contacts, 0);
            assert!(session.inner.pending.lock().unwrap().is_empty());
            session.stop();
        }
    }
    #[tokio::test]
    async fn unknown_or_expired_peers_receive_empty_challenges_without_table_disclosure() {
        let root = tempfile::tempdir().unwrap();
        let fixture = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let peer = node(fixture.local_addr().unwrap().port());
        let session = unseeded_session(root.path().join("routing.json")).await;
        let to = session.inner.socket.local_addr().unwrap();
        // Let the initial maintenance tick see an empty table before arranging test contacts.
        packet_barrier(&fixture, to).await;
        let mut firewalled = fixture_hello(0x11, peer.id, 8);
        firewalled[21] = 1;
        firewalled.extend_from_slice(&[9, 1, 0, 0xf2, 1]);
        fixture.send_to(&firewalled, to).await.unwrap();
        let (reply, _) = recv_packet(&fixture).await;
        assert_eq!(&reply[..2], &[0xe4, 0x19]);
        assert_eq!(session.status().contacts, 0);
        fixture
            .send_to(&fixture_hello(0x11, peer.id, 8), to)
            .await
            .unwrap();
        let (reply, _) = recv_packet(&fixture).await;
        assert_eq!(&reply[2..18], &[1; 16]);
        assert_eq!(&reply[18..], &[0x69, 0x7a, 5, 0]);
        assert_eq!(session.status().contacts, 1);
        assert_eq!(session.status().verified_contacts, 0);
        let mut other = node(4663);
        other.id = [3; 16];
        session.inner.add(other.clone());
        session
            .inner
            .contacts
            .lock()
            .unwrap()
            .get_mut(&other.id)
            .unwrap()
            .verified = Some(Instant::now());

        fixture
            .send_to(&fixture_challenge([7; 16], [1; 16]), to)
            .await
            .unwrap();
        let (reply, _) = recv_packet(&fixture).await;
        assert_eq!(reply, fixture_echo(&[7; 16]));
        session
            .inner
            .contacts
            .lock()
            .unwrap()
            .get_mut(&peer.id)
            .unwrap()
            .verified = Some(Instant::now());
        fixture
            .send_to(&fixture_challenge([7; 16], [1; 16]), to)
            .await
            .unwrap();
        let (reply, _) = recv_packet(&fixture).await;
        assert_eq!(reply[18], 2);
        session
            .inner
            .contacts
            .lock()
            .unwrap()
            .get_mut(&peer.id)
            .unwrap()
            .verified = Some(Instant::now() - VERIFIED_TTL);
        fixture
            .send_to(&fixture_challenge([7; 16], [1; 16]), to)
            .await
            .unwrap();
        let (reply, _) = recv_packet(&fixture).await;
        assert_eq!(reply, fixture_echo(&[7; 16]));
        fixture
            .send_to(&fixture_challenge([7; 16], [8; 16]), to)
            .await
            .unwrap();
        packet_barrier(&fixture, to).await; // Wrong intended node ID gets no challenge response.
        session.stop();
    }
    #[test]
    fn sources_require_correlated_ids_exact_lengths_and_direct_non_private_sources() {
        let data = sources_packet([2; 16], [9; 16], 1);
        let sources = source_response(&data, [2; 16], [9; 16], true).unwrap();
        assert_eq!(sources[0].host, Some(Ipv4Addr::LOCALHOST));
        assert_eq!(sources[0].port, 4662);
        assert!(source_response(&data, [3; 16], [9; 16], true).is_err());
        assert!(source_response(&data, [2; 16], [8; 16], true).is_err());
        assert!(source_response(&data[..data.len() - 1], [2; 16], [9; 16], true).is_err());
        assert!(source_response(&data, [2; 16], [9; 16], false)
            .unwrap()
            .is_empty());
        assert!(
            source_response(&sources_packet([2; 16], [9; 16], 3), [2; 16], [9; 16], true)
                .unwrap()
                .is_empty()
        );
        let mut extra = data.clone();
        extra.push(0);
        assert!(source_response(&extra, [2; 16], [9; 16], true).is_err());
    }
    #[test]
    fn routing_packets_have_strict_contact_count_and_target() {
        let mut data = vec![9; 16];
        data.push(1);
        write_node(&mut data, &node(4567));
        assert_eq!(routing_response(&data, [9; 16], true).unwrap().len(), 1);
        assert!(routing_response(&data, [9; 16], false).unwrap().is_empty());
        assert!(routing_response(&data, [8; 16], true).is_err());
        data.push(0);
        assert!(routing_response(&data, [9; 16], true).is_err());
    }
    #[test]
    fn xor_distance_compares_kad_words_in_numeric_order() {
        let a = [0; 16];
        let mut small = [0; 16];
        small[..4].copy_from_slice(&1u32.to_le_bytes());
        let mut large = [0; 16];
        large[..4].copy_from_slice(&256u32.to_le_bytes());
        assert!(distance(a, small) < distance(a, large));
    }
    #[tokio::test]
    async fn actual_udp_routing_and_sources_ignore_wrong_target_and_survive_restart() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("routing.json");
        let fixture = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let peer = node(fixture.local_addr().unwrap().port());
        let peer_copy = peer.clone();
        let responder = tokio::spawn(async move {
            let mut buf = [0; 2048];
            loop {
                let (n, from) = fixture.recv_from(&mut buf).await.unwrap();
                let data = &buf[..n];
                let mut response = vec![0xe4];
                match data.get(1) {
                    Some(0x11) => {
                        assert_eq!(n, 22);
                        assert_eq!(&data[18..], &[0x69, 0x7a, 5, 0]);
                        fixture
                            .send_to(&fixture_hello(0x19, peer_copy.id, 8), from)
                            .await
                            .unwrap();
                        continue;
                    }
                    Some(0x21) => {
                        assert_eq!(n, 35);
                        assert_eq!(&data[19..35], &peer_copy.id);
                        let mut wrong = vec![0xe4, 0x29];
                        wrong.extend([77; 16]);
                        wrong.push(0);
                        fixture.send_to(&wrong, from).await.unwrap();
                        response.push(0x29);
                        response.extend(&data[3..19]);
                        response.push(0);
                    }
                    Some(0x34) => {
                        assert_eq!(n, 28);
                        assert_eq!(
                            &data[2..18],
                            &[3, 2, 1, 0, 7, 6, 5, 4, 11, 10, 9, 8, 15, 14, 13, 12]
                        );
                        let target: Id = data[2..18].try_into().unwrap();
                        fixture
                            .send_to(&fixture_source_batch([77; 16], target, 50, 1), from)
                            .await
                            .unwrap();
                        // The first valid batch is callback-only; direct results follow in E5.
                        fixture
                            .send_to(&fixture_source_batch(peer_copy.id, target, 50, 3), from)
                            .await
                            .unwrap();
                        fixture.send_to(PACKED_SOURCES, from).await.unwrap();
                        for _ in 0..4 {
                            fixture
                                .send_to(&fixture_source_batch(peer_copy.id, target, 0, 1), from)
                                .await
                                .unwrap();
                        }
                        continue;
                    }
                    _ => continue,
                }
                fixture
                    .send_to(&fixture_packed(&response), from)
                    .await
                    .unwrap();
            }
        });
        let session = unseeded_session(path.clone()).await;
        session.inner.ensure_verified(&peer).await.unwrap();
        let nearest = session.lookup([8; 16]).await.unwrap();
        assert_eq!(nearest[0].id, peer.id);
        let sources = session
            .sources([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15], 100)
            .await
            .unwrap();
        assert_eq!(sources.len(), 1);
        assert_eq!(session.status().verified_contacts, 1);
        session.inner.save().unwrap();
        session.stop();
        responder.abort();
        let restored = KadSession::start_on_socket(
            [1; 16],
            31337,
            path,
            Vec::new(),
            true,
            Arc::new(UdpSocket::bind("127.0.0.1:0").await.unwrap()),
        )
        .await
        .unwrap();
        assert_eq!(restored.status().contacts, 1);
        assert_eq!(restored.status().verified_contacts, 0);
        restored.stop();
    }
    #[tokio::test]
    async fn transient_udp_errors_have_cancelable_backoff_but_structural_errors_stop() {
        for kind in [
            io::ErrorKind::ConnectionReset,
            io::ErrorKind::ConnectionRefused,
            io::ErrorKind::TimedOut,
            io::ErrorKind::Interrupted,
            io::ErrorKind::WouldBlock,
        ] {
            assert!(retryable_udp_error(&io::Error::from(kind)));
        }
        for kind in [
            io::ErrorKind::PermissionDenied,
            io::ErrorKind::InvalidInput,
            io::ErrorKind::NotConnected,
        ] {
            assert!(!retryable_udp_error(&io::Error::from(kind)));
        }
        #[cfg(windows)]
        assert!(retryable_udp_error(&io::Error::from_raw_os_error(10040)));
        let (shutdown, mut receiver) = watch::channel(false);
        let waiting = tokio::spawn(async move { udp_retry_delay(&mut receiver).await });
        tokio::task::yield_now().await;
        shutdown.send(true).unwrap();
        assert!(!tokio::time::timeout(Duration::from_secs(1), waiting)
            .await
            .unwrap()
            .unwrap());
    }
    #[tokio::test]
    async fn oversized_udp_does_not_stop_the_session() {
        let root = tempfile::tempdir().unwrap();
        let fixture = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let session = unseeded_session(root.path().join("routing.json")).await;
        let to = session.inner.socket.local_addr().unwrap();
        let mut oversized = vec![0; 40000];
        oversized[0] = 0xe4;
        oversized[1] = 0x11;
        fixture.send_to(&oversized, to).await.unwrap();
        packet_barrier(&fixture, to).await;
        assert!(session.status().running);
        assert_eq!(session.status().contacts, 0);
        session.stop();
    }
    #[tokio::test]
    async fn sources_wait_for_refresh_then_collect_the_direct_later_batch() {
        let root = tempfile::tempdir().unwrap();
        let fixture = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let peer = node(fixture.local_addr().unwrap().port());
        let session = unseeded_session(root.path().join("routing.json")).await;
        let inner = session.inner.clone();
        let candidate = peer.clone();
        let verify = tokio::spawn(async move { inner.ensure_verified(&candidate).await });
        let (_, from) = recv_packet(&fixture).await;
        fixture
            .send_to(&fixture_hello(0x19, peer.id, 8), from)
            .await
            .unwrap();
        let (challenge, _) = recv_packet(&fixture).await;
        fixture
            .send_to(&fixture_echo(&challenge[3..19]), from)
            .await
            .unwrap();
        verify.await.unwrap().unwrap();

        let inner = session.inner.clone();
        let candidate = peer.clone();
        let refresh = tokio::spawn(async move { inner.refresh_one(&candidate).await });
        let (refresh_request, from) = recv_packet(&fixture).await;
        assert_eq!(&refresh_request[3..19], &[1; 16]);
        let foreground = session.clone();
        let mut sources = tokio::spawn(async move { foreground.sources([9; 16], 100).await });
        // Hold the refresh response deliberately. A busy pending slot must not turn a
        // foreground source lookup into an immediately successful empty result.
        assert!(
            tokio::time::timeout(Duration::from_millis(50), &mut sources)
                .await
                .is_err()
        );
        fixture
            .send_to(&fixture_echo(&refresh_request[3..19]), from)
            .await
            .unwrap();
        let (lookup, from) = recv_packet(&fixture).await;
        assert_eq!(&lookup[..3], &[0xe4, 0x21, 10]);
        assert_eq!(&lookup[3..19], &[9; 16]);
        fixture
            .send_to(&fixture_echo(&lookup[3..19]), from)
            .await
            .unwrap();
        let (request, from) = recv_packet(&fixture).await;
        assert_eq!(request, fixture_source_request([9; 16]));
        for index in 0..6 {
            fixture
                .send_to(
                    &fixture_source_batch(peer.id, [9; 16], 50, if index == 5 { 1 } else { 3 }),
                    from,
                )
                .await
                .unwrap();
        }
        assert_eq!(sources.await.unwrap().unwrap().len(), 1);
        refresh.await.unwrap();
        assert!(session.inner.pending.lock().unwrap().is_empty());
        session.stop();
    }
    #[tokio::test]
    async fn source_stream_limits_packets_rows_and_rejects_wrong_identity() {
        let root = tempfile::tempdir().unwrap();
        let fixture = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let peer = node(fixture.local_addr().unwrap().port());
        let session = unseeded_session(root.path().join("routing.json")).await;
        for (per_packet, packets, expected) in [(1, 6, 6), (100, 3, 300)] {
            let inner = session.inner.clone();
            let candidate = peer.clone();
            let work = tokio::spawn(async move {
                inner
                    .query_sources(&candidate, [9; 16], fixture_source_request([9; 16]))
                    .await
            });
            let (_, from) = recv_packet(&fixture).await;
            fixture
                .send_to(&fixture_source_batch([3; 16], [9; 16], 100, 1), from)
                .await
                .unwrap();
            fixture
                .send_to(&fixture_source_batch(peer.id, [8; 16], 100, 1), from)
                .await
                .unwrap();
            packet_barrier(&fixture, from).await;
            for _ in 0..packets + 1 {
                fixture
                    .send_to(&fixture_source_batch(peer.id, [9; 16], per_packet, 1), from)
                    .await
                    .unwrap();
            }
            assert_eq!(work.await.unwrap().unwrap().len(), expected);
            packet_barrier(&fixture, from).await;
            assert!(session.inner.pending.lock().unwrap().is_empty());
        }
        session.stop();
    }
    #[tokio::test]
    async fn source_stream_releases_pending_on_cancellation_and_absolute_timeout() {
        let root = tempfile::tempdir().unwrap();
        let fixture = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let peer = node(fixture.local_addr().unwrap().port());
        let session = unseeded_session(root.path().join("routing.json")).await;
        for cancel in [true, false] {
            let inner = session.inner.clone();
            let candidate = peer.clone();
            let work = tokio::spawn(async move {
                inner
                    .query_sources(&candidate, [9; 16], fixture_source_request([9; 16]))
                    .await
            });
            let (_, from) = recv_packet(&fixture).await;
            fixture
                .send_to(&fixture_source_batch(peer.id, [9; 16], 50, 3), from)
                .await
                .unwrap();
            packet_barrier(&fixture, from).await;
            assert_eq!(session.inner.pending.lock().unwrap().len(), 1);
            if cancel {
                work.abort();
                let _ = work.await;
            } else {
                let result = tokio::time::timeout(QUERY_TIMEOUT + Duration::from_secs(1), work)
                    .await
                    .unwrap();
                assert!(result.unwrap().unwrap().is_empty());
            }
            assert!(session.inner.pending.lock().unwrap().is_empty());
        }
        session.stop();
    }
    #[tokio::test]
    async fn cancelled_queries_release_the_pending_slot() {
        let root = tempfile::tempdir().unwrap();
        let peer_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let peer = node(peer_socket.local_addr().unwrap().port());
        let session = KadSession::start_on_socket(
            [1; 16],
            31337,
            root.path().join("nodes.json"),
            Vec::new(),
            true,
            Arc::new(UdpSocket::bind("127.0.0.1:0").await.unwrap()),
        )
        .await
        .unwrap();
        for during_challenge in [false, true] {
            let inner = session.inner.clone();
            let candidate = peer.clone();
            let work = tokio::spawn(async move { inner.find(&candidate, [9; 16]).await });
            let (request, from) = recv_packet(&peer_socket).await;
            assert_eq!(&request[..2], &[0xe4, 0x11]);
            if during_challenge {
                peer_socket
                    .send_to(&fixture_hello(0x19, peer.id, 8), from)
                    .await
                    .unwrap();
                let (request, _) = recv_packet(&peer_socket).await;
                assert_eq!(&request[..3], &[0xe4, 0x21, 2]);
            }
            assert_eq!(session.inner.pending.lock().unwrap().len(), 1);
            let conflict = session
                .inner
                .query(&peer, [9; 16], 0x29, fixture_challenge([9; 16], peer.id))
                .await;
            assert_eq!(conflict.unwrap_err(), "kad_query_busy");
            assert_eq!(session.inner.pending.lock().unwrap().len(), 1);
            work.abort();
            let _ = work.await;
            assert!(session.inner.pending.lock().unwrap().is_empty());
            assert_eq!(session.status().verified_contacts, 0);
        }
        session.stop();
    }
}
