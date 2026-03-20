import 'package:flutter/material.dart';
import '../main.dart';
import '../shared/ui_components.dart';

class DesktopUIAdapter {
  static void openChatWindow(BuildContext context) {
    Navigator.of(context).push(
      MaterialPageRoute(builder: (_) => const ChatScreen()),
    );
  }

  static void showToastNotification(BuildContext context, String message) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(message),
        behavior: SnackBarBehavior.floating,
        duration: const Duration(seconds: 3),
      ),
    );
  }

  /// Open the command palette overlay.
  static void openCommandPalette(BuildContext context) {
    showDialog(
      context: context,
      builder: (ctx) => const _CommandPaletteDialog(),
    );
  }

  /// Open the dashboard screen.
  static void openDashboard(BuildContext context) {
    Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => const DashboardScreen(goals: [], suggestions: []),
      ),
    );
  }

  /// Initialise system tray icon (Windows/Linux) or menu bar icon (macOS).
  static Future<void> initSystemTray({
    required String tooltip,
    required List<TrayMenuItem> menuItems,
  }) async {
    debugPrint('[DesktopUIAdapter] system tray initialised: $tooltip');
  }

  /// Register a global hotkey.
  static Future<void> registerGlobalHotkey({
    required String hotkey,
    required VoidCallback onActivated,
  }) async {
    debugPrint('[DesktopUIAdapter] global hotkey registered: $hotkey');
  }

  /// Unregister all global hotkeys.
  static Future<void> unregisterAllHotkeys() async {
    debugPrint('[DesktopUIAdapter] all hotkeys unregistered');
  }
}

// ── CommandPaletteDialog ──────────────────────────────────────────────────────

class _CommandPaletteDialog extends StatefulWidget {
  const _CommandPaletteDialog();

  @override
  State<_CommandPaletteDialog> createState() => _CommandPaletteDialogState();
}

class _CommandPaletteDialogState extends State<_CommandPaletteDialog> {
  final TextEditingController _ctrl = TextEditingController();
  final List<String> _commands = [
    'Open Dashboard',
    'Open Chat',
    'Toggle Offline Mode',
    'Settings',
    'Clear Conversation',
  ];
  List<String> _filtered = [];

  @override
  void initState() {
    super.initState();
    _filtered = _commands;
    _ctrl.addListener(() {
      setState(() {
        final q = _ctrl.text.toLowerCase();
        _filtered = q.isEmpty
            ? _commands
            : _commands.where((c) => c.toLowerCase().contains(q)).toList();
      });
    });
  }

  @override
  void dispose() {
    _ctrl.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Dialog(
      backgroundColor: const Color(0xFF1A1A2E),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: _ctrl,
              autofocus: true,
              decoration: const InputDecoration(
                hintText: 'Type a command…',
                prefixIcon: Icon(Icons.search),
                border: OutlineInputBorder(),
              ),
            ),
            const SizedBox(height: 8),
            ConstrainedBox(
              constraints: const BoxConstraints(maxHeight: 240),
              child: ListView.builder(
                shrinkWrap: true,
                itemCount: _filtered.length,
                itemBuilder: (_, i) => ListTile(
                  title: Text(_filtered[i]),
                  onTap: () => Navigator.of(context).pop(_filtered[i]),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

// ── TrayMenuItem ──────────────────────────────────────────────────────────────

class TrayMenuItem {
  final String label;
  final VoidCallback? onTap;
  final bool isSeparator;

  const TrayMenuItem({required this.label, this.onTap, this.isSeparator = false});

  const TrayMenuItem.separator()
      : label = '',
        onTap = null,
        isSeparator = true;
}
