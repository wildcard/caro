import React from 'react';

interface PixelCardProps {
  children: React.ReactNode;
  title?: string;
  variant?: 'default' | 'neon' | 'gameboy';
  className?: string;
}

export const PixelCard: React.FC<PixelCardProps> = ({
  children,
  title,
  variant = 'default',
  className = '',
}) => {
  const variantStyles = {
    default: 'border-neon-blue',
    neon: 'border-neon-pink',
    gameboy: 'border-gameboy-light bg-gameboy-dark',
  };

  return (
    <div className={`pixel-card ${variantStyles[variant]} ${className}`}>
      {title && (
        <div className="pixel-text text-[10px] mb-4 text-neon-green">
          {title}
        </div>
      )}
      <div className="text-sm leading-relaxed">
        {children}
      </div>
    </div>
  );
};
