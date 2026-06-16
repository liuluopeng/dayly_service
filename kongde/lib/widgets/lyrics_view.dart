import 'dart:async';
import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:kongde/utils/lrc_parser.dart';

class LyricsView extends StatefulWidget {
  final List<LrcLine> lines;
  final Duration position;
  final Color textColor;
  final Color highlightColor;

  const LyricsView({
    super.key,
    required this.lines,
    required this.position,
    this.textColor = Colors.white54,
    this.highlightColor = Colors.white,
  });

  @override
  State<LyricsView> createState() => _LyricsViewState();
}

class _LyricsViewState extends State<LyricsView> {
  final ScrollController _scrollController = ScrollController();
  int _currentIndex = -1;
  bool _userScrolling = false;
  Timer? _resumeTimer;

  @override
  void didUpdateWidget(LyricsView oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.lines != oldWidget.lines) {
      _currentIndex = -1;
    }
    _updateCurrentIndex();
  }

  void _updateCurrentIndex() {
    if (widget.lines.isEmpty) return;

    int newIndex = -1;
    for (int i = widget.lines.length - 1; i >= 0; i--) {
      if (widget.position >= widget.lines[i].time) {
        newIndex = i;
        break;
      }
    }

    if (newIndex != _currentIndex) {
      _currentIndex = newIndex;
      if (!_userScrolling && _currentIndex >= 0) {
        _scrollToCurrent();
      }
    }
  }

  void _scrollToCurrent() {
    if (!_scrollController.hasClients) return;
    final targetOffset = (_currentIndex * 48.0) - 100.0;
    _scrollController.animateTo(
      targetOffset.clamp(0.0, _scrollController.position.maxScrollExtent),
      duration: const Duration(milliseconds: 300),
      curve: Curves.easeInOut,
    );
  }

  void _onUserScroll() {
    _userScrolling = true;
    _resumeTimer?.cancel();
    _resumeTimer = Timer(const Duration(seconds: 3), () {
      _userScrolling = false;
      if (_currentIndex >= 0) {
        _scrollToCurrent();
      }
    });
  }

  @override
  void dispose() {
    _resumeTimer?.cancel();
    _scrollController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (widget.lines.isEmpty) {
      return Center(
        child: Text(
          'lyrics.noLyrics'.tr,
          style: TextStyle(color: widget.textColor, fontSize: 16),
        ),
      );
    }

    return NotificationListener<ScrollNotification>(
      onNotification: (notification) {
        if (notification is UserScrollNotification) {
          _onUserScroll();
        }
        return false;
      },
      child: ListView.builder(
        controller: _scrollController,
        padding: const EdgeInsets.symmetric(vertical: 40),
        itemCount: widget.lines.length,
        itemBuilder: (context, index) {
          final isCurrent = index == _currentIndex;
          return Container(
            height: 48,
            alignment: Alignment.center,
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: Text(
              widget.lines[index].text,
              textAlign: TextAlign.center,
              style: TextStyle(
                color: isCurrent ? widget.highlightColor : widget.textColor,
                fontSize: isCurrent ? 18 : 15,
                fontWeight: isCurrent ? FontWeight.bold : FontWeight.normal,
              ),
            ),
          );
        },
      ),
    );
  }
}
