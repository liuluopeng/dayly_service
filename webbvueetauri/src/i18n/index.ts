import { createI18n } from 'vue-i18n'
import zh from './zh.json'
import en from './en.json'

const savedLocale = localStorage.getItem('dayly_locale') || 'zh'

const i18n = createI18n({
  legacy: false,
  locale: savedLocale,
  fallbackLocale: 'zh',
  messages: { zh, en }
})

export function setLocale(locale: 'zh' | 'en') {
  i18n.global.locale.value = locale
  localStorage.setItem('dayly_locale', locale)
}

export default i18n
