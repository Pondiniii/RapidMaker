document.addEventListener("DOMContentLoaded", function() {
    // Znajdź wszystkie komórki tabeli
    const cells = document.querySelectorAll(".table td");

    cells.forEach(cell => {
        // Policz liczbę kropek w komórce
        const dotCount = (cell.textContent || '').split('⚪').length - 1;

        // Dodaj odpowiednią klasę w zależności od liczby kropek
        if (dotCount >= 1 && dotCount <= 5) {
            cell.classList.add(`cell-${dotCount}`);
        }
    });
});
