/**
 * Multi-Language Code Snippet Synthesizer
 * Formats API requests into copy-ready code for cURL, Python, TypeScript, Rust, and Go.
 */

export interface SnippetRequest {
  method: string;
  url: string;
  headers?: Record<string, string>;
  body?: unknown;
}

export interface GeneratedSnippet {
  language: string;
  label: string;
  code: string;
}

export function generateAllSnippets(req: SnippetRequest): GeneratedSnippet[] {
  return [
    { language: 'curl', label: 'cURL', code: generateCurl(req) },
    { language: 'typescript', label: 'TypeScript (fetch)', code: generateTypeScript(req) },
    { language: 'python', label: 'Python (requests)', code: generatePython(req) },
    { language: 'rust', label: 'Rust (reqwest)', code: generateRust(req) },
    { language: 'go', label: 'Go (net/http)', code: generateGo(req) },
  ];
}

export function generateCurl(req: SnippetRequest): string {
  const method = req.method.toUpperCase();
  const parts: string[] = [`curl -X ${method} "${req.url}"`];

  const headers = req.headers || {};
  for (const [k, v] of Object.entries(headers)) {
    parts.push(`  -H "${k}: ${v}"`);
  }

  if (req.body && method !== 'GET' && method !== 'HEAD') {
    const jsonStr = typeof req.body === 'string' ? req.body : JSON.stringify(req.body, null, 2);
    // Escape quotes for bash/sh
    const escaped = jsonStr.replace(/"/g, '\\"');
    parts.push(`  -d "${escaped.replace(/\n/g, '\\n')}"`);
  }

  return parts.join(' \\\n');
}

export function generateTypeScript(req: SnippetRequest): string {
  const method = req.method.toUpperCase();
  const headers = req.headers || { 'Content-Type': 'application/json' };
  const hasBody = req.body && method !== 'GET' && method !== 'HEAD';

  const bodyStr = hasBody
    ? `,\n  body: JSON.stringify(${JSON.stringify(req.body, null, 4)})`
    : '';

  return `const response = await fetch("${req.url}", {
  method: "${method}",
  headers: ${JSON.stringify(headers, null, 4)}${bodyStr}
});

const data = await response.json();
console.log(data);`;
}

export function generatePython(req: SnippetRequest): string {
  const method = req.method.toLowerCase();
  const headers = req.headers || {};
  const hasBody = req.body && req.method.toUpperCase() !== 'GET';

  const lines: string[] = [
    'import requests',
    '',
    `url = "${req.url}"`,
    `headers = ${JSON.stringify(headers, null, 4)}`,
  ];

  if (hasBody) {
    lines.push(`payload = ${JSON.stringify(req.body, null, 4)}`);
    lines.push(`response = requests.${method}(url, headers=headers, json=payload)`);
  } else {
    lines.push(`response = requests.${method}(url, headers=headers)`);
  }

  lines.push('print(response.status_code)');
  lines.push('print(response.json())');

  return lines.join('\n');
}

export function generateRust(req: SnippetRequest): string {
  const method = req.method.toLowerCase();
  const hasBody = req.body && req.method.toUpperCase() !== 'GET';

  const lines: string[] = [
    'use reqwest::Client;',
    'use serde_json::Value;',
    '',
    '#[tokio::main]',
    'async fn main() -> Result<(), Box<dyn std::error::Error>> {',
    '    let client = Client::new();',
    `    let res = client.${method}("${req.url}")`,
  ];

  if (req.headers) {
    for (const [k, v] of Object.entries(req.headers)) {
      lines.push(`        .header("${k}", "${v}")`);
    }
  }

  if (hasBody) {
    lines.push(`        .json(&serde_json::json!(${JSON.stringify(req.body)}))`);
  }

  lines.push('        .send()');
  lines.push('        .await?;');
  lines.push('');
  lines.push('    let body = res.json::<Value>().await?;');
  lines.push('    println!("{:#?}", body);');
  lines.push('    Ok(())');
  lines.push('}');

  return lines.join('\n');
}

export function generateGo(req: SnippetRequest): string {
  const method = req.method.toUpperCase();
  const hasBody = req.body && method !== 'GET';

  const lines: string[] = [
    'package main',
    '',
    'import (',
    '    "fmt"',
    '    "io"',
    '    "net/http"',
  ];

  if (hasBody) {
    lines.push('    "strings"');
  }

  lines.push(')');
  lines.push('');
  lines.push('func main() {');

  if (hasBody) {
    const escaped = JSON.stringify(req.body).replace(/"/g, '\\"');
    lines.push(`    payload := strings.NewReader("${escaped}")`);
    lines.push(`    req, err := http.NewRequest("${method}", "${req.url}", payload)`);
  } else {
    lines.push(`    req, err := http.NewRequest("${method}", "${req.url}", nil)`);
  }

  lines.push('    if err != nil {');
  lines.push('        panic(err)');
  lines.push('    }');
  lines.push('');

  if (req.headers) {
    for (const [k, v] of Object.entries(req.headers)) {
      lines.push(`    req.Header.Add("${k}", "${v}")`);
    }
  }

  lines.push('');
  lines.push('    res, err := http.DefaultClient.Do(req)');
  lines.push('    if err != nil {');
  lines.push('        panic(err)');
  lines.push('    }');
  lines.push('    defer res.Body.Close()');
  lines.push('');
  lines.push('    body, _ := io.ReadAll(res.Body)');
  lines.push('    fmt.Println(string(body))');
  lines.push('}');

  return lines.join('\n');
}
