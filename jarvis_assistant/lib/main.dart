import 'package:flutter/material.dart';
import 'shared/ui_components.dart';

void main() {
  runApp(const JarvisApp());
}

class JarvisApp extends StatelessWidget {
  const JarvisApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Jarvis Assistant',
      theme: ThemeData.dark().copyWith(
        colorScheme: ColorScheme.fromSeed(
          seedColor: Colors.blueGrey,
          brightness: Brightness.dark,
        ),
        useMaterial3: true,
      ),
      home: const ChatScreen(),
    );
  }
}

class ChatScreen extends StatefulWidget {
  const ChatScreen({super.key});

  @override
  State<ChatScreen> createState() => _ChatScreenState();
}

class _ChatScreenState extends State<ChatScreen> {
  final List<ChatMessage> _messages = [];
  final TextEditingController _controller = TextEditingController();
  final ScrollController _scrollController = ScrollController();
  final ValueNotifier<bool> _isOffline = ValueNotifier(false);

  VoiceButtonState _voiceState = VoiceButtonState.idle;

  /// Simulated battery level. Null = desktop plugged in.
  double? _batteryLevel = 0.15; // demo: low battery

  @override
  void dispose() {
    _controller.dispose();
    _scrollController.dispose();
    _isOffline.dispose();
    super.dispose();
  }

  String _getResponse(String input) {
    return 'Logically speaking, I have processed your input.';
  }

  void _sendMessage() {
    final text = _controller.text.trim();
    if (text.isEmpty) return;

    final userMsg = ChatMessage(
      id: DateTime.now().millisecondsSinceEpoch.toString(),
      text: text,
      isUser: true,
      timestamp: DateTime.now(),
    );

    final assistantMsg = ChatMessage(
      id: '${DateTime.now().millisecondsSinceEpoch}_resp',
      text: _getResponse(text),
      isUser: false,
      timestamp: DateTime.now(),
    );

    setState(() {
      _messages.add(userMsg);
      _messages.add(assistantMsg);
    });

    _controller.clear();

    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (_scrollController.hasClients) {
        _scrollController.animateTo(
          _scrollController.position.maxScrollExtent,
          duration: const Duration(milliseconds: 300),
          curve: Curves.easeOut,
        );
      }
    });
  }

  void _onVoicePressed() {
    setState(() {
      _voiceState = _voiceState == VoiceButtonState.idle
          ? VoiceButtonState.listening
          : VoiceButtonState.idle;
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Jarvis'),
        actions: [
          // Battery-aware reduced-capability badge
          BatteryBadge(batteryLevel: _batteryLevel),
          const SizedBox(width: 8),
          // Offline indicator
          ValueListenableBuilder<bool>(
            valueListenable: _isOffline,
            builder: (_, offline, __) {
              if (!offline) return const SizedBox.shrink();
              return Padding(
                padding: const EdgeInsets.only(right: 8),
                child: Chip(
                  label: const Text(
                    'OFFLINE',
                    style: TextStyle(
                      color: Colors.white,
                      fontSize: 11,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                  backgroundColor: Colors.red.shade700,
                  padding: EdgeInsets.zero,
                  visualDensity: VisualDensity.compact,
                ),
              );
            },
          ),
        ],
      ),
      body: Column(
        children: [
          Expanded(
            child: ConversationHistory(
              messages: _messages,
              scrollController: _scrollController,
            ),
          ),
          _InputBar(
            controller: _controller,
            onSend: _sendMessage,
            voiceState: _voiceState,
            onVoicePressed: _onVoicePressed,
          ),
        ],
      ),
      floatingActionButton: ValueListenableBuilder<bool>(
        valueListenable: _isOffline,
        builder: (_, offline, __) {
          return FloatingActionButton(
            mini: true,
            tooltip: offline ? 'Go online' : 'Go offline',
            onPressed: () => _isOffline.value = !_isOffline.value,
            child: Icon(offline ? Icons.wifi_off : Icons.wifi),
          );
        },
      ),
    );
  }
}

class _InputBar extends StatelessWidget {
  final TextEditingController controller;
  final VoidCallback onSend;
  final VoiceButtonState voiceState;
  final VoidCallback onVoicePressed;

  const _InputBar({
    required this.controller,
    required this.onSend,
    required this.voiceState,
    required this.onVoicePressed,
  });

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: Padding(
        padding: const EdgeInsets.fromLTRB(12, 6, 8, 8),
        child: Row(
          children: [
            // Voice button
            VoiceButton(state: voiceState, onPressed: onVoicePressed),
            const SizedBox(width: 4),
            Expanded(
              child: TextField(
                controller: controller,
                textInputAction: TextInputAction.send,
                onSubmitted: (_) => onSend(),
                decoration: InputDecoration(
                  hintText: 'Message Jarvis...',
                  filled: true,
                  fillColor: const Color(0xFF1E1E1E),
                  contentPadding: const EdgeInsets.symmetric(
                    horizontal: 16,
                    vertical: 10,
                  ),
                  border: OutlineInputBorder(
                    borderRadius: BorderRadius.circular(24),
                    borderSide: BorderSide.none,
                  ),
                ),
              ),
            ),
            const SizedBox(width: 6),
            IconButton(
              icon: const Icon(Icons.send),
              onPressed: onSend,
              tooltip: 'Send',
            ),
          ],
        ),
      ),
    );
  }
}
