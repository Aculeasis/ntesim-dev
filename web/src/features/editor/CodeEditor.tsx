import {
  forwardRef,
  useImperativeHandle,
  useMemo,
  useRef,
  useState,
  type CSSProperties,
} from 'react';
import type { DisplayConfig } from '../../domain/contracts/display';
import type { SourceLocation } from '../../domain/contracts/script';
import { highlightTokens } from './highlight';
export interface EditorHandle {
  insert: (text: string) => void;
  reveal: (location: SourceLocation) => void;
}

export function textareaRange(
  source: string,
  location: Pick<SourceLocation, 'offset' | 'length'>,
): [number, number] {
  // HTML textareas normalize line endings; keep the authored source untouched.
  const offset = (raw: number) => source.slice(0, raw).replace(/\r\n?/g, '\n').length;
  return [offset(location.offset), offset(location.offset + location.length)];
}
export const CodeEditor = forwardRef<
  EditorHandle,
  { source: string; onChange: (source: string) => void; editor: DisplayConfig['editor'] }
>(({ source, onChange, editor }, handle) => {
  const input = useRef<HTMLTextAreaElement>(null),
    paint = useRef<HTMLPreElement>(null),
    numbers = useRef<HTMLPreElement>(null),
    [cursor, setCursor] = useState({ line: 1, column: 1 });
  const tokens = useMemo(() => highlightTokens(source), [source]);
  const updateCursor = () => {
    const before = input.current!.value.slice(0, input.current!.selectionStart).split('\n');
    setCursor({ line: before.length, column: before.at(-1)!.length + 1 });
  };
  const insert = (text: string) => {
    const area = input.current!;
    area.setRangeText(text, area.selectionStart, area.selectionEnd, 'end');
    onChange(area.value);
    area.focus();
    updateCursor();
  };
  useImperativeHandle(handle, () => ({
    insert,
    reveal: (location) => {
      const area = input.current!;
      area.focus();
      area.setSelectionRange(...textareaRange(source, location));
      const lineHeight = parseFloat(getComputedStyle(area).lineHeight);
      area.scrollTop = Math.max(0, (location.line - 3) * lineHeight);
      updateCursor();
    },
  }));
  return (
    <>
      <div
        className="editor-shell"
        data-theme={editor.theme}
        style={{ '--editor-font': editor.font + 'px' } as CSSProperties}
      >
        <pre ref={numbers} id="line-numbers" aria-hidden="true" hidden={editor.wrap}>
          {source
            .split('\n')
            .map((_, i) => i + 1)
            .join('\n') + '\n'}
        </pre>
        <div className={`code-content ${editor.wrap ? 'wrapped' : ''}`}>
          <pre ref={paint} id="code-highlight" aria-hidden="true">
            {tokens.map((token, i) =>
              token.type ? (
                <span className={'tok-' + token.type} key={i}>
                  {token.text}
                </span>
              ) : (
                token.text
              ),
            )}
          </pre>
          <textarea
            ref={input}
            id="script-input"
            aria-label="Simulation configuration"
            autoCapitalize="off"
            autoComplete="off"
            spellCheck={false}
            wrap={editor.wrap ? 'soft' : 'off'}
            value={source}
            onChange={(e) => onChange(e.target.value)}
            onSelect={updateCursor}
            onKeyDown={(e) => {
              if (e.key === 'Tab') {
                e.preventDefault();
                insert('  ');
              }
            }}
            onScroll={() => {
              const area = input.current!;
              paint.current!.scrollTop = area.scrollTop;
              paint.current!.scrollLeft = area.scrollLeft;
              numbers.current!.scrollTop = area.scrollTop;
            }}
          />
        </div>
      </div>
      <div className="editor-status">
        <span id="cursor-status">
          Ln {cursor.line} · Col {cursor.column}
        </span>
        <span>NTE DSL</span>
      </div>
    </>
  );
});
CodeEditor.displayName = 'CodeEditor';
