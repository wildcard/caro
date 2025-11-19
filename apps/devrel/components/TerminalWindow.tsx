'use client';

import React, { useState, useEffect } from 'react';

interface TerminalWindowProps {
  title?: string;
  children?: React.ReactNode;
  command?: string;
  output?: string;
  animate?: boolean;
  className?: string;
}

export const TerminalWindow: React.FC<TerminalWindowProps> = ({
  title = 'cmdai@terminal',
  children,
  command,
  output,
  animate = true,
  className = '',
}) => {
  const [displayedCommand, setDisplayedCommand] = useState('');
  const [displayedOutput, setDisplayedOutput] = useState('');
  const [showCursor, setShowCursor] = useState(true);

  useEffect(() => {
    if (!animate || !command) {
      setDisplayedCommand(command || '');
      setDisplayedOutput(output || '');
      return;
    }

    let currentIndex = 0;
    const typingSpeed = 50;

    const typeCommand = () => {
      if (currentIndex < (command?.length || 0)) {
        setDisplayedCommand(command!.substring(0, currentIndex + 1));
        currentIndex++;
        setTimeout(typeCommand, typingSpeed);
      } else {
        setTimeout(() => {
          setDisplayedOutput(output || '');
        }, 300);
      }
    };

    typeCommand();

    // Cursor blink
    const cursorInterval = setInterval(() => {
      setShowCursor((prev) => !prev);
    }, 500);

    return () => clearInterval(cursorInterval);
  }, [command, output, animate]);

  return (
    <div className={`terminal-window ${className}`}>
      {/* Terminal header */}
      <div className="flex items-center justify-between bg-pixel-bg-tertiary px-4 py-2 border-b-4 border-terminal-green">
        <div className="flex gap-2">
          <div className="w-3 h-3 bg-terminal-red"></div>
          <div className="w-3 h-3 bg-terminal-amber"></div>
          <div className="w-3 h-3 bg-terminal-green"></div>
        </div>
        <span className="pixel-text text-[8px] text-terminal-green">{title}</span>
        <div className="w-12"></div>
      </div>

      {/* Terminal content */}
      <div className="p-4 font-mono text-sm">
        {children || (
          <>
            {command && (
              <div className="flex items-start gap-2">
                <span className="text-neon-green">$</span>
                <span className="text-neon-blue">
                  {displayedCommand}
                  {animate && !displayedOutput && showCursor && (
                    <span className="inline-block w-2 h-4 bg-neon-green ml-1"></span>
                  )}
                </span>
              </div>
            )}
            {displayedOutput && (
              <div className="mt-2 text-terminal-green whitespace-pre-wrap">
                {displayedOutput}
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
};
