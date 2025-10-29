const API_BASE = '/api';

const DEFAULT_INFILL = 25;
const START_FEE = 50;
const form = document.getElementById('quoteForm');
const dropZone = document.getElementById('dropZone');
const fileInput = document.getElementById('fileInput');
const materialSelect = document.getElementById('materialSelect');
const feedbackArea = document.getElementById('feedbackArea');
const resultPanel = document.getElementById('resultPanel');
const resultBody = document.getElementById('resultBody');
const loader = document.getElementById('loader');
const uploadHint = document.getElementById('uploadHint');
const units = new Intl.NumberFormat('pl-PL', { minimumFractionDigits: 2, maximumFractionDigits: 2 });

async function init() {
  await hydrateMaterials();
  wireUpDropZone();
  wireUpForm();
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
    return;
  }
  const friendlySize = formatBytes(file.size);
  uploadHint.innerHTML = `<span class="font-semibold text-emerald-300">${file.name}</span> · ${friendlySize}`;
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

    toggleLoading(true);

    try {
      const payload = new FormData();
      payload.append('file', file, file.name);
      payload.append('material_id', materialSelect.value);

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
  resultPanel.classList.remove('hidden');
  resultPanel.classList.add('fade-in');

  const breakdown = quote.breakdown;
  const metadata = quote.metadata;
  const filament = metadata.filament ?? {};

  const lines = [];
  lines.push(`<div class="flex items-center justify-between">
      <p class="text-sm uppercase tracking-[0.25em] text-slate-400">Całkowity koszt</p>
      <span class="text-3xl font-semibold text-emerald-300">${formatCurrency(breakdown.total, breakdown.currency)}</span>
    </div>`);

  lines.push('<div class="mt-6 grid gap-3 text-sm text-slate-200">');
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
  lines.push(`<p class="leading-relaxed">Cena obejmuje opłatę startową ${formatCurrency(START_FEE, breakdown.currency)} za przygotowanie druku oraz koszt materiału liczony na podstawie objętości modelu.</p>`);
  if (breakdown.is_estimate) {
    lines.push('<p class="text-xs leading-relaxed text-slate-500">Szacujemy na bazie STL (infill 25%, podpory włączone). Jeśli masz G-code lub 3MF z Orca Slicer, użyj go dla pełnej dokładności.</p>');
  }
  lines.push('</div>');

  resultBody.innerHTML = lines.join('\n');
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
