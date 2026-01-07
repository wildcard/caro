import React from 'react';

interface PixelButtonProps {
  children: React.ReactNode;
  onClick?: () => void;
  href?: string;
  variant?: 'primary' | 'secondary' | 'danger';
  size?: 'sm' | 'md' | 'lg';
  className?: string;
}

export const PixelButton: React.FC<PixelButtonProps> = ({
  children,
  onClick,
  href,
  variant = 'primary',
  size = 'md',
  className = '',
}) => {
  const variantStyles = {
    primary: 'text-neon-green border-neon-green hover:bg-neon-green hover:text-pixel-bg-primary',
    secondary: 'text-neon-blue border-neon-blue hover:bg-neon-blue hover:text-pixel-bg-primary',
    danger: 'text-terminal-red border-terminal-red hover:bg-terminal-red hover:text-pixel-bg-primary',
  };

  const sizeStyles = {
    sm: 'text-[8px] px-3 py-2',
    md: 'text-[10px] px-4 py-3',
    lg: 'text-[12px] px-6 py-4',
  };

  const baseClasses = `pixel-button ${variantStyles[variant]} ${sizeStyles[size]} ${className}`;

  if (href) {
    return (
      <a
        href={href}
        className={baseClasses}
        target={href.startsWith('http') ? '_blank' : undefined}
        rel={href.startsWith('http') ? 'noopener noreferrer' : undefined}
      >
        {children}
      </a>
    );
  }

  return (
    <button onClick={onClick} className={baseClasses}>
      {children}
    </button>
  );
};
