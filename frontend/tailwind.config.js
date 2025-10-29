/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ['./dist/**/*.html', './dist/**/*.js'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        emerald: {
          300: '#3bcf74',
          400: '#2abf68',
        },
        rapid: '#00c896',
      },
      fontFamily: {
        sans: ['"Plus Jakarta Sans"', 'Inter', 'system-ui', 'sans-serif'],
      },
    },
  },
  plugins: [],
};
