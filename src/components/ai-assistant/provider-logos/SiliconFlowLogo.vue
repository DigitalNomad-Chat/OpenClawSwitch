<template>
  <svg width="32" height="32" viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg">
    <!-- 硅基流动 — 深紫黑背景 + 渐变 SF -->
    <defs>
      <linearGradient id="sfGrad" x1="6" y1="8" x2="26" y2="24" gradientUnits="userSpaceOnUse">
        <stop offset="0%" stop-color="#6366F1"/>
        <stop offset="100%" stop-color="#A855F7"/>
      </linearGradient>
    </defs>
    <rect width="32" height="32" rx="8" fill="#1A1A2E"/>
    <!-- "SF" 字母 -->
    <path
      d="M10 10H14V12.5H12V14.5H14V17H12V19.5H14.5V22H10V10Z"
      fill="url(#sfGrad)"
    />
    <path
      d="M16 10H22V12.5H18.5V14.5H21V17H18.5V22H16V10Z"
      fill="url(#sfGrad)"
    />
    <!-- 流动光点装饰 -->
    <circle cx="24" cy="10" r="2" fill="#6366F1" opacity="0.5"/>
    <circle cx="8" cy="24" r="1.5" fill="#A855F7" opacity="0.4"/>
  </svg>
</template>
