export interface MenuItem {
  path: string;
  title: string;
  icon: string;
  description: string;
  children?: Array<{
    path: string;
    title: string;
    icon: string;
    description: string;
  }>;
}

export function getMenuItems(t: (key: string) => string): MenuItem[] {
  return [
    {
      path: '/ggtt',
      title: t('menu.ggtt.title'),
      icon: t('menu.ggtt.icon'),
      description: t('menu.ggtt.desc')
    },
    {
      path: '/xiandaihanyu',
      title: t('menu.xiandaihanyu.title'),
      icon: t('menu.xiandaihanyu.icon'),
      description: t('menu.xiandaihanyu.desc')
    },
    {
      path: '/collins',
      title: t('menu.collins.title'),
      icon: t('menu.collins.icon'),
      description: t('menu.collins.desc')
    },
    {
      path: '/ldoce',
      title: t('menu.ldoce.title'),
      icon: t('menu.ldoce.icon'),
      description: t('menu.ldoce.desc')
    },
    {
      path: '/songs',
      title: t('menu.songs.title'),
      icon: '🎵',
      description: t('menu.songs.desc')
    },
    {
      path: '/songs?wasm=1',
      title: t('menu.songsWasm.title'),
      icon: '⚡',
      description: t('menu.songsWasm.desc')
    },
    {
      path: '/songs?lyrics=1',
      title: t('menu.songsLyrics.title'),
      icon: '🎶',
      description: t('menu.songsLyrics.desc')
    },
    {
      path: '/files',
      title: t('menu.files.title'),
      icon: '📂',
      description: t('menu.files.desc')
    },
    {
      path: '/short-notes',
      title: t('menu.shortNotes.title'),
      icon: '📝',
      description: t('menu.shortNotes.desc')
    },
    {
      path: '/openai-chat',
      title: t('menu.chat.title'),
      icon: '🤖',
      description: t('menu.chat.desc')
    },
    {
      path: '/chat',
      title: '聊天',
      icon: '💬',
      description: '与其他用户聊天'
    },
    {
      path: '/sharing',
      title: '局域网共享',
      icon: '🔗',
      description: '剪贴板同步和文件传输'
    },
    {
      path: '/clipboard-history',
      title: t('menu.clipboardHistory.title'),
      icon: '📋',
      description: t('menu.clipboardHistory.desc')
    },
    {
      path: '/game2048',
      title: t('menu.game2048.title'),
      icon: '🎮',
      description: t('menu.game2048.desc')
    },
    {
      path: '/notes',
      title: t('menu.notes.title'),
      icon: '📚',
      description: t('menu.notes.desc')
    },
    {
      path: '/search-history',
      title: t('menu.searchHistory.title'),
      icon: '📊',
      description: t('menu.searchHistory.desc')
    },
    {
      path: '/zici',
      title: t('menu.zici.title'),
      icon: '📖',
      description: t('menu.zici.desc'),
      children: [
        {
          path: '/zici/chars',
          title: t('menu.zici.chars.title'),
          icon: '🈶',
          description: t('menu.zici.chars.desc')
        },
        {
          path: '/zici/words',
          title: t('menu.zici.words.title'),
          icon: '📝',
          description: t('menu.zici.words.desc')
        },
        {
          path: '/zici/dictation',
          title: t('menu.zici.dictation.title'),
          icon: '✍️',
          description: t('menu.zici.dictation.desc')
        },
        {
          path: '/zici/keyboard',
          title: t('menu.zici.keyboard.title'),
          icon: '⌨️',
          description: t('menu.zici.keyboard.desc')
        },
        {
          path: '/zici/pinyin-picker',
          title: t('menu.zici.pinyin.title'),
          icon: '🔤',
          description: t('menu.zici.pinyin.desc')
        },
        {
          path: '/zici/word-frequency',
          title: t('menu.zici.frequency.title'),
          icon: '📊',
          description: t('menu.zici.frequency.desc')
        }
      ]
    },
    {
      path: '/tools',
      title: t('menu.tools.title'),
      icon: '🔧',
      description: t('menu.tools.desc'),
      children: [
        {
          path: '/tools/base64',
          title: t('menu.tools.base64.title'),
          icon: '🔢',
          description: t('menu.tools.base64.desc')
        },
        {
          path: '/tools/timestamp',
          title: t('menu.tools.timestamp.title'),
          icon: '⏰',
          description: t('menu.tools.timestamp.desc')
        },
        {
          path: '/tools/uuid',
          title: t('menu.tools.uuid.title'),
          icon: '🆔',
          description: t('menu.tools.uuid.desc')
        },
        {
          path: '/tools/password',
          title: t('menu.tools.password.title'),
          icon: '🔒',
          description: t('menu.tools.password.desc')
        },
        {
          path: '/tools/chinese-remover',
          title: t('menu.tools.chineseRemover.title'),
          icon: '✂️',
          description: t('menu.tools.chineseRemover.desc')
        },
        {
          path: '/tools/image-converter',
          title: t('menu.tools.imageConverter.title'),
          icon: '🖼️',
          description: t('menu.tools.imageConverter.desc')
        },
        {
          path: '/tools/qrcode',
          title: t('menu.tools.qrcode.title'),
          icon: '📱',
          description: t('menu.tools.qrcode.desc')
        },
        {
          path: '/tools/qrscan',
          title: t('menu.tools.qrscan.title'),
          icon: '📷',
          description: t('menu.tools.qrscan.desc')
        }
      ]
    },
    {
      path: '/images',
      title: t('menu.images.title'),
      icon: '🖼️',
      description: t('menu.images.desc')
    },
    {
      path: '/videos',
      title: t('menu.videos.title'),
      icon: '🎬',
      description: t('menu.videos.desc')
    },
    {
      path: '/admin/user-directories',
      title: t('menu.admin.title'),
      icon: '👥',
      description: t('menu.admin.desc')
    },
    {
      path: '/settings',
      title: t('menu.settings.title'),
      icon: '⚙️',
      description: t('menu.settings.desc')
    }
  ];
}
