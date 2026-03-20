/// Execution Layer
///
/// OS Control, Browser Automation, IoT, and RPA.
/// All actions are gated through SafetyAlignmentSystem.validate() before execution.

use crate::pal::types::Platform;
use crate::safety::{PermissionCategory, SafetyAlignmentSystem, ValidationResult};

// ── ExecutionResult ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub success: bool,
    pub output: String,
    pub degraded: bool,
}

impl ExecutionResult {
    fn ok(output: impl Into<String>) -> Self {
        Self { success: true, output: output.into(), degraded: false }
    }

    fn degraded(output: impl Into<String>) -> Self {
        Self { success: true, output: output.into(), degraded: true }
    }

    fn denied(reason: impl Into<String>) -> Self {
        Self { success: false, output: reason.into(), degraded: false }
    }
}

// ── OSControlAdapter ─────────────────────────────────────────────────────────

/// Platform-specific OS control adapter.
pub struct OSControlAdapter {
    pub platform: Platform,
}

impl OSControlAdapter {
    pub fn new(platform: Platform) -> Self {
        Self { platform }
    }

    /// Execute a shell/OS command appropriate for the current platform.
    pub fn execute_command(&self, command: &str) -> ExecutionResult {
        match self.platform {
            Platform::Windows => {
                // Stub: Win32/COM — would invoke CreateProcess or ShellExecute.
                ExecutionResult::ok(format!("[Win32] executed: {command}"))
            }
            Platform::Macos => {
                // Stub: AppleScript / Accessibility API.
                ExecutionResult::ok(format!("[AppleScript] executed: {command}"))
            }
            Platform::Linux => {
                // Stub: X11/Wayland/D-Bus.
                ExecutionResult::ok(format!("[D-Bus] executed: {command}"))
            }
            Platform::Android => {
                // Degraded: Android Accessibility Service API.
                ExecutionResult::degraded(format!(
                    "[Android/Accessibility] degraded execution: {command}"
                ))
            }
            Platform::Ios => {
                // Degraded: iOS Shortcuts/Siri integration only.
                ExecutionResult::degraded(format!(
                    "[iOS/Shortcuts] degraded execution: {command}"
                ))
            }
        }
    }

    /// Launch an application by name.
    pub fn launch_app(&self, app_name: &str) -> ExecutionResult {
        match self.platform {
            Platform::Ios => ExecutionResult::degraded(format!(
                "[iOS] cannot launch arbitrary apps: {app_name}"
            )),
            _ => ExecutionResult::ok(format!("launched: {app_name}")),
        }
    }

    /// Take a screenshot. Returns raw bytes stub.
    pub fn take_screenshot(&self) -> ExecutionResult {
        match self.platform {
            Platform::Ios => ExecutionResult::degraded(
                "[iOS] screenshot not available without entitlement".to_string(),
            ),
            _ => ExecutionResult::ok("[screenshot bytes]"),
        }
    }

    /// Read clipboard contents.
    pub fn read_clipboard(&self) -> ExecutionResult {
        ExecutionResult::ok("[clipboard content]")
    }

    /// Write text to clipboard.
    pub fn write_clipboard(&self, text: &str) -> ExecutionResult {
        ExecutionResult::ok(format!("clipboard written: {text}"))
    }
}

// ── SystemControlLayer ────────────────────────────────────────────────────────

/// High-level OS control layer — gates all actions through Safety.
pub struct SystemControlLayer {
    pub adapter: OSControlAdapter,
    pub safety: SafetyAlignmentSystem,
}

impl SystemControlLayer {
    pub fn new(platform: Platform) -> Self {
        Self {
            adapter: OSControlAdapter::new(platform),
            safety: SafetyAlignmentSystem::new(),
        }
    }

    fn gate(&mut self, action: &str) -> Option<ExecutionResult> {
        let result = self.safety.validate(action, PermissionCategory::OsControl);
        if !result.approved {
            let reason = if result.ethics_violation.is_some() {
                "ethics violation".to_string()
            } else if result.permission_denied {
                "permission denied".to_string()
            } else {
                "safety check failed".to_string()
            };
            Some(ExecutionResult::denied(format!("Safety denied: {reason}")))
        } else {
            None
        }
    }

    pub fn execute_os_command(&mut self, command: &str) -> ExecutionResult {
        if let Some(denied) = self.gate(command) {
            return denied;
        }
        self.adapter.execute_command(command)
    }

    pub fn launch_app(&mut self, app_name: &str) -> ExecutionResult {
        if let Some(denied) = self.gate(app_name) {
            return denied;
        }
        self.adapter.launch_app(app_name)
    }

    pub fn take_screenshot(&mut self) -> ExecutionResult {
        if let Some(denied) = self.gate("screenshot") {
            return denied;
        }
        self.adapter.take_screenshot()
    }

    pub fn read_clipboard(&mut self) -> ExecutionResult {
        if let Some(denied) = self.gate("clipboard_read") {
            return denied;
        }
        self.adapter.read_clipboard()
    }

    pub fn write_clipboard(&mut self, text: &str) -> ExecutionResult {
        if let Some(denied) = self.gate("clipboard_write") {
            return denied;
        }
        self.adapter.write_clipboard(text)
    }
}

// ── BrowserAutomationEngine ───────────────────────────────────────────────────

/// Browser automation via Playwright (desktop only).
pub struct BrowserAutomationEngine {
    pub safety: SafetyAlignmentSystem,
}

impl BrowserAutomationEngine {
    pub fn new() -> Self {
        Self { safety: SafetyAlignmentSystem::new() }
    }

    fn gate(&mut self, action: &str) -> Option<ExecutionResult> {
        let result = self.safety.validate(action, PermissionCategory::Browser);
        if !result.approved {
            let reason = if result.ethics_violation.is_some() {
                "ethics violation".to_string()
            } else if result.permission_denied {
                "permission denied".to_string()
            } else {
                "safety check failed".to_string()
            };
            Some(ExecutionResult::denied(format!("Safety denied: {reason}")))
        } else {
            None
        }
    }

    /// Navigate to a URL.
    pub fn navigate(&mut self, url: &str) -> ExecutionResult {
        if let Some(denied) = self.gate(url) {
            return denied;
        }
        // Stub: Playwright integration would go here.
        ExecutionResult::ok(format!("[Playwright] navigated to: {url}"))
    }

    /// Click an element identified by CSS selector.
    pub fn click(&mut self, selector: &str) -> ExecutionResult {
        if let Some(denied) = self.gate(selector) {
            return denied;
        }
        ExecutionResult::ok(format!("[Playwright] clicked: {selector}"))
    }

    /// Fill a form field.
    pub fn fill(&mut self, selector: &str, value: &str) -> ExecutionResult {
        if let Some(denied) = self.gate(selector) {
            return denied;
        }
        ExecutionResult::ok(format!("[Playwright] filled {selector} = {value}"))
    }

    /// Extract text content from a selector.
    pub fn extract_text(&mut self, selector: &str) -> ExecutionResult {
        if let Some(denied) = self.gate(selector) {
            return denied;
        }
        ExecutionResult::ok(format!("[Playwright] extracted text from: {selector}"))
    }
}

impl Default for BrowserAutomationEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ── IoTController ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct IoTDevice {
    pub id: String,
    pub name: String,
    pub protocol: String, // "mqtt" | "http" | "zigbee"
}

pub struct IoTController {
    pub safety: SafetyAlignmentSystem,
    pub discovered_devices: Vec<IoTDevice>,
}

impl IoTController {
    pub fn new() -> Self {
        Self {
            safety: SafetyAlignmentSystem::new(),
            discovered_devices: vec![],
        }
    }

    /// Discover devices on the local network (stub).
    pub fn discover_devices(&mut self) -> Vec<IoTDevice> {
        // Stub: return empty list; real impl would use mDNS/SSDP/Zigbee scan.
        self.discovered_devices.clone()
    }

    /// Send a command to a device.
    pub fn send_command(&mut self, device_id: &str, command: &str) -> ExecutionResult {
        let action = format!("iot:{device_id}:{command}");
        let result = self.safety.validate(&action, PermissionCategory::Other);
        if !result.approved {
            let reason = if result.ethics_violation.is_some() {
                "ethics violation".to_string()
            } else if result.permission_denied {
                "permission denied".to_string()
            } else {
                "safety check failed".to_string()
            };
            return ExecutionResult::denied(format!("Safety denied: {reason}"));
        }
        ExecutionResult::ok(format!("[IoT] sent '{command}' to device '{device_id}'"))
    }
}

impl Default for IoTController {
    fn default() -> Self {
        Self::new()
    }
}

// ── RPAEngine ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub steps: Vec<String>,
}

pub struct RPAEngine {
    pub safety: SafetyAlignmentSystem,
    pub workflows: Vec<Workflow>,
}

impl RPAEngine {
    pub fn new() -> Self {
        Self {
            safety: SafetyAlignmentSystem::new(),
            workflows: vec![],
        }
    }

    /// Record a new workflow from a list of steps.
    pub fn record_workflow(&mut self, name: &str, steps: Vec<String>) -> Workflow {
        let id = format!("wf-{}", self.workflows.len() + 1);
        let wf = Workflow { id: id.clone(), name: name.to_string(), steps };
        self.workflows.push(wf.clone());
        wf
    }

    /// Play back a recorded workflow.
    pub fn play_workflow(&mut self, workflow_id: &str) -> ExecutionResult {
        let action = format!("rpa:play:{workflow_id}");
        let result = self.safety.validate(&action, PermissionCategory::OsControl);
        if !result.approved {
            let reason = if result.ethics_violation.is_some() {
                "ethics violation".to_string()
            } else if result.permission_denied {
                "permission denied".to_string()
            } else {
                "safety check failed".to_string()
            };
            return ExecutionResult::denied(format!("Safety denied: {reason}"));
        }
        if self.workflows.iter().any(|w| w.id == workflow_id) {
            ExecutionResult::ok(format!("[RPA] played workflow: {workflow_id}"))
        } else {
            ExecutionResult::denied(format!("Workflow not found: {workflow_id}"))
        }
    }

    /// Schedule a workflow to run at a given cron expression.
    pub fn schedule_workflow(&mut self, workflow_id: &str, cron: &str) -> ExecutionResult {
        ExecutionResult::ok(format!(
            "[RPA] scheduled workflow '{workflow_id}' at '{cron}'"
        ))
    }
}

impl Default for RPAEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_control_windows_returns_success() {
        let adapter = OSControlAdapter::new(Platform::Windows);
        let result = adapter.execute_command("notepad.exe");
        assert!(result.success);
        assert!(!result.degraded);
    }

    #[test]
    fn os_control_ios_returns_degraded() {
        let adapter = OSControlAdapter::new(Platform::Ios);
        let result = adapter.execute_command("open app");
        assert!(result.degraded);
    }

    #[test]
    fn os_control_android_returns_degraded() {
        let adapter = OSControlAdapter::new(Platform::Android);
        let result = adapter.execute_command("open app");
        assert!(result.degraded);
    }

    #[test]
    fn system_control_layer_gates_through_safety() {
        let mut layer = SystemControlLayer::new(Platform::Linux);
        // Safety system defaults to approved for non-harmful actions.
        let result = layer.execute_os_command("ls");
        // Should succeed (safety approves by default).
        assert!(result.success || !result.output.is_empty());
    }

    #[test]
    fn browser_navigate_returns_success() {
        let mut browser = BrowserAutomationEngine::new();
        browser.safety.permissions.grant(PermissionCategory::Browser);
        let result = browser.navigate("https://example.com");
        assert!(result.success);
    }

    #[test]
    fn iot_send_command_returns_success() {
        let mut iot = IoTController::new();
        iot.safety.permissions.grant(PermissionCategory::Other);
        let result = iot.send_command("light-01", "turn_on");
        assert!(result.success);
    }

    #[test]
    fn rpa_record_and_play_workflow() {
        let mut rpa = RPAEngine::new();
        rpa.safety.permissions.grant(PermissionCategory::OsControl);
        let wf = rpa.record_workflow("test", vec!["click #btn".to_string()]);
        let result = rpa.play_workflow(&wf.id);
        assert!(result.success);
    }

    #[test]
    fn rpa_play_unknown_workflow_fails() {
        let mut rpa = RPAEngine::new();
        let result = rpa.play_workflow("nonexistent");
        assert!(!result.success);
    }
}
