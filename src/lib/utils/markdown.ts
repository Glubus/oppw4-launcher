function escapeHtml(value: string) {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function escapeAttribute(value: string) {
  return escapeHtml(value).replace(/`/g, "&#96;");
}

function isSafeUrl(value: string) {
  return /^https?:\/\//i.test(value);
}

function renderInline(value: string) {
  let output = escapeHtml(value);
  output = output.replace(/`([^`]+)`/g, "<code>$1</code>");
  output = output.replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");
  output = output.replace(/\*([^*]+)\*/g, "<em>$1</em>");
  output = output.replace(/\[([^\]]+)\]\(([^)]+)\)/g, (_match, label: string, url: string) => {
    const cleanUrl = url.trim();
    if (!isSafeUrl(cleanUrl)) return label;
    return `<a href="${escapeAttribute(cleanUrl)}" target="_blank" rel="noreferrer">${label}</a>`;
  });
  return output;
}

function closeList(lines: string[], currentList: "ul" | "ol" | null) {
  if (!currentList) return;
  lines.push(`</${currentList}>`);
}

export function markdownToHtml(markdown: string) {
  const source = markdown.trim();
  if (!source) return "";

  const lines = source.replace(/\r\n/g, "\n").split("\n");
  const html: string[] = [];
  let currentList: "ul" | "ol" | null = null;
  let inCodeBlock = false;
  let codeLines: string[] = [];

  for (const rawLine of lines) {
    const line = rawLine.trimEnd();

    if (line.trim().startsWith("```")) {
      if (inCodeBlock) {
        html.push(`<pre><code>${escapeHtml(codeLines.join("\n"))}</code></pre>`);
        codeLines = [];
        inCodeBlock = false;
      } else {
        closeList(html, currentList);
        currentList = null;
        inCodeBlock = true;
      }
      continue;
    }

    if (inCodeBlock) {
      codeLines.push(rawLine);
      continue;
    }

    if (!line.trim()) {
      closeList(html, currentList);
      currentList = null;
      continue;
    }

    const heading = /^(#{1,3})\s+(.+)$/.exec(line);
    if (heading) {
      closeList(html, currentList);
      currentList = null;
      const level = Math.min(heading[1].length + 1, 4);
      html.push(`<h${level}>${renderInline(heading[2])}</h${level}>`);
      continue;
    }

    const unordered = /^[-*]\s+(.+)$/.exec(line);
    if (unordered) {
      if (currentList !== "ul") {
        closeList(html, currentList);
        currentList = "ul";
        html.push("<ul>");
      }
      html.push(`<li>${renderInline(unordered[1])}</li>`);
      continue;
    }

    const ordered = /^\d+[.)]\s+(.+)$/.exec(line);
    if (ordered) {
      if (currentList !== "ol") {
        closeList(html, currentList);
        currentList = "ol";
        html.push("<ol>");
      }
      html.push(`<li>${renderInline(ordered[1])}</li>`);
      continue;
    }

    const quote = /^>\s?(.+)$/.exec(line);
    if (quote) {
      closeList(html, currentList);
      currentList = null;
      html.push(`<blockquote>${renderInline(quote[1])}</blockquote>`);
      continue;
    }

    closeList(html, currentList);
    currentList = null;
    html.push(`<p>${renderInline(line)}</p>`);
  }

  if (inCodeBlock) {
    html.push(`<pre><code>${escapeHtml(codeLines.join("\n"))}</code></pre>`);
  }
  closeList(html, currentList);

  return html.join("");
}

export function markdownToPlainText(markdown: string) {
  return markdown
    .replace(/```[\s\S]*?```/g, " ")
    .replace(/`([^`]+)`/g, "$1")
    .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1")
    .replace(/[#>*_`-]/g, " ")
    .replace(/\s+/g, " ")
    .trim();
}
