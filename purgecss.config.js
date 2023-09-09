module.exports = {
  content: [
    './templates/include/**/*.html',
    './static/js/**/*.js',
    './static/libs/bootstrap/bootstrap.min.js',
    './static/css/custom.css'
  ],
  defaultExtractor: content => {
    // Jeśli potrzebujesz dodatkowej logiki ekstraktora, możesz ją tutaj dodać
    return content.match(/[\w-/:]+(?<!:)/g) || [];
  },
  css: ['./static/libs/bootstrap/bootstrap.min.css'],
  output: './static/libs/bootstrap/',
  safelist: {
    // Uwzględnij klasy, które chcesz zachować w wynikowym pliku CSS, jeśli są jakieś konkretne
    standard: [],
    deep: [/data-bs-theme/], // Używając wyrażenia regularnego, możemy uwzględnić wszystkie klasy używające tego atrybutu
    greedy: []
  }
};
