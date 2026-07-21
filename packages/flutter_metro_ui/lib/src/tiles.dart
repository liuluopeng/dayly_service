import 'package:flutter/material.dart';

enum TileSize {
  /// 1×1 方块
  medium,
  /// 2×1 宽条
  wide,
}

class MetroTile extends StatelessWidget {
  final IconData? icon;
  final String? label;
  final String? imageUrl;
  final Color? color;
  final TileSize size;
  final VoidCallback? onTap;

  const MetroTile({
    super.key,
    this.icon,
    this.label,
    this.imageUrl,
    this.color,
    this.size = TileSize.medium,
    this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    final bg = color ?? const Color(0xFF1A1A1A);
    return GestureDetector(
      onTap: onTap,
      child: AspectRatio(
        aspectRatio: size == TileSize.wide ? 2 : 1,
        child: Container(
          color: bg,
          child: Stack(
            children: [
              if (icon != null)
                Center(
                  child: Opacity(
                    opacity: 0.25,
                    child: Icon(icon, size: 48, color: Colors.white),
                  ),
                ),
              if (imageUrl != null)
                Positioned.fill(
                  child: Image.network(imageUrl!, fit: BoxFit.cover),
                ),
              if (label != null)
                Positioned(
                  left: 10, right: 10, bottom: 8,
                  child: Text(label!,
                    style: const TextStyle(fontSize: 12, fontWeight: FontWeight.w600, color: Colors.white),
                    maxLines: 2, overflow: TextOverflow.ellipsis,
                  ),
                ),
            ],
          ),
        ),
      ),
    );
  }
}
