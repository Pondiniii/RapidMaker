const API_BASE = '/api';

const form = document.getElementById('quoteForm');
const dropZone = document.getElementById('dropZone');
const fileInput = document.getElementById('fileInput');
const materialSelect = document.getElementById('materialSelect');
const quantityInput = document.getElementById('quantityInput');
const manualNotice = document.getElementById('manualNotice');
const manualEmail = document.getElementById('manualEmail');
const manualMailto = document.getElementById('manualMailto');
const feedbackArea = document.getElementById('feedbackArea');
const resultPanel = document.getElementById('resultPanel');
const resultBody = document.getElementById('resultBody');
const loader = document.getElementById('loader');
const uploadHint = document.getElementById('uploadHint');
const submitButton = document.getElementById('quoteSubmit');
const submitLabel = document.getElementById('quoteSubmitLabel');
const turnaroundOptions = document.querySelectorAll('.turnaround-option');
let currentTurnaround = 'standard';
const units = new Intl.NumberFormat('pl-PL', { minimumFractionDigits: 2, maximumFractionDigits: 2 });

async function init() {
  await hydrateMaterials();
  wireUpDropZone();
  wireUpForm();
  setupTurnaroundToggle();
  setupQuantityWatcher();
}

async function hydrateMaterials() {
  try {
    const res = await fetch(`${API_BASE}/materials`);
    if (!res.ok) {
      throw new Error(`HTTP ${res.status}`);
    }
    const materials = await res.json();
    materials.sort((a, b) => a.display_name.localeCompare(b.display_name));
    for (const mat of materials) {
      const option = document.createElement('option');
      option.value = mat.id;
      option.textContent = `${mat.display_name} · ${mat.kind.toUpperCase()} (${mat.density_g_cm3.toFixed(2)} g/cm³)`;
      materialSelect.appendChild(option);
    }
  } catch (error) {
    pushFeedback(`Nie udało się pobrać listy materiałów: ${error.message}`, 'error');
  }
}

function wireUpDropZone() {
  ['dragenter', 'dragover'].forEach(evt => {
    dropZone.addEventListener(evt, e => {
      e.preventDefault();
      e.stopPropagation();
      dropZone.classList.add('is-dragover');
    });
  });

  ['dragleave', 'drop'].forEach(evt => {
    dropZone.addEventListener(evt, e => {
      e.preventDefault();
      e.stopPropagation();
      dropZone.classList.remove('is-dragover');
    });
  });

  dropZone.addEventListener('click', () => fileInput.click());

  dropZone.addEventListener('drop', e => {
    if (e.dataTransfer?.files?.length) {
      fileInput.files = e.dataTransfer.files;
      renderSelectedFile();
    }
  });

  fileInput.addEventListener('change', renderSelectedFile);
}

function renderSelectedFile() {
  const file = fileInput.files?.[0];
  if (!file) {
    uploadHint.innerHTML = 'Przeciągnij plik STL / 3MF / GCODE lub kliknij aby wybrać.';
    updateManualMailto();
    return;
  }
  const friendlySize = formatBytes(file.size);
  uploadHint.innerHTML = `<span class="font-semibold text-emerald-300">${file.name}</span> · ${friendlySize}`;
  updateManualMailto();
}

function setupTurnaroundToggle() {
  if (!turnaroundOptions?.length) {
    return;
  }
  turnaroundOptions.forEach(option => {
    option.addEventListener('click', () => {
      const value = option.dataset.turnaround || 'standard';
      currentTurnaround = value;
      turnaroundOptions.forEach(btn => btn.classList.toggle('is-active', btn === option));
      updateManualMailto();
    });
  });
}

function setupQuantityWatcher() {
  if (!quantityInput) {
    return;
  }
  quantityInput.addEventListener('input', () => {
    sanitizeQuantity();
    updateManualState();
  });
  quantityInput.addEventListener('blur', () => {
    sanitizeQuantity(true);
    updateManualState();
  });
  manualEmail?.addEventListener('input', () => {
    updateManualMailto();
  });
  updateManualState();
}

function sanitizeQuantity(force = false) {
  if (!quantityInput) return;
  const parsed = parseInt(quantityInput.value, 10);
  if (!Number.isFinite(parsed) || parsed <= 0 || force) {
    const safeValue = Number.isFinite(parsed) && parsed > 0 ? parsed : 1;
    quantityInput.value = String(safeValue);
  }
}

function getQuantity() {
  if (!quantityInput) return 1;
  const parsed = parseInt(quantityInput.value, 10);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : 1;
}

function isManualFlow(quantity = getQuantity()) {
  return quantity > 30;
}

function updateManualState() {
  const manual = isManualFlow();
  manualNotice?.classList.toggle('hidden', !manual);
  if (submitLabel && submitButton) {
    const label = manual
      ? submitButton.dataset.manualLabel || 'Wyślij do analizy'
      : submitButton.dataset.autoLabel || 'Policz koszt';
    submitLabel.textContent = label;
  }
  if (!manual && manualEmail) {
    manualEmail.value = manualEmail.value.trim();
  }
  updateManualMailto();
}

function updateManualMailto() {
  if (!manualMailto) return;
  const quantity = getQuantity();
  const file = fileInput?.files?.[0];
  const email = manualEmail?.value?.trim() ?? '';
  const turnaroundLabel = currentTurnaround === 'express' ? '1 dzień (express)' : '3–5 dni roboczych';
  const subject = encodeURIComponent(`Analiza serii – ${quantity} szt.`);
  const bodyLines = [
    'Cześć RapidMaker,',
    '',
    'Proszę o przygotowanie oferty dla większej serii:',
    `- Model: ${file ? file.name : 'nazwa pliku'}`,
    `- Ilość: ${quantity} sztuk`,
    `- Termin: ${turnaroundLabel}`,
    '',
    `Kontakt: ${email || 'podaj proszę e-mail zwrotny'}`,
    '',
    'Dodatkowe informacje:'
  ];
  const body = encodeURIComponent(bodyLines.join('\n'));
  manualMailto.href = `mailto:druk@rapidmaker.pl?subject=${subject}&body=${body}`;
}

function wireUpForm() {
  form.addEventListener('submit', async event => {
    event.preventDefault();
    clearFeedback();

    const file = fileInput.files?.[0];
    if (!file) {
      pushFeedback('Dodaj plik modelu zanim wyślesz formularz.', 'error');
      return;
    }

    const maxBytes = parseInt(form.dataset.maxBytes ?? '52428800', 10);
    if (file.size > maxBytes) {
      pushFeedback('Plik jest zbyt duży – limit to 50 MB.', 'error');
      return;
    }

    const quantity = getQuantity();
    if (isManualFlow(quantity)) {
      updateManualMailto();
      pushFeedback('Seria powyżej 30 sztuk: przygotowaliśmy szkic maila do indywidualnej wyceny.', 'info');
      manualMailto?.focus();
      return;
    }

    toggleLoading(true);

    try {
      const payload = new FormData();
      payload.append('file', file, file.name);
      payload.append('material_id', materialSelect.value);
      payload.append('quantity', String(quantity));
      payload.append('turnaround', currentTurnaround);

      const response = await fetch(`${API_BASE}/quote`, {
        method: 'POST',
        body: payload,
      });

      if (!response.ok) {
        const errorBody = await safeJson(response);
        const message = errorBody?.message ?? `Serwer zwrócił błąd ${response.status}`;
        throw new Error(message);
      }

      const quote = await response.json();
      renderQuote(quote);
      pushFeedback('Wycena gotowa ✨', 'success');
    } catch (error) {
      pushFeedback(`Nie udało się obliczyć kosztu: ${error.message}`, 'error');
    } finally {
      toggleLoading(false);
    }
  });
}

function renderQuote(quote) {
  const breakdown = quote.breakdown ?? {};

  if (breakdown.review_required) {
    showManualReview(quote);
    return;
  }

  resultPanel.classList.remove('hidden');
  resultPanel.classList.add('fade-in');
  manualNotice?.classList.add('hidden');
  if (submitLabel && submitButton) {
    submitLabel.textContent = submitButton.dataset.autoLabel || 'Policz koszt';
  }

  const metadata = quote.metadata;
  const filament = metadata.filament ?? {};
  const materialName = quote.material?.display_name ?? 'Materiał';
  const isExpress = breakdown.turnaround === 'express';
  const discountRate = breakdown.discount_rate ? breakdown.discount_rate * 100 : 0;

  const lines = [];
  lines.push(`<div class="flex items-center justify-between">
      <p class="text-sm uppercase tracking-[0.25em] text-slate-400">Całkowity koszt</p>
      <span class="text-3xl font-semibold text-emerald-300">${formatCurrency(breakdown.total, breakdown.currency)}</span>
    </div>`);

  lines.push('<div class="mt-6 grid gap-3 text-sm text-slate-200">');
  lines.push(renderFact('Materiał', materialName));
  lines.push(renderFact('Ilość modeli', `${breakdown.quantity ?? 1} szt.`));
  lines.push(renderFact('Cena za sztukę', formatCurrency(breakdown.unit_total, breakdown.currency)));
  lines.push(
    renderFact(
      'Termin',
      isExpress ? '1 dzień (express)' : '3–5 dni roboczych'
    )
  );
  if (discountRate > 0) {
    lines.push(renderFact('Zniżka ilościowa', `-${discountRate.toFixed(0)}%`));
  }
  if (isExpress) {
    lines.push(renderFact('Dopłata express', '+35% priorytet produkcyjny'));
  }
  lines.push(renderFact('Zużycie filamentu', `${formatNumber(filament.filament_used_g)} g`));
  if (filament.filament_used_mm) {
    lines.push(renderFact('Długość filamentu', `${formatNumber(filament.filament_used_mm / 1000)} m`));
  }
  if (filament.print_time_human) {
    lines.push(renderFact('Czas wydruku', filament.print_time_human));
  }
  if (metadata.geometry?.volume_cm3) {
    lines.push(renderFact('Objętość modelu', `${formatNumber(metadata.geometry.volume_cm3)} cm³`));
  }
  if (filament.infill_percent != null) {
    const suffix = filament.supports_enabled ? ' (z podporami)' : '';
    lines.push(renderFact('Infill', `${filament.infill_percent}%${suffix}`));
  }
  if (filament.solid_ratio_percent != null) {
    lines.push(renderFact('Szacowany wkład stały', `${formatNumber(filament.solid_ratio_percent)}%`));
  }
  lines.push('</div>');

  lines.push('<div class="mt-6 grid gap-3 text-sm text-slate-400">');
  lines.push(
    `<p class="leading-relaxed">Cena obejmuje opłatę startową ${formatCurrency(
      breakdown.base_fee,
      breakdown.currency
    )} oraz koszt materiału liczony na podstawie objętości i masy modelu.</p>`
  );
  if (discountRate > 0) {
    lines.push('<p class="leading-relaxed">Zastosowaliśmy rabat ilościowy na materiał – każda kolejna pula 10 sztuk powyżej 40 daje dodatkowe 5% zniżki.</p>');
  }
  if (isExpress) {
    lines.push('<p class="leading-relaxed">Tryb express rezerwuje drukarki na dedykowany slot, obejmuje wydłużone zmiany i ręczną kontrolę jakości.</p>');
  }
  if (breakdown.is_estimate) {
    lines.push('<p class="text-xs leading-relaxed text-slate-500">Szacujemy na bazie STL (infill 25%, podpory włączone). Jeśli masz G-code lub 3MF z Orca Slicer, użyj go dla pełnej dokładności.</p>');
  }
  lines.push('</div>');

  resultBody.innerHTML = lines.join('\n');
}

function showManualReview(quote) {
  manualNotice?.classList.remove('hidden');
  if (submitLabel && submitButton) {
    submitLabel.textContent = submitButton.dataset.manualLabel || 'Wyślij do analizy';
  }
  resultPanel.classList.add('hidden');
  updateManualMailto();
  pushFeedback('Ten model wymaga analizy inżynieryjnej – skontaktujemy się z ofertą indywidualną.', 'info');
}

function renderFact(title, value) {
  return `<div class="flex items-center justify-between">
    <span class="text-slate-400">${title}</span>
    <span class="text-slate-100 font-medium">${value ?? '—'}</span>
  </div>`;
}

function toggleLoading(isLoading) {
  loader.classList.toggle('hidden', !isLoading);
  form.querySelectorAll('input, button, select').forEach(el => (el.disabled = isLoading));
}

function pushFeedback(message, variant = 'info') {
  const colors = {
    info: 'text-slate-200 bg-slate-800/80 border-slate-700/60',
    success: 'text-emerald-200 bg-emerald-900/30 border-emerald-500/30',
    error: 'text-rose-200 bg-rose-900/30 border-rose-500/30',
  };
  const container = document.createElement('div');
  container.className = `glass-panel gradient-border rounded-xl border px-4 py-3 fade-in ${colors[variant] ?? colors.info}`;
  container.innerHTML = `<p class="text-sm font-medium">${message}</p>`;
  feedbackArea.appendChild(container);
}

function clearFeedback() {
  feedbackArea.innerHTML = '';
}

async function safeJson(response) {
  try {
    return await response.json();
  } catch (_) {
    return null;
  }
}

function formatCurrency(amount, currency) {
  return new Intl.NumberFormat('pl-PL', {
    style: 'currency',
    currency: currency || 'PLN',
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(amount ?? 0);
}

function formatNumber(value) {
  if (value == null || Number.isNaN(value)) {
    return '—';
  }
  return units.format(value);
}

function formatBytes(bytes) {
  const units = ['B', 'KB', 'MB', 'GB'];
  if (!bytes) return '0 B';
  const index = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  const num = bytes / Math.pow(1024, index);
  return `${num.toFixed(index === 0 ? 0 : 1)} ${units[index]}`;
}

init();
