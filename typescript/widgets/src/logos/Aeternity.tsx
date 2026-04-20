import React, { SVGProps, memo } from 'react';

function _AeternityLogo(props: SVGProps<SVGSVGElement>) {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" {...props}>
      <circle cx="16" cy="16" r="16" fill="#de3f6b" />
      <path
        fill="#fff"
        d="M22.3 21.5l-3.6-6.2 3.4-5.8h-3.2l-1.8 3.1-1.8-3.1h-3.2l3.4 5.8-3.6 6.2h3.2l2-3.5 2 3.5h3.2z"
      />
    </svg>
  );
}

export const AeternityLogo = memo(_AeternityLogo);
