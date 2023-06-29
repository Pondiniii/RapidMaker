const FDM = 'FDM';
const SLA = 'SLA';

const FDM_VOLUME = {x: 255, y: 255, z: 255};
const SLA_VOLUME = {x: 129, y: 80, z: 150};

const fdmMaterials = {
    'PLA': {price: 100, density: 1.24},
};

const slaMaterials = {
    'ABS-like': {price: 300, density: 1.05},
    'Nylon-like': {price: 400, density: 1.15},
};

const printerTypes = {
    [FDM]: fdmMaterials,
    [SLA]: slaMaterials,
}

function buildPrinterTypeSelect() {
    return `
        <select class="printerType" style="width: 100px;">
            <option value="${FDM}">${FDM}</option>
            <option value="${SLA}">${SLA}</option>
        </select>
    `;
}


function buildMaterialSelect() {
    return '<select class="material" style="width: 100px;"></select>';
}

// function buildCheckbox() {
//     return '<input type="checkbox" class="print-checkbox" checked>';
// }

function buildQuantityInput() {
    return '<input type="number" class="quantity" min="1" value="1" step="1" style="width: 50px;">';
}


function buildStrengthCheckbox() {
    return '<input type="checkbox" class="strength-checkbox checkbox">';
}

function buildDeleteButton() {
    return '<button class="delete-button btn btn-danger">X</button>';
}


function appendRowToTable(data) {
    let filename = data.filename;
    if(filename.length > 25) {
        filename = filename.substring(0, 25) + "...";
    }
    $("table tbody").append(
        `<tr>
            <td class="align-middle">${buildDeleteButton()}</td>
            <td class="align-middle">${buildPrinterTypeSelect()}</td>
            <td class="align-middle">${buildMaterialSelect()}</td>
            <td class="align-middle">${buildQuantityInput()}</td>
            <td class="align-middle">${buildStrengthCheckbox()}</td>
            <td class="align-middle">${filename}</td>
            <td class="fit align-middle">${fitsInPrinter(data, FDM) ? "Tak" : "Możliwe problemy ⚠"}</td>
            <td class="cost align-middle">0 zł</td>
        </tr>`
    );
}

function fitsInPrinter(item, printerType) {
    let printerVolume = printerType === 'FDM' ? FDM_VOLUME : SLA_VOLUME;

    // sprawdzamy, czy obiekt mieści się bez obrotu
    if (item.x <= printerVolume.x && item.y <= printerVolume.y && item.z <= printerVolume.z) {
        return true;
    }

    // obliczamy długość przekątnej bounding boxa obiektu
    let itemDiagonal = Math.sqrt(Math.pow(item.x, 2) + Math.pow(item.y, 2) + Math.pow(item.z, 2));

    // sprawdzamy, czy obiekt zmieści się po obróceniu
    if (itemDiagonal <= printerVolume.x && itemDiagonal <= printerVolume.y && itemDiagonal <= printerVolume.z) {
        return true;
    }

    // jeśli obiekt nie zmieścił się ani bez obrotu, ani po obróceniu, zwracamy false
    return false;
}

function updateTable(data) {
    // Loop: append all rows to the table, and update material options and trigger change
    for (let i = 0; i < data.length; i++) {
        let item = data[i];
        appendRowToTable(item);

        let row = $("table tbody tr").last(); // get the newly appended row
        updateMaterialOptions(row, FDM);
        row.find('.printerType, .material').change();

        row.find(".print-checkbox, .material, .quantity, .printerType, .strength-checkbox").on('change', (event) => onChange(event, item, null));

        row.find(".delete-button").on('click', function() {
            $(this).closest('tr').remove();
            calculateTotalCost(); // recalculate the total cost after a row is removed
        });

        onChange(null, item, row); // calculate initial cost
    }
    $(".quantity").on("input", blockNegativeInputs);
}



function blockNegativeInputs() {
    let val = parseInt(this.value);
    if (val < 0) {
        this.value = 1;
    }
}


function onChange(event, item, row) {
    console.log('onChange triggered');

    if (!row) { // if no row provided, determine it from the event
        row = $(event.target).closest('tr');
    }

    let printerType = row.find(".printerType").val();

    if (!item) { // if no item provided, determine it from the row index
        let rowIndex = row.index();
        console.log(`Row index: ${rowIndex}`);
        item = data[rowIndex];
    }

    if (event && event.target.className === 'printerType') {
        updateMaterialOptions(row, printerType);
        row.find('.fit').text(fitsInPrinter(item, printerType) ? "✓" : "X");
    }

    let volume = item.volume;
    let material = row.find(".material").val();
    let quantity = Number(row.find(".quantity").val());
    let materials = printerTypes[printerType];
    let isStrengthChecked = row.find(".strength-checkbox").prop('checked');
    let cost = calculateCost(volume, materials[material].density, materials[material].price, quantity, isStrengthChecked);
    row.find('.cost').text(`${cost.toFixed(2)} zł`);

    calculateTotalCost();
}


function updateMaterialOptions(row, printerType) {
    let materials = printerTypes[printerType];
    let materialSelect = row.find('.material');
    materialSelect.empty();
    for (let material in materials) {
        materialSelect.append(new Option(material, material));
    }
    materialSelect.change();
}

function calculateCost(volume, density, pricePerKg, quantity, isStrengthChecked) {
    let mass = volume * density;  // in grams
    let pricePerGram = pricePerKg / 1000;  // convert price to price per gram
    let cost = mass * pricePerGram * quantity;  // in zlotys
    if (isStrengthChecked) {
        cost *= 1.3;
    }
    return cost;
}

function calculateTotalCost() {
    let totalCost = 0;
    $("table tbody tr").each(function() {
        let row = $(this);
        // if(row.find(".print-checkbox").prop('checked')) {
        let cost = parseFloat(row.find('.cost').text());
        totalCost += cost;
        // }
    });
    $('#totalCost').text(`Razem: ${totalCost.toFixed(2)} zł`);
}