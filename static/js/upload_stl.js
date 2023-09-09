document.addEventListener("DOMContentLoaded", function() {
    let submitButton = document.querySelector(".wycena-button");

    document.querySelector("form").addEventListener("submit", function(e) {
        e.preventDefault();

        // Zmieniamy przycisk na wersję ładowania
        submitButton.innerHTML = '<span class="spinner-grow spinner-grow-sm" role="status" aria-hidden="true"></span> Loading...';
        submitButton.disabled = true;

        let formData = new FormData(this);

        fetch('/upload/stl', {
            method: 'POST',
            body: formData
        })
        .then(response => response.json())
        .then(data => {
            appendResult(data);
        })
        .catch((error) => {
            console.error('Error:', error);
        })
        .finally(() => {
            // Przywracamy przycisk do pierwotnego stanu
            submitButton.innerHTML = 'Zatwierdź';
            submitButton.disabled = false;
        });
    });
});


function appendResult(data) {
    if (!data.price_dict || Object.keys(data.price_dict).length === 0) {
        alert("Uwaga: plik ma zbyt duże rozmiary.");
        return; // Zwróć z funkcji, aby zapobiec dalszemu wykonaniu
    }
    let container = document.querySelector(".results-container");


    let table = container.querySelector(".table");
    if (!table) {
        table = document.createElement("table");
        table.classList.add("table", "table-striped", "table-bordered", "results");
        container.appendChild(table);
    }

    let tbody = table.querySelector("tbody") || document.createElement("tbody");
    table.appendChild(tbody);

    let tr = document.createElement("tr");

    // Przycisk usuwania wiersza
    let tdDelete = document.createElement("td");
    let deleteButton = document.createElement("button");
    deleteButton.textContent = "X";
    deleteButton.classList.add("btn", "btn-danger", "deleteButton");
    deleteButton.addEventListener("click", function() {
        tr.remove();
        updateTotalCost();
    });
    tdDelete.appendChild(deleteButton);
    tr.appendChild(tdDelete);

    // Nazwa pliku
    let tdFilename = document.createElement("td");
    tdFilename.textContent = data.filename;
    tdFilename.style.verticalAlign = "middle";
    tdFilename.classList.add("tdFilename")


    tr.appendChild(tdFilename);

    // Materiał
    let tdMaterial = document.createElement("td");
    let selectMaterial = document.createElement("select");
    selectMaterial.classList.add("form-select");
    selectMaterial.classList.add("selectMaterial");
    for (let mat in data.price_dict) {
        let option = document.createElement("option");
        option.value = mat;
        option.textContent = mat;
        selectMaterial.appendChild(option);
    }
    tdMaterial.appendChild(selectMaterial);
    tr.appendChild(tdMaterial);

    // Ilość sztuk
    let tdQuantity = document.createElement("td");
    let inputQuantity = document.createElement("input");
    inputQuantity.type = "number";
    inputQuantity.min = "1";
    inputQuantity.step = "1";
    inputQuantity.value = "1";
    inputQuantity.classList.add("form-control");
    inputQuantity.classList.add("inputQuantity");
    tdQuantity.appendChild(inputQuantity);
    tr.appendChild(tdQuantity);

    // Cena
    let tdPrice = document.createElement("td");
    tdPrice.textContent = data.price_dict[Object.keys(data.price_dict)[0]].toFixed(2) + " zł";
    tdPrice.style.verticalAlign = "middle";
    tdPrice.classList.add("tdPrice");

    tr.appendChild(tdPrice);

    tbody.appendChild(tr);

    let totalDiv = document.querySelector(".total-cost");
    if (!totalDiv) {
        totalDiv = document.createElement("div");
        totalDiv.classList.add("total-cost", "text-center", "w-50", "mx-auto");
        container.appendChild(totalDiv);
    }

    selectMaterial.addEventListener("change", function() {
        updateRowCost(data, this);
    });
    inputQuantity.addEventListener("input", function() {
        updateRowCost(data, this);
    });
    inputQuantity.addEventListener("blur", function() {
        validateAndAdjustQuantity(this);
        updateRowCost(data, this);
    });

    updateTotalCost();
}

function validateAndAdjustQuantity(inputElement) {
    let quantity = parseInt(inputElement.value);

    if (quantity < 1) {
        inputElement.value = "1";
    } else {
        inputElement.value = quantity; // Usuwa ewentualne liczby niecałkowite lub zbędne zera na początku
    }
}


function updateRowCost(data, element) {
    let row = element.closest("tr");
    let quantityInput = row.querySelector("input[type='number']");
    let quantity = 1;  // Domyślna wartość

    if (quantityInput) {
        quantity = parseFloat(quantityInput.value);
    } else {
        console.error("Nie znaleziono inputa dla ilości");
    }

    let materialSelect = row.querySelector("select");
    let material;

    if (materialSelect) {
        material = materialSelect.value;
    } else {
        console.error("Nie znaleziono selecta dla materiału");
        return;  // Wyjdź z funkcji, jeśli nie możemy znaleźć materiału
    }

    // Aby pobrać cenę dla wybranego materiału:
    let pricePerUnit = parseFloat(data.price_dict[material]);
    let totalPriceForRow = quantity * pricePerUnit;

    // Aktualizacja komórki z ceną dla tego wiersza
    if (row.cells[4]) { // Zaktualizuj indeks komórki, jeśli były wcześniejsze zmiany
        row.cells[4].textContent = totalPriceForRow.toFixed(2) + " zł";
    } else {
        console.error("Nie można znaleźć komórki z ceną");
    }

    updateTotalCost();
}

function updateTotalCost() {
    let rows = document.querySelectorAll(".results-container tbody tr");
    let total = 0;
    if (rows.length === 0) {
        let totalDiv = document.querySelector(".total-cost");
        totalDiv.innerHTML = "";  // Czyszczenie zawartości div'a, gdy nie ma wierszy
        return;  // Wyjdź z funkcji
    }

    rows.forEach(row => {
        let priceCellContent = row.cells[4].textContent;

        let priceString = priceCellContent.replace(/[^0-9.]/g, "");

        let price = parseFloat(priceString);

        if (!isNaN(price)) {
            total += price;
        }
    });

    let totalDiv = document.querySelector(".total-cost");
    totalDiv.innerHTML = `<strong>Cena razem:</strong> ${total.toFixed(2)} zł`;
}