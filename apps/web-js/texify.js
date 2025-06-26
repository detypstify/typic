import { pipeline } from 'https://cdn.jsdelivr.net/npm/@xenova/transformers/dist/transformers.min.js';

let texify;
const processBtn = document.getElementById('processBtn');

async function initialize() {
  texify = await pipeline('image-to-text', 'Xenova/texify');
  processBtn.disabled = false;
}

async function processImage() {
  const imageFile = document.getElementById('imageUpload').files[0];
  if (!imageFile) {
    alert('Please upload an image first.');
    return;
  }

  try {
    processBtn.disabled = true;
    // document.getElementById('status').innerText = 'Processing...';
    document.getElementById('latexOutput').innerText = '';
    document.getElementById('typstOutput').innerText = '';
    // document.getElementById('debugOutput').textContent = '';
    // document.getElementById('debug').classList.add('hidden');

    const imageUrl = URL.createObjectURL(imageFile);
    const latex = await texify(imageUrl, { max_new_tokens: 384 });
    const latexText = latex[0].generated_text;

    document.getElementById("latexOutput").innerText = latexText;
    const sanitized = sanitizeLatex(latexText);
    const typstOutput = tex2typst(sanitized);
    document.getElementById("typstOutput").innerText = typstOutput;
  } catch (error) {
    // document.getElementById('status').innerText = `An error occurred: ${error.message}`;
    console.error(error);
    // document.getElementById('debug').classList.remove('hidden');
    // document.getElementById('debugOutput').textContent = error.stack || error;
  } finally {
    processBtn.disabled = false;
  }
}

document.addEventListener('DOMContentLoaded', initialize);
document.getElementById('processBtn').addEventListener('click', processImage);

function sanitizeLatex(input) {
  let s = input;

  s = startProcessing(s);
  s = processMathcal(s);
  s = processSp(s);
  s = processTable(s);
  s = processVspace(s);
  s = processHspace(s);
  s = processBf(s);
  s = processTextMods(s);
  s = processPartial(s);
  s = nukeTrailing(s);
  s = processSmallSpace(s);
  s = processWeirdCharacters(s);
  s = postProcessing(s);

  return s;
}

function startProcessing(s) {
  return s
    .replace(/\\,/g, ' ')                        // remove thin space
    .replace(/\\left\{/g, '{').replace(/\\right\}/g, '}') // remove \left{ and \right}
    .replace(/\\\./g, '')                        // remove \. if present
    .replace(/\$\$/g, '')                        // remove display math delimiters
    .trim();
}
function processMathcal(s) {
  return s.replace(/\\cal/g, '\\mathcal');
}

function processSp(s) {
  return s.replace(/\\sp /g, '^ ').replace(/\\sb /g, '_');
}

function processTable(s) {
  return s.replace(/\\begin\{array\}\s+\{[\sclr]*\}/g, m => m.replace(/\s+/g, ''));
}

function processVspace(s) {
  if (/\\vspace|\\thinspace/.test(s)) return '';
  return s;
}

function processHspace(s) {
  if (/\\hspace\s*\{\s*[-~]/.test(s)) return '';
  if (/\\hspace\s*\{[\s0-9]*\./.test(s)) return '';
  if (/\\hspace\s*\{.*ex.*\}/.test(s)) return '';
  if (/\\hspace\s*\*\s*\{ /.test(s)) return '';
  if (/\\hspace\s*\{ ([\s\.0-9]*)mm \}/.test(s)) return '';

  s = s.replace(/\\hspace\s*\{([0-9]+)mm\}/g, (_, val) => {
    const converted = parseInt(val, 10) * 10;
    return `\\hspace{${converted}cm}`;
  });

  s = s.replace(/\\hspace\s*\{([0-9a-z\s]*)\}/g, (_, val) =>
    `\\hspace{${val.replace(/\s+/g, '')}}`
  );

  return s;
}

function processBf(s) {
  return s
    .replace(/\\bf/g, '\\mathbf')
    .replace(/\\boldmath/g, '\\mathbf')
    .replace(/\\it/g, '\\mathit')
    .replace(/\\tt/g, '\\texttt')
    .replace(/\\sf/g, '')
    .replace(/\\i /g, 'i ');
}

function processTextMods(s) {
  const blocklist = [
    /\\small/, /\\tiny/, /\\c /, /\\footnotesize/, /\\scriptsize/,
    /\\Bigl/, /\\Bigr/, /\\hline/, /\\raisebox/, /\\vphantom/,
    /\\textup/, /`/, /\\mit /, /\\do /, /\\em /, /\\atop/,
    /\\large /, /\\label/, /\\raise /
  ];
  for (const pattern of blocklist) {
    if (pattern.test(s)) return '';
  }
  return s;
}

function processPartial(s) {
  return s.replace(/\\d /g, '\\partial ');
}

function processSmallSpace(s) {
  return s.replace(/\\b /g, '\\! ');
}

function processWeirdCharacters(s) {
  return s.replace(/\\L /g, 'L ').replace(/\\l /g, 'l ');
}

function nukeTrailing(s) {
  return s.endsWith('\\') ? s.slice(0, -1) : s;
}

function postProcessing(s) {
  return processResizedParens(s);
}

function processResizedParens(s) {
  return s.replace(/(#scale\(x: \d+%, y: \d+%\)\[)(paren\.(l|r))\]/g, (_, start, token) => {
    const char = token === 'paren.l' ? '(' : ')';
    return `${start}${char}]`;
  });
}
