use eframe::egui;
use hmac::{Hmac, Mac};
use sha1::Sha1;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

type HmacSha1 = Hmac<Sha1>;

// ── Configuration ──────────────────────────────────────────────────────────
const TOTP_DIGITS: u32 = 6;
const TOTP_PERIOD: u64 = 30;
const DATA_FILE: &str = "accounts.json";

// ── Base32 decode (RFC 4648, no padding required) ─────────────────────────
fn base32_decode(input: &str) -> Result<Vec<u8>, String> {
    let cleaned: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    let cleaned = cleaned.to_uppercase();
    let alphabet: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

    let mut bits: u64 = 0;
    let mut bit_count: u32 = 0;
    let mut out = Vec::new();

    for ch in cleaned.chars() {
        if ch == '=' {
            break; // padding, ignore
        }
        let val = match alphabet.iter().position(|&a| a as char == ch) {
            Some(v) => v as u64,
            None => return Err(format!("Invalid Base32 character: '{}'", ch)),
        };
        bits = (bits << 5) | val;
        bit_count += 5;
        if bit_count >= 8 {
            bit_count -= 8;
            out.push((bits >> bit_count) as u8);
            bits &= (1 << bit_count) - 1;
        }
    }
    Ok(out)
}

// ── Base32 encode (RFC 4648, no padding) ──────────────────────────────────
fn base32_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut bits: u64 = 0;
    let mut bit_count: u32 = 0;
    let mut out = String::new();

    for &byte in data {
        bits = (bits << 8) | byte as u64;
        bit_count += 8;
        while bit_count >= 5 {
            bit_count -= 5;
            out.push(ALPHABET[(bits >> bit_count) as usize] as char);
            bits &= (1 << bit_count) - 1;
        }
    }
    if bit_count > 0 {
        out.push(ALPHABET[(bits << (5 - bit_count)) as usize] as char);
    }
    out
}

// ── TOTP generation (RFC 6238) ────────────────────────────────────────────
fn totp(secret_bytes: &[u8], time_step: u64, digits: u32) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let counter = now / time_step;

    let mut mac = HmacSha1::new_from_slice(secret_bytes).expect("HMAC key");
    mac.update(&counter.to_be_bytes());
    let result = mac.finalize().into_bytes();

    let offset = (result[19] & 0x0f) as usize;
    let code = u32::from_be_bytes([
        result[offset] & 0x7f,
        result[offset + 1],
        result[offset + 2],
        result[offset + 3],
    ]);
    let code = code % 10u32.pow(digits);
    format!("{:0width$}", code, width = digits as usize)
}

fn totp_remaining(period: u64) -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    period - (now % period)
}

fn totp_progress(period: u64) -> f32 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let elapsed = now % period;
    1.0 - (elapsed as f32 / period as f32)
}

// ── Data model ────────────────────────────────────────────────────────────
#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct Account {
    issuer: String,
    label: String,
    secret_base32: String,
}

impl Account {
    fn totp_code(&self) -> String {
        match base32_decode(&self.secret_base32) {
            Ok(bytes) => totp(&bytes, TOTP_PERIOD, TOTP_DIGITS),
            Err(_) => "??????".to_owned(),
        }
    }
}

// ── Application state ─────────────────────────────────────────────────────
struct TotpApp {
    accounts: Vec<Account>,
    data_path: PathBuf,

    // Add dialog
    show_add_dialog: bool,
    add_secret: String,
    add_issuer: String,
    add_label: String,
    toast: Option<(String, Instant)>,
    add_error: String,

    // Import dialog
    show_import_dialog: bool,
    import_text: String,
    import_error: String,

    // Export dialog
    show_export_dialog: bool,
    export_text: String,

}

impl TotpApp {
    fn new() -> Self {
        let data_path = Self::default_data_path();
        let accounts = Self::load_accounts(&data_path);
        Self {
            accounts,
            data_path,
            show_add_dialog: false,
            add_secret: String::new(),
            add_issuer: String::new(),
            add_label: String::new(),
            add_error: String::new(),
            show_import_dialog: false,
            import_text: String::new(),
            import_error: String::new(),
            show_export_dialog: false,
            export_text: String::new(),
            toast: None,
        }
    }

    fn default_data_path() -> PathBuf {
        std::env::current_dir().unwrap_or_default().join(DATA_FILE)
    }

    fn load_accounts(path: &PathBuf) -> Vec<Account> {
        if let Ok(data) = std::fs::read_to_string(path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    fn save_accounts(&self) {
        if let Ok(data) = serde_json::to_string_pretty(&self.accounts) {
            let _ = std::fs::write(&self.data_path, &data);
        }
    }

    fn show_toast(&mut self, msg: &str) {
        self.toast = Some((msg.to_owned(), Instant::now()));
    }

    /// Parse `otpauth://totp/...?secret=...&issuer=...` URI
    fn try_parse_otpauth(input: &str) -> Option<(String, String, String)> {
        let input = input.trim();
        if !input.starts_with("otpauth://totp/") {
            return None;
        }
        let rest = input.strip_prefix("otpauth://totp/")?;
        let (label_part, query) = rest.split_once('?')?;
        let mut issuer = String::new();
        let mut secret = String::new();
        let mut label = String::new();

        // label_part is "Issuer:Account" or just "Account"
        if let Some((iss, acc)) = label_part.split_once(':') {
            issuer = url_decode(iss);
            label = url_decode(acc);
        } else {
            label = url_decode(label_part);
        }

        for pair in query.split('&') {
            if let Some((k, v)) = pair.split_once('=') {
                match k {
                    "secret" => secret = v.to_uppercase(),
                    "issuer" if issuer.is_empty() => issuer = url_decode(v),
                    _ => {}
                }
            }
        }

        if secret.is_empty() {
            return None;
        }
        Some((issuer, label, secret))
    }

    fn add_account(&mut self, issuer: &str, label: &str, secret_base32: &str) {
        let account = Account {
            issuer: issuer.to_owned(),
            label: label.to_owned(),
            secret_base32: secret_base32.to_uppercase(),
        };
        self.accounts.push(account);
        self.save_accounts();
    }

    fn delete_account(&mut self, index: usize) {
        if index < self.accounts.len() {
            self.accounts.remove(index);
            self.save_accounts();
        }
    }

    fn export_json(&self) -> String {
        serde_json::to_string_pretty(&self.accounts).unwrap_or_default()
    }

    fn import_json(&mut self, json: &str) -> Result<usize, String> {
        let accounts: Vec<Account> =
            serde_json::from_str(json).map_err(|e| format!("Parse error: {}", e))?;
        if accounts.is_empty() {
            return Err("No accounts found".to_owned());
        }
        for acc in &accounts {
            if acc.secret_base32.trim().is_empty() {
                return Err("Account with empty secret".to_owned());
            }
        }
        let count = accounts.len();
        self.accounts.extend(accounts);
        self.save_accounts();
        Ok(count)
    }
}

fn url_decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                out.push(byte as char);
            } else {
                out.push('%');
                out.push_str(&hex);
            }
        } else if c == '+' {
            out.push(' ');
        } else {
            out.push(c);
        }
    }
    out
}

// ── eframe App ────────────────────────────────────────────────────────────
impl eframe::App for TotpApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Tick repaint for countdown
        ctx.request_repaint_after(Duration::from_millis(200));

        // ── Top bar ───────────────────────────────────────────────────────
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            egui::Frame::default()
                .fill(egui::Color32::from_rgb(20, 20, 30))
                .inner_margin(egui::Margin::symmetric(16.0, 10.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.heading("🔐  2FA 验证器");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("📥 导入").clicked() {
                                self.show_import_dialog = true;
                                self.import_text.clear();
                                self.import_error.clear();
                            }
                            if ui.button("📤 导出").clicked() {
                                self.export_text = self.export_json();
                                self.show_export_dialog = true;
                            }
                            if ui.button("＋ 添加").clicked() {
                                self.show_add_dialog = true;
                                self.add_secret.clear();
                                self.add_issuer.clear();
                                self.add_label.clear();
                                self.add_error.clear();
                            }
                        });
                    });
                });
        });

        // ── Account list ──────────────────────────────────────────────────
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.accounts.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(120.0);
                    ui.label("还没有添加账户");
                    ui.label("点击「＋ 添加」或「📥 导入」开始");
                });
            } else {
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        let mut delete_idx: Option<usize> = None;
                        for (i, account) in self.accounts.iter().enumerate() {
                            let code = account.totp_code();
                            let remaining = totp_remaining(TOTP_PERIOD);
                            let progress = totp_progress(TOTP_PERIOD);

                            egui::Frame::default()
                                .fill(egui::Color32::from_rgb(30, 30, 40))
                                .rounding(8.0)
                                .stroke(egui::Stroke::new(
                                    1.0,
                                    egui::Color32::from_rgb(50, 50, 65),
                                ))
                                .inner_margin(egui::Margin::symmetric(16.0, 14.0))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        // Left: issuer + label
                                        ui.vertical(|ui| {
                                            ui.label(
                                                egui::RichText::new(&account.issuer)
                                                    .size(14.0)
                                                    .color(egui::Color32::from_rgb(
                                                        200, 200, 210,
                                                    )),
                                            );
                                            if !account.label.is_empty() {
                                                ui.label(
                                                    egui::RichText::new(&account.label)
                                                        .size(11.0)
                                                        .color(egui::Color32::from_rgb(
                                                            120, 120, 140,
                                                        )),
                                                );
                                            }
                                        });
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                // Delete
                                                if ui
                                                    .button("🗑")
                                                    .on_hover_text("删除")
                                                    .clicked()
                                                {
                                                    delete_idx = Some(i);
                                                }
                                            },
                                        );
                                    });

                                    ui.add_space(8.0);

                                    // TOTP code row
                                    ui.horizontal(|ui| {
                                        let formatted = format!(
                                            "{} {}",
                                            &code[..3],
                                            &code[3..]
                                        );
                                        ui.label(
                                            egui::RichText::new(formatted)
                                                .size(34.0)
                                                .family(egui::FontFamily::Monospace)
                                                .color(egui::Color32::from_rgb(220, 220, 240)),
                                        );

                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                // Copy button
                                                if ui
                                                    .button("📋")
                                                    .on_hover_text("复制")
                                                    .clicked()
                                                {
                                                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                                        let _ = clipboard.set_text(&code);
                                                        self.show_toast("已复制到剪贴板");
                                                    }
                                                }

                                                ui.add_space(12.0);

                                                // Countdown circle
                                                let circle_size = 32.0;
                                                let (rect, _) = ui.allocate_exact_size(
                                                    egui::vec2(circle_size, circle_size),
                                                    egui::Sense::hover(),
                                                );
                                                let painter = ui.painter();
                                                let center = rect.center();
                                                let radius = circle_size / 2.0 - 2.0;

                                                // Background ring
                                                painter.circle_stroke(
                                                    center,
                                                    radius,
                                                    egui::Stroke::new(
                                                        3.0,
                                                        egui::Color32::from_rgb(50, 50, 60),
                                                    ),
                                                );

                                                // Progress arc
                                                let color = if progress < 0.17 {
                                                    egui::Color32::from_rgb(220, 50, 50)
                                                } else if progress < 0.33 {
                                                    egui::Color32::from_rgb(220, 180, 50)
                                                } else {
                                                    egui::Color32::from_rgb(50, 200, 80)
                                                };

                                                painter.circle_stroke(
                                                    center,
                                                    radius,
                                                    egui::Stroke::new(3.0, color),
                                                );

                                                // Remaining text
                                                painter.text(
                                                    center,
                                                    egui::Align2::CENTER_CENTER,
                                                    format!("{}", remaining),
                                                    egui::FontId::proportional(9.0),
                                                    egui::Color32::from_rgb(160, 160, 180),
                                                );
                                            },
                                        );
                                    });
                                });
                            ui.add_space(6.0);
                        }

                        // Defer deletion to avoid borrow conflict
                        if let Some(idx) = delete_idx {
                            self.delete_account(idx);
                        }
                    });
            }
        });

        // ── Toast (auto-dismiss after 2s) ─────────────────────────────────
        if let Some((msg, start)) = &self.toast {
            if start.elapsed() < Duration::from_secs(2) {
                let alpha = (1.0 - (start.elapsed().as_secs_f32() / 2.0)).min(1.0);
                egui::Area::new("toast")
                    .anchor(egui::Align2::CENTER_TOP, (0.0, 60.0))
                    .show(ctx, |ui| {
                        ui.with_alpha(alpha, |ui| {
                            egui::Frame::default()
                                .fill(egui::Color32::from_rgba_premultiplied(30, 30, 40, 220))
                                .rounding(6.0)
                                .inner_margin(egui::Margin::symmetric(24.0, 12.0))
                                .show(ui, |ui| {
                                    ui.label(
                                        egui::RichText::new(msg.as_str())
                                            .color(egui::Color32::WHITE)
                                            .size(14.0),
                                    );
                                });
                        });
                    });
            } else {
                self.toast = None;
            }
        }

        // ── Add dialog ────────────────────────────────────────────────────
        if self.show_add_dialog {
            let mut closed = false;
            egui::Window::new("添加账户")
                .resizable(false)
                .collapsible(false)
                .open(&mut self.show_add_dialog)
                .show(ctx, |ui| {
                    let mut from_uri = false;
                    ui.label("粘贴密钥或 otpauth:// 链接：");
                    if ui
                        .add(
                            egui::TextEdit::multiline(&mut self.add_secret)
                                .desired_rows(2)
                                .desired_width(320.0)
                                .hint_text("JBSWY3DPEHPK3PXP 或 otpauth://totp/..."),
                        )
                        .changed()
                    {
                        // Try auto-parse
                        if let Some((iss, lab, sec)) =
                            Self::try_parse_otpauth(&self.add_secret)
                        {
                            self.add_issuer = iss;
                            self.add_label = lab;
                            self.add_secret = sec;
                            from_uri = true;
                        }
                    }

                    if !from_uri {
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.label("发行方：");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.add_issuer)
                                    .desired_width(200.0)
                                    .hint_text("GitHub"),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("账户：");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.add_label)
                                    .desired_width(200.0)
                                    .hint_text("user@example.com"),
                            );
                        });
                    }

                    ui.add_space(8.0);
                    if !self.add_error.is_empty() {
                        ui.label(
                            egui::RichText::new(&self.add_error)
                                .color(egui::Color32::RED)
                                .size(12.0),
                        );
                    }

                    ui.horizontal(|ui| {
                        if ui.button("取消").clicked() {
                            closed = true;
                            self.show_add_dialog = false;
                        }
                        if ui.button("添加").clicked() {
                            let secret = self.add_secret.trim().to_uppercase();
                            // Clean the secret
                            let secret: String = secret.chars().filter(|c| !c.is_whitespace()).collect();

                            if secret.is_empty() {
                                self.add_error = "请输入密钥".to_owned();
                            } else if base32_decode(&secret).is_err() {
                                self.add_error = "密钥格式无效（需要 Base32 编码）".to_owned();
                            } else {
                                // If URI parse didn't set issuer, use default
                                if self.add_issuer.trim().is_empty() {
                                    self.add_issuer = "未知".to_owned();
                                }
                                self.add_account(
                                    self.add_issuer.trim(),
                                    self.add_label.trim(),
                                    &secret,
                                );
                                self.show_toast("账户已添加");
                                closed = true;
                                self.show_add_dialog = false;
                            }
                        }
                    });
                });
            if closed {
                self.show_add_dialog = false;
            }
        }

        // ── Import dialog ─────────────────────────────────────────────────
        if self.show_import_dialog {
            egui::Window::new("导入账户")
                .resizable(false)
                .collapsible(false)
                .open(&mut self.show_import_dialog)
                .show(ctx, |ui| {
                    ui.label("粘贴 JSON 备份内容：");
                    ui.add(
                        egui::TextEdit::multiline(&mut self.import_text)
                            .desired_rows(8)
                            .desired_width(360.0),
                    );
                    if !self.import_error.is_empty() {
                        ui.label(
                            egui::RichText::new(&self.import_error)
                                .color(egui::Color32::RED)
                                .size(12.0),
                        );
                    }
                    ui.horizontal(|ui| {
                        if ui.button("取消").clicked() {
                            self.show_import_dialog = false;
                        }
                        if ui.button("导入").clicked() {
                            match self.import_json(self.import_text.trim()) {
                                Ok(count) => {
                                    self.show_toast(&format!("成功导入 {} 个账户", count));
                                    self.show_import_dialog = false;
                                }
                                Err(e) => {
                                    self.import_error = e;
                                }
                            }
                        }
                    });
                });
        }

        // ── Export dialog ─────────────────────────────────────────────────
        if self.show_export_dialog {
            egui::Window::new("导出账户")
                .resizable(false)
                .collapsible(false)
                .open(&mut self.show_export_dialog)
                .show(ctx, |ui| {
                    ui.label("复制以下 JSON 内容以备份：");
                    ui.add(
                        egui::TextEdit::multiline(&mut self.export_text)
                            .desired_rows(8)
                            .desired_width(360.0),
                    );
                    if ui.button("复制并关闭").clicked() {
                        if let Ok(mut clipboard) = arboard::Clipboard::new() {
                            let _ = clipboard.set_text(&self.export_text);
                        }
                        self.show_toast("已复制到剪贴板");
                        self.show_export_dialog = false;
                    }
                });
        }
    }
}

// ── Entry point ───────────────────────────────────────────────────────────
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([480.0, 640.0])
            .with_min_inner_size([360.0, 400.0])
            .with_title("2FA 验证器"),
        ..Default::default()
    };

    eframe::run_native(
        "2FA 验证器",
        options,
        Box::new(|_cc| Box::new(TotpApp::new())),
    )
}
