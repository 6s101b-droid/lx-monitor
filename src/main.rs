use std::env;
use std::process::Command;
use sysinfo::{Components, System};
use eframe::egui;
use amdgpu_sysfs::gpu_handle::GpuHandle;

// Struktura z tumaczeniami
struct Translations {
    title: &'static str,
    gpu_load: &'static str,
    cpu_load: &'static str,
    cannot_read_gpu: &'static str,
    temperatures_tab: &'static str,
    hardware_tab: &'static str,
    processes_tab: &'static str,
    autostart_tab: &'static str,
    autostart_name: &'static str,
    autostart_command: &'static str,
    autostart_location: &'static str,
    autostart_status: &'static str,
    autostart_running: &'static str,
    autostart_not_running: &'static str,
    no_autostart: &'static str,
    process_name: &'static str,
    process_pid: &'static str,
    process_cpu: &'static str,
    process_memory: &'static str,
    no_processes: &'static str,
    component: &'static str,
    current: &'static str,
    maximum: &'static str,
    average: &'static str,
    no_sensors: &'static str,
    hardware_info: &'static str,
    os: &'static str,
    distribution: &'static str,
    version: &'static str,
    kernel_version: &'static str,
    architecture: &'static str,
    processor: &'static str,
    cpu_config: &'static str,
    motherboard: &'static str,
    bios: &'static str,
    graphics_card: &'static str,
    gpu_driver: &'static str,
    ram_total: &'static str,
    ram_used: &'static str,
    refresh_data: &'static str,
    reset_stats: &'static str,
    last_update_prefix: &'static str,
    last_update_suffix: &'static str,
    unknown: &'static str,
    gpu_clock: &'static str,
    vram_used: &'static str,
}

fn get_translations() -> &'static Translations {
    static ENGLISH: Translations = Translations {
        title: "Hardware Monitor",
        gpu_load: "GPU Load",
        cpu_load: "CPU Load",
        cannot_read_gpu: "Cannot read GPU load",
        temperatures_tab: "Temperatures",
        hardware_tab: "Hardware",
        processes_tab: "Processes",
        autostart_tab: "Autostart",
        autostart_name: "Name",
        autostart_command: "Command",
        autostart_location: "Location",
        autostart_status: "Status",
        autostart_running: "Running",
        autostart_not_running: "Not running",
        no_autostart: "No autostart programs found",
        process_name: "Name",
        process_pid: "PID",
        process_cpu: "CPU %",
        process_memory: "Memory",
        no_processes: "No processes available",
        component: "Component",
        current: "Current",
        maximum: "Maximum",
        average: "Average",
        no_sensors: "No temperature sensors available",
        hardware_info: "Hardware Information",
        os: "Operating System",
        distribution: "Distribution",
        version: "Version",
        kernel_version: "Kernel Version",
        architecture: "Architecture",
        processor: "Processor",
        cpu_config: "CPU Configuration",
        motherboard: "Motherboard",
        bios: "BIOS",
        graphics_card: "Graphics Card",
        gpu_driver: "GPU Driver",
        ram_total: "RAM (Total)",
        ram_used: "RAM (Used)",
        refresh_data: "Refresh Data",
        reset_stats: "Reset Statistics",
        last_update_prefix: "Last update: ",
        last_update_suffix: "s ago",
        unknown: "Unknown",
        gpu_clock: "GPU Clock",
        vram_used: "VRAM Usage",
    };

    static POLISH: Translations = Translations {
        title: "Monitor Sprzętowy",
        gpu_load: "Obciążenie Karty Graficznej",
        cpu_load: "Obciążenie Procesora",
        cannot_read_gpu: "Nie można odczytać obciążenia GPU",
        temperatures_tab: "Temperatury",
        hardware_tab: "Sprzęt",
        processes_tab: "Procesy",
        autostart_tab: "Autostart",
        autostart_name: "Nazwa",
        autostart_command: "Komenda",
        autostart_location: "Lokalizacja",
        autostart_status: "Status",
        autostart_running: "Uruchomiony",
        autostart_not_running: "Nie uruchomiony",
        no_autostart: "Brak programów autostartu",
        process_name: "Nazwa",
        process_pid: "PID",
        process_cpu: "CPU %",
        process_memory: "Pamięć",
        no_processes: "Brak dostępnych procesów",
        component: "Czujnik",
        current: "Aktualna",
        maximum: "Maksymalna",
        average: "Średnia",
        no_sensors: "Brak dostępnych czujników temperatury",
        hardware_info: "Informacje o Sprzęcie",
        os: "System operacyjny",
        distribution: "Dystrybucja",
        version: "Wersja",
        kernel_version: "Wersja jądra",
        architecture: "Architektura",
        processor: "Procesor",
        cpu_config: "Konfiguracja CPU",
        motherboard: "Płyta główna",
        bios: "BIOS",
        graphics_card: "Karta graficzna",
        gpu_driver: "Sterownik GPU",
        ram_total: "Pamięć RAM (całkowita)",
        ram_used: "Pamięć RAM (używana)",
        refresh_data: "Odśwież dane",
        reset_stats: "Resetuj statystyki",
        last_update_prefix: "Ostatnia aktualizacja: ",
        last_update_suffix: "s temu",
        unknown: "Nieznany",
        gpu_clock: "Taktowanie GPU",
        vram_used: "Zużycie VRAM",
    };

    // Wykryj jzyk systemowy
    let lang = std::env::var("LANG")
        .or_else(|_| std::env::var("LC_MESSAGES"))
        .or_else(|_| std::env::var("LANGUAGE"))
        .unwrap_or_default();

    if lang.to_lowercase().contains("pl") {
        &POLISH
    } else {
        &ENGLISH
    }
}

/// Snapshot jednego wiersza z /proc/stat
#[derive(Clone, Default)]
struct CpuStat {
    user: u64,
    nice: u64,
    system: u64,
    idle: u64,
    iowait: u64,
    irq: u64,
    softirq: u64,
    steal: u64,
}

impl CpuStat {
    fn total(&self) -> u64 {
        self.user + self.nice + self.system + self.idle
            + self.iowait + self.irq + self.softirq + self.steal
    }
    fn idle_total(&self) -> u64 {
        self.idle + self.iowait
    }
}

fn read_proc_stat() -> Vec<CpuStat> {
    let content = match std::fs::read_to_string("/proc/stat") {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    content.lines()
        .filter(|l| {
            l.starts_with("cpu")
                && l.len() > 3
                && l.chars().nth(3).map(|c| c.is_ascii_digit()).unwrap_or(false)
        })
        .map(|line| {
            let mut parts = line.split_whitespace().skip(1);
            let mut n = || parts.next().and_then(|v| v.parse().ok()).unwrap_or(0u64);
            CpuStat {
                user: n(), nice: n(), system: n(), idle: n(),
                iowait: n(), irq: n(), softirq: n(), steal: n(),
            }
        })
        .collect()
}

fn calc_cpu_usage(prev: &CpuStat, curr: &CpuStat) -> f32 {
    let total = curr.total().saturating_sub(prev.total());
    let idle  = curr.idle_total().saturating_sub(prev.idle_total());
    if total == 0 { return 0.0; }
    ((total.saturating_sub(idle)) as f32 / total as f32 * 100.0).clamp(0.0, 100.0)
}

fn get_private_memory_mb(pid: u32) -> Option<f64> {
    if let Ok(content) = std::fs::read_to_string(format!("/proc/{}/statm", pid)) {
        let mut parts = content.split_whitespace();
        let _ = parts.next()?;
        let rss = parts.next()?.parse::<u64>().ok()?;
        let shared = parts.next()?.parse::<u64>().ok()?;
        let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as u64;
        let priv_mem = rss.saturating_sub(shared) * page_size;
        return Some(priv_mem as f64 / 1024.0 / 1024.0);
    }
    None
}

struct AutostartProgram {
    name: String,
    command: String,
    location: String,
    path: String,
    running: bool,
}

fn get_autostart_programs(system: &System) -> Vec<AutostartProgram> {
    let mut programs = Vec::new();
    
    // User-specific autostart directory
    if let Some(home_dir) = std::env::var_os("HOME") {
        let user_autostart = std::path::PathBuf::from(home_dir).join(".config/autostart");
        if let Ok(entries) = std::fs::read_dir(user_autostart) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "desktop" {
                        if let Some(mut prog) = parse_desktop_file(&entry.path(), "User") {
                            prog.running = is_process_running(system, &prog.command);
                            programs.push(prog);
                        }
                    }
                }
            }
        }
    }
    
    // System-wide autostart directory
    let system_autostart = std::path::PathBuf::from("/etc/xdg/autostart");
    if let Ok(entries) = std::fs::read_dir(system_autostart) {
        for entry in entries.flatten() {
            if let Some(ext) = entry.path().extension() {
                if ext == "desktop" {
                    if let Some(mut prog) = parse_desktop_file(&entry.path(), "System") {
                        prog.running = is_process_running(system, &prog.command);
                        programs.push(prog);
                    }
                }
            }
        }
    }
    
    programs
}

fn is_process_running(system: &System, command: &str) -> bool {
    // Extract the executable name from the command
    let exe_name = command
        .split_whitespace()
        .next()
        .and_then(|cmd| {
            // Get just the filename without path
            cmd.split('/').last()
        })
        .unwrap_or("");
    
    // Check if any process with this name is running
    for process in system.processes().values() {
        if process.name().to_lowercase() == exe_name.to_lowercase() {
            return true;
        }
    }
    
    false
}

fn parse_desktop_file(path: &std::path::Path, location: &str) -> Option<AutostartProgram> {
    let content = std::fs::read_to_string(path).ok()?;
    
    let mut name = String::new();
    let mut command = String::new();
    let mut in_desktop_entry = false;
    
    for line in content.lines() {
        let line = line.trim();
        
        if line == "[Desktop Entry]" {
            in_desktop_entry = true;
            continue;
        }
        
        if line.starts_with('[') && in_desktop_entry {
            // End of Desktop Entry section
            break;
        }
        
        if in_desktop_entry {
            if let Some(value) = line.strip_prefix("Name=") {
                name = value.to_string();
            } else if let Some(value) = line.strip_prefix("Exec=") {
                command = value.to_string();
            }
        }
    }
    
    if name.is_empty() {
        // Fallback to filename if Name is not found
        name = path.file_stem()?.to_string_lossy().to_string();
    }
    
    if command.is_empty() {
        return None;
    }
    
    Some(AutostartProgram {
        name,
        command,
        location: location.to_string(),
        path: path.to_string_lossy().to_string(),
        running: false,
    })
}

fn draw_gauge(ui: &mut egui::Ui, percent: f32, color: egui::Color32, label: &str, extra_info: Option<&str>) {
    let (rect, _response) = ui.allocate_exact_size(egui::vec2(150.0, 120.0), egui::Sense::hover());
    let center = rect.center() + egui::vec2(0.0, 25.0);
    let radius = 60.0;
    let stroke_width = 12.0;

    let points: u32 = 64;
    let mut bg_points = vec![];
    for i in 0..=points {
        let angle = std::f32::consts::PI + (i as f32 / points as f32) * std::f32::consts::PI;
        bg_points.push(center + egui::vec2(angle.cos() * radius, angle.sin() * radius));
    }
    ui.painter().add(egui::Shape::Path(egui::epaint::PathShape::line(
        bg_points,
        egui::Stroke::new(stroke_width, egui::Color32::from_gray(50)),
    )));

    let mut fg_points = vec![];
    let end_angle = std::f32::consts::PI + (percent / 100.0).clamp(0.0, 1.0) * std::f32::consts::PI;
    let fg_steps = (points as f32 * (percent / 100.0)).max(1.0) as u32;
    for i in 0..=fg_steps {
        let angle = std::f32::consts::PI + (i as f32 / fg_steps as f32) * (end_angle - std::f32::consts::PI);
        fg_points.push(center + egui::vec2(angle.cos() * radius, angle.sin() * radius));
    }
    ui.painter().add(egui::Shape::Path(egui::epaint::PathShape::line(
        fg_points,
        egui::Stroke::new(stroke_width, color),
    )));

    let mut y_offset = -20.0;
    ui.put(
        egui::Rect::from_center_size(center + egui::vec2(0.0, y_offset), egui::vec2(rect.width(), 20.0)),
        egui::Label::new(
            egui::RichText::new(format!("{:.1}%", percent))
                .strong()
                .size(20.0)
                .color(color)
        ),
    );
    y_offset += 18.0;
    if let Some(info) = extra_info {
        for line in info.lines() {
            y_offset += 14.0;
            ui.put(
                egui::Rect::from_center_size(center + egui::vec2(0.0, y_offset), egui::vec2(rect.width(), 20.0)),
                egui::Label::new(egui::RichText::new(line).size(11.0).color(egui::Color32::from_gray(200))),
            );
        }
    }
    y_offset += 20.0;
    ui.put(
        egui::Rect::from_center_size(center + egui::vec2(0.0, y_offset), egui::vec2(rect.width(), 20.0)),
        egui::Label::new(egui::RichText::new(label).size(14.0)),
    );
}

struct HwMonitorApp {
    system: System,
    components: Components,
    gpu_handle: Option<GpuHandle>,
    last_update: std::time::Instant,
    last_process_update: std::time::Instant,
    max_temps: std::collections::HashMap<String, f32>,
    temp_sum: std::collections::HashMap<String, f32>,
    temp_count: std::collections::HashMap<String, u32>,
    temp_history: std::collections::HashMap<String, std::collections::VecDeque<f32>>,
    selected_tab: usize,
    cpu_brand: String,
    cpu_name: String,
    gpu_name: String,
    gpu_driver: String,
    distro_name: String,
    distro_version: String,
    motherboard: String,
    bios_version: String,
    translations: &'static Translations,
    gpu_usage: Option<u8>,
    gpu_clock_mhz: Option<u64>,
    gpu_vram_used_mb: Option<u64>,
    gpu_vram_total_mb: Option<u64>,
    cpu_clock_mhz: Option<u64>,
    ram_used_mb: Option<u64>,
    ram_total_mb: Option<u64>,
    cpu_prev: Vec<CpuStat>,
    cpu_usage: Vec<f32>,
    selected_pid: Option<u32>,
    selected_temp: Option<String>,
    selected_autostart: Option<String>,
}

impl Default for HwMonitorApp {
    fn default() -> Self {
        let mut system = System::new_all();
        system.refresh_cpu_usage();
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        system.refresh_cpu_usage();
        
        let components = Components::new_with_refreshed_list();
        let gpu_handle = amdgpu_sysfs::gpu_handle::GpuHandle::new_from_path(std::path::PathBuf::from("/sys/class/drm/card1/device")).ok();
        
        // Pobierz informacje o CPU
        let translations = get_translations();
        let cpu_brand = system.cpus().first().map(|c| c.brand().to_string()).unwrap_or_else(|| translations.unknown.to_string());
        let cpu_name = if translations.title == "Monitor Sprzętowy" {
            format!("{} rdzeni", system.cpus().len())
        } else {
            format!("{} cores", system.cpus().len())
        };
                // Pobierz informacje o GPU
        let (gpu_name, gpu_driver) = if gpu_handle.is_some() {
            // Najpierw sprbuj odczyta dokadny model z sysfs product_name
            let product_name = std::fs::read_to_string("/sys/class/drm/card1/device/product_name")
                .ok()
                .map(|s| s.trim().to_string());
            
            // Funkcja do mapowania PCI Device ID na konkretny model
            fn get_exact_gpu_model(device_id: u16) -> Option<String> {
                match device_id {
                    // Navi 23 (RX 6600 series)
                    0x73ff => Some("AMD Radeon RX 6600".to_string()),
                    0x73fe => Some("AMD Radeon RX 6600 XT".to_string()),
                    0x73e1 => Some("AMD Radeon RX 6600M".to_string()),
                    // Navi 22 (RX 6700 series)
                    0x73df => Some("AMD Radeon RX 6700 XT".to_string()),
                    0x73e0 => Some("AMD Radeon RX 6700".to_string()),
                    0x73e2 => Some("AMD Radeon RX 6700M".to_string()),
                    // Navi 21 (RX 6800/6900 series)
                    0x73bf => Some("AMD Radeon RX 6900 XT".to_string()),
                    0x73c0 => Some("AMD Radeon RX 6800".to_string()),
                    0x73c1 => Some("AMD Radeon RX 6800 XT".to_string()),
                    // Navi 24 (RX 6500/6400 series)
                    0x743f => Some("AMD Radeon RX 6500 XT".to_string()),
                    0x743e => Some("AMD Radeon RX 6400".to_string()),
                    // Navi 31 (RX 7900 series)
                    0x744c => Some("AMD Radeon RX 7900 XTX".to_string()),
                    0x744b => Some("AMD Radeon RX 7900 XT".to_string()),
                    // Navi 32 (RX 7800/7700 series)
                    0x747e => Some("AMD Radeon RX 7800 XT".to_string()),
                    0x7470 => Some("AMD Radeon RX 7700 XT".to_string()),
                    // Navi 33 (RX 7600)
                    0x7480 => Some("AMD Radeon RX 7600".to_string()),
                    0x7481 => Some("AMD Radeon RX 7600 XT".to_string()),
                    // Navi 10 (RX 5700/5600 series)
                    0x731f => Some("AMD Radeon RX 5700 XT".to_string()),
                    0x7310 => Some("AMD Radeon RX 5700".to_string()),
                    0x7340 => Some("AMD Radeon RX 5600 XT".to_string()),
                    // Vega
                    0x687f => Some("AMD Radeon RX Vega 64".to_string()),
                    0x6863 => Some("AMD Radeon RX Vega 56".to_string()),
                    _ => None,
                }
            }
            
            // Odczytaj PCI Device ID z sysfs
            let device_id_from_sysfs = std::fs::read_to_string("/sys/class/drm/card1/device/device")
                .ok()
                .and_then(|s| u16::from_str_radix(s.trim().trim_start_matches("0x"), 16).ok());
            
            let gpu_name = if let Some(name) = product_name {
                // product_name istnieje - uyj go jeli nie zawiera "Navi"
                if !name.contains("Navi") {
                    name
                } else if let Some(id) = device_id_from_sysfs {
                    // Zamapuj na konkretny model
                    get_exact_gpu_model(id).unwrap_or(name)
                } else {
                    name
                }
            } else {
                // Fallback do lspci z wyciganiem PCI ID
                Command::new("lspci")
                    .arg("-nn")
                    .output()
                    .ok()
                    .and_then(|output| String::from_utf8(output.stdout).ok())
                    .and_then(|s| {
                        s.lines()
                            .find(|line| line.contains("VGA") || line.contains("3D"))
                            .and_then(|line| {
                                // Wycignij PCI ID z formatu [1002:73ff]
                                let device_id = line.find("[1002:")
                                    .and_then(|start| {
                                        let id_start = start + 6; // Po "[1002:"
                                        line[id_start..].find(']')
                                            .and_then(|len| u16::from_str_radix(&line[id_start..id_start+len], 16).ok())
                                    });
                                
                                // Jeli mamy ID, zamapuj na nazw
                                if let Some(id) = device_id {
                                    if let Some(model) = get_exact_gpu_model(id) {
                                        return Some(model);
                                    }
                                }
                                
                                // Fallback: wycignij nazw marketingow z nawiasw kwadratowych
                                // Format: Navi 23 [Radeon RX 6600/6600 XT/6600M] [1002:73ff]
                                if let Some(start) = line.find('[') {
                                    if let Some(end) = line.find("] [") {
                                        let bracket_content = &line[start+1..end];
                                        // Jeli to lista (zawiera "/"), sprbuj znale dokadny model
                                        if bracket_content.contains('/') {
                                            if let Some(id) = device_id {
                                                return get_exact_gpu_model(id);
                                            }
                                        }
                                        return Some(bracket_content.to_string());
                                    }
                                }
                                None
                            })
                    })
                    .unwrap_or_else(|| "AMD GPU".to_string())
            };
            
            // Odczytaj wersj sterownika
            let driver_version = std::fs::read_to_string("/sys/module/amdgpu/version")
                .ok()
                .map(|s| format!("amdgpu ({})", s.trim()))
                .unwrap_or_else(|| "amdgpu".to_string());
            
            (gpu_name, driver_version)
        } else {
            ("Brak wykrytej karty".to_string(), "Brak".to_string())
        };
        
        // Pobierz informacje o dystrybucji
        let (distro_name, distro_version) = if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
            let name = content.lines()
                .find(|line| line.starts_with("PRETTY_NAME="))
                .and_then(|line| line.strip_prefix("PRETTY_NAME="))
                .map(|s| s.trim_matches('"').to_string())
                .unwrap_or_else(|| translations.unknown.to_string());
            
            let version = content.lines()
                .find(|line| line.starts_with("VERSION="))
                .and_then(|line| line.strip_prefix("VERSION="))
                .map(|s| s.trim_matches('"').to_string())
                .unwrap_or_else(|| translations.unknown.to_string());
            
            (name, version)
        } else {
            (translations.unknown.to_string(), translations.unknown.to_string())
        };
        
        // Pobierz informacje o pycie gwnej z sysfs DMI
        let motherboard = std::fs::read_to_string("/sys/class/dmi/id/board_name")
            .ok()
            .map(|s| s.trim().to_string())
            .or_else(|| {
                std::fs::read_to_string("/sys/class/dmi/id/board_vendor")
                    .ok()
                    .map(|s| s.trim().to_string())
            })
            .unwrap_or_else(|| translations.unknown.to_string());
        
        // Pobierz wersj BIOS z sysfs DMI
        let bios_version = std::fs::read_to_string("/sys/class/dmi/id/bios_version")
            .ok()
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| translations.unknown.to_string());
        
        Self {
            system,
            components,
            gpu_handle,
            last_update: std::time::Instant::now(),
            last_process_update: std::time::Instant::now(),
            max_temps: std::collections::HashMap::new(),
            temp_sum: std::collections::HashMap::new(),
            temp_count: std::collections::HashMap::new(),
            temp_history: std::collections::HashMap::new(),
            selected_tab: 0,
            cpu_brand,
            cpu_name,
            gpu_name,
            gpu_driver,
            distro_name,
            distro_version,
            motherboard,
            bios_version,
            translations: get_translations(),
            gpu_usage: None,
            gpu_clock_mhz: None,
            gpu_vram_used_mb: None,
            gpu_vram_total_mb: None,
            cpu_clock_mhz: None,
            ram_used_mb: None,
            ram_total_mb: None,
            cpu_prev: read_proc_stat(),
            cpu_usage: vec![],
            selected_pid: None,
            selected_temp: None,
            selected_autostart: None,
        }
    }
}

impl eframe::App for HwMonitorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Sticky footer na dole - zawsze widoczny
        egui::TopBottomPanel::bottom("footer_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("v0.1.2").color(egui::Color32::DARK_GRAY));
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Czas odświeżenia (po prawej)
                    ui.label(format!("{}{:.1}{}", self.translations.last_update_prefix, self.last_update.elapsed().as_secs_f32(), self.translations.last_update_suffix));
                
                ui.add_space(20.0);
                
                // Przycisk resetowania
                if ui.button(self.translations.reset_stats).clicked() {
                    self.max_temps.clear();
                    self.temp_sum.clear();
                    self.temp_count.clear();
                    self.temp_history.clear();
                }
                
                ui.add_space(10.0);
                
                // Przycisk odświeżania
                if ui.button(self.translations.refresh_data).clicked() {
                    self.system.refresh_cpu_usage();
                    self.components.refresh();
                    self.last_update = std::time::Instant::now();
                }
            }); // end right_to_left_layout
            }); // end horizontal layout
        }); // end bottom_panel
        // Główna zawartość z przewijaniem
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading(self.translations.title);
            
            // Aktualizuj dane co sekundę (CPU, GPU, temperatury)
            if self.last_update.elapsed() >= std::time::Duration::from_secs(1) {
                // Odczyt CPU z /proc/stat (jak GNOME System Monitor)
                let curr_stat = read_proc_stat();
                self.cpu_usage = curr_stat.iter().zip(self.cpu_prev.iter())
                    .map(|(curr, prev)| calc_cpu_usage(prev, curr))
                    .collect();
                self.cpu_prev = curr_stat;

                // Taktowanie CPU (MHz) - odczyt z cpufreq
                self.cpu_clock_mhz = std::fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq")
                    .ok()
                    .and_then(|s| s.trim().parse::<u64>().ok())
                    .map(|khz| khz / 1000);

                // Zużycie RAM (w bajtach -> MB) - odczyt z sysinfo
                self.ram_total_mb = Some(self.system.total_memory() / 1024 / 1024);
                self.ram_used_mb = Some(self.system.used_memory() / 1024 / 1024);

                self.components.refresh();
                
                // Odczyt obciążenia GPU
                if let Some(ref gpu) = self.gpu_handle {
                    self.gpu_usage = gpu.get_busy_percent().ok();
                    // Taktowanie GPU - aktywny poziom z pp_dpm_sclk (MHz)
                    if let Ok(levels) = gpu.get_clock_levels(amdgpu_sysfs::gpu_handle::PowerLevelKind::CoreClock) {
                        self.gpu_clock_mhz = levels.active_level().copied();
                    }
                    // Zużycie VRAM (w bajtach -> MB)
                    if let Ok(vram) = gpu.get_used_vram() {
                        self.gpu_vram_used_mb = Some(vram / 1024 / 1024);
                    }
                    if let Ok(vram_total) = gpu.get_total_vram() {
                        self.gpu_vram_total_mb = Some(vram_total / 1024 / 1024);
                    }
                }
                
                // Aktualizuj statystyki temperatur
                for component in &self.components {
                    let temp = component.temperature();
                    let label = component.label();
                    
                    // Aktualizuj maksymalną temperaturę
                    let max_temp = self.max_temps.entry(label.to_string()).or_insert(temp);
                    *max_temp = max_temp.max(temp);
                    
                    // Dodaj do historii (ogranicz do 100 pomiarów)
                    let history = self.temp_history.entry(label.to_string()).or_insert_with(|| std::collections::VecDeque::new());
                    history.push_back(temp);
                    if history.len() > 100 {
                        history.pop_front();
                    }
                    
                    // Przelicz średnią z ostatnich 100 pomiarów
                    let avg_temp: f32 = history.iter().sum::<f32>() / history.len() as f32;
                    self.temp_sum.insert(label.to_string(), avg_temp * history.len() as f32);
                    self.temp_count.insert(label.to_string(), history.len() as u32);
                }
                
                self.last_update = std::time::Instant::now();
            }
            
            // Aktualizuj procesy co 3 sekundy
            if self.last_process_update.elapsed() >= std::time::Duration::from_secs(3) {
                self.system.refresh_processes();
                self.last_process_update = std::time::Instant::now();
            }
            
            ui.separator();
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                let total_width = 150.0 + 80.0 + 150.0; // Szerokość zegarów i odstępu
                let padding = (ui.available_width() - total_width) / 2.0;
                if padding > 0.0 {
                    ui.add_space(padding);
                }
                
                // Oblicz CPU
                let avg_usage = if self.cpu_usage.is_empty() {
                    0.0
                } else {
                    self.cpu_usage.iter().sum::<f32>() / self.cpu_usage.len() as f32
                };
                
                let cpu_color = if avg_usage > 80.0 {
                    egui::Color32::RED
                } else if avg_usage > 50.0 {
                    egui::Color32::YELLOW
                } else {
                    egui::Color32::GREEN
                };
                
                // Rysuj CPU Gauge
                let cpu_clock = self.cpu_clock_mhz.unwrap_or(0);
                let ram_used = self.ram_used_mb.unwrap_or(0);
                let ram_total = self.ram_total_mb.unwrap_or(0);
                let cpu_extra_info = if ram_total > 0 {
                    Some(format!("{} MHz\nRAM: {} / {} MB", cpu_clock, ram_used, ram_total))
                } else {
                    Some(format!("{} MHz\nRAM: {} MB", cpu_clock, ram_used))
                };
                draw_gauge(ui, avg_usage, cpu_color, self.translations.cpu_load, cpu_extra_info.as_deref());
                
                ui.add_space(80.0); // Odstęp między łukami
                
                // Oblicz GPU
                let clock = self.gpu_clock_mhz.unwrap_or(0);
                let used = self.gpu_vram_used_mb.unwrap_or(0);
                let total = self.gpu_vram_total_mb.unwrap_or(0);
                let extra_info = if total > 0 {
                    Some(format!("{} MHz\nVRAM: {} / {} MB", clock, used, total))
                } else {
                    Some(format!("{} MHz\nVRAM: {} MB", clock, used))
                };
                
                if let Some(gpu_usage) = self.gpu_usage {
                    let gpu_usage_f32 = gpu_usage as f32;
                    let gpu_color = if gpu_usage > 80 {
                        egui::Color32::RED
                    } else if gpu_usage > 50 {
                        egui::Color32::YELLOW
                    } else {
                        egui::Color32::GREEN
                    };
                    draw_gauge(ui, gpu_usage_f32, gpu_color, self.translations.gpu_load, extra_info.as_deref());
                } else {
                    draw_gauge(ui, 0.0, egui::Color32::from_gray(100), self.translations.cannot_read_gpu, extra_info.as_deref());
                }
            });
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.selected_tab, 0, self.translations.temperatures_tab);
                ui.selectable_value(&mut self.selected_tab, 1, self.translations.hardware_tab);
                ui.selectable_value(&mut self.selected_tab, 2, self.translations.processes_tab);
                ui.selectable_value(&mut self.selected_tab, 3, self.translations.autostart_tab);
            });
            ui.separator();
            
            match self.selected_tab {
                0 => {
                    // Zakładka Temperatury
                    ui.add_space(10.0);
                    if self.components.is_empty() {
                        ui.label(self.translations.no_sensors);
                    } else {
                        // Nagłówki tabeli
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(self.translations.component).strong());
                            
                            let right_width = 80.0 * 3.0 + ui.spacing().item_spacing.x * 2.0;
                            let padding = ui.available_width() - right_width;
                            if padding > 0.0 { ui.add_space(padding); }
                            
                            ui.allocate_ui_with_layout(egui::vec2(80.0, 20.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new(self.translations.current).strong());
                            });
                            ui.allocate_ui_with_layout(egui::vec2(80.0, 20.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new(self.translations.maximum).strong());
                            });
                            ui.allocate_ui_with_layout(egui::vec2(80.0, 20.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new(self.translations.average).strong());
                            });
                        });
                        ui.separator();
                        
                        let mut sorted_components: Vec<_> = self.components.iter().collect();
                        sorted_components.sort_by(|a, b| {
                            let a_priority = get_component_priority(a.label());
                            let b_priority = get_component_priority(b.label());
                            a_priority.cmp(&b_priority)
                        });
                        
                        for component in sorted_components {
                            let temp = component.temperature();
                            let color = if temp > 70.0 {
                                egui::Color32::RED
                            } else if temp > 50.0 {
                                egui::Color32::YELLOW
                            } else {
                                egui::Color32::GREEN
                            };
                            
                            let polish_name = get_localized_name(component.label(), self.translations);
                            
                            let comp_label = component.label().to_string();
                            let is_selected_temp = self.selected_temp.as_ref() == Some(&comp_label);
                            
                            let fill = if is_selected_temp {
                                ui.visuals().selection.bg_fill
                            } else {
                                egui::Color32::TRANSPARENT
                            };
                            
                            let response = egui::Frame::none()
                                .fill(fill)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        // Nazwa podzespołu - wyrównana do lewej całkowicie
                                        ui.label(&polish_name);
                                        
                                        let right_width = 80.0 * 3.0 + ui.spacing().item_spacing.x * 2.0;
                                        let padding = ui.available_width() - right_width;
                                        if padding > 0.0 { ui.add_space(padding); }
                                        
                                        // Aktualna temperatura
                                        ui.allocate_ui_with_layout(egui::vec2(80.0, 20.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.label(egui::RichText::new(format!("{:.1}°C", temp)).color(color));
                                        });
                                        
                                        // Maksymalna temperatura
                                        let max_temp = self.max_temps.get(component.label()).unwrap_or(&temp);
                                        ui.allocate_ui_with_layout(egui::vec2(80.0, 20.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.label(egui::RichText::new(format!("{:.1}°C", max_temp)).color(egui::Color32::from_rgb(255, 140, 0)));
                                        });
                                        
                                        // Średnia temperatura
                                        ui.allocate_ui_with_layout(egui::vec2(80.0, 20.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if let Some(&sum) = self.temp_sum.get(component.label()) {
                                                if let Some(&count) = self.temp_count.get(component.label()) {
                                                    let avg_temp = sum / count as f32;
                                                    ui.label(egui::RichText::new(format!("{:.1}°C", avg_temp)).color(egui::Color32::BLUE));
                                                } else {
                                                    ui.label("-");
                                                }
                                            } else {
                                                ui.label("-");
                                            }
                                        });
                                    });
                                }).response.interact(egui::Sense::click());
                                
                            if response.clicked() {
                                self.selected_temp = Some(comp_label);
                            }
                        }
                    }
                }
                1 => {
                    // Zakładka Sprzęt
                    ui.add_space(10.0);
                    ui.heading(self.translations.hardware_info);
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", self.translations.os));
                        ui.label(std::env::consts::OS);
                    });

                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", self.translations.distribution));
                        ui.label(&self.distro_name);
                    });

                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", self.translations.version));
                        ui.label(&self.distro_version);
                    });

                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", self.translations.architecture));
                        ui.label(std::env::consts::ARCH);
                    });

                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", self.translations.kernel_version));
                        let kernel_ver = System::kernel_version()
                            .unwrap_or_else(|| self.translations.unknown.to_string());
                        ui.label(kernel_ver);
                    });

                    ui.separator();
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", self.translations.processor));
                        ui.label(&self.cpu_brand);
                    });

                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", self.translations.cpu_config));
                        ui.label(&self.cpu_name);
                    });

                    ui.separator();
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", self.translations.motherboard));
                        ui.label(&self.motherboard);
                    });

                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", self.translations.bios));
                        ui.label(&self.bios_version);
                    });

                    ui.separator();
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", self.translations.graphics_card));
                        ui.label(&self.gpu_name);
                    });

                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", self.translations.gpu_driver));
                        // Nazwa sterownika z wersją w nawiasie
                        ui.label(&self.gpu_driver);
                    });

                    ui.separator();
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", self.translations.ram_total));
                        ui.label(format!("{:.1} GB", self.system.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0));
                    });

                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", self.translations.ram_used));
                        ui.label(format!("{:.1} GB", self.system.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0));
                    });
                }
                2 => {
                    // Zakładka Procesy
                    ui.add_space(10.0);
                    
                    // Pobierz i posortuj procesy według użycia CPU (malejąco)
                    let mut processes: Vec<_> = self.system.processes().values().collect();
                    processes.sort_by(|a, b| {
                        b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap_or(std::cmp::Ordering::Equal)
                    });
                    
                    if processes.is_empty() {
                        ui.label(self.translations.no_processes);
                    } else {
                        // Nagłówki tabeli
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(self.translations.process_name).strong());
                            
                            let right_width = 80.0 + 80.0 + 100.0 + ui.spacing().item_spacing.x * 2.0;
                            let padding = ui.available_width() - right_width;
                            if padding > 0.0 { ui.add_space(padding); }
                            
                            ui.allocate_ui_with_layout(egui::vec2(80.0, 20.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new(self.translations.process_pid).strong());
                            });
                            ui.allocate_ui_with_layout(egui::vec2(80.0, 20.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new(self.translations.process_cpu).strong());
                            });
                            ui.allocate_ui_with_layout(egui::vec2(100.0, 20.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new(self.translations.process_memory).strong());
                            });
                        });
                        ui.separator();
                        
                        let num_cores = self.system.cpus().len().max(1) as f32;
                        
                        // Wyświetl top 30 procesów (ograniczenie dla wydajności)
                        for process in processes.iter().take(30) {
                            let raw_cpu = process.cpu_usage();
                            // Obliczamy użycie całkowite systemu na wzór Windows Task Manager
                            let cpu_usage = (raw_cpu / num_cores).clamp(0.0, 100.0);
                            
                            let color = if cpu_usage > 50.0 {
                                egui::Color32::RED
                            } else if cpu_usage > 20.0 {
                                egui::Color32::YELLOW
                            } else {
                                egui::Color32::GREEN
                            };
                            
                            let name = process.name().to_string();
                            // Ogranicz nazwę do 35 znaków
                            let display_name = if name.len() > 35 {
                                format!("{}...", &name[..32])
                            } else {
                                name
                            };
                            
                            let pid = process.pid().as_u32();
                            let is_selected = self.selected_pid == Some(pid);
                            
                            let fill = if is_selected {
                                ui.visuals().selection.bg_fill
                            } else {
                                egui::Color32::TRANSPARENT
                            };
                            
                            let response = egui::Frame::none()
                                .fill(fill)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(display_name);
                                        
                                        let right_width = 80.0 + 80.0 + 100.0 + ui.spacing().item_spacing.x * 2.0;
                                        let padding = ui.available_width() - right_width;
                                        if padding > 0.0 { ui.add_space(padding); }
                                        
                                        // PID
                                        ui.allocate_ui_with_layout(egui::vec2(80.0, 18.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.label(process.pid().to_string());
                                        });
                                        
                                        // CPU
                                        ui.allocate_ui_with_layout(egui::vec2(80.0, 18.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.label(egui::RichText::new(format!("{:.1}%", cpu_usage)).color(color));
                                        });
                                        
                                        // Pamięć
                                        let memory_mb = get_private_memory_mb(process.pid().as_u32())
                                            .unwrap_or_else(|| process.memory() as f64 / 1024.0 / 1024.0);
                                        ui.allocate_ui_with_layout(egui::vec2(100.0, 18.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.label(format!("{:.1} MB", memory_mb));
                                        });
                                    })
                                }).response.interact(egui::Sense::click());
                            
                            if response.clicked() {
                                self.selected_pid = Some(pid);
                            }
                            
                            response.context_menu(|ui| {
                                if ui.button("Zakończ proces").clicked() {
                                    process.kill();
                                    ui.close_menu();
                                }
                            });
                        }
                    }
                }
                3 => {
                    // Zakładka Autostart
                    ui.add_space(10.0);
                    
                    let autostart_programs = get_autostart_programs(&self.system);
                    
                    if autostart_programs.is_empty() {
                        ui.label(self.translations.no_autostart);
                    } else {
                        // Sztywne szerokości kolumn (w pikselach)
                        let status_width = 100.0;

                        // Nagłówki tabeli
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(self.translations.autostart_name).strong());
                            
                            let right_width = status_width;
                            let padding = ui.available_width() - right_width;
                            if padding > 0.0 { ui.add_space(padding); }
                            
                            ui.allocate_ui_with_layout(egui::vec2(status_width, 20.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new(self.translations.autostart_status).strong());
                            });
                        });
                        ui.separator();
                        
                        for program in &autostart_programs {
                            let is_selected = self.selected_autostart.as_ref() == Some(&program.name);
                            
                            let fill = if is_selected {
                                ui.visuals().selection.bg_fill
                            } else {
                                egui::Color32::TRANSPARENT
                            };
                            
                            let response = egui::Frame::none()
                                .fill(fill)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(&program.name);
                                        
                                        let right_width = status_width;
                                        let padding = ui.available_width() - right_width;
                                        if padding > 0.0 { ui.add_space(padding); }
                                        
                                        // Status
                                        ui.allocate_ui_with_layout(egui::vec2(status_width, 20.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            let status_text = if program.running {
                                                self.translations.autostart_running
                                            } else {
                                                self.translations.autostart_not_running
                                            };
                                            let status_color = if program.running {
                                                egui::Color32::GREEN
                                            } else {
                                                egui::Color32::GRAY
                                            };
                                            ui.label(egui::RichText::new(status_text).color(status_color));
                                        });
                                    });
                                }).response.interact(egui::Sense::click());
                            
                            if response.clicked() {
                                self.selected_autostart = Some(program.name.clone());
                            }
                            
                            response.context_menu(|ui| {
                                if ui.button("Otwórz lokalizację").clicked() {
                                    // Try to open file manager with the file selected
                                    let path = std::path::Path::new(&program.path);
                                    
                                    // Try different file managers with select option
                                    let file_managers = vec![
                                        ("nautilus", vec!["--select"]),
                                        ("dolphin", vec!["--select"]),
                                        ("nemo", vec!["--select"]),
                                        ("thunar", vec![]),
                                        ("pcmanfm", vec![]),
                                    ];
                                    
                                    let mut opened = false;
                                    for (cmd, args) in file_managers {
                                        if Command::new(cmd)
                                            .args(args.iter())
                                            .arg(&program.path)
                                            .spawn()
                                            .is_ok()
                                        {
                                            opened = true;
                                            break;
                                        }
                                    }
                                    
                                    // Fallback to xdg-open on the parent folder
                                    if !opened {
                                        if let Some(parent) = path.parent() {
                                            Command::new("xdg-open")
                                                .arg(parent)
                                                .spawn()
                                                .ok();
                                        }
                                    }
                                    
                                    ui.close_menu();
                                }
                            });
                        }
                    }
                }
                _ => {}
            }
            
            });
        });
        
        // Automatyczna aktualizacja co 1 sekundę
        ctx.request_repaint_after(std::time::Duration::from_secs(1));
    }
}

fn get_component_priority(label: &str) -> u8 {
    if label.contains("coretemp") {
        1 // CPU - najwyższy priorytet
    } else if label.contains("amdgpu") {
        2 // GPU - drugi priorytet
    } else {
        3 // Reszta - najniższy priorytet
    }
}

fn get_localized_name(original: &str, translations: &Translations) -> String {
    match original {
        "gigabyte_wmi temp1" => {
            if translations.title == "Monitor Sprzętowy" {
                "Płyta główna - Mostek północny".to_string()
            } else {
                "Motherboard - Northbridge".to_string()
            }
        }
        "gigabyte_wmi temp6" => {
            if translations.title == "Monitor Sprzętowy" {
                "Płyta główna - Mostek południowy".to_string()
            } else {
                "Motherboard - Southbridge".to_string()
            }
        }
        "gigabyte_wmi temp3" => {
            if translations.title == "Monitor Sprzętowy" {
                "Płyta główna - VRM 1".to_string()
            } else {
                "Motherboard - VRM 1".to_string()
            }
        }
        "gigabyte_wmi temp5" => {
            if translations.title == "Monitor Sprzętowy" {
                "Płyta główna - VRM 2".to_string()
            } else {
                "Motherboard - VRM 2".to_string()
            }
        }
        "gigabyte_wmi temp4" => {
            if translations.title == "Monitor Sprzętowy" {
                "Płyta główna - Chipset".to_string()
            } else {
                "Motherboard - Chipset".to_string()
            }
        }
        "gigabyte_wmi temp2" => {
            if translations.title == "Monitor Sprzętowy" {
                "Płyta główna - Obszar ogólny".to_string()
            } else {
                "Motherboard - General Area".to_string()
            }
        }
        "acpitz temp1" => {
            if translations.title == "Monitor Sprzętowy" {
                "Strefa termiczna ACPI 1".to_string()
            } else {
                "ACPI Thermal Zone 1".to_string()
            }
        }
        "acpitz temp2" => {
            if translations.title == "Monitor Sprzętowy" {
                "Strefa termiczna ACPI 2".to_string()
            } else {
                "ACPI Thermal Zone 2".to_string()
            }
        }
        name if name.contains("coretemp") && name.contains("Core") => {
            if translations.title == "Monitor Sprzętowy" {
                format!("Procesor - Rdzeń {}", name.split_whitespace().last().unwrap_or(""))
            } else {
                format!("CPU - Core {}", name.split_whitespace().last().unwrap_or(""))
            }
        }
        name if name.contains("coretemp") && name.contains("Package") => {
            if translations.title == "Monitor Sprzętowy" {
                "Procesor - Pakiet".to_string()
            } else {
                "CPU - Package".to_string()
            }
        }
        name if name.contains("amdgpu") && name.contains("mem") => {
            if translations.title == "Monitor Sprzętowy" {
                "Karta graficzna - Pamięć VRAM".to_string()
            } else {
                "Graphics Card - VRAM Memory".to_string()
            }
        }
        name if name.contains("amdgpu") && name.contains("junction") => {
            if translations.title == "Monitor Sprzętowy" {
                "Karta graficzna - Złącze".to_string()
            } else {
                "Graphics Card - Junction".to_string()
            }
        }
        name if name.contains("amdgpu") && name.contains("edge") => {
            if translations.title == "Monitor Sprzętowy" {
                "Karta graficzna - Krawędź".to_string()
            } else {
                "Graphics Card - Edge".to_string()
            }
        }
        _ => original.to_string(),
    }
}

fn main() {
    // Weryfikacja, czy proces działa z uprawnieniami roota (UID == 0)
    let is_root = unsafe { libc::geteuid() == 0 };

    if !is_root {
        // Pobranie ścieżki do aktualnie wykonywanego pliku
        let current_exe = env::current_exe().expect("Nie można ustalić ścieżki pliku wykonywalnego");
        
        // Wywołanie samego siebie poprzez pkexec (Polkit)
        let status = Command::new("pkexec")
            .arg(current_exe)
            .status()
            .expect("Nie udało się uruchomić pkexec. Upewnij się, że system obsługuje Polkit.");

        if !status.success() {
            eprintln!("Odmowa dostępu lub błąd uwierzytelniania.");
            return;
        }
    }

    // Uruchomienie GUI
    let translations = get_translations();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 700.0])
            .with_min_inner_size([400.0, 500.0])
            .with_title(translations.title)
            .with_resizable(true),
        ..Default::default()
    };

    let result = eframe::run_native(
        translations.title,
        options,
        Box::new(|_cc| Box::new(HwMonitorApp::default())),
    );

    if let Err(e) = result {
        eprintln!("Błąd uruchomienia GUI: {}", e);
    }
}