# Flutter Metro UI

Windows Phone 10 / Metro UI style widgets and theme for Flutter.

## Usage

```yaml
dependencies:
  flutter_metro_ui:
    path: packages/flutter_metro_ui
```

```dart
import 'package:flutter_metro_ui/flutter_metro_ui.dart';

ThemeData theme = wp10Theme(dark: false);
```

## Components

- `wp10Theme()` — complete WP10 ThemeData
- `MetroTile` — square/wide live tile
- `MetroPivot` / `MetroPivotPages` — horizontal pivot navigation
- `MetroHub` / `MetroHubSection` — scrollable hub layout
