import { createPortal } from 'react-dom';
import { useCallback, useMemo, useRef, type HTMLAttributes, type KeyboardEventHandler, type ReactNode } from 'react';

type RetainedFocus = { target: HTMLElement | null; root: boolean };

export type RetainedLayoutProps = HTMLAttributes<HTMLDivElement> & {
  geometryKey: string;
  children?: ReactNode;
};

type MovableParent = HTMLElement & {
  moveBefore?: (node: Node, child: Node | null) => void;
};

function moveHost(host: HTMLDivElement, parent: HTMLElement): void {
  if (host.parentNode === parent) return;
  const movable = parent as MovableParent;
  if (host.isConnected && parent.isConnected && typeof movable.moveBefore === 'function') {
    try {
      movable.moveBefore(host, null);
      return;
    } catch {
      // Older WebKit may expose moveBefore without supporting this move.
    }
  }
  parent.appendChild(host);
}

/**
 * Replace a stateless geometry shell while retaining stateful child fibers in one portal host.
 * The outer display-contents wrapper keeps logical event bubbling stable across shell swaps.
 */
export function RetainedLayout({ geometryKey, children, className, style, onKeyDown, ...attributes }: RetainedLayoutProps) {
  const host = useMemo<HTMLDivElement | null>(() => {
    if (typeof document === 'undefined') return null;
    const element = document.createElement('div');
    element.style.display = 'contents';
    element.setAttribute('data-retained-layout-host', 'true');
    return element;
  }, []);
  const ownerRef = useRef<HTMLDivElement>(null);
  const focusRef = useRef<RetainedFocus | null>(null);

  const attachShell = useCallback((shell: HTMLDivElement | null) => {
    if (!shell || !host) return;
    moveHost(host, shell);
    const focus = focusRef.current;
    focusRef.current = null;
    if (focus?.root) shell.focus({ preventScroll: true });
    else if (focus?.target && focus.target.isConnected) focus.target.focus({ preventScroll: true });
    return () => {
      const active = document.activeElement;
      if (active instanceof HTMLElement && shell.contains(active)) {
        focusRef.current = active === shell ? { target: null, root: true } : { target: active, root: false };
      }
      const owner = ownerRef.current ?? shell.parentElement;
      if (owner) moveHost(host, owner);
    };
  }, [host]);

  const handleKeyDown: KeyboardEventHandler<HTMLDivElement> | undefined = onKeyDown;
  return (
    <div ref={ownerRef} style={{ display: 'contents' }} onKeyDown={handleKeyDown}>
      <div key={geometryKey} ref={attachShell} className={className} style={style} {...attributes} />
      {host && createPortal(children, host)}
    </div>
  );
}
