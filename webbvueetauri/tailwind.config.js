/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
    "./src/App.vue"
  ],
  darkMode: 'media',
  theme: {
    extend: {
      colors: {
        primary: '#646cff',
        secondary: '#249b73',
        accent: '#24c8db',
        background: {
          light: '#f6f6f6',
          dark: '#2f2f2f'
        },
        text: {
          light: '#0f0f0f',
          dark: '#f6f6f6'
        }
      },
      fontFamily: {
        sans: ['Inter', 'Avenir', 'Helvetica', 'Arial', 'sans-serif'],
      },
      animation: {
        'pulse-slow': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
      }
    },
  },
  plugins: [],
}
