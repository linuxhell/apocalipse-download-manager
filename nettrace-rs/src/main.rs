#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use std::{
    collections::VecDeque,
    fs,
    net::{IpAddr, ToSocketAddrs},
    path::PathBuf,
    process::Command,
    sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex},
    thread,
    time::Duration,
};

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Family { V4, V6 }
impl Family {
    fn prefix(self) -> &'static str { if self == Self::V4 { "ipv4" } else { "ipv6" } }
    fn flag(self) -> &'static str { if self == Self::V4 { "-4" } else { "-6" } }
}

#[derive(Clone, Default)]
struct Stats {
    ip: String,
    samples: VecDeque<Option<f32>>,
    sent: u64,
    recv: u64,
    sum: f64,
    min: Option<f32>,
    max: Option<f32>,
    last: Option<f32>,
    jitter_sum: f64,
    jitter_n: u64,
    route: String,
    tracing: bool,
}

impl Stats {
    fn push(&mut self, ms: Option<f32>) {
        self.sent += 1;
        if let Some(v) = ms {
            if let Some(prev) = self.last {
                self.jitter_sum += (v - prev).abs() as f64;
                self.jitter_n += 1;
            }
            self.last = Some(v);
            self.recv += 1;
            self.sum += v as f64;
            self.min = Some(self.min.map_or(v, |x| x.min(v)));
            self.max = Some(self.max.map_or(v, |x| x.max(v)));
        }
        self.samples.push_back(ms);
        while self.samples.len() > 600 { self.samples.pop_front(); }
    }
    fn avg(&self) -> Option<f32> { (self.recv > 0).then(|| (self.sum / self.recv as f64) as f32) }
    fn loss(&self) -> f32 {
        if self.sent == 0 { 0.0 } else { ((self.sent - self.recv) as f32 / self.sent as f32) * 100.0 }
    }
    fn jitter(&self) -> Option<f32> {
        (self.jitter_n > 0).then(|| (self.jitter_sum / self.jitter_n as f64) as f32)
    }
}

struct Target {
    id: u64,
    host: String,
    family: Family,
    stats: Arc<Mutex<Stats>>,
    stop: Arc<AtomicBool>,
}
impl Target { fn spec(&self) -> String { format!("{}:{}", self.family.prefix(), self.host) } }

struct App {
    targets: Vec<Target>,
    selected: Option<u64>,
    next_id: u64,
    input: String,
    config: PathBuf,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        let mut a = Self {
            targets: vec![], selected: None, next_id: 1,
            input: "google.com".into(), config: config_path()
        };
        let specs = fs::read_to_string(&a.config).unwrap_or_default();
        if specs.lines().all(|x| x.trim().is_empty()) {
            a.add("google.com".into(), Family::V4);
            a.add("google.com".into(), Family::V6);
        } else {
            for line in specs.lines() {
                if let Some((h, f)) = parse(line) { a.add(h, f); }
            }
        }
        a.save();
        a
    }

    fn add(&mut self, host: String, family: Family) {
        if host.trim().is_empty() { return; }
        if let Some(t) = self.targets.iter().find(|t| t.host.eq_ignore_ascii_case(&host) && t.family == family) {
            self.selected = Some(t.id); return;
        }
        let id = self.next_id; self.next_id += 1;
        let stats = Arc::new(Mutex::new(Stats { route: "Clique em Traceroute para capturar a rota.".into(), ..Default::default() }));
        let stop = Arc::new(AtomicBool::new(false));
        start_monitor(host.clone(), family, stats.clone(), stop.clone());
        self.targets.push(Target { id, host, family, stats, stop });
        if self.selected.is_none() { self.selected = Some(id); }
    }

    fn save(&self) {
        if let Some(p) = self.config.parent() { let _ = fs::create_dir_all(p); }
        let body = self.targets.iter().map(Target::spec).collect::<Vec<_>>().join("\n");
        let _ = fs::write(&self.config, body);
    }

    fn add_input(&mut self, dual: bool) {
        let raw = self.input.trim().to_string();
        if dual {
            let host = raw.strip_prefix("ipv4:").or_else(|| raw.strip_prefix("ipv6:")).unwrap_or(&raw).to_string();
            self.add(host.clone(), Family::V4); self.add(host, Family::V6);
        } else if let Some((h, f)) = parse(&raw) { self.add(h, f); }
        self.save();
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| { ui.heading("NetTrace RS"); ui.label("Rust • portátil • IPv4/IPv6"); });
            ui.horizontal(|ui| {
                ui.add_sized([340.0, 28.0], egui::TextEdit::singleline(&mut self.input).hint_text("ipv4:google.com | ipv6:google.com"));
                if ui.button("Adicionar").clicked() { self.add_input(false); }
                if ui.button("IPv4 + IPv6").clicked() { self.add_input(true); }
                if ui.button("Remover").clicked() {
                    if let Some(id) = self.selected {
                        if let Some(i) = self.targets.iter().position(|t| t.id == id) {
                            self.targets[i].stop.store(true, Ordering::Relaxed);
                            self.targets.remove(i);
                            self.selected = self.targets.first().map(|t| t.id);
                            self.save();
                        }
                    }
                }
            });
            ui.add_space(6.0);
        });

        egui::SidePanel::left("targets").default_width(275.0).show(ctx, |ui| {
            ui.heading("Destinos");
            ui.separator();
            for t in &self.targets {
                let s = t.stats.lock().unwrap().clone();
                if ui.selectable_label(self.selected == Some(t.id), t.spec()).clicked() { self.selected = Some(t.id); }
                ui.label(format!("{}  • perda {:.1}%", fmt_ms(s.last), s.loss()));
                ui.add_space(8.0);
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let Some(id) = self.selected else { ui.label("Adicione um destino."); return };
            let Some(t) = self.targets.iter().find(|x| x.id == id) else { return };
            let s = t.stats.lock().unwrap().clone();

            ui.heading(t.spec());
            ui.label(format!("IP resolvido: {}", if s.ip.is_empty() { "aguardando..." } else { &s.ip }));
            ui.horizontal_wrapped(|ui| {
                card(ui, "AGORA", fmt_ms(s.last));
                card(ui, "MÉDIA", fmt_ms(s.avg()));
                card(ui, "MÍN / MÁX", match (s.min,s.max) { (Some(a),Some(b)) => format!("{a:.1} / {b:.1} ms"), _ => "--".into() });
                card(ui, "JITTER", fmt_ms(s.jitter()));
                card(ui, "PERDA", format!("{:.2}%", s.loss()));
                card(ui, "AMOSTRAS", format!("{}/{}", s.recv, s.sent));
            });

            ui.add_space(12.0);
            ui.heading("Latência em tempo real");
            graph(ui, &s.samples);

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.add_enabled(!s.tracing, egui::Button::new(if s.tracing { "Executando..." } else { "Traceroute" })).clicked() {
                    start_trace(t.host.clone(), t.family, t.stats.clone());
                }
                ui.label("ipv4:host força IPv4 • ipv6:host força IPv6");
            });
            ui.add_space(8.0);
            ui.heading("Rota atual");
            egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
                let mut route = s.route.clone();
                ui.add(egui::TextEdit::multiline(&mut route).font(egui::TextStyle::Monospace).desired_width(f32::INFINITY).interactive(false));
            });
        });
        ctx.request_repaint_after(Duration::from_millis(250));
    }
}

impl Drop for App {
    fn drop(&mut self) {
        for t in &self.targets { t.stop.store(true, Ordering::Relaxed); }
        self.save();
    }
}

fn card(ui: &mut egui::Ui, title: &str, value: String) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_min_width(115.0);
        ui.label(egui::RichText::new(title).small().weak());
        ui.label(egui::RichText::new(value).strong().size(16.0));
    });
}

fn fmt_ms(v: Option<f32>) -> String { v.map(|x| format!("{x:.1} ms")).unwrap_or_else(|| "--".into()) }

fn graph(ui: &mut egui::Ui, samples: &VecDeque<Option<f32>>) {
    let size = egui::vec2(ui.available_width().max(300.0), 260.0);
    let (r, p) = ui.allocate_painter(size, egui::Sense::hover());
    let rect = r.rect;
    p.rect_filled(rect, 5.0, egui::Color32::from_rgb(18,22,28));
    let max = samples.iter().filter_map(|v| *v).fold(0.0_f32, f32::max).max(50.0) * 1.2;
    for i in 0..=4 {
        let t = i as f32 / 4.0;
        let y = egui::lerp(rect.bottom()..=rect.top(), t);
        p.line_segment([egui::pos2(rect.left(),y), egui::pos2(rect.right(),y)], egui::Stroke::new(1.0, egui::Color32::from_gray(45)));
        p.text(egui::pos2(rect.left()+5.0,y), egui::Align2::LEFT_TOP, format!("{:.0} ms", max*t), egui::FontId::monospace(10.0), egui::Color32::GRAY);
    }
    if samples.len() < 2 { return; }
    let n = (samples.len()-1) as f32;
    let mut seg = Vec::new();
    for (i,v) in samples.iter().enumerate() {
        let x = egui::lerp(rect.left()..=rect.right(), i as f32/n);
        match v {
            Some(ms) => seg.push(egui::pos2(x, egui::lerp(rect.bottom()..=rect.top(), (*ms/max).clamp(0.0,1.0)))),
            None => {
                if seg.len()>1 { p.add(egui::Shape::line(seg.clone(), egui::Stroke::new(2.0, egui::Color32::LIGHT_GREEN))); }
                seg.clear();
                p.line_segment([egui::pos2(x,rect.top()),egui::pos2(x,rect.bottom())], egui::Stroke::new(1.3,egui::Color32::LIGHT_RED));
            }
        }
    }
    if seg.len()>1 { p.add(egui::Shape::line(seg, egui::Stroke::new(2.0, egui::Color32::LIGHT_GREEN))); }
}

fn start_monitor(host: String, family: Family, stats: Arc<Mutex<Stats>>, stop: Arc<AtomicBool>) {
    thread::spawn(move || while !stop.load(Ordering::Relaxed) {
        if let Some(ip) = resolve(&host, family) { stats.lock().unwrap().ip = ip.to_string(); }
        let ms = ping(&host, family);
        stats.lock().unwrap().push(ms);
        thread::sleep(Duration::from_secs(1));
    });
}

fn ping(host: &str, family: Family) -> Option<f32> {
    let mut c = Command::new("ping.exe");
    c.args([family.flag(), "-n", "1", "-w", "1500", host]);
    #[cfg(windows)] c.creation_flags(CREATE_NO_WINDOW);
    let o = c.output().ok()?;
    if !o.status.success() { return None; }
    parse_ms(&String::from_utf8_lossy(&o.stdout))
}

fn parse_ms(text: &str) -> Option<f32> {
    for line in text.lines() {
        let lower = line.to_ascii_lowercase().replace(',', ".");
        if let Some(ms) = lower.find("ms") {
            let before = &lower[..ms];
            let digits: String = before.chars().rev().take_while(|c| c.is_ascii_digit() || *c=='.').collect::<String>().chars().rev().collect();
            if let Ok(v) = digits.parse::<f32>() { return Some(v.max(0.1)); }
        }
    }
    None
}

fn resolve(host: &str, family: Family) -> Option<IpAddr> {
    (host,0).to_socket_addrs().ok()?.map(|x| x.ip()).find(|ip| matches!((family,ip),(Family::V4,IpAddr::V4(_))|(Family::V6,IpAddr::V6(_))))
}

fn start_trace(host: String, family: Family, stats: Arc<Mutex<Stats>>) {
    stats.lock().unwrap().tracing = true;
    thread::spawn(move || {
        let mut c = Command::new("tracert.exe");
        c.args([family.flag(), "-d", "-h", "30", "-w", "1200", &host]);
        #[cfg(windows)] c.creation_flags(CREATE_NO_WINDOW);
        let text = c.output().map(|o| String::from_utf8_lossy(&o.stdout).to_string()).unwrap_or_else(|e| e.to_string());
        let mut s = stats.lock().unwrap(); s.route = text; s.tracing = false;
    });
}

fn parse(input: &str) -> Option<(String,Family)> {
    let s=input.trim();
    if let Some(h)=s.strip_prefix("ipv4:") { Some((h.trim().into(),Family::V4)) }
    else if let Some(h)=s.strip_prefix("ipv6:") { Some((h.trim().into(),Family::V6)) }
    else if !s.is_empty() { Some((s.into(),Family::V4)) } else { None }
}

fn config_path() -> PathBuf {
    std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.join("data").join("targets.txt"))).unwrap_or_else(|| PathBuf::from("data/targets.txt"))
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0,820.0]).with_min_inner_size([900.0,620.0]),
        ..Default::default()
    };
    eframe::run_native("NetTrace RS", options, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}
