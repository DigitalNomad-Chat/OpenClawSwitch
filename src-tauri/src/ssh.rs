// SSH 閺夆晜绮庨埢鍏兼交閻愭潙澶嶉柡宥囶焾缁烘儳螣閳ヨ櫕鍋?
// 濞达綀娉曢弫?ssh2 crate 閻庡湱鍋熼獮?SSH 閺夆晝鍋炵敮鎾媌娴ｇ瓔鍚囬悹鍥︾筏缁辨繈鏌呭宕囩畺 channel 闁告稒鍨濋幎銈夊箥瑜戦、鎴犫偓鍦仧楠炲洭寮崶锔筋偨闁瑰灝绉崇紞?

use obfstr::obfstr as s;
use serde::{Deserialize, Serialize};
use ssh2::Session;
use std::collections::HashSet;
use std::fs;
use std::io::Read;
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::State;

// ============================================================================
// Known Hosts 缂佺媴绱曢幃?
// ============================================================================

fn get_known_hosts_path() -> PathBuf {
    s! { let openclaw_dir = ".openclaw"; }
    s! { let known_hosts = "known_hosts"; }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(openclaw_dir)
        .join(known_hosts)
}

fn load_known_hosts() -> HashSet<String> {
    let path = get_known_hosts_path();
    if !path.exists() {
        return HashSet::new();
    }

    fs::read_to_string(&path)
        .ok()
        .map(|content| {
            content
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| line.trim().to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn save_known_host(fingerprint: &str) -> Result<(), String> {
    let path = get_known_hosts_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("闁告帗绋戠紓鎾绘儎椤旇偐绉垮鎯扮簿鐟? {}", e))?;
    }

    let mut hosts = load_known_hosts();
    hosts.insert(fingerprint.to_string());

    let content = hosts.into_iter().collect::<Vec<_>>().join("\n");
    fs::write(&path, content).map_err(|e| format!("闁告劖鐟ラ崣鍡涘棘閸ワ附顐藉鎯扮簿鐟? {}", e))?;

    Ok(())
}

fn is_host_known(fingerprint: &str) -> bool {
    load_known_hosts().contains(fingerprint)
}

// ============================================================================
// 缂佇偉顕ч悗椋庘偓瑙勭煯缁?
// ============================================================================

/// SSH 閻犱降鍊涢惁澶愬棘閻熸壆纭€
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SshAuthMode {
    Password,
    PrivateKey,
}

/// SSH 閺夆晝鍋炵敮鎾煀瀹ュ洨鏋?
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SshProfile {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_mode: SshAuthMode,
    pub password: Option<String>,
    pub key_path: Option<String>,
}

/// 闁圭娲ㄥЧ妤佺┍閳╁啩绱?
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FingerprintInfo {
    pub sha256: String,
    pub md5: String,
    pub host: String,
    pub is_known: bool,
}

/// 閺夆晜绮庨埢濂稿棘閸ワ附顐介柡澶涚磿濞?
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteFileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
}

/// 闂佹澘绉堕悿鍡涘棘閸ワ附顐介柟鍏肩矌閸屻劎绱掗幘瀵镐函
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigSearchResult {
    pub path: String,
    pub file_name: String,
    pub dir_path: String,
}

// ============================================================================
// SSH 閺夆晝鍋炵敮瀵哥不閿涘嫭銈為柛?
// ============================================================================

/// SSH 閺夆晝鍋炵敮鎾偐閼哥鍋撴笟濠勭闁归晲鐒﹀﹢?Session 闁告粌鐭佺换娑㈠箳閵夈倓绻嗛柟?
struct SshConnection {
    session: Session,
    #[allow(dead_code)]
    host: String,
    username: String,
}

/// SSH 缂佺媴绱曢幃濠囧闯椤帞绀夌紒鎹愭硶閳昏偐鈧怀顦崣蹇涙儍閸曨喚绠鹃柟鎭掑劤婵悂骞€娴ｅ壊鍟囬柛?
pub struct SshManager {
    connection: Mutex<Option<SshConnection>>,
}

impl SshManager {
    pub fn new() -> Self {
        SshManager {
            connection: Mutex::new(None),
        }
    }
}

/// 閻忓繐妫欑€垫氨鐥悷鎵憻闁煎搫銈归弳鐔虹磼閸曨剛澹愮€殿喖绻愮€靛弶绋夐崫鍕）闁稿浚鍙€缁绘﹢宕氱捄铏规憻缂佹缂氱憰?
fn format_fingerprint_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(":")
}

/// 閻忓繐妫欑€垫氨鐥悷鎵憻闁煎搫銈归弳鐔虹磼閸曨剛澹愮€殿喖绻愮€靛弶绋?Base64闁挎稑婀疕A-256 閻㈩垰鎽滈弫銈夊冀閻撳海纭€闁?
fn format_fingerprint_base64(bytes: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = Vec::new();
    for chunk in bytes.chunks(3) {
        match chunk.len() {
            3 => {
                result.push(CHARS[(chunk[0] >> 2) as usize]);
                result.push(CHARS[(((chunk[0] & 0x03) << 4) | (chunk[1] >> 4)) as usize]);
                result.push(CHARS[(((chunk[1] & 0x0f) << 2) | (chunk[2] >> 6)) as usize]);
                result.push(CHARS[(chunk[2] & 0x3f) as usize]);
            }
            2 => {
                result.push(CHARS[(chunk[0] >> 2) as usize]);
                result.push(CHARS[(((chunk[0] & 0x03) << 4) | (chunk[1] >> 4)) as usize]);
                result.push(CHARS[((chunk[1] & 0x0f) << 2) as usize]);
                result.push(b'=');
            }
            1 => {
                result.push(CHARS[(chunk[0] >> 2) as usize]);
                result.push(CHARS[((chunk[0] & 0x03) << 4) as usize]);
                result.push(b'=');
                result.push(b'=');
            }
            _ => {}
        }
    }
    String::from_utf8(result).unwrap_or_default()
}

// ============================================================================
// Tauri 闁告稒鍨濋幎?
// ============================================================================

/// 鐎点倛娅ｉ悵?SSH 閺夆晝鍋炵敮鎾嵁閹壆绠查柛銉у仦鐎垫氨鐥柅娑楃箚闁诡収鍨界槐娆戜焊濮橆厽寮撻悹浣靛€涢惁澶愭晬?
#[tauri::command]
pub fn ssh_connect(
    manager: State<SshManager>,
    host: String,
    port: u16,
    username: String,
) -> Result<FingerprintInfo, String> {
    // 闁稿繐鐗婇弻鍥ь嚕閳ь剙顔忛崣澶嬬畳閺夆晝鍋炵敮?
    {
        let mut conn = manager.connection.lock().map_err(|e| format!("{}{}", s!("闂佸じ绶氶弫濠勬嫚? "), e))?;
        *conn = None;
    }

    let addr = format!("{}:{}", host, port);
    let tcp = TcpStream::connect(&addr).map_err(|e| format!("閺夆晝鍋炵敮瀛樺緞鏉堫偉袝: {}", e))?;
    tcp.set_nodelay(true).ok();

    let mut session = Session::new().map_err(|e| format!("闁告帗绋戠紓鎾村濮樺磭妯堝鎯扮簿鐟? {}", e))?;
    session.set_tcp_stream(tcp);
    session
        .handshake()
        .map_err(|e| format!("闁圭儵鍓濇晶婊勫緞鏉堫偉袝: {}", e))?;

    // 闁圭儵鍓濇晶婊呪偓鐟版湰閸ㄦ岸宕ユ惔鈥虫櫃閻犱礁澧介悿鍡欐惥閸涱喗顦ч柛?keepalive闁挎稑鐭傛导鈺呭礂瀹ュ懎鍙￠柟鐢靛瑜版瑩骞嶇€ｎ亝瀚?SFTP 闁告帗绻傞～鎰板礌?
    session.set_timeout(0); // 濞戞挸绉烽鏇犵磾?session 閻℃帒鎳忓錂炴晬瀹€鈧弫閬嶅触閸曨剚鎯欏ù锝嗙矎閸ゆ粎鎮扮仦鎯т粯闁?
    session.set_keepalive(true, 30); // 婵?30 缂佸甯掕ぐ錂炴焻?keepalive 闂傚啫寮堕娑㈠棘椤擄紕绠?

    // 闁兼儳鍢茶ぐ鍥ㄧ▔缂佹ɑ绨氶柟绋挎川濮?
    let md5 = format_fingerprint_hex(
        &session
            .host_key_hash(ssh2::HashType::Md5)
            .ok_or(s!("failed to read SSH host key MD5 fingerprint").to_string())?,
    );

    let sha256_bytes = session
        .host_key_hash(ssh2::HashType::Sha256)
        .ok_or(s!("failed to read SSH host key SHA-256 fingerprint").to_string())?;
    let sha256 = format!("{}{}", s!("SHA256:"), format_fingerprint_base64(sha256_bytes));

    // 婵☆偀鍋撻柡?known_hosts
    let is_known = is_host_known(&sha256);

    let fingerprint = FingerprintInfo {
        sha256,
        md5,
        host: host.clone(),
        is_known,
    };

    // 濞ｅ洦绻傞悺銊﹀濮樺磭妯堥柨娑樼墕閻ㄥ寮甸鍥跺悋閻犲洣绶ょ槐?
    let mut conn = manager.connection.lock().map_err(|e| format!("{}{}", s!("闂佸じ绶氶弫濠勬嫚? "), e))?;
    *conn = Some(SshConnection {
        session,
        host,
        username,
    });

    Ok(fingerprint)
}

/// 濞ｅ洦绻傞悺銊︾▔缂佹ɑ绨氶柟绋挎川濮规宕?known_hosts
#[tauri::command]
pub fn ssh_save_fingerprint(fingerprint: String) -> Result<(), String> {
    save_known_host(&fingerprint)
}

/// 濞达綀娉曢弫銈団偓闈涙闉栨粎鎷嬮妶鍫㈡
#[tauri::command]
pub fn ssh_auth_password(
    manager: State<SshManager>,
    password: String,
) -> Result<(), String> {
    let conn = manager.connection.lock().map_err(|e| format!("{}{}", s!("闂佸じ绶氶弫濠勬嫚? "), e))?;
    let conn = conn.as_ref().ok_or(s!("SSH connection not found").to_string())?;

    conn.session
        .userauth_password(&conn.username, &password)
        .map_err(|e| format!("閻庨潧妫涢悥婊呮媼閵堝牏妲堝鎯扮簿鐟? {}", e))?;

    if !conn.session.authenticated() {
        return Err(s!("SSH password authentication did not complete").to_string());
    }

    Ok(())
}

/// 濞达綀娉曢弫銈囩矓娓氣偓閹告粎鎷嬮妶鍫㈡
#[tauri::command]
pub fn ssh_auth_key(
    manager: State<SshManager>,
    key_path: String,
    passphrase: Option<String>,
) -> Result<(), String> {
    let conn = manager.connection.lock().map_err(|e| format!("{}{}", s!("闂佸じ绶氶弫濠勬嫚? "), e))?;
    let conn = conn.as_ref().ok_or(s!("SSH connection not found").to_string())?;

    let key = Path::new(&key_path);
    if !key.exists() {
        return Err(format!("缂佸绶氶幐婊堝棘閸ワ附顐藉☉鎾崇Т閵°劑宕? {}", key_path));
    }

    conn.session
        .userauth_pubkey_file(
            &conn.username,
            None,
            key,
            passphrase.as_deref(),
        )
        .map_err(|e| format!("缂佸绶氶幐婊呮媼閵堝牏妲堝鎯扮簿鐟? {}", e))?;

    if !conn.session.authenticated() {
        return Err(s!("SSH public key authentication did not complete").to_string());
    }

    Ok(())
}

/// 闁哄偆鍘肩槐?SSH 閺夆晝鍋炵敮?
#[tauri::command]
pub fn ssh_disconnect(manager: State<SshManager>) -> Result<(), String> {
    let mut conn = manager.connection.lock().map_err(|e| format!("{}{}", s!("闂佸じ绶氶弫濠勬嫚? "), e))?;
    if let Some(c) = conn.as_ref() {
        s! { let client_disconnect = "client disconnect"; }
        let _ = c.session.disconnect(None, client_disconnect, None);
    }
    *conn = None;
    Ok(())
}

/// 闁兼儳鍢茶ぐ鍥ㄦ交閻愭潙澶嶉柣妯垮煐閳?
#[tauri::command]
pub fn ssh_get_status(manager: State<SshManager>) -> Result<bool, String> {
    let conn = manager.connection.lock().map_err(|e| format!("{}{}", s!("闂佸じ绶氶弫濠勬嫚? "), e))?;
    Ok(conn
        .as_ref()
        .map(|c| c.session.authenticated())
        .unwrap_or(false))
}

/// 闁告帗顨呴崵顓熸交濠婂應鏌ら柣鈺婂枛缂嶅秹鏁嶉崼銉㈠亾濮樺磭绠?ls 闁告稒鍨濋幎銈夋晬?
#[tauri::command]
pub fn ssh_list_dir(
    manager: State<SshManager>,
    path: String,
) -> Result<Vec<RemoteFileEntry>, String> {
    let conn = manager.connection.lock().map_err(|e| format!("{}{}", s!("闂佸じ绶氶弫濠勬嫚? "), e))?;
    let conn = conn.as_ref().ok_or(s!("SSH connection not found").to_string())?;

    if !conn.session.authenticated() {
        return Err(s!("SSH authentication required").to_string());
    }

    s! { let dot = "."; }
    s! { let dotdot = ".."; }
    s! { let total = "total"; }

    // 濞达綀娉曢弫?ls -la 闁兼儳鍢茶ぐ鍥儎椤旇偐绉块柛鎺擃殙閵嗗啴鏁?-time-style 缁绢収鍠曠换姘綇閹惧啿姣夐柡宥囧帶缁扁剝绋夐埀顒勬嚊?
    let cmd = format!(
        "ls -la --time-style=+%s '{}' 2>/dev/null || ls -la '{}' 2>/dev/null",
        path, path
    );
    let output = exec_remote_command(&conn.session, &cmd)?;

    let mut entries = Vec::new();
    for line in output.lines().skip(1) {
        // 閻犲搫鐤囩换?"total" 閻?
        let line = line.trim();
        if line.is_empty() || line.starts_with(total) {
            continue;
        }
        if let Some(entry) = parse_ls_line(line, &path) {
            if entry.name != dot && entry.name != dotdot {
                entries.push(entry);
            }
        }
    }

    // 闁烩晩鍠栫紞宥夊捶閵娿儱顤呴柨娑樻湰閺嬪啯绂掔捄鐑樿含闁告艾鍑界槐婵嬪触閸曨喖娈伴柟绋款槸閹洜绮旈悧鍫濈瑩閹?
    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.cmp(&b.name))
    });

    Ok(entries)
}

/// 閻熸瑱绲鹃悗?ls -la 閺夊牊鎸搁崵顓㈡儍閸曨偄绀嬮悶?
fn parse_ls_line(line: &str, parent_path: &str) -> Option<RemoteFileEntry> {
    // ls -la 闁哄秶鍘х槐? drwxr-xr-x 2 root root 4096 1234567890 dirname
    // 闁? -rw-r--r-- 1 root root 1234 Jan 01 12:00 filename
    let parts: Vec<&str> = line.splitn(9, char::is_whitespace)
        .filter(|s| !s.is_empty())
        .collect();

    if parts.len() < 7 {
        return None;
    }

    let is_dir = parts[0].starts_with('d');
    let size: u64 = parts[4].parse().unwrap_or(0);

    // 闁哄倸娲ｅ▎銏ゅ触瀹ュ棙笑闁哄牃鍋撻柛姘凹缁斿瓨绋夐鍕憻婵炲牏顣槐娆撳矗椤栨繂鍘撮柛鏍ф噹閹绮氶悜妯煎闁?
    // 閻庣敻鈧稓鑹?--time-style=+%s 闁哄秶鍘х槐锟犳晬鐏炵晫鎽熸繛鍫濈仛閺嗙喐绋?7+
    // 閻庣敻鈧稓鑹鹃柡宥呮搐閸ｎ垶寮介悡搴ｇ闁挎稑鑻悺褍鈻撻崹顐ｆ濞?8+
    let name = if parts.len() >= 9 {
        parts[8..].join(" ")
    } else if parts.len() >= 8 {
        parts[7..].join(" ")
    } else {
        parts[parts.len() - 1].to_string()
    };

    // 閻犲搫鐤囩换鍐箔閿曗偓瑜板潡鏌ч悙顒€澶嶉柣?-> 闂侇喓鍔岄崹?
    s! { let arrow = " -> "; }
    let name = name.split(arrow).next().unwrap_or(&name).to_string();

    if name.is_empty() {
        return None;
    }

    Some(RemoteFileEntry {
        path: format!("{}/{}", parent_path.trim_end_matches('/'), name),
        name,
        is_dir,
        size,
    })
}

/// 閻犲洩顕цぐ鍥ㄦ交濠婂應鏌ら柡鍌氭矗濞嗐垽宕橀崨顓у晣闁挎稑鐗撻埀顒佷亢缁?cat 闁告稒鍨濋幎銈夋晬?
#[tauri::command]
pub fn ssh_read_file(
    manager: State<SshManager>,
    path: String,
) -> Result<String, String> {
    let conn = manager.connection.lock().map_err(|e| format!("{}{}", s!("闂佸じ绶氶弫濠勬嫚? "), e))?;
    let conn = conn.as_ref().ok_or(s!("SSH connection not found").to_string())?;

    if !conn.session.authenticated() {
        return Err(s!("SSH authentication required").to_string());
    }

    let cmd = format!("cat '{}'", path.replace('\'', "'\\''"));
    exec_remote_command(&conn.session, &cmd)
}

/// 闁告劖鐟ラ崣鍡樻交濠婂應鏌ら柡鍌氭矗濞嗐垽鏁嶉崼銉㈠亾濮樺磭绠?channel stdin闁?
#[tauri::command]
pub fn ssh_write_file(
    manager: State<SshManager>,
    path: String,
    content: String,
) -> Result<(), String> {
    let conn = manager.connection.lock().map_err(|e| format!("{}{}", s!("闂佸じ绶氶弫濠勬嫚? "), e))?;
    let conn = conn.as_ref().ok_or(s!("SSH connection not found").to_string())?;

    if !conn.session.authenticated() {
        return Err(s!("SSH authentication required").to_string());
    }

    let escaped_path = path.replace('\'', "'\\''");
    let cmd = format!("cat > '{}'", escaped_path);

    let mut channel = conn.session
        .channel_session()
        .map_err(|e| format!("闁告帗绋戠紓鎾绘焻濮樻湹澹曞鎯扮簿鐟? {}", e))?;
    channel
        .exec(&cmd)
        .map_err(|e| format!("闁圭瑳鍡╂斀闁告稒鍨濋幎銈嗗緞鏉堫偉袝: {}", e))?;

    use std::io::Write;
    channel
        .write_all(content.as_bytes())
        .map_err(|e| format!("闁告劖鐟ラ崣鍡樺緞鏉堫偉袝: {}", e))?;

    // 闁稿繑濞婂Λ?stdin 闂侇偅姘ㄩ悡鈩冩交濠婂應鏌?cat 闁告稒鍨濋幎銈囩磼閹惧瓨灏?
    channel.send_eof().map_err(|e| format!("闁告瑦鍨块埀?EOF 濠㈡儼绮剧憴? {}", e))?;
    channel.wait_eof().ok();
    channel.wait_close().ok();

    let exit = channel.exit_status().unwrap_or(-1);
    if exit != 0 {
        return Err(format!("闁告劖鐟ラ崣鍡樺緞鏉堫偉袝闁挎稑鐭傞埀顑藉亾闁告垼娅ｉ悥? {}", exit));
    }

    Ok(())
}

/// 閺夆晜绮庨埢濂告煂瀹ュ懏鍎欑紓鍐╁灥閸?
#[tauri::command]
pub fn ssh_restart_gateway(
    manager: State<SshManager>,
) -> Result<String, String> {
    s! { let action_restart = "restart"; }
    ssh_run_gateway_command(manager, action_restart)
}

/// 閺夆晜绮庨埢濂稿触椤栨艾袟缂傚啯鍨甸崣?
#[tauri::command]
pub fn ssh_start_gateway(
    manager: State<SshManager>,
) -> Result<String, String> {
    s! { let action_start = "start"; }
    ssh_run_gateway_command(manager, action_start)
}

/// 閺夆晜绮庨埢濂稿磻濠婂嫷鍓剧紓鍐╁灥閸?
#[tauri::command]
pub fn ssh_stop_gateway(
    manager: State<SshManager>,
) -> Result<String, String> {
    s! { let action_stop = "stop"; }
    ssh_run_gateway_command(manager, action_stop)
}

/// 閺夆晜绮庨埢濂稿箥瑜戦、鎴犵磾閹存繂褰犻柛娑欏灊閹躲倝鏁嶉崸鐨宎rt / stop / restart闁?
fn ssh_run_gateway_command(
    manager: State<SshManager>,
    action: &str,
) -> Result<String, String> {
    let conn = manager.connection.lock().map_err(|e| format!("{}{}", s!("闂佸じ绶氶弫濠勬嫚? "), e))?;
    let conn = conn.as_ref().ok_or(s!("SSH connection not found").to_string())?;

    if !conn.session.authenticated() {
        return Err(s!("SSH authentication required").to_string());
    }

    s! { let exit_marker = "__EXIT__"; }

    let cmd = format!(
        r#"
tmp=$(mktemp /tmp/clawlite-gateway-{}.XXXXXX 2>/dev/null || echo /tmp/clawlite-gateway-{}.log)
openclaw gateway {} >"$tmp" 2>&1
code=$?
echo "__EXIT__$code"
cat "$tmp" 2>/dev/null || true
rm -f "$tmp" 2>/dev/null || true
"#,
        action, action, action
    );

    let output = exec_remote_command(&conn.session, &cmd)?;
    let exit_code = output
        .lines()
        .find(|line| line.starts_with(exit_marker))
        .and_then(|line| line.trim_start_matches(exit_marker).parse::<i32>().ok())
        .unwrap_or(-1);

    let details = output
        .lines()
        .filter(|line| !line.starts_with(exit_marker))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string();

    s! { let completed_msg = "remote gateway {} command completed"; }
    s! { let failed_msg = "remote gateway {} command failed"; }

    if exit_code == 0 {
        if details.is_empty() {
            return Ok(format!("{}", completed_msg.replace("{}", action)));
        }
        return Ok(details);
    }

    Err(if details.is_empty() {
        format!("{}", failed_msg.replace("{}", action))
    } else {
        format!("{}: {}", failed_msg.replace("{}", action), details)
    })
}

/// 閺夆晜绮庨埢濂稿磻閵夈儲銈ｆ信顐熷亾闁哄被鍎荤槐?27.0.0.1:18789闁?
#[tauri::command]
pub fn ssh_health_check(
    manager: State<SshManager>,
) -> Result<bool, String> {
    let conn = manager.connection.lock().map_err(|e| format!("{}{}", s!("闂佸じ绶氶弫濠勬嫚? "), e))?;
    let conn = conn.as_ref().ok_or(s!("SSH connection not found").to_string())?;

    if !conn.session.authenticated() {
        return Err(s!("SSH authentication required").to_string());
    }

    s! { let healthy = "__HEALTHY__"; }

    let cmd = r#"
if curl -fsS --max-time 3 http://127.0.0.1:18789 >/dev/null 2>&1 || \
   wget -q --timeout=3 -O- http://127.0.0.1:18789 >/dev/null 2>&1; then
  echo "__HEALTHY__"
else
  echo "__UNHEALTHY__"
fi
"#;

    let output = exec_remote_command(&conn.session, cmd)?;
    Ok(output.contains(healthy))
}

/// 闁煎浜滄慨鈺呭箹濠婂喚鍤㈤弶鈺傜矌閳诲ジ寮靛鍛潳闁革絻鍔嬬粭錂炴儍?OpenClaw 闂佹澘绉堕悿鍡涘棘閸ワ附顐介柨娑樼墦閳ь剚淇虹换?test -f 闁告稒鍨濋幎銈夋晬?
#[tauri::command]
pub fn ssh_search_config(
    manager: State<SshManager>,
) -> Result<Vec<ConfigSearchResult>, String> {
    let conn = manager.connection.lock().map_err(|e| format!("{}{}", s!("闂佸じ绶氶弫濠勬嫚? "), e))?;
    let conn = conn.as_ref().ok_or(s!("SSH connection not found").to_string())?;

    if !conn.session.authenticated() {
        return Err(s!("SSH authentication required").to_string());
    }

    s! { let openclaw_dir = ".openclaw"; }
    s! { let config_openclaw = ".config/openclaw"; }
    s! { let etc_openclaw = "/etc/openclaw"; }
    s! { let opt_openclaw = "/opt/openclaw"; }
    s! { let root_home = "/root"; }
    s! { let openclaw_json = "openclaw.json"; }
    s! { let openclaw_yaml = "openclaw.yaml"; }
    s! { let openclaw_yml = "openclaw.yml"; }

    // 闁兼儳鍢茶ぐ鍥ㄦ交濠婂應鏌?home 闁烩晩鍠栫紞?
    let home = get_remote_home(&conn.session).unwrap_or_else(|| root_home.to_string());

    let search_dirs = [
        format!("{}/{}", home, openclaw_dir),
        home.clone(),
        format!("{}/{}", home, config_openclaw),
        etc_openclaw.to_string(),
        opt_openclaw.to_string(),
    ];

    let config_names = [
        openclaw_json,
        openclaw_yaml,
        openclaw_yml,
    ];

    // 闁哄瀚紓鎾诲箥瑜版帒娅ゆ信顐熷亾闁哄被鍎遍幊鈩冪?
    let mut checks = Vec::new();
    for dir in &search_dirs {
        for name in &config_names {
            let p = format!("{}/{}", dir.trim_end_matches('/'), name);
            checks.push(format!("test -f '{}' && echo '{}'", p, p));
        }
    }
    let cmd = checks.join("; ");

    let mut results = Vec::new();
    if let Ok(output) = exec_remote_command(&conn.session, &cmd) {
        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() || !line.starts_with('/') {
                continue;
            }
            let path = Path::new(line);
            if let Some(file_name) = path.file_name() {
                let dir_path = path
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();
                results.push(ConfigSearchResult {
                    path: line.to_string(),
                    file_name: file_name.to_string_lossy().to_string(),
                    dir_path,
                });
            }
        }
    }

    Ok(results)
}

// ============================================================================
// 閺夊牆鎳庢修顏堝礄閼恒儲娈?
// ============================================================================

/// 闂侇偅淇虹换鍐箥瑜戦、鎴炴交濠婂應鏌ら柛娑欏灊閹躲倝鎳㈠畡鏉跨悼闁活潿鍔嶉崺?home 闁烩晩鍠栫紞?
fn get_remote_home(session: &Session) -> Option<String> {
    s! { let echo_home = "echo $HOME"; }
    let output = exec_remote_command(session, echo_home).ok()?;
    let home = output.trim().to_string();
    if home.is_empty() { None } else { Some(home) }
}

/// 闁圭瑳鍡╂斀閺夆晜绮庨埢濂稿川閹存帗濮㈡鐐村劶缁绘垿宕堕悙瀵割伕闁荤偛妫楅幃妤呮儍閸曨喚缈婚柛?
fn exec_remote_command(session: &Session, cmd: &str) -> Result<String, String> {
    let mut channel = session
        .channel_session()
        .map_err(|e| format!("闁告帗绋戠紓鎾绘焻濮樻湹澹曞鎯扮簿鐟? {}", e))?;
    channel
        .exec(cmd)
        .map_err(|e| format!("闁圭瑳鍡╂斀闁告稒鍨濋幎銈嗗緞鏉堫偉袝: {}", e))?;

    let mut output = String::new();
    channel
        .read_to_string(&mut output)
        .map_err(|e| format!("閻犲洩顕цぐ鍥ㄦ綇閹惧啿姣夊鎯扮簿鐟? {}", e))?;

    // 闁圭儤甯為埞?stderr
    let mut stderr_buf = String::new();
    let _ = channel.stderr().read_to_string(&mut stderr_buf);

    let _ = channel.send_eof();
    let _ = channel.wait_eof();
    let _ = channel.wait_close();

    Ok(strip_ansi_escapes(&output))
}

/// 婵炴挸鎳愰幃?ANSI/OSC/CSI 缂備礁鐗忛顒佹姜椤戣法鐤呴幖鏉戠箰閸?
fn strip_ansi_escapes(input: &str) -> String {
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut result = Vec::with_capacity(len);
    let mut i = 0;

    while i < len {
        if bytes[i] == 0x1b {
            if i + 1 < len {
                match bytes[i + 1] {
                    b'[' => {
                        // CSI: ESC [ ... (缂備礁鐗婇娑欑@-~)
                        i += 2;
                        while i < len && !(bytes[i] >= b'@' && bytes[i] <= b'~') {
                            i += 1;
                        }
                        if i < len { i += 1; }
                    }
                    b']' => {
                        // OSC: ESC ] ... (缂備礁鐗婇娑欑BEL 闁?ESC \)
                        i += 2;
                        while i < len {
                            if bytes[i] == 0x07 { i += 1; break; }
                            if bytes[i] == 0x1b && i + 1 < len && bytes[i + 1] == b'\\' {
                                i += 2; break;
                            }
                            i += 1;
                        }
                    }
                    _ => { i += 2; }
                }
            } else {
                i += 1;
            }
        } else if bytes[i] == 0x07 || bytes[i] == 0x0e || bytes[i] == 0x0f {
            i += 1;
        } else {
            result.push(bytes[i]);
            i += 1;
        }
    }

    String::from_utf8(result).unwrap_or_else(|_| input.to_string())
}

// ============================================================================
// SSH 閺夆晜绮庨埢濂告偝椤栨凹鏆旀信顐熷亾婵?
// ============================================================================

/// 闂侇偅淇虹换?SSH 婵☆偀鍋撴繛鏉戭儓缁绘瑧绮欑€ｎ偅绠涢柛鏂衡偓铏彜闁?OpenClaw 闁绘粠鍨伴。銊╂偐閼哥鍋?
#[tauri::command]
pub fn ssh_check_environment(
    manager: State<SshManager>,
) -> Result<crate::installer::EnvironmentStatus, String> {
    let conn = manager
        .connection
        .lock()
        .map_err(|e| format!("{}{}", s!("闂佸じ绶氶弫濠勬嫚? "), e))?;
    let conn = conn.as_ref().ok_or(s!("SSH connection not found").to_string())?;
    if !conn.session.authenticated() {
        return Err(s!("SSH authentication required").to_string());
    }

    s! { let not_installed = "__NOT_INSTALLED__"; }
    s! { let not_found = "__NOT_FOUND__"; }
    s! { let macos = "macos"; }
    s! { let windows = "windows"; }
    s! { let linux = "linux"; }
    s! { let aarch64 = "aarch64"; }
    s! { let x86_64 = "x86_64"; }
    s! { let unknown = "unknown"; }

    // 濞戞挴鍋撴繛鍡忓墲閳ь儸鍕挃閻炴稑鑻ˇ鍧楀级閳╁喚姊炬繛鏉戭儏閹斥剝绂掗妶蹇曠闁告垵绻愰惃?round-trip
    let detect_script = r#"
echo "===OPENCLAW_VERSION==="
openclaw --version 2>/dev/null || echo "__NOT_INSTALLED__"
echo "===OPENCLAW_PATH==="
which openclaw 2>/dev/null || echo "__NOT_FOUND__"
echo "===NODE_VERSION==="
node --version 2>/dev/null || echo "__NOT_INSTALLED__"
echo "===GIT_VERSION==="
git --version 2>/dev/null || echo "__NOT_INSTALLED__"
echo "===FNM_VERSION==="
fnm --version 2>/dev/null || echo "__NOT_INSTALLED__"
echo "===SYSTEM_INFO==="
uname -s && uname -m && basename "$SHELL" 2>/dev/null || echo "unknown"
echo "===END==="
"#;

    let output = exec_remote_command(&conn.session, detect_script)?;

    // 閻熸瑱绲鹃悗鑺ユ綇閹惧啿姣?
    let get_section = |key: &str| -> String {
        let start_marker = format!("==={}===", key);
        let lines: Vec<&str> = output.lines().collect();
        let mut capture = false;
        let mut result = Vec::new();
        for line in &lines {
            if line.contains(&start_marker) {
                capture = true;
                continue;
            }
            if capture && line.contains("===") {
                break;
            }
            if capture {
                result.push(line.trim());
            }
        }
        result.join("\n").trim().to_string()
    };

    // OpenClaw
    let oc_version_raw = get_section("OPENCLAW_VERSION");
    let oc_path_raw = get_section("OPENCLAW_PATH");
    let oc_installed = !oc_version_raw.contains(not_installed);
    let openclaw = crate::installer::OpenClawStatus {
        installed: oc_installed,
        version: if oc_installed {
            Some(oc_version_raw)
        } else {
            None
        },
        path: if oc_path_raw.contains(not_found) {
            None
        } else {
            Some(oc_path_raw)
        },
        compatibility: None, // SSH 远程检测暂不计算兼容性
    };

    // Node.js
    let node_raw = get_section("NODE_VERSION");
    let node_installed = !node_raw.contains(not_installed);
    let node_version = if node_installed {
        Some(node_raw.trim_start_matches('v').to_string())
    } else {
        None
    };
    let node_major: u32 = node_version
        .as_deref()
        .and_then(|v| v.split('.').next())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let node = crate::installer::NodeStatus {
        installed: node_installed,
        version: node_version,
        meets_requirement: node_major >= 22,
    };

    // Git
    let git_raw = get_section("GIT_VERSION");
    let git_installed = !git_raw.contains(not_installed);
    let git = crate::installer::GitStatus {
        installed: git_installed,
        version: if git_installed {
            Some(git_raw.replace("git version ", "").trim().to_string())
        } else {
            None
        },
    };

    // fnm
    let fnm_raw = get_section("FNM_VERSION");
    let fnm_installed = !fnm_raw.contains(not_installed);
    let fnm = crate::installer::FnmStatus {
        installed: fnm_installed,
        version: if fnm_installed {
            Some(fnm_raw.replace("fnm ", "").trim().to_string())
        } else {
            None
        },
    };

    // 缂侇垵宕电划鐑樼┍閳╁啩绱?
    let sys_raw = get_section("SYSTEM_INFO");
    let sys_lines: Vec<&str> = sys_raw.lines().collect();
    let os_name = sys_lines.first().unwrap_or(&linux).to_lowercase();
    let os = if os_name.contains("darwin") {
        macos
    } else if os_name.contains("windows") || os_name.contains("mingw") {
        windows
    } else {
        linux
    }
    .to_string();
    let arch_raw = sys_lines.get(1).unwrap_or(&x86_64).to_lowercase();
    let arch = if arch_raw.contains("aarch64") || arch_raw.contains("arm64") {
        aarch64
    } else {
        x86_64
    }
    .to_string();
    let shell = sys_lines
        .get(2)
        .unwrap_or(&"sh")
        .to_string();

    let system = crate::installer::SystemInfo { os, arch, shell };

    Ok(crate::installer::EnvironmentStatus {
        openclaw,
        node,
        git,
        fnm,
        system,
        network_region: unknown.to_string(),
    })
}
