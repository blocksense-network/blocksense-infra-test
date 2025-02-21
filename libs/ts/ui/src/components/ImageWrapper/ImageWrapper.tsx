import React from 'react';
import Image from 'next/image';

type ImageProps = {
  src: string;
  alt?: string;
  className?: string;
  onClick?: (e: React.MouseEvent) => void;
};

export const ImageWrapper = ({
  src,
  alt = ' ',
  className = ' ',
  onClick,
}: ImageProps) => {
  return <img src={src} alt={alt} className={className} onClick={onClick} />;
};
