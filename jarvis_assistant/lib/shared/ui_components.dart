import 'package:flutter/material.dart';

class ChatMessage {
  final String id;
  final String text;
  final bool isUser;
  final DateTime timestamp;

  const ChatMessage({
    required this.id,
    required this.text,
    required this.isUser,
    required this.timestamp,
  });
}

class ConversationHistory extends StatelessWidget {
  final List<ChatMessage> messages;
  final ScrollController? scrollController;

  const ConversationHistory({
    super.key,
    required this.messages,
    this.scrollController,
  });

  @override
  Widget build(BuildContext context) {
    return ListView.builder(
      controller: scrollController,
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      itemCount: messages.length,
      itemBuilder: (context, index) {
        final msg = messages[index];
        return _MessageBubble(message: msg);
      },
    );
  }
}

class _MessageBubble extends StatelessWidget {
  final ChatMessage message;

  const _MessageBubble({required this.message});

  @override
  Widget build(BuildContext context) {
    final isUser = message.isUser;
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Column(
        crossAxisAlignment:
            isUser ? CrossAxisAlignment.end : CrossAxisAlignment.start,
        children: [
          if (!isUser)
            const Padding(
              padding: EdgeInsets.only(left: 4, bottom: 2),
              child: Text(
                'Ayanokoji',
                style: TextStyle(
                  fontSize: 11,
                  color: Colors.blueGrey,
                  fontWeight: FontWeight.w600,
                ),
              ),
            ),
          Container(
            constraints: BoxConstraints(
              maxWidth: MediaQuery.of(context).size.width * 0.72,
            ),
            padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
            decoration: BoxDecoration(
              color: isUser
                  ? Colors.blueAccent.shade700
                  : const Color(0xFF2A2A2A),
              borderRadius: BorderRadius.only(
                topLeft: const Radius.circular(16),
                topRight: const Radius.circular(16),
                bottomLeft: Radius.circular(isUser ? 16 : 4),
                bottomRight: Radius.circular(isUser ? 4 : 16),
              ),
            ),
            child: Text(
              message.text,
              style: const TextStyle(fontSize: 14, color: Colors.white),
            ),
          ),
        ],
      ),
    );
  }
}

class SettingsPanel extends StatelessWidget {
  const SettingsPanel({super.key});

  @override
  Widget build(BuildContext context) {
    return const Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          'Settings',
          style: TextStyle(fontSize: 20, fontWeight: FontWeight.bold),
        ),
        SizedBox(height: 12),
        Text('Settings coming soon.'),
      ],
    );
  }
}

// ── PersonaSettingsPanel ──────────────────────────────────────────────────────

class PersonaSettings {
  final double intensity;   // [0.0, 1.0]
  final double verbosity;   // [0.0, 1.0]

  const PersonaSettings({required this.intensity, required this.verbosity});
}

class PersonaSettingsPanel extends StatefulWidget {
  final PersonaSettings initial;
  final ValueChanged<PersonaSettings> onChanged;

  const PersonaSettingsPanel({
    super.key,
    required this.initial,
    required this.onChanged,
  });

  @override
  State<PersonaSettingsPanel> createState() => _PersonaSettingsPanelState();
}

class _PersonaSettingsPanelState extends State<PersonaSettingsPanel> {
  late double _intensity;
  late double _verbosity;

  @override
  void initState() {
    super.initState();
    _intensity = widget.initial.intensity;
    _verbosity = widget.initial.verbosity;
  }

  void _notify() {
    widget.onChanged(PersonaSettings(intensity: _intensity, verbosity: _verbosity));
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text('Persona Settings',
            style: TextStyle(fontSize: 16, fontWeight: FontWeight.bold)),
        const SizedBox(height: 12),
        _SliderRow(
          label: 'Intensity',
          value: _intensity,
          onChanged: (v) {
            setState(() => _intensity = v);
            _notify();
          },
        ),
        _SliderRow(
          label: 'Verbosity',
          value: _verbosity,
          onChanged: (v) {
            setState(() => _verbosity = v);
            _notify();
          },
        ),
      ],
    );
  }
}

class _SliderRow extends StatelessWidget {
  final String label;
  final double value;
  final ValueChanged<double> onChanged;

  const _SliderRow({
    required this.label,
    required this.value,
    required this.onChanged,
  });

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        SizedBox(width: 80, child: Text(label)),
        Expanded(
          child: Slider(
            value: value,
            min: 0.0,
            max: 1.0,
            divisions: 10,
            label: value.toStringAsFixed(1),
            onChanged: onChanged,
          ),
        ),
        SizedBox(
          width: 32,
          child: Text(value.toStringAsFixed(1),
              style: const TextStyle(fontSize: 12)),
        ),
      ],
    );
  }
}

// ── VoiceButton ───────────────────────────────────────────────────────────────

enum VoiceButtonState { idle, listening, processing }

class VoiceButton extends StatelessWidget {
  final VoiceButtonState state;
  final VoidCallback onPressed;

  const VoiceButton({
    super.key,
    required this.state,
    required this.onPressed,
  });

  @override
  Widget build(BuildContext context) {
    final (icon, color, tooltip) = switch (state) {
      VoiceButtonState.idle       => (Icons.mic_none, Colors.blueGrey, 'Start listening'),
      VoiceButtonState.listening  => (Icons.mic, Colors.redAccent, 'Listening…'),
      VoiceButtonState.processing => (Icons.hourglass_top, Colors.orange, 'Processing…'),
    };

    return Tooltip(
      message: tooltip,
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 200),
        decoration: BoxDecoration(
          shape: BoxShape.circle,
          color: color.withOpacity(0.15),
        ),
        child: IconButton(
          icon: Icon(icon, color: color),
          iconSize: 28,
          onPressed: state == VoiceButtonState.processing ? null : onPressed,
        ),
      ),
    );
  }
}

// ── BatteryBadge ──────────────────────────────────────────────────────────────

/// Displays a reduced-capability badge when battery is low.
class BatteryBadge extends StatelessWidget {
  /// Battery level in [0.0, 1.0]. Pass null when not applicable (desktop plugged in).
  final double? batteryLevel;

  const BatteryBadge({super.key, this.batteryLevel});

  @override
  Widget build(BuildContext context) {
    final level = batteryLevel;
    if (level == null || level > 0.2) return const SizedBox.shrink();

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
      decoration: BoxDecoration(
        color: Colors.orange.shade800,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Icon(Icons.battery_alert, size: 14, color: Colors.white),
          const SizedBox(width: 4),
          Text(
            'Low power — reduced capability',
            style: Theme.of(context)
                .textTheme
                .labelSmall
                ?.copyWith(color: Colors.white),
          ),
        ],
      ),
    );
  }
}

// ── GoalTracker ───────────────────────────────────────────────────────────────

class GoalItem {
  final String id;
  final String title;
  final double progress; // [0.0, 1.0]

  const GoalItem({required this.id, required this.title, required this.progress});
}

class GoalTracker extends StatelessWidget {
  final List<GoalItem> goals;

  const GoalTracker({super.key, required this.goals});

  @override
  Widget build(BuildContext context) {
    if (goals.isEmpty) {
      return const Padding(
        padding: EdgeInsets.all(16),
        child: Text('No active goals.', style: TextStyle(color: Colors.grey)),
      );
    }
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: goals.map((g) => _GoalRow(goal: g)).toList(),
    );
  }
}

class _GoalRow extends StatelessWidget {
  final GoalItem goal;
  const _GoalRow({required this.goal});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 6),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(goal.title, style: const TextStyle(fontSize: 13)),
          const SizedBox(height: 4),
          LinearProgressIndicator(
            value: goal.progress.clamp(0.0, 1.0),
            backgroundColor: Colors.grey.shade800,
            color: Colors.blueAccent,
          ),
        ],
      ),
    );
  }
}

// ── ProactiveSuggestionCard ───────────────────────────────────────────────────

class ProactiveSuggestion {
  final String id;
  final String message;
  final double urgency; // [0.0, 1.0]

  const ProactiveSuggestion({
    required this.id,
    required this.message,
    required this.urgency,
  });
}

class ProactiveSuggestionCard extends StatelessWidget {
  final ProactiveSuggestion suggestion;
  final VoidCallback onAccept;
  final VoidCallback onReject;

  const ProactiveSuggestionCard({
    super.key,
    required this.suggestion,
    required this.onAccept,
    required this.onReject,
  });

  @override
  Widget build(BuildContext context) {
    return Card(
      color: const Color(0xFF1E2A3A),
      margin: const EdgeInsets.symmetric(vertical: 4, horizontal: 8),
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(suggestion.message,
                style: const TextStyle(fontSize: 13, color: Colors.white)),
            const SizedBox(height: 8),
            Row(
              mainAxisAlignment: MainAxisAlignment.end,
              children: [
                TextButton(
                  onPressed: onReject,
                  child: const Text('Dismiss',
                      style: TextStyle(color: Colors.grey)),
                ),
                const SizedBox(width: 8),
                ElevatedButton(
                  onPressed: onAccept,
                  child: const Text('Accept'),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

// ── DashboardScreen ───────────────────────────────────────────────────────────

class DashboardScreen extends StatelessWidget {
  final List<GoalItem> goals;
  final List<ProactiveSuggestion> suggestions;

  const DashboardScreen({
    super.key,
    this.goals = const [],
    this.suggestions = const [],
  });

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Dashboard')),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text('Goals',
                style: TextStyle(fontSize: 16, fontWeight: FontWeight.bold)),
            const SizedBox(height: 8),
            GoalTracker(goals: goals),
            const SizedBox(height: 24),
            if (suggestions.isNotEmpty) ...[
              const Text('Suggestions',
                  style: TextStyle(fontSize: 16, fontWeight: FontWeight.bold)),
              const SizedBox(height: 8),
              ...suggestions.map((s) => ProactiveSuggestionCard(
                    suggestion: s,
                    onAccept: () {},
                    onReject: () {},
                  )),
            ],
          ],
        ),
      ),
    );
  }
}
