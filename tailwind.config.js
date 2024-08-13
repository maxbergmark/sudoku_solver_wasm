/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: 'class',
  content: {
    files: ["*.html", "./src/**/*.rs"],
  },
  theme: {
    extend: {
      colors: {
        'cerulean-blue': {
          '50': '#f0f5fe',
          '100': '#dce9fd',
          '200': '#c2d8fb',
          '300': '#97c1f9',
          '400': '#66a0f4',
          '500': '#437cee',
          '600': '#2d5ee3',
          '700': '#264dd9',
          '800': '#243da9',
          '900': '#223886',
          '950': '#192352',
        },
      },
    },
  },
  plugins: [],
}