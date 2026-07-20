import 'package:get/get.dart';

class Messages extends Translations {
  @override
  Map<String, Map<String, String>> get keys => {
    'zh_CN': {
      // Common
      'common.cancel': '取消',
      'common.confirm': '确定',
      'common.delete': '删除',
      'common.save': '保存',
      'common.retry': '重试',
      'common.refresh': '刷新',
      'common.back': '返回',
      'common.scan': '扫描',
      'common.play': '播放',
      'common.copy': '复制',
      'common.copied': '已复制到剪贴板',
      'common.loading': '加载中',
      'common.loadFailed': '加载失败',
      'common.loadFailedWith': '加载失败: @error',
      'common.required': '必填',
      'common.invalid': '无效',
      'common.name': '名称',
      'common.address': '地址',
      'common.port': '端口',
      'common.username': '用户名',
      'common.password': '密码',
      'common.emptyState': '暂无数据',
      'common.retryOrRefresh': '请稍后刷新',
      'common.search': '查询',
      'common.history': '历史',
      'common.hide': '隐藏',
      'common.highFrequency': '高频查询',
      'common.recentSearch': '最近查询',
      'common.noHighFrequencyData': '暂无高频查询数据',
      'common.noRecentData': '暂无最近查询数据',
      'common.searchError': '查询出错: @error',
      'common.searchCount': '查询 @count 次',

      // App
      'app.exitHint': '再按一次返回键退出应用',

      // Navigation
      'nav.home': '首页',
      'nav.menu': '菜单',
      'nav.profile': '我',

      // Home
      'home.title': '首页',
      'home.listenMusic': '听歌',

      // Profile
      'profile.title': '我',
      'profile.settings': '设置',

      // Menu
      'menu.title': '应用菜单',
      'menu.wubiQuery': '五笔查询',
      'menu.videoLibrary': '视频库',
      'menu.imageGallery': '图片库',
      'menu.melatoninMovies': 'Melatonin 电影',
      'menu.scan': '扫码',
      'menu.unifiedDict': '词典',
      'menu.onlineMusic': '在线听歌',
      'menu.noteManager': '笔记管理',
      'menu.pianoKeyboard': '钢琴键盘',
      'menu.tools': '工具函数',
      'menu.fileManager': '文件管理',
      'unifiedDict.title': '词典',
      'unifiedDict.hint': '输入要查询的词语...',
      'unifiedDict.searchCount': '「{word}」已查询 {count} 次',
      'unifiedDict.frequency': '词频',
      'unifiedDict.history': '历史',
      'menu.clipboardHistory': '剪贴板历史',
      'menu.clipboardHistoryText': '文本',
      'menu.clipboardHistoryImage': '图片',
      'menu.clipboardHistoryNoPreview': '无预览',

      // Settings
      'settings.title': '设置',
      'settings.appearance': '外观设置',
      'settings.player': '播放器设置',
      'settings.server': '服务器',
      'settings.data': '数据',
      'settings.resetAll': '重置所有数据',
      'settings.resetAllSubtitle': '清除服务器配置和登录信息，回到欢迎页',
      'settings.light': '明亮',
      'settings.lightDesc': '始终使用明亮主题',
      'settings.dark': '黑暗',
      'settings.darkDesc': '始终使用黑暗主题',
      'settings.system': '跟随系统',
      'settings.systemDesc': '根据系统设置自动切换',
      'settings.solidBg': '纯色背景',
      'settings.solidBgDesc': '使用歌曲主题色背景',
      'settings.blurBg': '模糊背景',
      'settings.blurBgDesc': '使用歌曲封面模糊背景',
      'settings.defaultColor': '默认颜色',
      'settings.defaultColorDesc': '使用默认灰色背景',
      'settings.addServer': '添加服务器',
      'settings.editServer': '编辑服务器',
      'settings.deleteServer': '删除服务器',
      'settings.confirmDelete': '确定删除 "@name"？',
      'settings.confirmReset': '将清除所有服务器配置和登录信息，回到欢迎页面。确定继续？',
      'settings.confirmResetBtn': '确定重置',
      'settings.language': '语言',
      'settings.languageDesc': '切换显示语言',
      'settings.chinese': '中文',
      'settings.english': 'English',

      // Setup
      'setup.defaultServer': '默认服务器',
      'setup.welcome': '欢迎使用',
      'setup.welcomeDesc': '请先配置服务器和账号',
      'setup.nameHint': '例如：家里、公司',
      'setup.nameRequired': '请输入名称',
      'setup.serverAddress': '服务器地址',
      'setup.serverAddressHint': '192.168.1.100 或 example.com',
      'setup.serverAddressRequired': '请输入地址',
      'setup.portRequired': '请输入端口',
      'setup.portInvalid': '端口无效',
      'setup.usernameRequired': '请输入用户名',
      'setup.passwordRequired': '请输入密码',
      'setup.start': '开始使用',

      // Tools
      'tools.title': '工具中心',
      'tools.base64': 'Base64 工具',
      'tools.calculator': '计算器',
      'tools.password': '密码生成器',
      'tools.timestamp': '时间戳工具',
      'tools.uuid': 'UUID 生成器',

      // Base64 Tool
      'base64.title': 'Base64 工具',
      'base64.switchToDecode': '切换到解码',
      'base64.switchToEncode': '切换到编码',
      'base64.encodeInput': '输入要编码的文本：',
      'base64.decodeInput': '输入要解码的 Base64 字符串：',
      'base64.encodeHint': '请输入文本',
      'base64.decodeHint': '请输入 Base64 字符串',
      'base64.encode': '编码',
      'base64.decode': '解码',
      'base64.encodeResult': '编码结果：',
      'base64.decodeResult': '解码结果：',
      'base64.decodeFailed': '解码失败',

      // Calculator Tool
      'calculator.title': '计算器',
      'calculator.numberA': '数字 A',
      'calculator.numberB': '数字 B',
      'calculator.calculate': '计算',
      'calculator.result': '计算结果：',

      // Password Tool
      'password.title': '密码生成器',
      'password.length': '密码长度：',
      'password.generateRandom': '生成随机密码',
      'password.generateStrong': '生成强密码',
      'password.randomPassword': '随机密码：',
      'password.strongPassword': '强密码：',

      // Timestamp Tool
      'timestamp.title': '时间戳工具',
      'timestamp.refresh': '刷新时间',
      'timestamp.currentInfo': '当前时间信息：',
      'timestamp.timestamp': '时间戳：@value',
      'timestamp.localTime': '本地时间：@value',
      'timestamp.utcTime': 'UTC时间：@value',

      // UUID Tool
      'uuid.title': 'UUID 生成器',
      'uuid.generate': '生成新UUID',
      'uuid.result': 'UUID 生成结果：',
      'uuid.v4': 'UUID v4：@value',
      'uuid.v6': 'UUID v6：@value',
      'uuid.v7': 'UUID v7：@value',

      // Utils Page
      'utils.title': '工具函数测试',
      'utils.base64Section': 'Base64 编码/解码',
      'utils.inputString': '输入字符串',
      'utils.encodeResult': '编码结果: @value',
      'utils.base64String': 'Base64 编码字符串',
      'utils.decodeResult': '解码结果: @value',
      'utils.calculatorSection': '计算器',
      'utils.passwordSection': '密码生成',
      'utils.passwordLength': '密码长度',
      'utils.randomPassword': '随机密码: @value',
      'utils.strongPassword': '强密码: @value',
      'utils.timestampSection': '时间戳',
      'utils.currentTimestamp': '当前时间戳: @value',
      'utils.currentLocalTime': '当前本地时间: @value',
      'utils.currentUtcTime': '当前UTC时间: @value',
      'utils.uuidSection': 'UUID 生成',

      // Collins Dict
      'collins.title': '柯林斯词典',
      'collins.inputHint': '输入单词',

      // LDOCE Dict
      'ldoce.title': '朗文词典',
      'ldoce.inputHint': '输入单词',

      // Wubi Query
      'wubi.title': '五笔查询',
      'wubi.inputLabel': '输入一个字',
      'wubi.placeholder': '查询结果将显示在这里',
      'wubi.code86': '86版编码  @code',
      'wubi.queryError': '查询错误: @error',
      'wubi.displayError': '无法显示@key: @error',

      // Scan
      'scan.title': '扫码',
      'scan.rescan': '重新扫描',
      'scan.instruction': '将二维码放入框内',
      'scan.result': '扫描结果：',
      'scan.copyText': '复制文本',

      // File Manager
      'fileManager.title': '文件管理',
      'fileManager.listView': '列表视图',
      'fileManager.gridView': '网格视图',
      'fileManager.emptyDir': '空目录',
      'fileManager.openFullscreen': '全屏打开',
      'fileManager.loadFailed': '加载失败',
      'fileManager.cannotLoad': '无法加载: @error',
      'fileManager.audioPreview': '🎵 音频文件 - 点击全屏打开',
      'fileManager.pdfPreview': '📄 PDF 文件 - 点击全屏打开',
      'fileManager.epubPreview': '📖 EPUB 书籍 - 点击全屏打开',
      'fileManager.noPreview': '不支持预览',

      // EPUB Reader
      'epub.cannotOpen': '无法打开 EPUB: @error',
      'epub.decreaseFont': '字号减小',
      'epub.increaseFont': '字号增大',
      'epub.switchToLight': '切换亮色',
      'epub.switchToDark': '切换暗色',
      'epub.toc': '目录',
      'epub.prevPage': '上一页',
      'epub.nextPage': '下一页',

      // EPUB Parser
      'epub.unknownTitle': '未知标题',
      'epub.chapter': '章节 @number',

      // Image Gallery
      'imageGallery.title': '图片库',
      'imageGallery.columns': '@count 列',
      'imageGallery.scanStarted': '扫描已开始，请稍后刷新',
      'imageGallery.scanFailed': '扫描失败: @error',
      'imageGallery.noFolders': '暂无图片文件夹，请先添加媒体路径并扫描',
      'imageGallery.noImages': '该文件夹暂无图片',

      // Image Viewer
      'imageViewer.loadFailedHttp': '加载图片失败: HTTP @statusCode',
      'imageViewer.loadFailed': '加载图片失败: @error',

      // Video Library
      'videoLibrary.title': '视频库',
      'videoLibrary.scanVideo': '扫描视频',
      'videoLibrary.loadFailed': '加载失败',

      // Video Player
      'videoPlayer.memoryNotSupported': '内存播放暂不支持，请使用 network 或 asset 类型',
      'videoPlayer.loadFailed': '视频加载失败，格式或编码可能不受支持',
      'videoPlayer.playFailed': '播放失败: @error',

      // API Movies
      'apiMelatoninMovies.title': 'Melatonin 电影',
      'apiMelatoninMovies.noMovies': '暂无电影',
      'apiMelatoninMovies.loadFailed': '加载失败: @error',

      // Actor Movies
      'actorMovies.title': '@name 的电影',
      'actorMovies.noMovies': '暂无 @name 的电影',
      'actorMovies.loadFailed': '加载失败: @error',

      // Movie Detail
      'movieDetail.loadFailed': '加载失败: @error',
      'movieDetail.videoUnavailable': '视频地址不可用',
      'movieDetail.year': '年份',
      'movieDetail.rating': '评分',
      'movieDetail.duration': '时长',
      'movieDetail.releaseDate': '上映日期',
      'movieDetail.genre': '类型',
      'movieDetail.director': '导演',
      'movieDetail.actors': '演员',
      'movieDetail.plot': '剧情简介',
      'movieDetail.play': '播放',

      // Online Music
      'onlineMusic.title': '在线听歌',
      'onlineMusic.loadFailed': '获取歌曲列表失败: @error',

      // Piano
      'piano.title': '钢琴键盘',
      'piano.standardPitch': 'A4 = 440 Hz (标准音高)',
      'piano.whiteKey': '白键',
      'piano.blackKey': '黑键',
      'piano.middleC': 'C4 (中间位置)',

      // Music Service
      'music.unknownArtist': '未知艺术家',
      'music.unknownAlbum': '未知专辑',
      'music.loadDeviceFailed': '获取设备音乐失败: @error',
      'music.selectionCancelled': '已取消选择',
      'music.noAudioFiles': '所选文件夹中没有找到音频文件',
      'music.noFilesSelected': '没有选择音频文件',

      // Audio Player Handler
      'audio.onlineAudio': '在线音频',
      'audio.onlineSong': '在线歌曲',
      'audio.unknownSong': '未知歌曲',

      // Song Info Widget
      'songInfo.noAudio': '暂无音频',

      // Lyrics
      'lyrics.noLyrics': '暂无歌词',

      // Common App Bar
      'appBar.log': '日志',
      'appBar.music': '音乐',

      // Notes
      'note.editNote': '编辑笔记',
      'note.createNote': '创建笔记',
      'note.save': '保存',
      'note.title': '标题',
      'note.content': '内容',
      'note.saveFailed': '保存笔记失败: @error',
      'note.manager': '笔记管理',
      'note.loadFailed': '加载笔记失败: @error',
      'note.noTitle': '无标题',
      'note.noContent': '无内容',

      // Collins Dictionary Widget
      'collinsWidget.title': '柯林斯词典小组件',
      'collinsWidget.enterWord': '请输入要查询的单词',
      'collinsWidget.updated': '小组件已更新，请在桌面添加',
      'collinsWidget.inputHint': '输入要查询的单词',
      'collinsWidget.updateWidget': '更新小组件',
      'collinsWidget.instructions': '使用说明',
      'collinsWidget.step1': '1. 输入要查询的单词',
      'collinsWidget.step2': '2. 点击"更新小组件"将单词保存到桌面小组件',
      'collinsWidget.step3': '3. 长按桌面空白处添加"柯林斯词典"小组件',
      'collinsWidget.step4': '4. 点击小组件即可快速跳转到应用内的柯林斯词典页面',

      // AppBar Mini Window
      'miniWindow.log': '日志',

      // API Error Codes
      'error.EMPTY_CREDENTIALS': '用户名和密码不能为空',
      'error.EMPTY_PATH': '路径不能为空',
      'error.INVALID_MEDIA_TYPE': '无效的媒体类型',
      'error.NO_AUTHORIZED_DIR': '用户未配置授权目录',
      'error.PATH_NOT_IN_DIR': '路径不在任何授权目录内',
      'error.NOT_A_DIRECTORY': '路径不是目录',
      'error.OPENAI_KEY_MISSING': 'OpenAI API Key 未配置',
      'error.USER_NOT_FOUND': '用户不存在',
      'error.SONG_NOT_FOUND': '歌曲不存在',
      'error.SONG_COVER_NOT_FOUND': '封面图片不存在',
      'error.LYRICS_NOT_FOUND': '歌词文件不存在',
      'error.NO_TTML_LYRICS': '该歌曲没有 TTML 歌词',
      'error.DIR_NOT_FOUND': '目录不存在',
      'error.CHARACTER_NOT_FOUND': '未找到该汉字',
      'error.PATH_NOT_FOUND': '路径不存在',
      'error.SESSION_NOT_FOUND': '会话不存在',
      'error.MEDIA_PATH_NOT_FOUND': '媒体路径不存在',
      'error.SHORT_NOTE_NOT_FOUND': '短笔记不存在',
      'error.NOTE_NOT_FOUND': '笔记不存在',
      'error.NOTE_CONTENT_NOT_FOUND': '笔记内容不存在',
      'error.RESOURCE_NOT_FOUND': '资源不存在',
      'error.WRONG_PASSWORD': '用户名或密码错误',
      'error.INVALID_TOKEN': '无效的认证令牌',
      'error.TOKEN_EXPIRED': '认证令牌已过期',
      'error.ADMIN_REQUIRED': '需要管理员权限',
      'error.DIR_ACCESS_DENIED': '目录不存在或无权访问',
      'error.INTERNAL_ERROR': '服务器内部错误',
    },
    'en_US': {
      // Common
      'common.cancel': 'Cancel',
      'common.confirm': 'Confirm',
      'common.delete': 'Delete',
      'common.save': 'Save',
      'common.retry': 'Retry',
      'common.refresh': 'Refresh',
      'common.back': 'Back',
      'common.scan': 'Scan',
      'common.play': 'Play',
      'common.copy': 'Copy',
      'common.copied': 'Copied to clipboard',
      'common.loading': 'Loading',
      'common.loadFailed': 'Load failed',
      'common.loadFailedWith': 'Load failed: @error',
      'common.required': 'Required',
      'common.invalid': 'Invalid',
      'common.name': 'Name',
      'common.address': 'Address',
      'common.port': 'Port',
      'common.username': 'Username',
      'common.password': 'Password',
      'common.emptyState': 'No data',
      'common.retryOrRefresh': 'Please try again later',
      'common.search': 'Search',
      'common.history': 'History',
      'common.hide': 'Hide',
      'common.highFrequency': 'Top Searches',
      'common.recentSearch': 'Recent Searches',
      'common.noHighFrequencyData': 'No top search data',
      'common.noRecentData': 'No recent search data',
      'common.searchError': 'Search error: @error',
      'common.searchCount': 'Searched @count times',

      // App
      'app.exitHint': 'Press back again to exit',

      // Navigation
      'nav.home': 'Home',
      'nav.menu': 'Menu',
      'nav.profile': 'Me',

      // Home
      'home.title': 'Home',
      'home.listenMusic': 'Music',

      // Profile
      'profile.title': 'Me',
      'profile.settings': 'Settings',

      // Menu
      'menu.title': 'App Menu',
      'menu.wubiQuery': 'Wubi Query',
      'menu.videoLibrary': 'Video Library',
      'menu.imageGallery': 'Image Gallery',
      'menu.melatoninMovies': 'Melatonin Movies',
      'menu.scan': 'QR Scanner',
      'menu.unifiedDict': 'Dictionary',
      'menu.onlineMusic': 'Online Music',
      'menu.noteManager': 'Notes',
      'menu.pianoKeyboard': 'Piano',
      'menu.tools': 'Tools',
      'menu.fileManager': 'File Manager',
      'unifiedDict.title': 'Dictionary',
      'unifiedDict.hint': 'Search any word...',
      'unifiedDict.searchCount': '"{word}" searched {count} times',
      'unifiedDict.frequency': 'Top Words',
      'unifiedDict.history': 'History',
      'menu.clipboardHistory': 'Clipboard History',
      'menu.clipboardHistoryText': 'Text',
      'menu.clipboardHistoryImage': 'Image',
      'menu.clipboardHistoryNoPreview': 'No Preview',

      // Settings
      'settings.title': 'Settings',
      'settings.appearance': 'Appearance',
      'settings.player': 'Player',
      'settings.server': 'Server',
      'settings.data': 'Data',
      'settings.resetAll': 'Reset All Data',
      'settings.resetAllSubtitle': 'Clear server config and login info, return to welcome page',
      'settings.light': 'Light',
      'settings.lightDesc': 'Always use light theme',
      'settings.dark': 'Dark',
      'settings.darkDesc': 'Always use dark theme',
      'settings.system': 'System',
      'settings.systemDesc': 'Follow system settings',
      'settings.solidBg': 'Solid',
      'settings.solidBgDesc': 'Use song theme color background',
      'settings.blurBg': 'Blur',
      'settings.blurBgDesc': 'Use blurred album cover background',
      'settings.defaultColor': 'Default',
      'settings.defaultColorDesc': 'Use default gray background',
      'settings.addServer': 'Add Server',
      'settings.editServer': 'Edit Server',
      'settings.deleteServer': 'Delete Server',
      'settings.confirmDelete': 'Delete "@name"?',
      'settings.confirmReset': 'This will clear all server configs and login info, returning to the welcome page. Continue?',
      'settings.confirmResetBtn': 'Confirm Reset',
      'settings.language': 'Language',
      'settings.languageDesc': 'Switch display language',
      'settings.chinese': '中文',
      'settings.english': 'English',

      // Setup
      'setup.defaultServer': 'Default Server',
      'setup.welcome': 'Welcome',
      'setup.welcomeDesc': 'Please configure your server and account first',
      'setup.nameHint': 'e.g. Home, Office',
      'setup.nameRequired': 'Please enter a name',
      'setup.serverAddress': 'Server Address',
      'setup.serverAddressHint': '192.168.1.100 or example.com',
      'setup.serverAddressRequired': 'Please enter an address',
      'setup.portRequired': 'Please enter a port',
      'setup.portInvalid': 'Invalid port',
      'setup.usernameRequired': 'Please enter username',
      'setup.passwordRequired': 'Please enter password',
      'setup.start': 'Get Started',

      // Tools
      'tools.title': 'Tools',
      'tools.base64': 'Base64 Tool',
      'tools.calculator': 'Calculator',
      'tools.password': 'Password Generator',
      'tools.timestamp': 'Timestamp Tool',
      'tools.uuid': 'UUID Generator',

      // Base64 Tool
      'base64.title': 'Base64 Tool',
      'base64.switchToDecode': 'Switch to Decode',
      'base64.switchToEncode': 'Switch to Encode',
      'base64.encodeInput': 'Enter text to encode:',
      'base64.decodeInput': 'Enter Base64 string to decode:',
      'base64.encodeHint': 'Enter text',
      'base64.decodeHint': 'Enter Base64 string',
      'base64.encode': 'Encode',
      'base64.decode': 'Decode',
      'base64.encodeResult': 'Encode result:',
      'base64.decodeResult': 'Decode result:',
      'base64.decodeFailed': 'Decode failed',

      // Calculator Tool
      'calculator.title': 'Calculator',
      'calculator.numberA': 'Number A',
      'calculator.numberB': 'Number B',
      'calculator.calculate': 'Calculate',
      'calculator.result': 'Result:',

      // Password Tool
      'password.title': 'Password Generator',
      'password.length': 'Password length:',
      'password.generateRandom': 'Generate Random',
      'password.generateStrong': 'Generate Strong',
      'password.randomPassword': 'Random password:',
      'password.strongPassword': 'Strong password:',

      // Timestamp Tool
      'timestamp.title': 'Timestamp Tool',
      'timestamp.refresh': 'Refresh',
      'timestamp.currentInfo': 'Current time info:',
      'timestamp.timestamp': 'Timestamp: @value',
      'timestamp.localTime': 'Local time: @value',
      'timestamp.utcTime': 'UTC time: @value',

      // UUID Tool
      'uuid.title': 'UUID Generator',
      'uuid.generate': 'Generate UUID',
      'uuid.result': 'UUID Result:',
      'uuid.v4': 'UUID v4: @value',
      'uuid.v6': 'UUID v6: @value',
      'uuid.v7': 'UUID v7: @value',

      // Utils Page
      'utils.title': 'Utility Tests',
      'utils.base64Section': 'Base64 Encode/Decode',
      'utils.inputString': 'Input string',
      'utils.encodeResult': 'Encode result: @value',
      'utils.base64String': 'Base64 string',
      'utils.decodeResult': 'Decode result: @value',
      'utils.calculatorSection': 'Calculator',
      'utils.passwordSection': 'Password',
      'utils.passwordLength': 'Password length',
      'utils.randomPassword': 'Random: @value',
      'utils.strongPassword': 'Strong: @value',
      'utils.timestampSection': 'Timestamp',
      'utils.currentTimestamp': 'Timestamp: @value',
      'utils.currentLocalTime': 'Local time: @value',
      'utils.currentUtcTime': 'UTC time: @value',
      'utils.uuidSection': 'UUID',

      // Collins Dict
      'collins.title': 'Collins Dictionary',
      'collins.inputHint': 'Enter a word',

      // LDOCE Dict
      'ldoce.title': 'LDOCE Dictionary',
      'ldoce.inputHint': 'Enter a word',

      // Wubi Query
      'wubi.title': 'Wubi Query',
      'wubi.inputLabel': 'Enter a character',
      'wubi.placeholder': 'Results will appear here',
      'wubi.code86': 'Wubi86: @code',
      'wubi.queryError': 'Query error: @error',
      'wubi.displayError': 'Cannot display @key: @error',

      // Scan
      'scan.title': 'QR Scanner',
      'scan.rescan': 'Rescan',
      'scan.instruction': 'Place QR code in the frame',
      'scan.result': 'Scan result:',
      'scan.copyText': 'Copy Text',

      // File Manager
      'fileManager.title': 'File Manager',
      'fileManager.listView': 'List View',
      'fileManager.gridView': 'Grid View',
      'fileManager.emptyDir': 'Empty directory',
      'fileManager.openFullscreen': 'Open fullscreen',
      'fileManager.loadFailed': 'Load failed',
      'fileManager.cannotLoad': 'Cannot load: @error',
      'fileManager.audioPreview': '🎵 Audio file - tap to open fullscreen',
      'fileManager.pdfPreview': '📄 PDF file - tap to open fullscreen',
      'fileManager.epubPreview': '📖 EPUB book - tap to open fullscreen',
      'fileManager.noPreview': 'Preview not supported',

      // EPUB Reader
      'epub.cannotOpen': 'Cannot open EPUB: @error',
      'epub.decreaseFont': 'Decrease font',
      'epub.increaseFont': 'Increase font',
      'epub.switchToLight': 'Switch to light',
      'epub.switchToDark': 'Switch to dark',
      'epub.toc': 'Table of Contents',
      'epub.prevPage': 'Previous',
      'epub.nextPage': 'Next',

      // EPUB Parser
      'epub.unknownTitle': 'Unknown Title',
      'epub.chapter': 'Chapter @number',

      // Image Gallery
      'imageGallery.title': 'Image Gallery',
      'imageGallery.columns': '@count columns',
      'imageGallery.scanStarted': 'Scan started, please refresh later',
      'imageGallery.scanFailed': 'Scan failed: @error',
      'imageGallery.noFolders': 'No image folders, please add media paths and scan',
      'imageGallery.noImages': 'No images in this folder',

      // Image Viewer
      'imageViewer.loadFailedHttp': 'Failed to load image: HTTP @statusCode',
      'imageViewer.loadFailed': 'Failed to load image: @error',

      // Video Library
      'videoLibrary.title': 'Video Library',
      'videoLibrary.scanVideo': 'Scan Videos',
      'videoLibrary.loadFailed': 'Load failed',

      // Video Player
      'videoPlayer.memoryNotSupported': 'Memory playback not supported, please use network or asset type',
      'videoPlayer.loadFailed': 'Video load failed, format or codec may not be supported',
      'videoPlayer.playFailed': 'Playback failed: @error',

      // API Movies
      'apiMelatoninMovies.title': 'Melatonin Movies',
      'apiMelatoninMovies.noMovies': 'No movies',
      'apiMelatoninMovies.loadFailed': 'Load failed: @error',

      // Actor Movies
      'actorMovies.title': '@name\'s Movies',
      'actorMovies.noMovies': 'No movies for @name',
      'actorMovies.loadFailed': 'Load failed: @error',

      // Movie Detail
      'movieDetail.loadFailed': 'Load failed: @error',
      'movieDetail.videoUnavailable': 'Video URL unavailable',
      'movieDetail.year': 'Year',
      'movieDetail.rating': 'Rating',
      'movieDetail.duration': 'Duration',
      'movieDetail.releaseDate': 'Release Date',
      'movieDetail.genre': 'Genre',
      'movieDetail.director': 'Director',
      'movieDetail.actors': 'Cast',
      'movieDetail.plot': 'Synopsis',
      'movieDetail.play': 'Play',

      // Online Music
      'onlineMusic.title': 'Online Music',
      'onlineMusic.loadFailed': 'Failed to load songs: @error',

      // Piano
      'piano.title': 'Piano',
      'piano.standardPitch': 'A4 = 440 Hz (Standard Pitch)',
      'piano.whiteKey': 'White Key',
      'piano.blackKey': 'Black Key',
      'piano.middleC': 'C4 (Middle C)',

      // Music Service
      'music.unknownArtist': 'Unknown Artist',
      'music.unknownAlbum': 'Unknown Album',
      'music.loadDeviceFailed': 'Failed to load device music: @error',
      'music.selectionCancelled': 'Selection cancelled',
      'music.noAudioFiles': 'No audio files found in selected folder',
      'music.noFilesSelected': 'No audio files selected',

      // Audio Player Handler
      'audio.onlineAudio': 'Online Audio',
      'audio.onlineSong': 'Online Song',
      'audio.unknownSong': 'Unknown Song',

      // Song Info Widget
      'songInfo.noAudio': 'No audio',

      // Lyrics
      'lyrics.noLyrics': 'No lyrics',

      // Common App Bar
      'appBar.log': 'Log',
      'appBar.music': 'Music',

      // Notes
      'note.editNote': 'Edit Note',
      'note.createNote': 'Create Note',
      'note.save': 'Save',
      'note.title': 'Title',
      'note.content': 'Content',
      'note.saveFailed': 'Failed to save note: @error',
      'note.manager': 'Notes',
      'note.loadFailed': 'Failed to load note: @error',
      'note.noTitle': 'Untitled',
      'note.noContent': 'No content',

      // Collins Dictionary Widget
      'collinsWidget.title': 'Collins Dict Widget',
      'collinsWidget.enterWord': 'Please enter a word',
      'collinsWidget.updated': 'Widget updated, please add it on home screen',
      'collinsWidget.inputHint': 'Enter a word to look up',
      'collinsWidget.updateWidget': 'Update Widget',
      'collinsWidget.instructions': 'Instructions',
      'collinsWidget.step1': '1. Enter a word to look up',
      'collinsWidget.step2': '2. Tap "Update Widget" to save the word to the home screen widget',
      'collinsWidget.step3': '3. Long press on home screen to add the "Collins Dict" widget',
      'collinsWidget.step4': '4. Tap the widget to quickly open Collins Dictionary in the app',

      // AppBar Mini Window
      'miniWindow.log': 'Log',

      // API Error Codes
      'error.EMPTY_CREDENTIALS': 'Username and password cannot be empty',
      'error.EMPTY_PATH': 'Path cannot be empty',
      'error.INVALID_MEDIA_TYPE': 'Invalid media type',
      'error.NO_AUTHORIZED_DIR': 'No authorized directory configured',
      'error.PATH_NOT_IN_DIR': 'Path is not within any authorized directory',
      'error.NOT_A_DIRECTORY': 'Path is not a directory',
      'error.OPENAI_KEY_MISSING': 'OpenAI API Key is not configured',
      'error.USER_NOT_FOUND': 'User not found',
      'error.SONG_NOT_FOUND': 'Song not found',
      'error.SONG_COVER_NOT_FOUND': 'Cover image not found',
      'error.LYRICS_NOT_FOUND': 'Lyrics file not found',
      'error.NO_TTML_LYRICS': 'This song has no TTML lyrics',
      'error.DIR_NOT_FOUND': 'Directory not found',
      'error.CHARACTER_NOT_FOUND': 'Character not found',
      'error.PATH_NOT_FOUND': 'Path not found',
      'error.SESSION_NOT_FOUND': 'Session not found',
      'error.MEDIA_PATH_NOT_FOUND': 'Media path not found',
      'error.SHORT_NOTE_NOT_FOUND': 'Short note not found',
      'error.NOTE_NOT_FOUND': 'Note not found',
      'error.NOTE_CONTENT_NOT_FOUND': 'Note content not found',
      'error.RESOURCE_NOT_FOUND': 'Resource not found',
      'error.WRONG_PASSWORD': 'Incorrect username or password',
      'error.INVALID_TOKEN': 'Invalid authentication token',
      'error.TOKEN_EXPIRED': 'Authentication token expired',
      'error.ADMIN_REQUIRED': 'Admin privileges required',
      'error.DIR_ACCESS_DENIED': 'Directory not found or access denied',
      'error.INTERNAL_ERROR': 'Internal server error',
    },
  };
}
