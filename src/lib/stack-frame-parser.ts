export interface StackFrame {
  function: string;
  file: string;
  line: number;
  column?: number;
  isInternal: boolean;
}

function checkIsInternalFrame(path: string): boolean {
  return (
    path.includes('node_modules') ||
    path.includes('@tauri-apps') ||
    path.includes('react-dom') ||
    path.includes('react.development') ||
    path.includes('scheduler')
  );
}

function parseLocationTail(location: string): { file: string; line: number; column: number } | null {
  const match = location.match(/^(.*):(\d+):(\d+)$/);
  if (!match) {
    return null;
  }
  return {
    file: match[1],
    line: parseInt(match[2], 10) || 0,
    column: parseInt(match[3], 10) || 0,
  };
}

/** Parse one V8-style stack line (`at …`) into a frame, or null if the line does not match. */
export function parseStackLine(line: string): StackFrame | null {
  const trimmed = line.trim();
  if (!trimmed.startsWith('at ')) {
    return null;
  }

  let rest = trimmed.slice(3);
  let prefixAsync = false;
  if (rest.startsWith('async ')) {
    prefixAsync = true;
    rest = rest.slice(6);
  }

  const namedMatch = rest.match(/^(.+?)\s+\((.+):(\d+):(\d+)\)$/);
  if (namedMatch) {
    const filePath = namedMatch[2];
    const lineNum = parseInt(namedMatch[3], 10) || 0;
    const colNum = parseInt(namedMatch[4], 10) || 0;
    return {
      function: namedMatch[1].trim(),
      file: filePath,
      line: lineNum,
      column: colNum,
      isInternal: checkIsInternalFrame(filePath),
    };
  }

  const bare = parseLocationTail(rest);
  if (!bare) {
    return null;
  }

  const fnName = prefixAsync ? 'async <anonymous>' : '<anonymous>';
  return {
    function: fnName,
    file: bare.file,
    line: bare.line,
    column: bare.column,
    isInternal: checkIsInternalFrame(bare.file),
  };
}
