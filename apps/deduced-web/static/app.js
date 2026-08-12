const CATEGORY_ICONS = {
  cars: '🚗',
  companies: '🏢',
  countries: '🌎',
};

const COMPARISON_SYMBOL = {
  match: '=',
  different: '×',
  higher: '↑',
  lower: '↓',
  partial: '~',
};

let lastState = null;
let categoryAnswers = [];
let roundGuesses = [];

function showScreen(id) {
  document.querySelectorAll('.screen').forEach((screen) => screen.classList.remove('active'));
  document.getElementById(id).classList.add('active');
  document.getElementById('bottomNav').classList.toggle('hidden', id === 'game');
  window.scrollTo(0, 0);
}

function nav(id, button) {
  showScreen(id);
  document.querySelectorAll('.nav-btn').forEach((btn) => btn.classList.remove('active'));
  if (button) button.classList.add('active');
}

function categoryIcon(id) {
  return CATEGORY_ICONS[id] || '🎯';
}

async function api(path, options) {
  const response = await fetch(path, {
    headers: { 'Content-Type': 'application/json' },
    ...options,
  });
  const body = await response.json().catch(() => null);
  if (!response.ok) {
    const message = body && body.error ? body.error : `request to ${path} failed`;
    throw new Error(message);
  }
  return body;
}

async function goToCategories() {
  showScreen('categories');
  const grid = document.getElementById('categoryGrid');
  grid.innerHTML = '<p style="color:var(--muted)">Loading categories…</p>';

  const categories = await api('/api/categories');

  grid.innerHTML = '';
  for (const category of categories) {
    const card = document.createElement('div');
    card.className = 'category-card';
    card.onclick = () => startGame(category.id);
    card.innerHTML = `
      <div class="icon">${categoryIcon(category.id)}</div>
      <h3>${category.name}</h3>
      <small>${category.answer_count} to discover · ${category.attempts} attempts</small>
    `;
    grid.appendChild(card);
  }
}

function exitGame() {
  document.getElementById('resultOverlay').classList.remove('visible');
  goToCategories();
}

async function startGame(categoryId) {
  const [state, answers] = await Promise.all([
    api('/api/round', {
      method: 'POST',
      body: JSON.stringify({ category_id: categoryId }),
    }),
    api(`/api/categories/${categoryId}/answers`),
  ]);

  lastState = state;
  categoryAnswers = answers;
  roundGuesses = [];

  document.getElementById('resultOverlay').classList.remove('visible');
  renderGameShell(state);
  showScreen('game');
  setTimeout(() => document.getElementById('guessInput').focus(), 120);
}

async function restartRound() {
  if (!lastState) {
    exitGame();
    return;
  }
  await startGame(lastState.category_id);
}

function renderAttemptDots(container, usedCount, maxCount) {
  container.innerHTML = '';
  for (let i = 0; i < maxCount; i += 1) {
    const dot = document.createElement('div');
    dot.className = 'attempt-dot' + (i < usedCount ? ' used' : '');
    container.appendChild(dot);
  }
}

function renderRevealGrid(usedCount, maxCount) {
  const grid = document.getElementById('revealGrid');
  grid.innerHTML = '';

  const totalCells = 30;
  const revealedThreshold = usedCount * Math.ceil((totalCells + 5) / maxCount);

  for (let i = 0; i < totalCells; i += 1) {
    const cell = document.createElement('div');
    const cellThreshold = (i * 7) % totalCells;
    cell.style.opacity = cellThreshold < revealedThreshold ? '.05' : (i % 3 === 0 ? '.32' : '.88');
    grid.appendChild(cell);
  }
}

function chipHtml(label, value, type, symbol) {
  return `<span class="clue-chip ${type}">${label}: ${value} <strong>${symbol}</strong></span>`;
}

function comparisonChipClass(tag) {
  if (tag === 'match') return 'good';
  if (tag === 'different') return 'bad';
  return tag;
}

function comparisonChipsHtml(comparisons) {
  return comparisons
    .map((c) => chipHtml(c.label, c.guessed_value, comparisonChipClass(c.comparison), COMPARISON_SYMBOL[c.comparison] || '?'))
    .join('');
}

function computeKnownFacts() {
  const facts = [];
  const bounds = {};

  for (const guess of roundGuesses) {
    for (const c of guess.comparisons) {
      if (c.comparison === 'match') {
        if (!facts.some((f) => f.label === c.label && f.value === c.guessed_value)) {
          facts.push({ label: c.label, value: c.guessed_value, type: 'good', symbol: COMPARISON_SYMBOL.match });
        }
      } else if (c.comparison === 'higher' || c.comparison === 'lower') {
        const numeric = Number.parseFloat(c.guessed_value);
        if (Number.isNaN(numeric)) continue;

        const bound = bounds[c.key] || { label: c.label, min: null, max: null };
        if (c.comparison === 'higher') {
          bound.min = bound.min === null ? numeric : Math.max(bound.min, numeric);
        } else {
          bound.max = bound.max === null ? numeric : Math.min(bound.max, numeric);
        }
        bounds[c.key] = bound;
      }
    }
  }

  for (const bound of Object.values(bounds)) {
    if (bound.min !== null) facts.push({ label: bound.label, value: `> ${bound.min}`, type: 'higher', symbol: COMPARISON_SYMBOL.higher });
    if (bound.max !== null) facts.push({ label: bound.label, value: `< ${bound.max}`, type: 'lower', symbol: COMPARISON_SYMBOL.lower });
  }

  return facts;
}

function renderKnownFacts() {
  const facts = computeKnownFacts();
  const panel = document.getElementById('knownPanel');

  if (!facts.length) {
    panel.classList.remove('visible');
    document.getElementById('knownChips').innerHTML = '';
    return;
  }

  panel.classList.add('visible');
  document.getElementById('knownChips').innerHTML = facts
    .slice(0, 6)
    .map((f) => chipHtml(f.label, f.value, f.type, f.symbol))
    .join('');
}

function renderHistory() {
  const box = document.getElementById('guessHistory');

  if (!roundGuesses.length) {
    box.innerHTML = '<div style="color:var(--muted);font-size:12px;padding:6px 2px">Your clues will build here after each guess.</div>';
    return;
  }

  box.innerHTML = '';
  roundGuesses.slice().reverse().forEach((guess, idx) => {
    const div = document.createElement('div');
    div.className = 'guess-entry ' + (idx === 0 ? 'latest' : 'compact');
    div.innerHTML = `
      <div class="guess-row">
        <div class="guess-name">${guess.name}</div>
        <div class="guess-result">INCORRECT</div>
      </div>
      <div class="guess-chips">${comparisonChipsHtml(guess.comparisons)}</div>
    `;
    box.appendChild(div);
  });
}

function renderSuggestions() {
  const wrap = document.getElementById('suggestions');
  wrap.innerHTML = '';

  if (roundGuesses.length > 0) return;

  for (const answer of categoryAnswers.slice(0, 6)) {
    const pill = document.createElement('button');
    pill.className = 'pill';
    pill.type = 'button';
    pill.textContent = answer.name;
    pill.onclick = () => quickGuess(answer.name);
    wrap.appendChild(pill);
  }
}

function quickGuess(name) {
  document.getElementById('guessInput').value = name;
  submitGuess();
}

function renderGameShell(state) {
  const remaining = state.max_attempts - state.attempts_used;

  document.getElementById('gameCategoryLabel').textContent =
    `${categoryIcon(state.category_id)} ${state.category_name}`;
  document.getElementById('gameHeartsLabel').textContent = `❤️ ${remaining}`;
  document.getElementById('gameRoundLabel').textContent = state.category_name.toUpperCase();
  document.getElementById('guessCounter').textContent =
    `${state.attempts_used} of ${state.max_attempts} guesses`;

  renderAttemptDots(document.getElementById('attemptDots'), state.attempts_used, state.max_attempts);
  renderRevealGrid(state.attempts_used, state.max_attempts);

  const revealLevel = Math.min(state.attempts_used + 1, state.max_attempts);
  document.getElementById('revealFill').style.width = `${(revealLevel / state.max_attempts) * 100}%`;
  document.getElementById('revealCount').textContent = `${revealLevel}/${state.max_attempts}`;

  const card = document.getElementById('imageCard');
  card.className = 'image-card' + (state.attempts_used ? ` shrink-${Math.min(state.attempts_used, 4)}` : '');

  renderKnownFacts();
  renderHistory();
  renderSuggestions();

  document.getElementById('guessInput').value = '';
  document.getElementById('autocomplete').classList.remove('visible');
}

function flashWrong() {
  const toast = document.getElementById('wrongToast');
  toast.classList.add('show');
  setTimeout(() => toast.classList.remove('show'), 850);
}

function formatElapsed(seconds) {
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${mins}:${String(secs).padStart(2, '0')}`;
}

function showResultOverlay(state) {
  const won = state.status === 'won';

  const badge = document.getElementById('resultBadge');
  badge.textContent = won ? 'Solved' : 'Round Over';
  badge.classList.toggle('lost', !won);

  const headline = document.getElementById('resultHeadline');
  headline.textContent = won ? 'DEDUCED!' : 'ANSWER REVEALED';
  headline.style.color = won ? 'var(--green)' : 'var(--red)';

  document.getElementById('resultSub').textContent = won
    ? 'You figured it out.'
    : 'Here is the answer you were after.';

  document.getElementById('resultAnswerName').textContent = state.answer_name;
  document.getElementById('resultAnswerInitial').textContent = state.answer_name.charAt(0).toUpperCase();
  document.getElementById('resultAttempts').textContent = `${state.attempts_used}/${state.max_attempts}`;
  document.getElementById('resultTime').textContent = formatElapsed(state.elapsed_seconds || 0);
  document.getElementById('resultScore').textContent = state.score || 0;

  document.getElementById('resultOverlay').classList.add('visible');
}

async function submitGuess() {
  const input = document.getElementById('guessInput');
  const raw = input.value.trim();
  if (!raw) return;

  const match = categoryAnswers.find((a) => a.name.toLowerCase() === raw.toLowerCase());
  if (!match) {
    input.animate(
      [
        { transform: 'translateX(0)' },
        { transform: 'translateX(-5px)' },
        { transform: 'translateX(5px)' },
        { transform: 'translateX(0)' },
      ],
      { duration: 220 },
    );
    return;
  }

  if (roundGuesses.some((g) => g.name === match.name)) {
    input.value = '';
    return;
  }

  document.getElementById('autocomplete').classList.remove('visible');

  const state = await api('/api/guess', {
    method: 'POST',
    body: JSON.stringify({ name: match.name }),
  });

  lastState = state;
  input.value = '';

  if (state.last_guess) {
    roundGuesses.push({ name: state.last_guess.guessed_name, comparisons: state.last_guess.comparisons });
  }

  renderGameShell(state);

  if (state.status === 'playing') {
    flashWrong();
    setTimeout(() => input.focus(), 60);
  } else {
    showResultOverlay(state);
  }
}

function setupAutocomplete() {
  const input = document.getElementById('guessInput');
  const box = document.getElementById('autocomplete');

  input.addEventListener('input', () => {
    const query = input.value.trim().toLowerCase();
    box.innerHTML = '';

    if (!query) {
      box.classList.remove('visible');
      return;
    }

    const guessedNames = new Set(roundGuesses.map((g) => g.name));
    const matches = categoryAnswers
      .filter((a) => a.name.toLowerCase().includes(query) && !guessedNames.has(a.name))
      .slice(0, 5);

    for (const match of matches) {
      const button = document.createElement('button');
      button.type = 'button';
      button.textContent = match.name;
      button.onclick = () => {
        box.classList.remove('visible');
        input.value = match.name;
        submitGuess();
      };
      box.appendChild(button);
    }

    box.classList.toggle('visible', matches.length > 0);
  });

  input.addEventListener('keydown', (event) => {
    if (event.key === 'Enter') submitGuess();
  });
}

setupAutocomplete();

const themes = {
  purple: { primary: '#7c4dff', light: '#9b7cff', dark: '#5128cf' },
  blue: { primary: '#078eea', light: '#29b7ff', dark: '#045aa4' },
  green: { primary: '#20aa61', light: '#45d784', dark: '#126f40' },
  orange: { primary: '#f47c16', light: '#ffa33f', dark: '#a74c09' },
  gold: { primary: '#c9982f', light: '#efc55a', dark: '#805d15' },
  teal: { primary: '#0f969c', light: '#31c9cb', dark: '#075d64' },
};

function theme(name) {
  const t = themes[name];
  if (!t) return;
  const root = document.documentElement;
  root.style.setProperty('--primary', t.primary);
  root.style.setProperty('--primary-light', t.light);
  root.style.setProperty('--primary-dark', t.dark);
}

const DEVICE_PRESETS = {
  phone: { width: 390, height: 844 },
  tablet: { width: 834, height: 1194 },
  desktop: { width: 1440, height: 900 },
};

function setDevice(name, button) {
  document.querySelectorAll('.device-btn').forEach((btn) => btn.classList.remove('active'));
  if (button) button.classList.add('active');

  const frame = document.getElementById('deviceFrame');
  const scaler = document.getElementById('deviceScaler');
  const stage = document.getElementById('deviceStage');
  const preset = DEVICE_PRESETS[name];

  if (!preset) {
    document.body.classList.remove('framed');
    frame.style.cssText = '';
    scaler.style.cssText = '';
    return;
  }

  document.body.classList.add('framed');

  const availableWidth = stage.clientWidth - 32;
  const availableHeight = window.innerHeight - 120;
  const scale = Math.min(1, availableWidth / preset.width, availableHeight / preset.height);

  frame.style.width = `${preset.width}px`;
  frame.style.height = `${preset.height}px`;
  frame.style.transform = `scale(${scale})`;

  scaler.style.width = `${preset.width * scale}px`;
  scaler.style.height = `${preset.height * scale}px`;
}

window.addEventListener('resize', () => {
  const active = document.querySelector('.device-btn.active');
  if (active) setDevice(active.dataset.device, active);
});
